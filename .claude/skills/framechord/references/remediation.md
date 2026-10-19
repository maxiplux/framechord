# Remediation workflows and acceptance gates

Implementation work is separate from live API verification and human acceptance.
Existing character likeness failed acceptance and remains pending. No new candidates
are automatically accepted, and a selected model is never silently replaced.

## Versioned authoring and character reuse

Jobs, configuration and runs read versions 1 and 2. New direct jobs, new configs and
new runs use v2; character, infographic, review and other existing contracts stay v1.
`cast` and `assembly` require v2. Older binaries cannot read v2; keep the current
binary with exported projects. `migrate --kind job|config|run --file FILE` previews
without editing. `--apply` validates and writes a non-overwriting `.v1.bak` backup.
Active runs cannot be migrated. Ordinary reads do not rewrite old runs.

An image/infographic cast is an ordered list of `{alias, character, direction}`.
`direction` is an optional Markdown path relative to the job. Cast conflicts with
singular `character`; aliases must be unique. Each character needs canonical visual
references and compatible pinned models/identity settings. The CLI conservatively
limits the combined reference set to eight. No identities are blended, rewritten,
or dropped to fit a request. Reference order, hashes and directions affect caching.

`plan --character MANIFEST --asset FILE` can be repeated. The binary copies selected
characters and media into reserved `_inputs/` directories before any planning call.
The model can reference but cannot replace these files. Copied characters preserve
voice permission dependencies and original image provenance. Model-authored bundles
still undergo contained-path and capability validation before atomic publication.

## Verification and quality

```sh
framechord verify --suite suite.yaml --json
# Only after approving the concrete bounded batch:
framechord verify --suite suite.yaml --execute --preview-hash HASH --json
framechord review report --artifact ARTIFACT_ID --output comparison.md --json
```

Suites contain `schema_version: 1`, `id`, `description`, `jobs`, `limits` and
`criteria`. Job paths are relative to the suite. Verification uses one attempt per
request, no external output destinations, content-bound preview hashes, and an
idempotent run ID mapping. Repeating execution of the same preview returns the
existing run; a failed run is not silently retried. Changes to scripts, profiles,
references, settings or limits require a fresh preview. Declared limits bound
requests/media volume, not currency charges. Estimates are unknown when no reliable
provider price is available. Missing credentials and dependent jobs are skipped.
Provider failure does not imply an unsupported capability or a passed test.

Verification executes generation jobs. Planner calls, AI assessments and provider
voice creation are separate explicit operations and must appear separately in the
approved test batch. Repository test cases and external gates are in `docs/ACCEPTANCE.md`. In initialized
projects, author suites using `templates/verification.yaml`; development fixtures are
not copied into unrelated projects.

`review report` exports comparisons and reference copies; it never records acceptance.
Use the reference-sheet, identity-checklist, and acceptance-review templates. Reviews
retain criterion-specific expected/observed/evidence fields. Human review must check
likeness, exact infographic labels/values, narration/lyrics, clipping and transitions.
A candidate remains a candidate until selected through persona acceptance.

## Readiness, usage and recovery

`doctor --live` remains a read-only authentication check. `models --live --provider
openai,google,openrouter,elevenlabs` reads one catalog page per selected provider,
with no retries, a 10 second timeout, a 1 MiB response ceiling, and no registry edits.
It reports catalog visibility separately from generation readiness. Missing models
may be unlisted or beyond the first Google page. Fish has no implemented model-catalog
lookup; its explicit voice catalog and local model registry remain available.
`shutdown_date`, when returned by OpenAI, is informational; it never triggers a switch.

`usage --json` reports provider-returned usage and shared reserved budgets separately.
Unknown cost is not zero; no exact aggregate currency total is invented. Review and
voice-operation usage is included separately; historical attempt rows must not be
summed with their current operation total. Cached job usage describes the original
response and is not a new charge. Labelled list-price estimates (`cost_estimate`, `cost_status`, `estimated_total_usd`) come from the dated bundled price table; see [cost](COST.md).

For an uncertain job, first try supported provider reconciliation. Otherwise:

```sh
framechord runs recover RUN --job JOB --file result.png --format png \
  --fingerprint RECORDED_FINGERPRINT --evidence recovery-evidence.md
framechord runs resume RUN
```

Recovery checks the exact uncertain leaf job, media format, bytes and evidence. It
stores a `user_attested` receipt alongside the original request ID and attempts.
Resume publishes through the normal pipeline without submitting the job again.
Do not put credentials, account details, or billing data in evidence.

## Local audio and storage

Audiobook jobs (`media: audiobook`, schema v2) reuse `assembly.pause_ms` as the gap
between chapters and `assembly.loudness_lufs` per chapter; `turn_gain_db` is rejected.
Chapters and the full book are assembled locally by streaming, so memory stays bounded
to one clip. See the audiobook skill reference.

Podcast `assembly` optionally contains `pause_ms` (0–10000), `turn_gain_db` (known
turn IDs, -24 to 24), and `loudness_lufs` (-36 to -10). Omission preserves legacy
assembly. Output stays mono 24 kHz WAV; original clips are retained. Integrated
loudness uses the native [EBU R128 analyzer](https://docs.rs/ebur128/0.1.10/ebur128/).
A -1 dBFS sample-peak ceiling takes precedence over the requested loudness target;
silence has no measurable loudness. This is not a true-peak mastering limiter.

New v2 run YAML stores reference blobs by SHA-256 under `framechord/blobs/`. Loading
checks hashes before hydrating adapter inputs. Legacy inline references remain
readable. Metadata indexes are derived/rebuildable; YAML is canonical. Ignore blobs
alongside runs/cache, not the entire framechord directory.

`storage inspect` reports size and orphan candidates. `storage plan --output
cleanup.yaml` saves a preview **outside framechord/**. `storage clean --plan
cleanup.yaml` rejects stale plans, symlinks and active workers. It removes only
unreferenced blobs. All runs, outputs, reviews, authoring files and history are
retained, including unknown submissions. Archive/deletion of whole runs is not
implemented.

## Provider references checked during implementation

- [OpenAI model listing](https://developers.openai.com/api/reference/resources/models/methods/list)
- [Google model listing](https://ai.google.dev/api/models)
- [OpenRouter model listing](https://openrouter.ai/docs/api/api-reference/models/list-all-models-and-their-properties)
- [ElevenLabs model listing](https://elevenlabs.io/docs/api-reference/models/list)

Catalog documentation was consulted October 14, 2026. Model access and media quality
require separate live verification; these references do not establish account access.
