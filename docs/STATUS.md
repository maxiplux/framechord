# Implementation status

FrameChord is the project name from 0.6.1 onward. Earlier evidence below keeps its original MediaAgent commands, release names and artifact paths. Existing `mediaagent/` projects remain supported in place. See [rename and upgrade guidance](../assets/skill/references/renaming.md).

Development snapshot, October 15, 2026. The first-version command families are implemented within the provider boundaries below. Visual character consistency has failed user acceptance and must not be described as complete. Native validation is separate from media-quality acceptance; public binary availability is recorded on the [Releases page](https://github.com/maxiplux/agentmedia/releases/latest).

Implemented: Rust package/library/CLI, versioned contracts and schemas, offline preserving initialization, embedded skills/templates and explicit bundled-file refresh, portable character import/export and reference acceptance, direct/job/batch validation and explicit job overrides, infographic compilation, text planning with contained validated authoring bundles, concurrent durable runs, dependencies, finite limits, verified cache, submission idempotency, worker startup/loss handling, cancellation/resume, native audio assembly, review records, durable image AI review receipts/reuse/retry history, ElevenLabs descriptive voice previews and explicit saved-voice creation/binding/verification refresh, child revisions, targeted podcast revisions, audiobooks by chapter or full book (manuscript splitting, WAV/MP3 with ID3 chapters, chapter index and cue sheet, whole-book budget preflight, chapter subsets and chapter revisions), consent-gated recorded-voice cloning, portable exact-transcript voice samples, Fish binary reference synthesis, Google audio AI review/retrieval, Google generation reconciliation, bounded paginated/filterable voice discovery, a terminal configuration wizard, and explicit user host concurrency controls. Reference media is decoded before acceptance or generation. Failed cached publication preserves independent jobs.

| Provider | Adapter paths implemented | Selected live verification |
| --- | --- | --- |
| OpenAI | PNG generation/editing, saved/built-in voice speech, text planning, image review | Image generation/editing, built-in speech, job planning, image review passed |
| Google | Interactions image/reference generation, speech, Lyria music, text planning, filterable paginated prebuilt/custom voice catalog, audio review, saved-interaction retrieval | Image generation and JPEG recovery, speech, instrumental clip, filtered catalog, short audio review passed; retrieval has mocked coverage |
| Fish Audio | Speech with explicit saved voice binding/model header or authorized binary reference samples, filterable paginated owned/licensed public catalogs | Licensed public voice speech and two-page filtered public catalog passed; inline samples have mocked coverage |
| OpenRouter | PNG image generation, text planning, image review | Image generation passed |
| ElevenLabs | Saved voice speech, instrumental/vocal music, sound effects, filterable paginated account voice catalog, descriptive preview design, explicit saved-voice creation/binding and verification lookup, consent-gated recorded cloning | Premade voice speech, short instrumental music, sound effect, catalog, three preview candidates passed; creation/binding/cloning tested with mocks only; two-page filtered catalog passed |

See [live evidence](LIVE_TESTS.md) for exact models, runs, decoded durations, one earlier rejected placeholder voice request, and untested paths. Live tests prove selected API/file handling, not human-perceived quality, exact script pronunciation, character consistency, or all model variants. No saved provider voice was created or cloned, and no candidates were accepted automatically. The descriptive preview test returned three candidates; provider-side creation/binding/verification refresh has offline coverage, with creation/binding untested live.

Ordinary tests use mocked adapters/transports and small local fixtures. They verify contracts, concurrency, limits, identity resolution, authoring validation, audio assembly, cache, revisions, and recovery without paid calls even when credentials exist. The baseline had 100 passing offline tests on macOS; remediation adds independent contract/recovery/storage tests. See the remediation evidence below for the current count. Historical local packages passed their isolated offline checks, including initialization and embedded skill export without the checkout; historical 0.4.0 and current 0.5.1 release evidence is below. See [packaging](PACKAGING.md). Native CI covers Linux, macOS and Windows. See the installation and release notes below for exact versioned results.

Verification still requiring external inputs: selected authorized speaker recordings for live cloning/reference synthesis, human listening and repeated character scenes. These do not become passed merely because implementation or mock tests succeeded.

Provider boundaries: voice preview design and saved-voice creation/cloning are implemented for ElevenLabs; other providers can reuse their accessible bindings. Fish supports stateless synthesis from authorized recordings with exact transcripts. Google supports audio review with a 20 MiB inline request ceiling; larger jobs can review individual clips. Google Interactions has retrieval by saved ID; the other adapter endpoints do not expose an implemented media-result retrieval path, so uncertain calls require account reconciliation before explicit retry. Provider professional/biometric verification must be completed by the speaker through the provider's supported account flow. No placeholder verification submission or video adapter is advertised. Public binary installation, updates and release publishing are implemented in 0.4.0; signing and notarization remain unverified.

The opt-in smoke script makes at most one bounded request per invocation and reports unavailable credentials as skipped. Image identity, voice quality, vocal lyrics, and infographic typography/data accuracy still require review using repeated scenes/clips and the exact intended source content.

Official implementation references: [OpenAI image API](https://developers.openai.com/api/docs/guides/image-generation), [OpenAI speech](https://developers.openai.com/api/reference/resources/audio/subresources/speech/methods/create), [Google images](https://ai.google.dev/gemini-api/docs/image-generation), [Google speech](https://ai.google.dev/gemini-api/docs/speech-generation), [Google music](https://ai.google.dev/gemini-api/docs/music-generation), [Fish TTS](https://docs.fish.audio/api-reference/endpoint/openapi-v1/text-to-speech), [OpenRouter images](https://openrouter.ai/docs/guides/overview/multimodal/image-generation), [ElevenLabs speech](https://elevenlabs.io/docs/api-reference/text-to-speech/convert), [music](https://elevenlabs.io/docs/api-reference/music/compose), [effects](https://elevenlabs.io/docs/api-reference/text-to-sound-effects/convert), [recorded cloning](https://elevenlabs.io/docs/api-reference/voices/ivc/create), [Google audio](https://ai.google.dev/gemini-api/docs/audio), [background retrieval](https://ai.google.dev/gemini-api/docs/background-execution), and [ElevenLabs speech limits](https://elevenlabs.io/docs/overview/models).


The bundled skill includes every implemented command/flag generated from Clap,
all current schemas, denied-permission consent scaffolds, clone/sample templates,
configuration/concurrency guidance, and recovery/review procedures. Project skills
are refreshed explicitly; existing credentials and unrelated assets are preserved.


Character-consistency correction: the first paired forest scene was rejected by
the user for changed identities and is recorded as `needs_revision`. The CLI now
supports explicit identity-reference roles in image jobs and repeated
`--identity-reference` flags. On supported OpenAI models it records and sends high
input fidelity, rejects low-fidelity identity requests before submission, and keeps
reference labels/order in the effective snapshot. Offline tests validate those
controls and the actual multipart body. These controls are prerequisites, not proof
that the generated face/body matches. Corrected visual output still needs user
acceptance; model/provider changes remain explicit.


Original-model continuity is now enforced before submission. Local original
receipts and declared portable origins pin identity references to their source
provider/model and carry compatible settings; conflicting model overrides fail.
Initialization supports complete config files or offline terminal setup, including
models, settings, voices, locales, consistency policy, and limits, before enabling
the initial persona profiles. New template and skill guidance cover these fields.
These changes have offline regression coverage; no additional paid generation was
performed to verify them. The earlier explicit alternative-model comparison is a
historical candidate, not evidence that changing the original model satisfies this
contract. Visual likeness still needs user acceptance.


Read-only API diagnostics are implemented through `doctor --live`, with provider
selection, bounded parallel GET checks, timeouts, secret/account-data redaction,
structured diagnostics schema, and actionable failure classes. Default doctor stays
offline. A bounded live authentication check passed for all five provider endpoints
with HTTP 200; media generation/model access and billing readiness were not tested
by this check. See [connection checks](CONNECTIONS.md) and [live evidence](LIVE_TESTS.md).


## Remediation implementation (0.2.0)

Implemented: v2 job/config/run readers with v1 compatibility and explicit backed-up
migration; ordered casts with canonical references and pinned-model conflict
rejection; selected character/asset planner bundles with immutable reserved inputs;
preview-hash-bound verification suites; read-only bounded model catalogs; user-attested
recovered-response attachment; usage reports including separate voice/review operations;
hash-addressed reference blobs, lightweight metadata indexes and conservative orphan
cleanup; optional podcast pauses/gain/EBU R128 normalization; reference comparison
reports, identity templates and complete generated skill command/schema documentation.

The project now has a local baseline commit and explicit generation defaults.
No remote or publication was added. The initial defaults are previously tested
selections, not a new assertion of current account access.

Prepared acceptance inputs are under `mediaagent/verification/`. The original-model
reference-sheet preview resolves two requests; complete-media acceptance resolves
13 requests; designed-voice preview is a separate one-request operation. The approved
two-image reference-sheet batch has now run successfully, one attempt per image;
both outputs were marked `needs_revision` after agent visual comparison for identity
drift and cropping. Human acceptance remains pending. Other paid remediation batches
remain unexecuted. Repeated scenes correctly remain blocked until references
are selected; saved-voice reuse remains blocked until explicit creation/binding.
Cloning/reference synthesis still needs authorized recordings and consent evidence.

Fresh Codex/Claude automatic-discovery sessions remain pending. Windows native execution is covered by the current CI evidence below.
Generic process-caller and isolated package tests are separate from those gates.
See [acceptance](ACCEPTANCE.md) and [remediation workflows](REMEDIATION.md).


Remediation local evidence: 119 offline tests pass on macOS, with formatting and
strict all-target/all-feature Clippy checks passing. The 0.2.0 Apple Silicon package
passes 12 isolated offline checks. No paid requests were made during implementation.
Subsequent approved character acceptance testing made two image requests, documented
in [acceptance](ACCEPTANCE.md). No provider voice creation, cloned voices, or canonical
reference acceptance were performed.


Final Linux x86-64 verification also passes 119 offline tests, formatting, strict
Clippy and 12 isolated package checks, executing under Docker Desktop emulation.
The verified source manifest matches the current Rust, test, asset and packaging
sources. Mac and Linux 0.2.0 archives/checksums are under `target/package-dist/`;
`target/remediation-verification.json` records their exact hashes. Windows was pending in that historical local snapshot; current native results follow. Neither historical local archive was publicly published or signed.

## Guided setup and current stable models (0.3.0)

The terminal initializer now loops through a searchable, colored checklist, supports explicit draft saving/resumption and masks missing project credential entry. Its default preset selects OpenAI GPT Image 2.5 Flare for images/infographics, Fish s2.1-pro for speech/podcasts and OpenAI GPT-6 Luna for planning. Optional features remain disabled until selected. `config status --json` shares the same local readiness checks; initialization remains offline.

Only reviewed current stable model families may submit new provider requests, including retries and side operations. Historical configuration, model locks, receipts, cached results and response recovery remain readable. Existing retired-model identities need an explicit new character version with human-accepted references. The bundled catalog has dated official-source metadata; live discovery is bounded and cannot authorize execution. GPT Image 2.5 request translation has mocked coverage, including exact model IDs/snapshots, references and supported size/quality settings; paid live acceptance was not performed for this change.

Local macOS formatting, strict Clippy and 140 offline tests pass, including real terminal navigation, masked credentials, draft/resume, cancellation and current-model policy tests. All 140 offline tests, formatting and strict Clippy also pass on native Windows, Linux and macOS runners: [Rust checks](https://github.com/maxiplux/agentmedia/actions/runs/38670199603). Each platform archive passes 13 isolated offline workflows: [native packages and downloadable artifacts](https://github.com/maxiplux/agentmedia/actions/runs/38670199579). Downloaded archive/executable hashes were verified, including the Windows executable’s 8 MiB stack reserve. These runs tested code revision `3395a58`; subsequent changes only update documentation. No claim of human identity/quality acceptance is added.

## Init destination correction (0.3.1)

Reproduced `File exists (os error 17)` when initialization runs beside a downloaded executable named `mediaagent`: its path conflicts with the required `mediaagent/` configuration directory. Setup now checks the destination before collecting choices or writing files and offers another project folder in a terminal. Noninteractive callers receive `init_path_conflict` with corrective arguments. The executable and existing credentials are preserved. `--project PATH init` now selects the destination when the positional path is omitted; an explicit `init PATH` takes precedence. Setup displays the full destination before the checklist and save review.

Formatting, strict Clippy and all 144 offline tests pass on native Windows, Linux and macOS runners: [CI results](https://github.com/maxiplux/agentmedia/actions/runs/38673081222). This includes a real terminal regression that redirects from the colliding folder and saves an unfinished project with both skills. The local macOS package passes 14 isolated checks, including the downloaded-executable collision. The [native package workflow and downloads](https://github.com/maxiplux/agentmedia/actions/runs/38673081221) record each platform’s package checks. These runs test code revision `5318faa`; subsequent changes only update documentation. No paid provider calls were made.


## User installation and updates (0.4.0)

Added offline `install`, read-only `update --check`, stable public-release updates, exact versions, explicit downgrades and offline rollback. Installation uses user-local directories, preserves unrelated executables unless replacement is explicit, and optionally configures user PATH. Updates verify GitHub's asset digest, size and executable version before replacement, keep a previous executable, and preserve project state and skills. Unix shell profile changes preserve existing text; Windows uses the user PATH rather than machine settings. No provider keys or paid calls are used.

Public shell/PowerShell installers select the native target, pin one stable release, verify SHA-256 and version, and call the binary's installer. CI adds native Intel Mac coverage. Tagged release publication validates all native archives and raw executables against the exact clean source commit, uploads a draft, verifies GitHub's stored digests, then publishes it as latest. Checksums are not code signing or notarization.

Version [0.4.0](https://github.com/maxiplux/agentmedia/releases/tag/v0.4.0) is publicly published from `bfe6c3303dd18bed5935edc794663a0dd8075337` with 20 assets. Formatting, strict Clippy and 157 offline Rust tests pass; each of the four native packages passes 16 isolated checks. The auxiliary ignored test entry is invoked in isolated subprocesses by the update/rollback tests. [Native pre-merge checks](https://github.com/maxiplux/agentmedia/actions/runs/38711974447) and the [tagged release and public acceptance workflow](https://github.com/maxiplux/agentmedia/actions/runs/38713232173) passed on macOS Apple Silicon, macOS Intel, Linux x86-64 and Windows x86-64 on October 15, 2026.

Public acceptance installed through the hosted installer, checked the published release, performed checksum-verified self-reinstallation, restored the offline rollback, and preserved project fixtures on all four platforms. An additional local Apple Silicon check verified default latest-version installation and `update --check`. The hosted HTML and both installer scripts match the merged source. No media-provider calls were made and character/media quality acceptance is unchanged.

The first publication stopped safely while trying to read a draft through GitHub's published-only by-tag endpoint. All 20 draft assets were independently checked against the native release manifest and GitHub's stored digests before publication; rerunning the workflow verified the immutable published assets and passed all four public acceptance jobs. A subsequent source-only publisher correction uses the numeric release endpoint for drafts, with an offline regression covering digest rejection, draft resume and refusal to overwrite published assets. Future tags run that regression before publishing. This correction changes release tooling rather than the 0.4.0 executables. See [installation](INSTALLATION.md).

## Google-first project setup (0.5.1)

New projects use Google Nano Banana 2.1 for images/infographics, Gemini 3.8 Flash TTS with built-in `Kore` for speech/podcasts, Lyria 3.5 for music, and Gemini 3.8 Flash for planning. These stable defaults were checked against official Google references on October 15, 2026; their sources and review date are recorded in `assets/models.json`. `--preset google` and `--preset recommended` select the same defaults. The second terminal welcome option and `--preset openai` explicitly select OpenAI images, speech/podcasts with built-in `coral`, and planning. Music/effects stay disabled in that alternative. Standalone sound effects remain optional through ElevenLabs. Provider preference never authorizes automatic fallback, unsupported generation or changes to existing character provenance.

Init reads the selected project's `.env`, creates it when absent and appends missing provider key names with empty values. Existing secrets, comments, exports, newline conventions and unrelated content are preserved. It does not copy keys from the process environment, executable folder or ancestor projects. Existing environment precedence remains intact. Invalid dotenv syntax stops noninteractive initialization before configuration or skill writes, without showing parser snippets. Both Codex and Claude project skills are installed by default; modified installed skills and existing configurations/characters are preserved. The result identifies the credential file and available project skill directories.

Local formatting, strict Clippy and 165 offline Rust tests pass. New checks exercise single-key readiness, Google/OpenAI selection in real terminals, inherited built-in voice bindings, target-folder dotenv ownership, preserving partial credential files, empty entries and CRLF/comments, process/ancestor key isolation, secret-free output and invalid-file rejection. Linked credential files remain linked; required changes are refused with a clear action rather than detaching the link. Shared setup locking prevents a waiting initializer from overwriting credentials saved by another setup process. Ordinary checks make no provider calls. Version [0.5.1](https://github.com/maxiplux/agentmedia/releases/tag/v0.5.1) is published from `3464b8831b1a02689891db4bdd86288538427dcf` with 20 original assets. All native builds passed 165 offline Rust tests and 16 package checks. Public Windows and Linux acceptance passed; the initial Apple Silicon CI acceptance failed at the read-only update check, with its diagnostic hidden by the harness. A local public Apple Silicon install/check/reinstall/rollback passed, as did a real 0.4.0 → 0.5.1 upgrade and offline rollback preserving existing configuration, identity, dotenv contents and installed skills. The upgraded binary initialized a new Google project with both skills and single-key offline readiness. CI diagnosis is tracked below; local success does not mark the failed CI job as passed. Media quality and character consistency acceptance remain unchanged.

The initial v0.5.0 tag was not published: an additional Windows check exposed a worker completing between the launcher’s state read and child-exit check, incorrectly reporting worker_start. Version 0.5.1 reloads the recorded state before declaring startup failure, with a deterministic receipt-preservation regression. Actual startup failures still require inspecting the saved run; no requests are automatically resubmitted.


## Supported release platforms and acceptance diagnostics

Current build, packaging, publication and public acceptance matrices target only
Windows x86-64, Linux x86-64 and macOS Apple Silicon (ARM64). Intel Mac builds are
removed from active CI and the hosted installer; historical immutable releases
retain their original assets. New releases contain 16 assets across three native
targets, including installers, checksums and the source/hash manifest.

Public acceptance can be rerun independently through the `Public release
acceptance` workflow with an explicit published version. Updater nonzero exits
now expose their structured stdout and stderr; timeout failures remain bounded
and are not automatically retried. Three offline diagnostic regressions verify
exit failures, structured failures and timeouts. The three-target publisher's
offline integrity/idempotency regression and native bootstrap checks pass locally.
The shell installer also rejects Intel Macs before downloading. No media-provider
requests are made by these checks.


[Independent public acceptance for 0.5.1](https://github.com/maxiplux/agentmedia/actions/runs/38722360350)
passes on Windows x86-64, Linux x86-64 and Apple Silicon macOS from tooling commit
`6342d7198d6511e83a6e0fd4b59ef42813a89ef5`. Each job verifies a public
installation, read-only update check, checksum-verified forced reinstallation,
offline rollback and preservation of project fixtures. The original failed job's
underlying diagnostic cannot be recovered because its harness discarded captured
output; successful independent checks do not establish the original cause. The
new harness records failures without hiding or automatically retrying them.
All 20 immutable 0.5.1 release assets agree with the original manifest and
GitHub's stored hashes/sizes. The hosted guide and both installer scripts match
current master; the hosted shell installer supports Apple Silicon and Linux
x86-64. Future releases publish only the three supported native targets.


[Three-platform Rust checks](https://github.com/maxiplux/agentmedia/actions/runs/38722356648)
pass formatting, strict all-target/all-feature Clippy and 165 offline Rust tests
on Windows x86-64, Linux x86-64 and Apple Silicon macOS for the platform-tooling
commit. The independent public acceptance and hosted documentation deployment
are also green. These software checks do not imply media-quality acceptance.


[Three-platform native packages](https://github.com/maxiplux/agentmedia/actions/runs/38722363095)
also pass on all three supported platforms, with 16 isolated checks per package.
The downloaded packages from the exact clean tooling commit pass the publisher's
dry run as a complete 16-asset, three-target release set. This validates future
publication without changing the immutable 0.5.1 release or creating another tag.


## FrameChord rename (0.6.1)

The package, executable and bundled skill are renamed to `framechord`; new
projects initialize `framechord/`. Existing `mediaagent/` project directories
remain supported in place, including original configuration, jobs, character
references, reference blobs and run receipts. Initialization returns the actual
`data_directory` and `legacy_layout`; roots containing both directories are
rejected before selecting or modifying a project. Old installed skills are
preserved; renamed project skills install in their own directories. The shared
host concurrency lock namespace is retained to coordinate old and new binaries.

Release tooling, three-platform native builds, installers, self-updates,
maintained skill copies, templates, command help and downloadable examples use
the FrameChord name. Older executables require an explicit new installation;
`mediaagent update` does not rename the product. Historical release bytes,
acceptance evidence and authored project files are preserved.

Local formatting, strict Clippy and 171 offline Rust tests pass, including
legacy initialization/configuration/skill preservation, unchanged readable run
receipts and rejection of ambiguous project roots. Native release and public
installation acceptance are recorded after they execute. No paid provider calls
are made by rename validation. The owned `framechord.com` domain requires
separate DNS/HTTPS verification; see [domain setup](DOMAIN.md).


The v0.6.0 source tag was stopped before publication after rename review found
that an existing user-level host ceiling would be bypassed by the new settings
path. No 0.6.0 release assets were published. Version 0.6.1 retains existing
`~/.config/mediaagent/host.yaml`, rejects conflicting old/new host settings and
keeps the shared slot locks; two offline regressions cover preservation and
conflict refusal.


FrameChord [0.6.1](https://github.com/maxiplux/framechord/releases/tag/v0.6.1) is
publicly published from `8280ea0edd22617f67886c0a176ee6b50aabaf2b`. Its 16 assets
agree with the source manifest and GitHub's stored hashes/sizes; no Intel Mac
asset is included. The [release workflow](https://github.com/maxiplux/framechord/actions/runs/38735561872)
passes native checks, 17 package checks per target, publication and public
installation/update/reinstallation/offline rollback on all three platforms.
A local real public-download transition from MediaAgent 0.5.1 to FrameChord
0.6.1 preserves legacy configuration, exact persona content, dotenv bytes, a
modified old skill, the old executable and its installation receipt; new project
skills and FrameChord update/rollback also pass. No provider calls were made.

## Authenticated release metadata (0.6.2)

A further [post-domain acceptance run](https://github.com/maxiplux/framechord/actions/runs/38737600285)
exposed HTTP 403 from a runner's anonymous GitHub API update check. FrameChord now
supports optional process-only `GH_TOKEN` / `GITHUB_TOKEN`; CI supplies its built-in
read-only token to avoid depending on the shared anonymous quota. Authorization
is scoped to this repository's GitHub API metadata URLs and marked sensitive;
release downloads and media providers receive no GitHub credential. Invalid header
characters fail before sending a request, with no secret value in the error.
An exhausted quota has a specific actionable error. Public anonymous operation
remains available, with no automatic update retries or project dotenv changes.
Three offline regressions verify token precedence/redaction, malformed credential
rejection and origin/path scoping. Release results follow execution.

Cloudflare's four apex A records and www CNAME are DNS-only. GitHub's domain check
accepts both hosts, the certificate covers both, Enforce HTTPS is enabled and a
TLS-verified origin request serves the exact guide. Earlier negative DNS answers
remain cached by the local network resolver; independent checks distinguish that
propagation delay from the accepted records and certificate.

Local 0.6.2 validation passes formatting, strict Clippy and 174 offline Rust
tests. Ten README generation/validation examples pass offline with isolated
Google defaults, explicit OpenAI narration and ElevenLabs effects; no provider
calls are made. All 65 website command pages match the generated reference.
The publisher and acceptance diagnostic regressions also pass offline.

FrameChord [0.6.2](https://github.com/maxiplux/framechord/releases/tag/v0.6.2)
is publicly published from `751ae425255a6bd74a2fba4cfd1d14904e23cbd4`. The
[release workflow](https://github.com/maxiplux/framechord/actions/runs/38739260296)
passes native format/Clippy/tests, 17 isolated package checks per target, verified
publication and public installer/update/reinstallation/offline rollback on Windows
x86-64, Linux x86-64 and macOS Apple Silicon. Both the
[master checks](https://github.com/maxiplux/framechord/actions/runs/38739257958) and
[tag checks](https://github.com/maxiplux/framechord/actions/runs/38739260073) pass.
All 16 public assets match the manifest commit and GitHub's stored sizes/digests;
no Intel Mac asset is included. The same real public-download legacy transition
to 0.6.2 passes locally, preserving exact configuration, persona, dotenv, modified
old skill, old executable and installation receipt, while installing the new
skills and verifying FrameChord update/offline rollback. No paid provider calls
were made.

The [Pages deployment](https://github.com/maxiplux/framechord/actions/runs/38739258045)
passes. The custom-domain guide and both installers match merged source over
verified HTTPS, `www` redirects to the apex, and the local DNS cache has now
refreshed successfully. Fresh public acceptance runners also followed the hosted
installer path successfully on all three supported platforms.

The standard public acceptance script also passes locally for 0.6.2 using normal
DNS and HTTPS (no address override): public installer, read-only update check,
verified reinstall, offline rollback and project-file preservation. Its isolated
environment excludes media-provider keys and makes zero provider calls.

## Cost estimates, agent targets and site generator (0.7.0)

October 15, 2026. Three implementation changes back the new cost and integration
documentation. The release evidence follows the local validation below.

- `usage --json` now reports labelled list-price estimates beside provider-reported
  units: `cost_estimate`, `cost_status` (`estimated`, `provider_reported`,
  `cached_no_charge`, `not_estimable`; operations and local podcast and audiobook assembly jobs
  `not_estimated`), `cost_note`,
  `estimated_total_usd` and an `estimate_summary`. Rates come from the bundled
  `assets/pricing.json`, verified against the official Google, OpenAI and ElevenLabs
  pricing pages on October 15, 2026, or from an explicit validated `--price-table`.
  Provider-returned `cost` is never replaced and `currency_total` stays null.
  `models --json` carries the matching `pricing` entries. See [cost](COST.md).
- Skill install targets `cursor`, `gemini`, `opencode`, `copilot` and `none` join
  `codex` and `claude`, with discovery paths verified against each vendor's
  documentation on the same date. `config status --json` reports `skills`,
  `skill_targets` and `skills_shared_by`; the wizard gained a per-agent toggle menu.
  See [integration](INTEGRATION.md).
- `scripts/build-site.py` regenerates the guide's command pages, model catalog,
  example archives and version stamps from the binary, `assets/models.json` and the
  checked-in `docs/site/examples/`; `--check` runs in the Linux CI job. The guide gained
  hand-written cost and integration sections and a Google-default hero.

Local validation on macOS Apple Silicon: formatting, strict Clippy, 196 passing
offline Rust tests (one ignored platform test), 8 generator unit tests,
`build-site.py --check` clean against the regenerated 0.7.0 guide (65 command pages,
18 sections), and `package-release.py --offline` with 18 isolated native checks
including the new native-target and estimate assertions. The guide was opened locally
in a browser at desktop and phone widths with no console errors or horizontal overflow.
No provider calls were made.

The changes merged to master through [pull request #6](https://github.com/maxiplux/framechord/pull/6)
(`14068fb`). Its [Windows Rust checks](https://github.com/maxiplux/framechord/actions/runs/38786018023)
failed in one new test, `status_reports_shared_discovery_for_every_known_agent`, which
compared a native skill path joined as a single `".cursor/skills/framechord"` string
against the report's `root.join(dir).join("framechord")` construction; Windows
serializes the two with different separators. Linux and macOS passed. The test-only
alignment `7fde6338cb51d19297407ee22ec42c3e92c5f115` uses the same join order, and the
[master checks](https://github.com/maxiplux/framechord/actions/runs/38787016204) on that
commit pass on Windows x86-64, Linux x86-64 and macOS Apple Silicon. No product code
changed; on Windows the reported native paths keep mixed separators (for example
`...\.cursor/skills\framechord`), which resolve correctly.

FrameChord [0.7.0](https://github.com/maxiplux/framechord/releases/tag/v0.7.0) is
publicly published from `7fde6338cb51d19297407ee22ec42c3e92c5f115`. The
[release workflow](https://github.com/maxiplux/framechord/actions/runs/38787795161)
passes native format/Clippy/tests (196 offline tests on Linux and macOS, 195 on
Windows, one ignored on each), 18 isolated package checks per target, verified
publication of 16 assets and public installer/update/reinstallation/offline rollback on
Windows x86-64, Linux x86-64 and macOS Apple Silicon. The
[tag checks](https://github.com/maxiplux/framechord/actions/runs/38787794745) also pass.
The 15 assets listed in the release manifest match GitHub's stored sizes and digests,
the manifest records the tag commit, `latest/download/version.txt` serves `0.7.0`, and
no Intel Mac asset is included. The
[independent public acceptance run](https://github.com/maxiplux/framechord/actions/runs/38792124986)
for `v0.7.0` passes on all three platforms, each reporting a successful public
installation, read-only update check, checksum-verified reinstallation, offline
rollback and project-file preservation with zero paid provider calls.

The [Pages deployment](https://github.com/maxiplux/framechord/actions/runs/38786017995)
for the merge passes: <https://framechord.com/> serves the guide with the `v0.7.0`
header, the hosted guide and both installers byte-match `docs/site` on master, and
`www` redirects to the apex. On this Mac, `curl -fsSL https://framechord.com/install.sh | sh`
(into an isolated `--bin-dir` with `--no-modify-path`) installed `framechord 0.7.0`
whose SHA-256 equals the published Apple Silicon sidecar. The standard public
acceptance script also passes locally for 0.7.0 with normal DNS and HTTPS: public
installer, read-only update check, verified reinstall, offline rollback and
project-file preservation, with zero provider calls. No paid provider calls were made
at any point of this release.

## Guide media made with FrameChord

October 15, 2026. The guide now carries a hero illustration, eight section
illustrations, four infographics, a six-slide English/Spanish narrated tour, a
two-voice podcast and a music bed, all generated by the installed `framechord 0.7.0`
from the versioned jobs in `docs/site/media/jobs/` and re-encoded for the web
(JPEG, MP3). The bounded live batch, its cache reuse, AI audio reviews and usage are
recorded in [live evidence](LIVE_TESTS.md#site-showcase-batch-070); per-asset hashes are
in `docs/site/media/provenance.json`. The new `#showcase` section brings the guide from
18 to 19 sections; the 0.7.0 entry above records the guide as it was at that release.
`build-site.py --check` and the generator tests pass, and the page was checked locally
at desktop and phone widths with no console errors. Human review of media quality is
still pending.

## Agent delegation guide and agent-readable docs

October 16, 2026. The guide gains a hand-written `#delegate` section, "Hand it to your
agent", placed after the showcase. It covers installing the project skill, a free
readiness prompt, the author/check/generate/report loop, twelve copyable prompts by use
case, and a case study. The case study quotes, lightly edited, the request that
produced the guide's media and lists the steps recorded in
[live evidence](LIVE_TESTS.md#site-showcase-batch-070). The prompts are documentation:
none was run as part of this change, and no paid call was made. The guide now has 20
sections.

`scripts/build-site.py` now also derives `docs/site/index.md` (the guide's `<main>`
content as Markdown) and `docs/site/llms.txt` (an llms.txt index of install commands,
agent rules and one link per section) from the finished page. `--check` covers both
files, and the offline generator tests cover the converter. The README and the
integration guide link the new section and both files. The page was checked locally at
desktop and phone widths: copy buttons and search work, there is no horizontal scroll,
and there are no console errors. Publication to framechord.com follows the normal
`master` deployment.

## Audiobooks

October 19, 2026. A request for an audiobook previously produced a podcast, because
the only long-form choices were single-text speech and multi-speaker podcasts, and the
skill pointed article adaptations at podcasts. `media: audiobook` is now a separate
job type: one narrator reads exact text from a Markdown manuscript split at `#`
headings or from an explicit chapter list. It publishes one WAV or MP3 per chapter and,
in `full` mode, the whole book with ID3 chapter frames, a `chapters.json` index and a
`.cue` sheet. `validate` shows the chapter table and budget fit; a whole book is
checked against the run limits before any run exists; `generate --chapter` produces a
subset; `revise --chapter` re-reads one chapter and reassembles the book. Chapters and
the book are assembled by streaming, off the scheduler thread. The skill now routes
book and read-aloud requests to audiobooks and keeps podcasts for conversations, and
`plan --source` copies a manuscript into the bundle without sending its words to the
planner. Revising a long podcast turn now also directs every part of that turn; before,
its cached parts were reused unchanged.

MP3 uses LAME 3.100, statically linked under the LGPL (`framechord notices`,
[packaging](PACKAGING.md#mp3-encoder-and-the-lgpl)); `--no-default-features` builds
without it. Speech models without a recorded request limit (Google, Fish) are split
at 4,000 characters per request. That value is a conservative choice inside Gemini 3.8
Flash TTS's documented 8,192-token input limit, checked on 2026-10-19; Fish publishes
no per-request character cap. Output is 24 kHz mono and is not an ACX/Audible delivery
specification; WAV books are limited to about 24.8 hours.

Offline: 15 new tests in `tests/audiobook.rs` plus splitter and chunker unit tests
cover splitting rules, exact text, narrator enforcement, schema gates, unchanged
fingerprints for existing jobs, the preflight, WAV and MP3 books with ID3 chapters
read back, chapters mode, subsets reused by a later full run, partial failure and
resume, chapter revision, streamed versus in-memory assembly, pricing, templates,
planner source containment and the CLI. The statically linked LAME build, formatting,
strict Clippy, the full test suite and the isolated package checks passed natively on
Windows x86-64, Linux x86-64 and macOS Apple Silicon for commit `1587cff`
([checks](https://github.com/maxiplux/framechord/actions/runs/39316394373),
[packages](https://github.com/maxiplux/framechord/actions/runs/39316394369)). One approved live run
(two Google Gemini TTS requests) produced the guide's two-chapter sample with correct
ID3 chapters; see [live evidence](LIVE_TESTS.md#audiobook-sample-october-19-2026).
Narration quality has not been reviewed by a listener, and Fish, ElevenLabs and OpenAI
audiobooks have only mocked coverage.

The guide gains a `#audiobooks` section (now 21 sections), a playable two-chapter
sample in the showcase, a thirteenth delegation prompt and a ninth scenario, and six
example files (41 in total). `build-site.py --check` and its tests pass; the page was
checked locally at desktop and phone widths with no console errors.
