# Execution contracts (schema version 1)

Configuration lives in the visible `framechord/` directory. Quick generation needs no init; it still writes durable records under that directory. Credentials come from process environment first, then the target root `.env`, never ancestors. Names: OPENAI_API_KEY, GEMINI_API_KEY, FISH_API_KEY, OPENROUTER_API_KEY, ELEVENLABS_API_KEY. Never put values in manifests or prompts.

`--output` means one final artifact filename. `--output-dir` chooses run output storage, including podcast clips plus combined WAV and audiobook folders (`<dir>/<book-id>/`, see [audiobooks](audiobooks.md)). Paths returned in JSON are absolute. Existing destinations require `--overwrite`. Provider/model and individual setting flags can override a saved job without modifying its YAML. Character bindings remain pinned; conflicting overrides fail. Podcast voice/settings must be configured per turn. All jobs validate before submission. Unsupported model/settings/reference combinations fail before paid calls.

Normal generation waits. `--async` starts a local worker and returns a durable run record. Workers use no hosted service. Terminal progress belongs to stderr, with real job counts and a spinner while provider completion is unknown. `--json` has no animation or ANSI. `--progress plain` works in redirected logs; `--progress off` suppresses updates. `NO_COLOR` disables color unless explicitly overridden by `--color always`.

States: queued, running, cancelling, succeeded, partial, failed, cancelled, blocked, unknown. Wait timeout stops waiting, not generation. Cancel prevents queued work; in-flight calls finish and retain results. Worker loss leaves queued work resumable, and running submissions unknown. Resume retains successful artifacts. No automatic provider failover. Ambiguous requests require `--retry-unknown`; these may have been charged already.

Identical effective inputs reuse only hash-verified artifacts unless `--fresh`. Cache keys include content, settings, identity, reference bytes, and provider/model/locale. Fresh results are not guaranteed deterministic. `--request-id` is separate submission idempotency; conflicting input under the same ID is rejected.

Default shared run budget: 20 requests, 8 images, 1,800 reserved audio seconds, 60,000 speech characters, 3 revision operations, 3 attempts per job. Default host concurrency is 4 globally and 2 per provider. Per-run limits allow up to 32/16; actual execution also respects shared host ceilings. `config host show` reads them; explicit `config host setup --file host-limits.yaml` writes only framechord user configuration. Lowering host ceilings waits for active work to drain before admitting more calls. Reservations happen before calls, and revisions inherit the budget group. Speech reservations are estimates at 12 characters/second; actual decoded duration is recorded. These are volume limits, not dollar caps. `usage --json` adds `cost_estimate`, `cost_status` (`estimated`, `provider_reported`, `cached_no_charge`, `not_estimable`; operations and local podcast and audiobook assembly jobs `not_estimated`) and `cost_note` per job plus `estimated_total_usd` and `estimate_summary`, computed from the dated bundled price table or an explicit `--price-table`; `currency_total` stays null. A duration overrun saves the artifact and blocks further affected work. Cached reuse does not consume a generation reservation. No automatic retries after ambiguous submission.

Local integrity validation checks decoded image/audio formats, hashes, dimensions or duration. Semantic quality starts unchecked. `review record` imports a human/agent assessment with provenance. `review check --ai` is explicit, uses a configured compatible text reviewer, and may be inconclusive. `revise` creates a child run, preserving the source and identity bindings. Image revisions use the original as an editing reference when supported; audio is regenerated. Exact copy changes require an updated source spec/script.

JSON envelope: schema_version, command, ok, data, errors, warnings. Errors contain code, message, field, retryable, corrective_argv. Exit codes: 0 success/accepted/pending; 1 internal; 2 invalid input; 3 provider/local operation; 4 partial; 5 needs revision; 6 limit; 7 unknown; 130 interruption. Limit and unknown take priority over partial success.

Ordinary tests and initialization are offline. Live generation is opt-in and requires provider access. Provider voice IDs belong to an account and do not become portable universal voices. Schema validation cannot prove image identity or natural voice quality; assess repeated scenes/clips with human review.

`skills install --overwrite`, `skills export DIRECTORY --overwrite`, and `templates export DIRECTORY --overwrite` explicitly refresh bundled files, including modified bundled assets; unrelated files remain intact. Default installation and repeated initialization preserve existing content.

`plan` makes a text-model request and produces reviewable YAML/Markdown/text files only. Its bundle is limited to 32 files and 1 MiB, with dependencies contained inside the bundle and shared schema/capability validation before publication. Raw text responses stay in the recorded run if authoring validation fails; planning never calls a media generator.

CLI result views omit inline reference base64 while preserving names, MIME types, roles, and hashes, and include a warning when bytes are omitted. Complete reference snapshots remain in persisted run YAML. The file schemas describe persisted contracts; CLI views avoid sending encoded pixels into agent context. Dry runs persist no run.

`review status --artifact ARTIFACT_ID --json` inspects review attempts and receipts without a provider call. Status marks abandoned running operations unknown while preserving attempts and request IDs. AI image and audio reviews persist operation records under the source run, including per-attempt request IDs, returned usage, results, and uncertain outcomes. Repeat identical review commands reuse a completed assessment without another request. `review check --ai --retry-unknown` is explicit and may incur another charge. Unsupported reviewers and unfinished/corrupt generation results fail before review submission. Voice design/create operations use the same durable receipt and retry rules; read [voices](voices.md) for the candidate-selection workflow.


`config wizard` helps terminal users choose explicit supported defaults without a
network call. It shows the complete configuration before saving. Automation uses
`config setup --file` instead; the wizard rejects JSON/noninteractive invocations.
Host setup is a separate user-scope operation and never edits agent settings.

`runs reconcile RUN_ID` retrieves uncertain Google interactions with recorded IDs
using GET requests only. Completed responses are saved locally; `resume` then
publishes without another paid generation or attempt. Pending interactions remain
unknown and are never silently resubmitted. Providers whose adapter has no retrieval
endpoint need account reconciliation before an explicit retry. Missing/expired IDs
are actionable unknown outcomes rather than proof of failure.

Audio AI review uses explicit Google `gemini-3.8-flash`, actual
WAV/MP3/FLAC bytes, exact script/ordered podcast turns or music/effect briefs, and
accepted recording evidence. It can assess transcription, pronunciation, omissions,
numbers, timing/clipping, and voice similarity where references exist. Image AI
review uses OpenAI/OpenRouter multimodal text models. Neither guarantees quality.
Google inline review requests are limited to 20 MiB including encoded audio and
source text; use individual clips for larger jobs. Reviews preserve receipts and
actual returned usage. `review reconcile --artifact ID` retrieves uncertain Google
audio reviews without a new assessment request. It records a completed assessment,
or returns unknown when provider completion cannot be established. For definitive
quality acceptance, listen/view and use `review record` with actual provenance.


Long speech on models with documented character caps splits at Unicode-safe
boundaries without changing the source text: OpenAI 4096; ElevenLabs Flash v2.5
40000, Multilingual v2 10000, v3 5000 per request. Chunks and final assembled WAV
remain recorded. Limits reserve each actual chunk call; explicit lower request
budgets can leave a partial narration. Other models keep their script as supplied.
Podcasts may define every turn's selection/character and need no unrelated default
on the local assembly job.

`init` uses the looping offline setup described in [setup](setup.md). New noninteractive projects receive Google defaults with a built-in voice and report incomplete readiness until GEMINI_API_KEY is available. The OpenAI alternative is explicit with --preset openai. Missing key variable names are appended to the chosen project .env without replacing existing values; both project skills are installed by default and are also discovered by Cursor, Gemini CLI, OpenCode and Copilot; `--agents none` installs no skill. `--preset empty` opts into an empty scaffold. Initialization never makes network calls; masked key entry writes only on explicit save. Existing project defaults and character identities are preserved. Only current stable models may make new provider requests, while older records and verified results remain inspectable.
