#!/usr/bin/env python3
"""Regenerate the generated regions of docs/site/index.html (command pages, model catalog,
example archive and version stamps) from the local binary, assets/models.json and
docs/site/examples, then derive the agent-readable docs/site/index.md and docs/site/llms.txt
from the finished page; --check fails when any of them has drifted. Offline: the only
subprocess is the local framechord executable."""
import argparse
import base64
import datetime
import difflib
import hashlib
import html
import html.parser
import io
import json
import pathlib
import re
import subprocess
import sys
import tempfile
import tomllib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
SITE_URL = 'https://framechord.com/'
REPO_URL = 'https://github.com/maxiplux/framechord'
RAW_URL = 'https://raw.githubusercontent.com/maxiplux/framechord/master/'
ENTRY = ('<details class="command-entry"><summary>{path}</summary><div class="code-block"><div class="code-bar">'
         '<span>{path}</span><button type="button" class="copy transition-colors motion-reduce:transition-none">Copy</button>'
         '</div><pre><code>{help}</code></pre></div></details>')
MARKERS = {
    'commands': ('<!-- generated:commands -->', '<!-- /generated:commands -->'),
    'catalog': ('<!-- generated:catalog -->', '<!-- /generated:catalog -->'),
}
PROVIDER_ORDER = ['openai', 'fish', 'google', 'elevenlabs', 'openrouter']
ZIP_DATE = (2026, 10, 14, 22, 15, 34)
DATA_URI = re.compile(r'href="data:application/zip;base64,([A-Za-z0-9+/=]+)"')
COMMAND_BLOCK = re.compile(r'\n## (framechord[^\n]*)\n\n```text\n(.*?)\n```\n', re.DOTALL)
STAMPS = {
    'version': (re.compile(r'<span class="version">v[^<]*</span>'), '<span class="version">v{version}</span>'),
    'target': (re.compile(r'This guide targets <strong>FrameChord [^<]*</strong>'), 'This guide targets <strong>FrameChord {version}</strong>'),
    'software version': (re.compile(r'"softwareVersion":"[^"]*"'), '"softwareVersion":"{version}"'),
    'command pages': (re.compile(r'all \d+ command pages'), 'all {count} command pages'),
    'catalog date': (re.compile(r'The bundled catalog was verified on [^.]*\.'), 'The bundled catalog was verified on {date}.'),
}


def crate_version(root=ROOT):
    return tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']


def default_binary(root=ROOT):
    name = 'framechord.exe' if sys.platform == 'win32' else 'framechord'
    for profile in ['release', 'debug']:
        candidate = root / 'target' / profile / name
        if candidate.is_file():
            return candidate
    raise SystemExit('No built framechord executable under target/; run cargo build --release --locked or pass --binary')


def export_commands(binary):
    """Exports the bundled skill into a scratch directory and returns commands.md."""
    with tempfile.TemporaryDirectory(prefix='framechord-site-') as scratch:
        target = pathlib.Path(scratch) / 'skill'
        subprocess.run([str(binary), '--json', '--non-interactive', 'skills', 'export', str(target)],
                       capture_output=True, text=True, check=True, timeout=120)
        return (target / 'references' / 'commands.md').read_text(encoding='utf-8')


def binary_version(binary):
    output = subprocess.run([str(binary), '--version'], capture_output=True, text=True, check=True, timeout=60).stdout
    return output.strip().split()[-1]


def parse_commands(markdown):
    """Real command paths in declaration order; clap's `help` pseudo-commands are dropped."""
    entries = []
    for path, body in COMMAND_BLOCK.findall(markdown):
        if 'help' in path.split():
            continue
        # Trailing whitespace is stripped per line; each page ends with exactly one newline.
        entries.append((path, '\n'.join(line.rstrip() for line in body.split('\n')).rstrip('\n') + '\n'))
    return entries


def render_commands(entries):
    return ''.join(ENTRY.format(path=html.escape(path, quote=True), help=html.escape(body, quote=True)) for path, body in entries)


def render_catalog(models):
    current = [m for m in models['models'] if m.get('current') and m.get('supported')]
    parts = []
    for provider in PROVIDER_ORDER:
        items = ''.join(
            f'<li><code>{html.escape(m["model"])}</code> — {m["media"]}: {html.escape(m["description"])}</li>'
            for m in current if m['provider'] == provider)
        parts.append(f'<h3>{provider}</h3><ul>{items}</ul>')
    return ''.join(parts)


def build_zip(examples):
    """Deterministic archive of the example sources; fixed metadata keeps the bytes stable."""
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, 'w', zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(p for p in pathlib.Path(examples).rglob('*') if p.is_file()):
            info = zipfile.ZipInfo('examples/' + path.relative_to(examples).as_posix(), date_time=ZIP_DATE)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o600 << 16
            info.create_system = 3
            archive.writestr(info, path.read_bytes())
    return buffer.getvalue()


def zip_manifest(payload):
    """Name → bytes, so archives compare by content rather than by compressor output."""
    with zipfile.ZipFile(io.BytesIO(payload)) as archive:
        if archive.testzip() is not None:
            raise ValueError('corrupt example archive')
        return {name: archive.read(name) for name in archive.namelist()}


def splice(text, name, generated):
    opening, closing = MARKERS[name]
    if text.count(opening) != 1 or text.count(closing) != 1:
        raise ValueError(f'expected exactly one {opening} … {closing} pair in the guide')
    start = text.index(opening) + len(opening)
    end = text.index(closing)
    if end < start:
        raise ValueError(f'{opening} appears after its closing marker')
    return text[:start] + generated + text[end:]


def apply_stamps(text, version, count, date):
    for label, (pattern, template) in STAMPS.items():
        matches = pattern.findall(text)
        if len(matches) != 1:
            raise ValueError(f'expected exactly one {label} stamp, found {len(matches)}')
        text = pattern.sub(template.format(version=version, count=count, date=date), text)
    return text


def catalog_date(models):
    date = datetime.date.fromisoformat(models['verified_at'])
    return f'{date:%B} {date.day}, {date.year}'


def verify_anchors(text):
    ids = set(re.findall(r'(?<![-\w])id="([^"]+)"', text))
    missing = sorted({anchor for anchor in re.findall(r'href="#([^"]+)"', text) if anchor not in ids})
    if missing:
        raise ValueError('nav anchors without a matching id: ' + ', '.join('#' + m for m in missing))


def replace_archives(text, payload):
    """Every data:application/zip link on the page is the example pack; all are replaced together."""
    encoded = base64.b64encode(payload).decode('ascii')
    matches = DATA_URI.findall(text)
    if not matches:
        raise ValueError('no embedded example archive found in the guide')
    if all(zip_manifest(base64.b64decode(m)) == zip_manifest(payload) for m in matches) and len(set(matches)) == 1:
        return text  # identical content: keep the committed bytes to avoid churn
    return DATA_URI.sub(lambda _: f'href="data:application/zip;base64,{encoded}"', text)


def regenerate(text, *, commands_markdown, models, examples, version):
    entries = parse_commands(commands_markdown)
    if not entries:
        raise ValueError('no command pages parsed from commands.md')
    text = splice(text, 'commands', render_commands(entries))
    text = splice(text, 'catalog', render_catalog(models))
    text = replace_archives(text, build_zip(examples))
    text = apply_stamps(text, version, len(entries), catalog_date(models))
    if text.count('class="command-entry"') != len(entries):
        raise ValueError('command entry count does not match the stamped page count')
    verify_anchors(text)
    return text


# Agent-readable copies. index.md mirrors <main> of the finished page; llms.txt
# (https://llmstxt.org) lists it with the facts an agent needs before reading further.
VOID_TAGS = {'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'source', 'track', 'wbr'}
SKIP_TAGS = {'script', 'style', 'button', 'input', 'label', 'nav', 'svg', 'template', 'textarea', 'select'}
# Copy-button bars, screen-reader status regions and decorative terminal chrome carry no content.
SKIP_CLASSES = {'code-bar', 'sr-only', 'skip', 'terminal-caption', 'status-dot', 'deck-count', 'slide-num'}
BLOCK_TAGS = {'article', 'aside', 'blockquote', 'caption', 'dd', 'details', 'div', 'dl', 'dt', 'figcaption',
              'figure', 'footer', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'header', 'hr', 'li', 'main', 'ol', 'p',
              'pre', 'section', 'summary', 'table', 'ul'}
HEADING_LEVEL = {'h1': 2, 'h2': 2, 'h3': 3, 'h4': 4, 'h5': 5, 'h6': 6}  # the document title is the only '#'
EXAMPLES_URL = REPO_URL + '/tree/master/docs/site/examples'
MD_ANCHOR = re.compile(r'<a id="([^"]+)"></a>')
MD_LOCAL_LINK = re.compile(r'\]\(#([^)\s]+)\)')


class Node:
    __slots__ = ('tag', 'attrs', 'children')

    def __init__(self, tag, attrs):
        self.tag, self.attrs, self.children = tag, dict(attrs), []

    def classes(self):
        return set((self.attrs.get('class') or '').split())


class TreeBuilder(html.parser.HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.root = Node('#root', [])
        self.stack = [self.root]

    def handle_starttag(self, tag, attrs):
        node = Node(tag, attrs)
        self.stack[-1].children.append(node)
        if tag not in VOID_TAGS:
            self.stack.append(node)

    def handle_startendtag(self, tag, attrs):
        self.stack[-1].children.append(Node(tag, attrs))

    def handle_endtag(self, tag):
        for depth in range(len(self.stack) - 1, 0, -1):
            if self.stack[depth].tag == tag:
                del self.stack[depth:]
                return

    def handle_data(self, data):
        self.stack[-1].children.append(data)


def parse_html(text):
    builder = TreeBuilder()
    builder.feed(text)
    builder.close()
    return builder.root


def elements(node):
    return [child for child in node.children if isinstance(child, Node)]


def find_all(node, predicate):
    found = []
    for child in elements(node):
        if predicate(child):
            found.append(child)
        found.extend(find_all(child, predicate))
    return found


def find_first(node, predicate):
    matches = find_all(node, predicate)
    return matches[0] if matches else None


def skipped(node):
    if node.tag in SKIP_TAGS or 'hidden' in node.attrs or node.attrs.get('aria-hidden') == 'true':
        return True
    # The narrated tour is bilingual; the Markdown copy keeps the English text and links both audio sets.
    return node.attrs.get('lang') == 'es' or bool(node.classes() & SKIP_CLASSES)


def raw_text(node):
    return ''.join(child if isinstance(child, str) else raw_text(child)
                   for child in node.children if isinstance(child, str) or not skipped(child))


def plain(node):
    return re.sub(r'\s+', ' ', raw_text(node)).strip()


def escape_md(text):
    return re.sub(r'([\\`*<\[\]])', r'\\\1', text)


def code_span(text):
    text = re.sub(r'\s+', ' ', text)
    fence = '`' * (max((len(run) for run in re.findall(r'`+', text)), default=0) + 1)
    pad = ' ' if text.startswith('`') or text.endswith('`') else ''
    return f'{fence}{pad}{text}{pad}{fence}'


def fenced(text):
    text = text.strip('\n')
    fence = '`' * max(3, max((len(run) for run in re.findall(r'`+', text)), default=0) + 1)
    return f'{fence}\n{text}\n{fence}'


def link_target(href):
    if href.startswith('data:'):
        return EXAMPLES_URL
    return f'<{href}>' if re.search(r'[\s()]', href) else href


def wrap(marker, inner):
    core = inner.strip()
    if not core:
        return inner
    lead, trail = inner[:len(inner) - len(inner.lstrip())], inner[len(inner.rstrip()):]
    return f'{lead}{marker}{core}{marker}{trail}'


def inline_node(node):
    if isinstance(node, str):
        return escape_md(node)
    if skipped(node):
        return ''
    tag = node.tag
    if tag == 'br':
        return ' '
    if tag in ('strong', 'b'):
        return wrap('**', inline(node))
    if tag in ('em', 'i'):
        return wrap('*', inline(node))
    if tag == 'code':
        return code_span(raw_text(node))
    if tag == 'img':
        return f'![{escape_md(node.attrs.get("alt", ""))}]({link_target(node.attrs.get("src", ""))})'
    if tag == 'audio':
        source = node.attrs.get('src') or next((s.attrs.get('src') for s in elements(node) if s.tag == 'source'), '')
        return f' [Audio: {escape_md(source.rsplit("/", 1)[-1])}]({link_target(source)}) ' if source else ''
    if tag == 'a':
        text = inline(node).strip()
        href = node.attrs.get('href', '')
        return f'[{text}]({link_target(href)})' if text and href and href != '#' else text
    return inline(node)


def inline(node):
    return re.sub(r'\s+', ' ', ''.join(inline_node(child) for child in node.children))


def render_list(node):
    items = []
    for number, item in enumerate((li for li in elements(node) if li.tag == 'li' and not skipped(li)), 1):
        body = '\n\n'.join(blocks(item))
        if not body:
            continue
        if item.attrs.get('id'):
            body = f'<a id="{item.attrs["id"]}"></a>{body}'
        marker = f'{number}. ' if node.tag == 'ol' else '- '
        lines = body.split('\n')
        items.append(marker + lines[0] + ''.join('\n' + (' ' * len(marker) + line if line else '') for line in lines[1:]))
    if not items:
        return []
    return ['\n'.join(items) if all('\n' not in item for item in items) else '\n\n'.join(items)]


def render_table(node):
    def cells(row):
        return [inline(cell).strip().replace('|', '\\|') for cell in elements(row) if cell.tag in ('th', 'td')]
    rows = [cells(row) for row in find_all(node, lambda n: n.tag == 'tr')]
    if not rows:
        return []
    out = []
    caption = find_first(node, lambda n: n.tag == 'caption')
    if caption is not None:
        out.append(f'**{inline(caption).strip()}**')
    width = max(len(row) for row in rows)
    lines = ['| ' + ' | '.join(row + [''] * (width - len(row))) + ' |' for row in rows]
    lines.insert(1, '|' + ' --- |' * width)
    return out + ['\n'.join(lines)]


def render_definitions(node):
    out, label = [], None
    for item in find_all(node, lambda n: n.tag in ('dt', 'dd')):
        if item.tag == 'dt':
            label = inline(item).strip()
        else:
            out.append(f'**{label}:** {inline(item).strip()}' if label else inline(item).strip())
            label = None
    return out


def render_block(node):
    tag, classes = node.tag, node.classes()
    if tag in HEADING_LEVEL:
        text = inline(node).strip()
        return ['#' * HEADING_LEVEL[tag] + ' ' + text] if text else []
    if tag in ('p', 'figcaption', 'dd'):
        text = inline(node).strip()
        return [text] if text else []
    if tag in ('summary', 'dt', 'caption'):
        text = inline(node).strip()
        return [f'**{text}**'] if text else []
    if tag == 'pre':
        return [fenced(raw_text(node))]
    if tag in ('ul', 'ol'):
        return render_list(node)
    if tag == 'table':
        return render_table(node)
    if tag == 'dl':
        return render_definitions(node)
    if tag == 'hr':
        return ['---']
    if tag in ('aside', 'blockquote'):
        inner = '\n\n'.join(blocks(node))
        return ['\n'.join('> ' + line if line else '>' for line in inner.split('\n'))] if inner else []
    if tag == 'details' and 'command-entry' in classes:
        summary = find_first(node, lambda n: n.tag == 'summary')
        pre = find_first(node, lambda n: n.tag == 'pre')
        if summary is None or pre is None:  # generated entries always carry both; anything else is a bug
            raise ValueError('a command-entry needs a <summary> and a <pre> with its help text')
        return [f'### {plain(summary)}', fenced(raw_text(pre))]
    if classes & {'hero-actions', 'media-tags'}:  # side-by-side links and tags with no text between them
        parts = [inline_node(child).strip() for child in elements(node)]
        return [' · '.join(part for part in parts if part)] if any(parts) else []
    if 'resource-links' in classes:  # cards: <a><strong>title</strong><span>description</span></a>
        items = []
        for card in (child for child in elements(node) if child.tag == 'a'):
            title = find_first(card, lambda n: n.tag == 'strong')
            note = find_first(card, lambda n: n.tag == 'span')
            items.append(f'- [{inline(title).strip() if title is not None else plain(card)}]'
                         f'({link_target(card.attrs.get("href", ""))})' + (f': {inline(note).strip()}' if note is not None else ''))
        return ['\n'.join(items)] if items else []
    if 'code-block' in classes:
        bar = find_first(node, lambda n: 'code-bar' in n.classes())
        span = find_first(bar, lambda n: n.tag == 'span') if bar is not None else None
        label = plain(span) if span is not None else ''
        return ([f'{escape_md(label)}:'] if label else []) + [fenced(raw_text(pre)) for pre in find_all(node, lambda n: n.tag == 'pre')]
    return blocks(node)


def blocks(node):
    out, run = [], []

    def flush():
        text = re.sub(r'\s+', ' ', ''.join(run)).strip()
        if text:
            out.append(text)
        run.clear()

    for child in node.children:
        if isinstance(child, Node) and child.tag in BLOCK_TAGS and not skipped(child):
            flush()
            rendered = render_block(child)
            if rendered and child.attrs.get('id'):
                out.append(f'<a id="{child.attrs["id"]}"></a>')
            out.extend(rendered)
        else:
            run.append(inline_node(child))
    flush()
    return out


def guide_sections(root):
    """(id, label, heading, summary) for every sidebar link, in navigation order."""
    nav = find_first(root, lambda n: n.tag == 'nav')
    if nav is None:
        raise ValueError('the guide has no section navigation')
    sections = []
    for link in find_all(nav, lambda n: n.tag == 'a' and n.attrs.get('href', '').startswith('#')):
        anchor = link.attrs['href'][1:]
        label = re.sub(r'\s+', ' ', ''.join(c for c in link.children if isinstance(c, str))).strip()
        section = find_first(root, lambda n, a=anchor: n.attrs.get('id') == a)
        heading = find_first(section, lambda n: 'section-heading' in n.classes()) if section is not None else None
        title = find_first(heading, lambda n: n.tag == 'h2') if heading is not None else None
        summary = find_first(heading, lambda n: n.tag == 'p') if heading is not None else None
        sections.append((anchor, label, plain(title) if title is not None else label,
                         plain(summary) if summary is not None else ''))
    return sections


def render_markdown(page, version):
    root = parse_html(page)
    main = find_first(root, lambda n: n.tag == 'main')
    title = find_first(root, lambda n: n.tag == 'title')
    if main is None or title is None:
        raise ValueError('the guide needs a <title> and a <main> element')
    contents = '\n'.join(f'- [{escape_md(label)}](#{anchor})' for anchor, label, _, _ in guide_sections(root))
    document = [
        '<!-- Generated by scripts/build-site.py from docs/site/index.html; edit the HTML, not this file. -->',
        f'# {plain(title)}',
        f'> Markdown version of {SITE_URL} for FrameChord {version}, for agents and readers without a browser. '
        f'The short index for agents is {SITE_URL}llms.txt. Media paths are relative to {SITE_URL}.',
        '## Contents', contents,
    ] + blocks(main)
    markdown = '\n\n'.join(document) + '\n'
    ids = set(MD_ANCHOR.findall(markdown))
    missing = sorted({anchor for anchor in MD_LOCAL_LINK.findall(markdown) if anchor not in ids})
    if missing:
        raise ValueError('Markdown links without a matching anchor: ' + ', '.join('#' + m for m in missing))
    return markdown


def render_llms(page, version, command_count):
    sections = guide_sections(parse_html(page))
    lines = [
        '# FrameChord',
        '',
        '> FrameChord is a local command-line tool, one binary, that turns YAML and Markdown briefs into images, '
        'infographics, speech, podcasts, audiobooks, music and sound effects through OpenAI, Google Gemini, Fish Audio, '
        'OpenRouter and ElevenLabs. Coding agents such as Claude Code, Codex, Cursor, Gemini CLI, OpenCode and '
        f'GitHub Copilot drive it through a bundled skill. This file describes FrameChord {version}.',
        '',
        'Before you act for a user:',
        '',
        '- Install: `curl -fsSL https://framechord.com/install.sh | sh` on macOS or Linux, '
        '`irm https://framechord.com/install.ps1 | iex` in Windows PowerShell. Check with `framechord --version`.',
        '- Set up a project with `framechord init` at its root. It is offline, preserves existing files and installs '
        'the skill in `.claude/skills/framechord/` and `.agents/skills/framechord/`. Read that `SKILL.md` and its '
        'references before authoring files.',
        '- Credentials come from the environment or the project `.env`: `GEMINI_API_KEY`, `OPENAI_API_KEY`, '
        '`FISH_API_KEY`, `OPENROUTER_API_KEY`, `ELEVENLABS_API_KEY`. Never print, paste or commit key values.',
        '- Loop: author a YAML job and Markdown brief, run `framechord validate`, preview with `--dry-run --json` '
        '(no provider calls), show the user that preview and wait for their explicit approval before generating, '
        'then report the artifact paths from the JSON result. A request to make media is not approval of the '
        'preview.',
        '- Pass `--json --non-interactive`. Keep scripts, lyrics and figures exact. Do not switch provider, model '
        'or voice unless asked. Get fresh approval before resuming a stopped run or batch, and never retry an '
        'uncertain paid request without explicit approval.',
        '- A request for an audiobook, or to read a book or long document aloud, is `media: audiobook`: one '
        'narrator, the exact source text, one file per chapter and optionally the full book. A podcast is a '
        'multi-speaker conversation; do not turn a book into one unless the user asks.',
        '',
        '## Guide',
        '',
        f'- [Full guide in Markdown]({SITE_URL}index.md): every section below and all {command_count} command pages',
    ]
    for anchor, label, heading, summary in sections:
        description = ' — '.join(part for part in (heading if heading != label else '', summary) if part)
        lines.append(f'- [{label}]({SITE_URL}index.md#{anchor})' + (f': {description}' if description else ''))
    lines += [
        '',
        '## Source documentation',
        '',
        f'- [README]({RAW_URL}README.md): overview, quick start and an example for every media type',
        f'- [Integration guide]({RAW_URL}docs/INTEGRATION.md): existing repositories, scripts, CI and other agents',
        f'- [Installation]({RAW_URL}docs/INSTALLATION.md): installers, updates and rollback',
        f'- [Cost]({RAW_URL}docs/COST.md): pay-per-asset pricing and `usage` estimates',
        f'- [Connections]({RAW_URL}docs/CONNECTIONS.md): `doctor` checks and provider key failures',
        '',
        '## Optional',
        '',
        f'- [Example pack]({EXAMPLES_URL}): the job files used by the guide scenarios',
        f'- [Guide media provenance]({SITE_URL}media/provenance.json): how the guide\'s own media was generated',
        f'- [Releases]({REPO_URL}/releases/latest): native binaries and checksums',
    ]
    return '\n'.join(lines) + '\n'


def check_text(current, regenerated, name):
    if current == regenerated:
        return True, f'{name} is up to date'
    diff = list(difflib.unified_diff(current.splitlines(), regenerated.splitlines(), f'committed {name}',
                                     f'regenerated {name}', lineterm='', n=1))
    report = [line if len(line) < 400 else line[:400] + '…' for line in diff[:80]]
    if len(diff) > 80:
        report.append(f'… {len(diff) - 80} more diff lines')
    return False, '\n'.join(report) or f'{name} differs'


def check(current, regenerated):
    """Byte-exact outside the archives; archives compare by content. Returns (ok, report)."""
    if current == regenerated:
        return True, 'guide is up to date'
    def strip(text):
        return DATA_URI.sub('href="data:application/zip;base64,<archive>"', text)
    report = []
    archives_current = [zip_manifest(base64.b64decode(m)) for m in DATA_URI.findall(current)]
    archives_new = [zip_manifest(base64.b64decode(m)) for m in DATA_URI.findall(regenerated)]
    if archives_current != archives_new:
        report.append('embedded example archives differ from docs/site/examples')
    def lines(text):
        # The guide packs whole sections onto one line; diff at tag boundaries instead.
        return re.sub(r'(?<=>)(?=<)', '\n', strip(text)).splitlines()
    if strip(current) != strip(regenerated):
        diff = difflib.unified_diff(lines(current), lines(regenerated), 'committed', 'regenerated', lineterm='', n=1)
        diff_lines = [line if len(line) < 400 else line[:400] + '…' for line in diff]
        report.extend(diff_lines[:80])
        if len(diff_lines) > 80:
            report.append(f'… {len(diff_lines) - 80} more diff lines')
    return not report, '\n'.join(report) if report else 'guide is up to date'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=pathlib.Path, help='framechord executable (default: target/release, then target/debug)')
    parser.add_argument('--check', action='store_true', help='fail when the committed guide differs from the regenerated one')
    parser.add_argument('--html', type=pathlib.Path, default=ROOT / 'docs/site/index.html')
    parser.add_argument('--examples', type=pathlib.Path, default=ROOT / 'docs/site/examples')
    parser.add_argument('--models', type=pathlib.Path, default=ROOT / 'assets/models.json')
    parser.add_argument('--markdown', type=pathlib.Path, default=ROOT / 'docs/site/index.md')
    parser.add_argument('--llms', type=pathlib.Path, default=ROOT / 'docs/site/llms.txt')
    args = parser.parse_args()
    binary = args.binary or default_binary()
    version = crate_version()
    built = binary_version(binary)
    if built != version:
        raise SystemExit(f'{binary} reports {built} but Cargo.toml says {version}; rebuild before generating')
    if not args.examples.is_dir():
        raise SystemExit(f'missing example sources: {args.examples}')
    current = args.html.read_text(encoding='utf-8')
    models = json.loads(args.models.read_text(encoding='utf-8'))
    regenerated = regenerate(current, commands_markdown=export_commands(binary), models=models,
                             examples=args.examples, version=version)
    command_pages = regenerated.count('class="command-entry"')
    outputs = [(args.html, current, regenerated, check(current, regenerated))]
    for path, fresh in [(args.markdown, render_markdown(regenerated, version)),
                        (args.llms, render_llms(regenerated, version, command_pages))]:
        committed = path.read_text(encoding='utf-8') if path.is_file() else ''
        outputs.append((path, committed, fresh, check_text(committed, fresh, path.name)))
    if args.check:
        print('\n'.join(report for _, _, _, (_, report) in outputs))
        raise SystemExit(0 if all(ok for _, _, _, (ok, _) in outputs) else 1)
    written = []
    for path, _, fresh, (ok, _) in outputs:
        if ok:
            continue
        path.write_text(fresh, encoding='utf-8', newline='\n')
        written.append({'path': str(path), 'bytes': len(fresh.encode('utf-8')),
                        'sha256': hashlib.sha256(fresh.encode('utf-8')).hexdigest()})
    if not written:
        print('guide already up to date')
        return
    print(json.dumps({'written': written, 'command_pages': command_pages, 'version': version}))


if __name__ == '__main__':
    main()
