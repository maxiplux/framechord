# framechord

## Purpose and current state

Build a local Rust CLI that lets any agent with process access, including Codex and Claude Code, and terminal users generate images, infographics, speech, podcasts, audiobooks, music, and sound effects through reusable skills. Characters are optional; when needed, define visual and voice identity once and reuse it across jobs and projects.

The intended package, binary, and skill name is `framechord`. The initialized configuration directory is the visible `framechord/` folder, without a leading dot. Existing `mediaagent/` projects remain supported in place to preserve paths, receipts and reference hashes; do not move or rewrite their content as part of branding. Refuse ambiguous roots containing both directories.

The repository now contains the Rust 2024 `framechord` package, a reusable library and CLI, versioned contracts, offline initialization/validation, embedded skills/templates, portable characters, concurrent durable runs, limits/cache/recovery, native audio assembly, reviews/revisions, and initial adapters for all five providers. Local offline tests verify contracts and mocked execution. Bounded live tests have verified selected image, speech, music, sound-effect, and OpenAI planning paths; media quality remains unverified, and the first paired character scene failed user review for identity drift. Explicit identity-reference roles, original provider/model provenance locks, complete offline init setup, and supported high-fidelity controls are implemented; do not describe visual consistency as accepted. Bounded voice discovery is implemented for Google, Fish, and ElevenLabs. ElevenLabs descriptive voice previews and explicit saved-voice creation/binding are implemented, with live preview testing and mocked creation/binding checks. Durable AI image review records preserve request receipts and explicit retry history. Consent-gated ElevenLabs cloning, portable authorized voice samples, Fish reference synthesis, Google audio review and interaction retrieval, paginated/filterable catalogs, an offline configuration wizard, and explicit host ceilings are implemented. Live audio review and filtered catalogs passed; cloning/reference synthesis use offline fixtures until selected authorized recordings are supplied. Human quality/consistency and public release signing remain unverified. Native CI/package results for each version are tracked in docs/STATUS.md. Local Apple Silicon release packaging has passed isolated offline checks. See `docs/STATUS.md` for exact implementation boundaries. Do not describe planned capabilities or unexecuted live tests as completed.


Remediation adds v2 jobs/config/runs (v1 readers retained), cast authoring, planner
selected-input bundles, bounded verification previews, read-only model discovery,
user-attested response recovery, usage/storage/migration commands, reference blobs,
and optional podcast assembly controls. Keep older character/infographic/review
contracts at v1. Paid acceptance batches require the user's approval of their
concrete preview; media quality, selected voices and canonical references still
require actual human acceptance. See docs/REMEDIATION.md and docs/ACCEPTANCE.md.

## Current setup and model policy

The terminal initializer is a resumable checklist with arrow/search menus and
masked optional key entry. New CLI projects use Google defaults for supported image, speech, music and text features,
with OpenAI as an explicit alternative; standalone effects remain optional with ElevenLabs.
Init preserves/appends project .env key names and installs Codex/Claude project skills
by default; `cursor`, `gemini`, `opencode`, `copilot` and `none` are explicit targets.
`--preset empty` preserves empty-scaffold behavior. Initialization stays offline.
Only reviewed current stable model families may submit new provider requests,
including retries, planning, reviews and voice design. Historical contracts,
receipts, cached successes and response recovery remain readable. Never silently
upgrade a pinned character: create a new version and explicitly accept new
references. Maintain dated official-source metadata in `assets/models.json` and
`assets/pricing.json` and review both each release. Discovery cannot authorize unimplemented or retired models.

## First-version scope

- Generate character images, spoken dialogue and narration, instrumental and vocal music, and sound effects.
- Integrate OpenAI, Google Gemini, Fish Audio, OpenRouter, and ElevenLabs for their supported capabilities. Every provider does not need to implement every media type.
- Support unified characters with separate visual and voice profiles, saved voices, reference recordings, and voice creation/cloning where the provider supports them.
- Support agent-authored YAML and Markdown, plus a standalone CLI planning workflow using a configured text model.
- Initialize projects offline, install project or user skills, and execute independent generation jobs in parallel.
- Reuse characters between projects through portable import/export bundles.
- Test locally first. Video generation is a future extension; do not add placeholder video adapters or advertise video support in the first version.

Character voice consistency applies to speech in this version. Vocal music follows its music brief; do not promise that its singer matches the character's speaking voice.

## Architecture

Keep command handling, project storage, typed configuration and validation, character resolution, text planning, provider adapters, and concurrent execution separate. Prefer a reusable Rust library behind a thin command-line entry point.

Provider adapters own authentication, request translation, streaming or asynchronous provider responses, and provider-specific errors. Shared orchestration owns validation, dependencies, concurrency, local artifacts, and run status. Use a capability registry to describe support by provider and model, including reference images, voice references, cloning, formats, and supported settings. Reject unsupported requests before submitting generation calls.

Agents and the CLI planner must produce the same file contracts. Keep request construction and credential handling inside the binary rather than duplicating API calls in skills. Bundle the maintained skill and template assets with the executable so an installed binary can initialize another project without access to this source checkout.

Use local files for project state. No hosted backend or service is required beyond calls to the selected provider APIs.

## Project files and templates

Use this intended initialized layout:

```text
<project>/
  .env                         # Local credentials; ignored by Git
  .env.example                 # Credential variable names; no secrets
  framechord/
    config.yaml
    templates/
    characters/<character-id>/
      character.yaml
      persona.md
      visual.md
      voice.md
      references/images/
      references/audio/
    jobs/<job-id>/
      job.yaml
      brief.md
    runs/<run-id>/
      run.yaml
      outputs/
  .agents/skills/framechord/    # Project skill for Codex
  .claude/skills/framechord/    # Project skill for Claude Code
```

YAML is the canonical structured format for configuration, character definitions, job manifests, and generation records. Version these file contracts. YAML identifies the relevant Markdown files and local media references; Markdown holds readable descriptions, exact speech scripts, lyrics, and generation briefs. Do not replace this workflow with a database or bury all authoring content in one prompt string.

Include templates for character identity, visual appearance, voice, image briefs, speech scripts, music briefs, and sound-effect briefs. Give each template the sections and examples needed to author a useful request. Distinguish stable identity from job-specific direction: a pose, emotion, outfit variation, or delivery change must not silently rewrite the canonical character.

Resolve referenced paths relative to their containing manifest and keep exported character dependencies inside the bundle. Validate YAML, required content, reference existence, and capability compatibility before paid calls. Treat Markdown and YAML as data; do not execute embedded commands while parsing them.

Persist effective generation inputs and results, including character and reference hashes, provider, model, settings, output paths, status, and usage/cost information when available. Do not invent usage or cost numbers when a provider does not return them. Labelled list-price estimates computed by `usage` from the dated bundled price table are a separate read-only view: they never replace provider-reported cost, never become a currency total, and leave unpriceable work unknown.

## Character identity and reuse

Each character has a stable ID and a unified identity, with independently configurable visual and voice profiles. Its portable folder carries its YAML, Markdown, reference images, and reference recordings. Provider voice IDs are provider/account-specific bindings, not universally portable voices.

Reuse accepted visual references and the saved voice binding or fixed reference recordings on subsequent jobs. Derived visual identity jobs must use the original asset’s provider and exact model, resolved from verified local receipts or declared hash-bound origin metadata. Reject conflicting model overrides and mixed identity model lineages before generation. Keep original provenance through acceptance/export/import. Initialization can save models, compatible settings, locales, consistency policy, and limits before creating configured initial profiles; repeated init preserves existing content. Pin the selected provider, model, and identity settings; seeds alone do not guarantee consistent characters. Do not silently replace a missing voice, change providers, discard references, or claim exact identity preservation across providers.

Bootstrap may import existing references or generate candidate character images and voices. Voice workflows may use existing provider voices, descriptive voice design, or cloning where supported and accessible to the account. Creating a provider-side voice is an explicit operation. Honor the provider's required consent and verification inputs without pretending unsupported cloning is available.

Candidate references remain candidates until explicitly accepted by the user or through an agent carrying the user's selection. Once accepted, they become canonical references. Routine generation must not automatically promote its outputs to character references.

Import/export must preserve character content and local references without depending on an absolute path in the original project. Detect ID conflicts instead of overwriting another character. Reusing an imported provider voice requires an accessible binding in the destination account; otherwise explain the missing binding and require explicit configuration or bootstrap.

## CLI workflows

The following command families now have an initial implementation; use `--help` and `docs/STATUS.md` for implemented options and remaining limitations:

- `framechord init [path]`: create configuration, templates, an initial persona scaffold, and project-local skills. Allow agent targets to be selected (codex, claude, cursor, gemini, opencode, copilot, none); default to Codex and Claude Code, whose directories Cursor, Gemini CLI, OpenCode and Copilot also read. Perform no network calls or media generation.
- `framechord skills install`: install the bundled skill for selected agents at project or user scope.
- `framechord persona`: create, inspect, bootstrap, accept references, import, and export characters.
- `framechord plan`: use a configured text model to author character descriptions or generation jobs from the shared templates. Produce reviewable files before generation; planning must not implicitly bootstrap voices or generate media.
- `framechord validate`: check configuration, characters, jobs, and references without generation calls.
- `framechord doctor`: inspect key presence offline; explicit `--live` checks selected provider authentication endpoints with bounded read-only requests. Read per-provider status/actions; do not confuse authentication with generation/model/billing readiness.
- `framechord generate` and `framechord batch`: execute a job or a collection of jobs using saved defaults and explicit overrides.
- `framechord resume`: continue a recorded run while preserving successful results and handling uncertain provider requests explicitly.

Initialization saves explicit provider/model choices per media type. Require valid choices before generation; do not guess a model or use an unrecorded `latest` selection. Job overrides are supported, but provider or voice changes must remain explicit. Standalone planning also uses an explicitly configured text provider/model.

Human-facing prompts may help initialization and authoring. Every workflow used by the skill must also support noninteractive operation. Provide structured JSON results for agent callers and put progress and diagnostics on stderr. Failures must have nonzero exit status and actionable errors.

Initialization and skill installation must preserve existing credentials, unrelated skills, configuration, character references, and user-authored files. Repeated initialization should report existing content rather than overwrite it. Updates that would replace modified assets must be explicit.

## Skill integration

Maintain one provider-neutral `framechord` skill source with YAML frontmatter containing `name` and `description`, followed by concise workflow instructions. Link substantial template/schema guidance through supporting references rather than duplicating it throughout the skill. Normal automatic skill discovery remains enabled.

Install the skill in the documented discovery locations:

- Codex: project `.agents/skills/framechord/`; user `~/.agents/skills/framechord/`.
- Claude Code: project `.claude/skills/framechord/`; user `~/.claude/skills/framechord/`.
- Cursor: project `.cursor/skills/framechord/`; user `~/.cursor/skills/framechord/` (also reads the Codex and Claude directories).
- Gemini CLI: project `.gemini/skills/framechord/`; user `~/.gemini/skills/framechord/` (also reads `.agents/skills/`).
- OpenCode: project `.opencode/skills/framechord/`; user `~/.config/opencode/skills/framechord/` (also reads the Codex and Claude directories).
- GitHub Copilot: project `.github/skills/framechord/`; user `~/.copilot/skills/framechord/` (also reads the Codex and Claude directories).

Native directories are explicit choices; the default pair already reaches every agent above. Verify these paths against each vendor's documentation when they change.

The skill helps the agent inspect configuration and available characters, author YAML and Markdown using templates, validate files, bootstrap references when requested, and call the binary for generation. It must preserve exact requested dialogue/lyrics, reuse selected characters, and report generated artifact paths and incomplete jobs accurately.

Project initialization defaults to project-local installation. User installation is a separate explicit command. Do not rewrite global agent settings, command permissions, or unrelated project instructions as part of installing this skill. If the binary cannot be found, explain how to build/install it rather than silently switching to another media tool.

## Parallel generation and recovery

Execute independent jobs concurrently with configurable global and per-provider limits. Resolve bootstrap and other job dependencies before dependent requests. An initialized character can be reused by many jobs without regenerating its references.

Persist each job's status and outputs separately so concurrent writes cannot corrupt manifests or overwrite another job's files. Preserve successful jobs when others fail; report partial success and return a failure status for an incomplete batch. Publish completed local artifacts atomically.

Fingerprint the effective request, including brief/script content, character definition, reference bytes, provider/model, locale, and generation settings. Reuse verified local results for identical requests unless fresh generation is explicitly requested. Cached reuse is not a promise that fresh model output will be deterministic.

Handle rate limits, timeouts, streaming interruptions, and asynchronous provider jobs. Use bounded retries and honor provider retry guidance only when safe. Do not blindly resubmit a generation after an ambiguous timeout or interruption: it may already have incurred charges. Save request/job IDs when available and reconcile provider status or require an explicit retry when the outcome cannot be established. Never silently fail over to another provider.

## Credentials and local artifacts

Read credentials from the process environment or the target project's root `.env`; process environment values take precedence. Do not pick up credentials from unrelated ancestor projects. Use `OPENAI_API_KEY`, `GEMINI_API_KEY`, `FISH_API_KEY`, `OPENROUTER_API_KEY`, and `ELEVENLABS_API_KEY` as the project-facing credential names.

Local development and live smoke tests use this repository's `.env`. Never print or inspect its secret values in agent output. Keep keys, authorization headers, credential-bearing URLs, and billing information out of YAML, Markdown, skills, prompts, logs, fixtures, and version control. Configure providers by credential variable name rather than literal secrets.

Preserve the existing `.env` and `.env.*` ignore rules, with `.env.example` as the secret-free exception. Never populate examples with working credentials. Initializing another project must not copy this development repository's keys. Load credentials only for the provider needed by the operation.

Keep build products, caches, and generated run artifacts out of this development repository's version control. Do not ignore the entire `framechord/` directory: configuration, templates, and authored definitions may be versioned. Add only targeted ignore entries when introducing artifacts, and verify them. Use deliberate, small fixtures for tests.

## Development and validation

Supported native release targets are Windows x86-64, Linux x86-64 and macOS
Apple Silicon (ARM64). Do not add Intel Mac builds to CI or release publication.
Historical immutable releases may retain their original Intel assets.

Keep provider details grounded in current official documentation and record explicit model choices. Do not assume website subscriptions, API credits, model access, or cloning permissions are interchangeable. Check account limitations during explicit live operations and report them accurately.

For Rust code changes, run appropriate checks:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Normal tests must run offline with mocked providers and must never become live simply because credentials are present. Provide explicit, opt-in live smoke tests using short scripts, small images, short music/effect clips, bounded request counts, and selected providers. Report unavailable credentials or capabilities as untested/skipped rather than passed. Do not launch paid calls while running ordinary validation or tests.

Meaningful implementation checks must cover:

- Offline initialization, repeat initialization, existing-file preservation, and both project/user skill targets.
- Character import/export with relative references and conflicting IDs.
- Reuse of accepted visual/voice references, explicit bootstrap acceptance, and capability rejection before API calls.
- Agent-authored and CLI-planned files accepted by the same validator.
- Actual concurrent execution, configured limits, dependencies, isolated writes, partial failures, cache invalidation, and resume without repeating successful or ambiguous requests.
- Provider response handling, environment precedence, and credential redaction.
- Opt-in live generation for each implemented provider capability, with recorded artifact paths and honest test status.

Human review is needed to assess image identity and voice quality; schema validation and successful downloads alone do not prove character consistency. Verify this with repeated image scenes and speech clips using the same accepted persona.

For documentation-only changes, verify accuracy, naming, links, and agreement with the project requirements. Do not run paid generation or add tests that only mirror documentation wording.

## Official references

- OpenAI: <https://developers.openai.com/api/docs>
- Google Gemini: <https://ai.google.dev/gemini-api/docs>
- Google audio understanding: <https://ai.google.dev/gemini-api/docs/audio>
- Google speech generation: <https://ai.google.dev/gemini-api/docs/speech-generation>
- Fish Audio API: <https://docs.fish.audio/api-reference/endpoint/openapi-v1/text-to-speech>
- Fish Audio plans: <https://fish.audio/plan/>
- OpenRouter: <https://openrouter.ai/docs/quickstart>
- ElevenLabs: <https://elevenlabs.io/docs/overview/capabilities/voices> and <https://elevenlabs.io/docs/overview/capabilities/music>
- Codex skills: <https://learn.chatgpt.com/docs/build-skills>
- Claude Code skills: <https://code.claude.com/docs/en/skills>
- Cursor skills: <https://cursor.com/docs/skills>
- Gemini CLI skills: <https://geminicli.com/docs/cli/tutorials/skills-getting-started/>
- OpenCode skills: <https://opencode.ai/docs/skills/>
- GitHub Copilot agent skills: <https://docs.github.com/en/copilot/concepts/agents/about-agent-skills>
- Pricing pages for the bundled price table: <https://ai.google.dev/gemini-api/docs/pricing>, <https://developers.openai.com/api/docs/pricing>, <https://elevenlabs.io/pricing/api>

Google audio understanding is distinct from speech generation. Use the correct API for the requested modality, and consult the current provider reference before implementing an adapter.
