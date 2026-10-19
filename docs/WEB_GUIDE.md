# Single-file FrameChord guide

FrameChord is the project name from 0.6.1 onward. Earlier evidence below keeps its original MediaAgent commands, release names and artifact paths. Existing `mediaagent/` projects remain supported in place. See [rename and upgrade guidance](../assets/skill/references/renaming.md).

The complete deliverable is `docs/site/index.html`. Open it in a browser or serve
that one file. No other assets, CDN, build step or internet access are required to
render the guide. The only external request it can make is opt-in Google Analytics,
described under [Analytics consent](#analytics-consent).

Two agent-readable files sit beside it and are derived from the finished page, never
edited by hand: `docs/site/index.md`, the guide's `<main>` content as Markdown (English
narration only, media links relative to the site, the example pack linked on GitHub),
and `docs/site/llms.txt`, an [llms.txt](https://llmstxt.org/) index with install
commands, working rules for agents and one link per guide section. The page links both
from its footer, its `#delegate` section and a `<link rel="alternate"
type="text/markdown">` in the head. They are published at
<https://framechord.com/index.md> and <https://framechord.com/llms.txt>.

Three static files sit beside the page for search engines and agents, and are edited by
hand. `robots.txt` allows all crawlers, points to the sitemap and names `llms.txt` and
`index.md`. `sitemap.xml` lists the one canonical page. `favicon.svg` is the brand mark.
The page head declares the canonical URL, robots directives, Open Graph and Twitter card
tags using `media/hero.jpg`, `alternate` links to `index.md` and `llms.txt`, and
schema.org JSON-LD: `WebSite`, `SoftwareApplication` and a `TechArticle` whose encoding
is the Markdown copy. The JSON-LD has no version or date fields, so it needs no update
per release.

The FrameChord guide is published at <https://framechord.com/>. The GitHub Pages
address redirects there; see [domain setup](DOMAIN.md).
The `Publish documentation` GitHub Actions workflow publishes only `docs/site/`
to GitHub Pages after changes to that folder or the workflow on `master`. It can
also be started manually:

```sh
gh workflow run pages.yml --ref master
```

Edit `docs/site/index.html` and push to `master` to update the live guide. Check
the workflow's deployment result before treating an update as published. The hosted
document is the same standalone file used locally. Four regions are generated:

- the command pages between `<!-- generated:commands -->` markers, from the binary's
  own `skills export` reference (clap `help` pseudo-commands are dropped);
- the current-model lists between `<!-- generated:catalog -->` markers, from
  `assets/models.json`;
- the four embedded `framechord-examples.zip` links, from `docs/site/examples/`;
- the version, page-count and catalog-date stamps, from `Cargo.toml` and the catalog.

After those regions, the generator rewrites `index.md` and `llms.txt` from the result.
Their section list comes from the sidebar navigation, and each section's heading and
summary come from its `section-heading`, so a new section needs only a nav link and
that heading block. A Markdown link to an anchor the copy does not contain stops the
generator.

Regenerate them after any CLI, catalog or example change, and verify before pushing:

```sh
cargo build --release --locked
python3 scripts/build-site.py
python3 scripts/build-site.py --check
python3 scripts/test-build-site.py
```

The generator needs Python 3.11 or newer and refuses to run when the built binary's
version differs from `Cargo.toml`. `--check` compares the committed guide with a fresh
regeneration (archives by content, everything else byte for byte), compares `index.md`
and `llms.txt` byte for byte, and is part of the Linux job in `check.yml`. Hand-written
sections, including the cost and integration sections added on October 15, 2026 and
the `#delegate` agent-prompt section added on October 16, 2026, live outside the markers
and are never touched.

The HTML contains:

- Compiled Tailwind CSS 4.3.3 with the guide's component styles in a `<style>` tag.
- Embedded JavaScript for search, navigation and copy buttons.
- Twenty-one sections: the narrated showcase (with the audiobook sample), “Hand it to
  your agent” with thirteen copyable prompts and the case study of the guide's own
  media, nine practical scenarios, and eighteen workflow/reference sections ending with
  66 expandable command help pages.
- Four download links carrying the same embedded ZIP of 41 example files.
- An opt-in Google Analytics consent banner; see [Analytics consent](#analytics-consent).

Example downloads extract into `examples/` in your target project. They retain
relative YAML dependencies. The cast example intentionally requires your own
accepted character references. Credential variable names contain no secret values.

From the repository root, an optional local preview is:

```sh
python3 -m http.server 8765 --bind 127.0.0.1 --directory docs/site
```

Open http://127.0.0.1:8765/ while that process runs. A server is optional for normal
browser use; local-file clipboard restrictions use a selection/copy fallback.

The Tailwind CLI compiled the component rules using `@apply` and utility classes;
there is no runtime Tailwind script. Official build reference:
https://tailwindcss.com/docs/installation/tailwind-cli

Verified after bundling, before the opt-in analytics below existed: no external
scripts, styles, images or file links; unique element IDs and valid internal anchors; embedded ZIP integrity and all 35
files; browser command search and copying; a mobile layout without page overflow.
The 0.3.0 guide and extracted examples pass 22 offline initialization, dry-run and
validation checks. No provider calls were made for this work. Earlier mobile
layout checks apply to the base layout; the updated setup and command sections
were rechecked in a desktop browser.

The scenario section covers article narration, a podcast asset package, a data
infographic, a visual/audio lesson, recurring characters, app visuals and sounds,
a vocal jingle, and bilingual narration. Each has input/output descriptions,
commands, authoring files, an agent prompt and review criteria. The four new batch
manifests passed eight offline validation and dry-run checks. Browser checks
confirmed expandable walkthroughs, command copying and mobile layout.

Version 0.3.0 updates initialization to a resumable keyboard-driven checklist, optional masked credential entry, OpenAI/Fish defaults, local readiness and a reviewed current-stable model policy. Its 63 command pages and 35 downloadable examples match the updated executable and templates. New-model execution uses mocked requests; this documentation update makes no paid provider requests.

Version 0.3.1 documents the executable/configuration folder name conflict, early destination selection, full-path display and `--project` precedence. The command reference is regenerated from the corrected CLI. The example archive remains unchanged.

Version 0.4.0 adds public installer links, the install/update/rollback workflow and two generated command pages. The guide remains a standalone HTML document; the installer scripts are separate downloads used only when explicitly installing software.

The 0.4.0 public guide and both installer downloads were fetched from GitHub Pages and matched byte-for-byte with the merged source on October 15, 2026. Public native installation, self-update and offline rollback passed on all four supported release targets; see [the recorded results](STATUS.md#user-installation-and-updates-040).

Version 0.5.1 updates the setup walkthrough and generated command help for Google-first defaults, the explicit OpenAI preset, built-in speech voices, target-project dotenv creation/update and default installation of Codex/Claude project skills. Examples that name other providers remain explicit provider choices and require their corresponding credentials.


Version 0.6.1 renames the guide, all 65 generated command pages, installers and
all four embedded example archives to FrameChord. The upgrade note distinguishes
new `framechord/` projects from preserved legacy `mediaagent/` directories.
The custom domain's registration is separate from DNS and HTTPS deployment.

Version 0.6.2 refreshes generated command help for optional release metadata
authentication. The guide is bound to `https://framechord.com/` with an approved
apex/www certificate and enforced HTTPS. Local negative DNS caches can take longer
to refresh than the accepted authoritative records. README workflows now cover
every supported media type, exact input files, character reuse, agent operation,
parallel batches and review/recovery; ten generation/validation examples pass
offline, without paid calls.

The 0.6.2 guide and both installers match the merged source byte-for-byte over
valid HTTPS at the custom domain. The `www` host redirects to the apex. This local
check used an independently verified public GitHub Pages address to bypass the
network resolver's cached earlier NXDOMAIN, without disabling TLS verification.
Fresh-runner public acceptance results are tracked in STATUS.md.

After propagation, the local resolver also returns all four expected apex
addresses. Public release 0.6.2 installation, update and offline rollback pass on
all three supported native platforms, including the Pages-to-custom-domain
installer redirect. See [release evidence](STATUS.md#authenticated-release-metadata-062).

## Analytics consent

Since October 16, 2026 the guide can use Google Analytics (measurement ID
`G-5QFTHWRP9W`), but only after a visitor clicks **Allow analytics** in the consent
banner. Until then the page requests nothing from Google and writes no analytics
cookie. The Consent Mode default (analytics, ad storage, ad user data and ad
personalization all denied) is only queued in `window.dataLayer`.

- **Allow** sends a consent update granting analytics storage only, then loads
  `https://www.googletagmanager.com/gtag/js`. That script is the guide's one external
  script, and it records a page view on load. Hash navigation between sections is not
  tracked.
- **No thanks**, or withdrawing later from **Analytics preferences** in the footer, sets
  Google's disable flag, sends a denied update and deletes the `_ga` cookies. A
  `storage` listener applies the change in every open tab of the site.
- The choice is kept in `localStorage` under `framechord-analytics-consent`. Without
  storage, the banner asks again on the next visit.

Offline copies and consumers with a strict Content Security Policy keep working: if the
script is blocked or there is no network, only analytics is lost. To remove analytics,
delete the `#consent` banner, its footer button and the consent script at the end of
`index.html`. The footer control is a button, which the Markdown converter skips, so
`index.md` and `llms.txt` contain no analytics text.
