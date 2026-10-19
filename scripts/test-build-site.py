#!/usr/bin/env python3
"""Offline regression for the guide generator: no binary, no network, fixtures only."""
import base64
import io
import pathlib
import runpy
import tempfile
import unittest
import zipfile
from unittest.mock import patch


SCRIPT = pathlib.Path(__file__).with_name('build-site.py')


def fixture_html(entries='', catalog='', version='0.6.2', count=1, date='October 14, 2026', zip_b64='UEsFBgAAAAAAAAAAAAAAAAAAAAAAAA=='):
    return (
        '<html><body><nav><a href="#commands">x</a><a href="#start">y</a></nav>'
        f'<span class="version">v{version}</span><script type="application/ld+json">{{"softwareVersion":"{version}"}}</script>'
        f'<section id="start"><p>The bundled catalog was verified on {date}. More.</p>'
        f'<!-- generated:catalog -->{catalog}<!-- /generated:catalog -->'
        f'<p><a class="button" href="data:application/zip;base64,{zip_b64}" download="framechord-examples.zip">Get</a></p></section>'
        f'<p>This guide targets <strong>FrameChord {version}</strong>. Exact bundled help for all {count} command pages.</p>'
        '<section id="commands"><p id="command-status" role="status" aria-live="polite" class="text-sm mb-4"></p>'
        f'<!-- generated:commands -->{entries}<!-- /generated:commands --></section></div><footer></footer></body></html>\n'
    )


class Generator(unittest.TestCase):
    def setUp(self):
        with patch('subprocess.run', side_effect=AssertionError('no subprocess during import')):
            self.site = runpy.run_path(str(SCRIPT))

    def test_help_pseudo_commands_are_dropped_and_order_is_kept(self):
        markdown = (
            '# framechord 0.6.2 command reference\n\nIntro.\n'
            '\n## framechord\n\n```text\nRoot help\n```\n'
            '\n## framechord help\n\n```text\nPrint help\n```\n'
            '\n## framechord generate\n\n```text\nGenerate help  \n```\n'
            '\n## framechord generate help image\n\n```text\nPseudo\n```\n'
        )
        entries = self.site['parse_commands'](markdown)
        self.assertEqual([path for path, _ in entries], ['framechord', 'framechord generate'])
        self.assertEqual(entries[1][1], 'Generate help\n')

    def test_entry_template_escapes_help_text(self):
        rendered = self.site['render_commands']([('framechord x', "a <b> & 'c' \"d\"\n")])
        self.assertIn('<details class="command-entry"><summary>framechord x</summary>', rendered)
        self.assertIn('<pre><code>a &lt;b&gt; &amp; &#x27;c&#x27; &quot;d&quot;\n</code></pre>', rendered)
        self.assertEqual(rendered.count('class="command-entry"'), 1)

    def test_catalog_lists_current_supported_models_in_guide_provider_order(self):
        models = {'models': [
            {'provider': 'google', 'model': 'g-1', 'media': 'image', 'description': 'Google <one>', 'current': True, 'supported': True},
            {'provider': 'openai', 'model': 'o-1', 'media': 'speech', 'description': 'OpenAI', 'current': True, 'supported': True},
            {'provider': 'openai', 'model': 'o-old', 'media': 'speech', 'description': 'Retired', 'current': False, 'supported': True},
            {'provider': 'fish', 'model': 'f-1', 'media': 'speech', 'description': 'Fish', 'current': True, 'supported': False},
        ]}
        rendered = self.site['render_catalog'](models)
        self.assertEqual(rendered, '<h3>openai</h3><ul><li><code>o-1</code> — speech: OpenAI</li></ul><h3>fish</h3><ul></ul><h3>google</h3><ul><li><code>g-1</code> — image: Google &lt;one&gt;</li></ul><h3>elevenlabs</h3><ul></ul><h3>openrouter</h3><ul></ul>')

    def test_stamps_require_exactly_one_match(self):
        text = fixture_html()
        stamped = self.site['apply_stamps'](text, '0.7.0', 65, 'October 15, 2026')
        self.assertIn('<span class="version">v0.7.0</span>', stamped)
        self.assertIn('FrameChord 0.7.0</strong>', stamped)
        self.assertIn('"softwareVersion":"0.7.0"', stamped)
        self.assertIn('all 65 command pages', stamped)
        self.assertIn('verified on October 15, 2026.', stamped)
        with self.assertRaisesRegex(ValueError, 'version'):
            self.site['apply_stamps'](text.replace('class="version"', 'class="other"'), '0.7.0', 65, 'x')
        with self.assertRaisesRegex(ValueError, 'command pages'):
            self.site['apply_stamps'](text + ' all 3 command pages', '0.7.0', 65, 'x')

    def test_zip_is_compared_by_content_not_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            examples = pathlib.Path(temporary)
            (examples / 'b.yaml').write_text('b: 1\n')
            (examples / 'a.md').write_text('# a\n')
            first = self.site['build_zip'](examples)
            with zipfile.ZipFile(io.BytesIO(first)) as archive:
                self.assertEqual(archive.namelist(), ['examples/a.md', 'examples/b.yaml'])
                self.assertIsNone(archive.testzip())
            stored = io.BytesIO()
            with zipfile.ZipFile(stored, 'w', zipfile.ZIP_STORED) as archive:
                archive.writestr('examples/a.md', '# a\n')
                archive.writestr('examples/b.yaml', 'b: 1\n')
            self.assertNotEqual(first, stored.getvalue())
            self.assertEqual(self.site['zip_manifest'](first), self.site['zip_manifest'](stored.getvalue()))
            (examples / 'a.md').write_text('# changed\n')
            self.assertNotEqual(self.site['zip_manifest'](first), self.site['zip_manifest'](self.site['build_zip'](examples)))

    def test_regenerate_fills_markers_and_check_detects_drift(self):
        markdown = '\n## framechord\n\n```text\nRoot\n```\n\n## framechord help\n\n```text\nH\n```\n'
        models = {'verified_at': '2026-10-15', 'models': []}
        with tempfile.TemporaryDirectory() as temporary:
            examples = pathlib.Path(temporary)
            (examples / 'job.yaml').write_text('schema_version: 2\n')
            regenerated = self.site['regenerate'](fixture_html(), commands_markdown=markdown, models=models, examples=examples, version='0.7.0')
            self.assertIn('<!-- generated:commands --><details class="command-entry"><summary>framechord</summary>', regenerated)
            self.assertEqual(regenerated.count('class="command-entry"'), 1)
            self.assertIn('all 1 command pages', regenerated)
            self.assertIn('<h3>openai</h3><ul></ul>', regenerated)
            self.assertIn('verified on October 15, 2026.', regenerated)
            payload = base64.b64decode(self.site['DATA_URI'].search(regenerated).group(1))
            self.assertEqual(self.site['zip_manifest'](payload), {'examples/job.yaml': b'schema_version: 2\n'})
            ok, report = self.site['check'](regenerated, regenerated)
            self.assertTrue(ok, report)
            ok, report = self.site['check'](regenerated.replace('Root', 'Changed'), regenerated)
            self.assertFalse(ok)
            self.assertIn('Changed', report)
            self.assertNotIn('base64,UEs', report)
            again = self.site['regenerate'](regenerated, commands_markdown=markdown, models=models, examples=examples, version='0.7.0')
            self.assertEqual(again, regenerated)

    def test_missing_or_duplicate_markers_are_refused(self):
        with self.assertRaisesRegex(ValueError, 'generated:commands'):
            self.site['splice'](fixture_html().replace('<!-- generated:commands -->', ''), 'commands', 'x')
        with self.assertRaisesRegex(ValueError, 'generated:commands'):
            self.site['splice'](fixture_html() + '<!-- generated:commands -->', 'commands', 'x')

    def markdown_page(self, main):
        return (
            '<html><head><title>Guide · Test</title><style>.x{}</style></head><body>'
            '<a class="skip" href="#main">Skip</a><aside class="sidebar"><nav>'
            '<a href="#start"><span>01</span>Getting started</a><a href="#ask"><span>✦</span>Ask an agent</a></nav></aside>'
            f'<main id="main">{main}</main><div id="copy-status" class="sr-only">Copied</div>'
            '<script>const x = "<p>not content</p>";</script></body></html>\n'
        )

    def test_markdown_mirrors_the_main_content(self):
        main = (
            '<section class="guide-section" id="start"><div class="section-heading"><span class="eyebrow">01 / SETUP</span>'
            '<h2>Set up once</h2><p>Use <code>init</code> &amp; <strong>preview</strong> with <a href="#ask">an agent</a>.</p></div>'
            '<div class="code-block"><div class="code-bar"><span>Terminal</span><button class="copy">Copy</button></div>'
            '<pre><code><span class="terminal-comment"># first</span>\nframechord init\n</code></pre></div>'
            '<h3><span lang="en">English</span><span lang="es">Español</span></h3>'
            '<div lang="es"><p>Solo en español</p></div>'
            '<div class="table-wrap"><table><caption>Paths</caption><thead><tr><th>Path</th><th>Commit?</th></tr></thead>'
            '<tbody><tr><td><code>a|b</code></td><td>Yes</td></tr></tbody></table></div>'
            '<aside class="note"><strong>Note.</strong> Keys stay in <code>.env</code>.</aside>'
            '<figure><img src="media/a.jpg" alt="An [image]"><figcaption>Made with it</figcaption></figure>'
            '<audio controls src="media/talk.mp3"></audio>'
            '<a class="button" href="data:application/zip;base64,UEsFBg==">Examples</a></section>'
            '<section class="guide-section" id="ask"><div class="section-heading"><h2>Ask an agent</h2>'
            '<p>Delegate the work.</p></div><details class="agent-prompt"><summary>Ask your agent</summary>'
            '<p>Preview first.</p></details>'
            '<details class="command-entry"><summary>framechord init</summary><div class="code-block"><div class="code-bar">'
            '<span>framechord init</span></div><pre><code>Usage: framechord init &lt;PATH&gt;\n</code></pre></div></details></section>'
        )
        markdown = self.site['render_markdown'](self.markdown_page(main), '0.7.0')
        self.assertTrue(markdown.startswith('<!-- Generated by scripts/build-site.py'))
        self.assertIn('# Guide · Test\n', markdown)
        self.assertIn('FrameChord 0.7.0', markdown)
        self.assertIn('## Contents\n\n- [Getting started](#start)\n- [Ask an agent](#ask)', markdown)
        self.assertIn('<a id="start"></a>\n\n01 / SETUP\n\n## Set up once\n\n'
                      'Use `init` & **preview** with [an agent](#ask).', markdown)
        self.assertIn('Terminal:\n\n```\n# first\nframechord init\n```', markdown)
        self.assertIn('### English\n', markdown)
        self.assertNotIn('Español', markdown)
        self.assertNotIn('Solo en', markdown)
        self.assertIn('**Paths**\n\n| Path | Commit? |\n| --- | --- |\n| `a\\|b` | Yes |', markdown)
        self.assertIn('> **Note.** Keys stay in `.env`.', markdown)
        self.assertIn('![An \\[image\\]](media/a.jpg)\n\nMade with it', markdown)
        self.assertIn('[Audio: talk.mp3](media/talk.mp3)', markdown)
        self.assertIn('[Examples](https://github.com/maxiplux/framechord/tree/master/docs/site/examples)', markdown)
        self.assertIn('**Ask your agent**\n\nPreview first.', markdown)
        self.assertIn('### framechord init\n\n```\nUsage: framechord init <PATH>\n```', markdown)
        for absent in ('Copy', 'Skip', 'Copied', 'not content', '.x{}', 'base64'):
            self.assertNotIn(absent, markdown)
        self.assertTrue(markdown.endswith('\n') and not markdown.endswith('\n\n'))

    def test_markdown_lists_indent_nested_blocks(self):
        main = ('<section id="start"><ol class="steps"><li><h3>Install</h3><p>Run it.</p>'
                '<div class="code-block"><pre><code>framechord init</code></pre></div></li>'
                '<li><h3>Ask</h3></li></ol><ul><li>one</li><li>two</li></ul></section><section id="ask"><p>Ask.</p></section>')
        markdown = self.site['render_markdown'](self.markdown_page(main), '0.7.0')
        self.assertIn('1. ### Install\n\n   Run it.\n\n   ```\n   framechord init\n   ```\n\n2. ### Ask', markdown)
        self.assertIn('- one\n- two', markdown)

    def test_markdown_handles_incomplete_blocks(self):
        main = ('<section id="start"><div class="code-block"><div class="code-bar"><button>Copy</button></div>'
                '<pre><code>framechord init</code></pre></div></section><section id="ask"><p>Ask.</p></section>')
        markdown = self.site['render_markdown'](self.markdown_page(main), '0.7.0')
        self.assertIn('<a id="start"></a>\n\n```\nframechord init\n```', markdown)
        broken = '<section id="start"><details class="command-entry"><summary>framechord x</summary></details></section>'
        with self.assertRaisesRegex(ValueError, 'command-entry'):
            self.site['render_markdown'](self.markdown_page(broken + '<section id="ask"><p>Ask.</p></section>'), '0.7.0')

    def test_markdown_refuses_links_without_anchors(self):
        main = '<section id="start"><p><a href="#gone">gone</a></p></section><section id="ask"><p>Ask.</p></section>'
        with self.assertRaisesRegex(ValueError, '#gone'):
            self.site['render_markdown'](self.markdown_page(main), '0.7.0')

    def test_llms_txt_indexes_every_section_of_the_markdown_guide(self):
        main = ('<section id="start"><div class="section-heading"><h2>Set up once</h2><p>Offline init.</p></div></section>'
                '<section id="ask"><div class="section-heading"><h2>Ask an agent</h2><p>Delegate it.</p></div></section>')
        llms = self.site['render_llms'](self.markdown_page(main), '0.7.0', 65)
        self.assertTrue(llms.startswith('# FrameChord\n\n> '))
        self.assertIn('FrameChord 0.7.0', llms)
        self.assertIn('[Full guide in Markdown](https://framechord.com/index.md): every section below and all 65 command pages', llms)
        self.assertIn('- [Getting started](https://framechord.com/index.md#start): Set up once — Offline init.', llms)
        self.assertIn('- [Ask an agent](https://framechord.com/index.md#ask): Delegate it.', llms)
        self.assertIn('## Optional\n', llms)
        self.assertNotIn('KEY=', llms)

    def test_check_text_reports_a_diff(self):
        ok, report = self.site['check_text']('a\nb\n', 'a\nc\n', 'llms.txt')
        self.assertFalse(ok)
        self.assertIn('-b', report)
        self.assertIn('+c', report)
        self.assertEqual(self.site['check_text']('same\n', 'same\n', 'llms.txt'), (True, 'llms.txt is up to date'))

    def test_nav_anchors_must_resolve(self):
        with self.assertRaisesRegex(ValueError, '#missing'):
            self.site['verify_anchors'](fixture_html().replace('href="#start"', 'href="#missing"'))
        with self.assertRaisesRegex(ValueError, '#start'):
            self.site['verify_anchors'](fixture_html().replace('id="start"', 'data-id="start"'))


if __name__ == '__main__':
    unittest.main()
