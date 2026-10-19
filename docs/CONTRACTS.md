> Remediation update: job/config/run support v1 and v2. New cast and podcast assembly
> fields require v2. Other manifest versions remain v1. See [remediation](REMEDIATION.md)
> and the CLI-generated schemas for current contracts; older v1 examples remain readable.

# Contracts (v1 and v2 manifests)

The executable and library share typed, unknown-field-rejecting Serde contracts. Schemars emits JSON Schema 2020-12. Every versioned document requires a supported `schema_version` (1, or 2 for job, config and run); unsupported versions are rejected. Schemas: config, character, infographic, job, batch, effective, run, artifact, review, revision, budget, operation, voice_design, voice_consent, voice_clone, host_limits, diagnostics, verification. Limits are a typed section of config and can also be supplied independently with `--limits-file`.

YAML owns structured state. Markdown owns readable identity, visual/voice descriptions, exact scripts/lyrics, and creative direction. Neither is executed. Effective input snapshots include complete prompt/script content, provider/model, locale/settings, identity hashes, ordered references with role/hash/bytes, dependencies, and output destinations. Run records include isolated job status, attempts, request IDs when available, usage when returned, artifacts, hashes, dimensions/duration, cache provenance, and quality.

Commands wait by default. Async submission confirms a local worker has started or completed before returning accepted status. Worker ownership uses an OS file lock, not a process-name guess. Each run has one manifest writer; job stages and response files have separate paths. Outputs publish through temporary files on the destination volume. No background hosted service is installed.

Run states are queued, running, cancelling, succeeded, partial, failed, cancelled, blocked, unknown. A lost worker makes submitted requests unknown; queued work and local assembly remain resumable. Cancel stops queued jobs and allows in-flight requests to finish, retaining results. Wait timeout/interruption stops waiting and leaves generation running. Resume verifies successful artifacts, reuses complete locally saved responses, and never repeats unknown submissions without `--retry-unknown`. `runs reconcile` retrieves saved Google interaction IDs with bounded GET calls; completed responses resume through local publication without a new reservation. Pending/requires-action interactions remain unknown. Other adapters require account reconciliation before an explicit retry.

Batch dependencies must be acyclic and refer to existing unique job IDs. Shared destinations are rejected. Independent work executes concurrently under project and host locks. A failed dependency blocks downstream work. Partial success preserves successful artifacts and returns nonzero. Cached dependencies progress to completion in the same invocation. Podcast and chunked narration retain ordered clips and combine them natively into WAV without ffmpeg.

Budget reservations are atomic and shared by revisions. Requests/images/audio/speech/revision counters persist in `budget.yaml`. Rejected and explicitly retried requests consume attempts; cache reuse and local response recovery do not create another request reservation. Rate-limit retry guidance is persisted. Default host ceilings are 4 globally and 2 per provider. Explicit user configuration permits up to 32/16; per-run limits and shared host locks both apply. Lowered ceilings count already-active calls before admitting more. All generation types require explicit supported settings. Lyria Clip requests the 30-second preset (decoded output duration is recorded separately); full-song Lyria requires an explicit longer reservation and prompt-guided duration is not exact.

Review does not change generation status or produce media. Quality is unchecked, passed, needs_revision, or inconclusive, with provenance and concrete findings. Image AI review receives the actual result plus references and source copy. Google audio review receives actual audio, exact source text/turns/brief, and accepted voice evidence. Inline review is bounded to 20 MiB and oversized requests fail before a budget reservation. Review reconciliation retrieves a saved Google interaction without another paid assessment. A review may not prove accuracy or identity solely from successful download. Revisions preserve provider/model/identity/voice bindings, create a child artifact, and never replace accepted references. Script/copy changes are supplied explicitly with `--source-job`; a podcast `--turn` revision reuses verified unchanged clips.

JSON response fields: schema_version, command, ok, data, errors, warnings. Command names follow parsed command paths, never a guessed position in argv. Error fields: code, message, field, retryable, corrective_argv, optional request_id/retry_after_seconds. Credentials and provider error bodies are excluded. Exit codes: 0 success/accepted/pending; 1 internal; 2 invalid input/config; 3 operational; 4 partial; 5 needs_revision; 6 limit; 7 unknown; 130 interrupted wait. Unknown and limit states take priority over partial success.

Saved job overrides apply explicitly to provider/model, locale, and individual settings without rewriting authored files. Pinned character provider/model/voice conflicts fail during resolution. Podcast settings belong on turns rather than the assembly job. Model-authored planning bundles contain at most 32 YAML/Markdown/text files and 1 MiB of JSON; all manifest dependencies must remain inside the bundle before the shared validator reads them. Reference validation decodes media and rejects a mismatch between file extension and bytes before acceptance or generation.

CLI result views omit inline reference base64 while preserving names, MIME types, roles, and hashes, and include a warning when bytes are omitted. Complete reference snapshots remain in persisted run YAML. The file schemas describe persisted contracts; CLI views avoid sending encoded pixels into agent context. Dry runs persist no run.

Review and voice operations have durable versioned operation records. Per-attempt history preserves provider IDs and outcomes across explicit retries. Complete results are reused when local recording or binding is interrupted. Lost running operations become unknown and require explicit retry; safe provider rejection still respects retry guidance and attempt limits. AI review consumes the source lineage request budget and cannot run on an unfinished source. Voice designs have their own bounded lineage, exact 100–1000 character preview script, explicit design/speech models, and at most three candidates. Provider-side creation uses a selected candidate; character binding is explicit and verification-aware. Voice-design status uses `persona voice-status`; these operations are synchronous and are not media worker runs.


`VoiceSample` paths resolve against the character/job manifest and carry audio,
exact transcript, and permission YAML. `VoiceConsent` requires actual granted
permission, subject, authorizer, listed purpose, and nonempty local evidence.
`reference_synthesis` and `saved_voice_clone` are distinct purposes. Fingerprints
include recording bytes, exact transcript, and semantic authorization/evidence
hashes. Portable export rewrites local dependency paths without changing the
permission hash. Samples never authorize their own use merely by existing.

Recorded clones are explicit ElevenLabs account operations, with 1–5 recordings
and a 48 MiB encoded budget. Identity, authorization, name, model, and recordings
are pinned in the operation snapshot. Binding and canonical recording acceptance
are separate actions; required verification must be confirmed before binding.
Templates are denied-permission scaffolds. Provider professional/biometric
verification remains an account flow, not an impersonated CLI submission.


Image job `references` accepts legacy plain paths (content role) or strict objects
`{path, role}` with identity/style/layout/content. `--identity-reference` maps to
identity role without canonically accepting the file. Identity images precede other
roles and are labelled by filename in the compiled guide. Selected images remain
immutable bytes/hashes in the effective snapshot. `input_fidelity` is optional,
included in fingerprints when set, and omitted from serialization when unset to
preserve existing no-fidelity request fingerprints. OpenAI image 1/1.5 identity
requests resolve to high; low-only mini identity requests fail. Image 2 omits this
API parameter. Fidelity never constitutes automatic identity acceptance.


Configuration adds optional `default_settings` and `default_locales` maps keyed by
media, alongside explicit provider/model `defaults`. Defaults must pass capability
validation without network access. `consistency.require_known_identity_model` is
true for new unsupplied init configs; legacy configs default false. Known identity
origins always pin provider/model, independently of that compatibility flag.
`consistency.inherit_source_settings` defaults true. Default settings apply only to
the matching selected model; common original settings, visual profile settings,
and explicit job settings overlay in that order. Edit-only input fidelity is omitted
for originals without references. Podcast defaults apply to compatible individual
turns rather than the assembly job.

Typed references optionally carry `origin` with `selection`, `settings`, `locale`,
`sha256`, and local `artifact` ID when known. Receipt lookup prefers an exact original
path, then unambiguous identical bytes; changed files, conflicting declared models,
mixed identity model lineages, and rejected identity sources fail before submission.
Accepted visual profiles carry `settings` and `origins` keyed by media hash so renamed
portable references retain provenance after export/import. External declarations
are caller-supplied metadata, not independent proof of their generation history.
Missing optional origin fields preserve compatibility with old manifests.


`usage` is a read-only report. Each job row keeps `provider_reported_usage` raw and adds
`cost_estimate`, `cost_status` (`estimated`, `provider_reported`, `cached_no_charge`,
`not_estimable`; operations and local podcast and audiobook assembly jobs are `not_estimated`) and `cost_note`. Estimates come from the
dated bundled price table (`assets/pricing.json`, or an explicit validated `--price-table`),
are computed from recorded artifact dimensions/durations, script characters or returned
token counts, never replace a provider-returned `cost`, and are summed only into
`estimated_total_usd` with an `estimate_summary`; `currency_total` remains null.

`diagnostics` is the read-only provider/key report contract. `doctor` is offline by
default; `--live` explicitly checks the selected providers, returns all per-provider
statuses with safe actions, and exits 3 when any selected check fails. It creates no
generation records and does not infer media endpoint/model/billing readiness. See
[connection checks](CONNECTIONS.md) for the exact endpoints and response fields.

## Audiobooks

`media: audiobook` requires job `schema_version: 2`. It takes exactly one text
source: `manuscript` (one Markdown file, optionally with `split: {heading_level,
empty_chapters, preamble}`) or `chapters` (ordered `{id, title, text_file, brief,
settings}` entries). `text_file`, `turns`, `cast`, `spec`, `references`, `output`,
`settings.format` and `assembly.turn_gain_db` are rejected. Packaging is in
`audiobook: {output_mode: chapters|full, format: wav|mp3, mp3_bitrate_kbps,
announce_titles, max_chars_per_request, title, author}`; `assembly.pause_ms` is the
gap between chapters and `assembly.loudness_lufs` is applied to each chapter.

Splitting reads only ATX `#` headings outside fenced code; YAML front matter is
excluded, a leading byte-order mark is dropped, heading lines are never narrated and
chapter bodies are exact slices of the source. The narrator is the job `character`,
an explicit `selection`, `defaults.audiobook` or `defaults.speech`, recorded as
`selection_source`; every chapter must resolve to the same selection, character hash,
voice, locale and references.

The resolved effective input is a book (`media: audiobook`) whose turns are chapters
(`media: speech`, `audiobook.role: chapter`), whose turns are provider parts. Parts are
plain speech requests, paragraph-aligned and at most `min(max_chars_per_request or
4000, model limit)` characters; their fingerprints exclude packaging, so changing the
format, mode or pause reuses every cached part. Chapters and the book are assembled
locally, never cached and never sent to a provider. A run selected with `generate
--chapter` contains no book job. Before a run is created, the whole audiobook is
checked against `requests`, `speech_chars` and `audio_seconds`, counting only parts
without a verified cached result plus budget already used by the group; a book that
does not fit exits 6 with no run.

With `--output-dir DIR`, files are published to `DIR/<book>/`: `chapter-NN-<id>.<ext>`
per chapter, and in `full` mode `<book>.<ext>`, `<book>.chapters.json` (the `chapters`
schema) and `<book>.cue` (omitted above 99 chapters). MP3 is constant bitrate at 24 kHz
mono with an ID3v2.3 tag; the full book carries one `CHAP` frame per chapter and an
ordered top-level `CTOC`. WAV output is limited to about 24.8 hours. Result JSON adds
an `audiobook` summary with chapter states, paths, durations, cached parts, missing
chapters and the next step; long narration strings in result views are replaced by
their character count and hash. `revise --chapter ID` adds direction to that chapter
and every part beneath it.
