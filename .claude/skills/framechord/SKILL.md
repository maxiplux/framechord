---
name: framechord
description: Generate images, infographics, speech, podcasts, audiobooks (by chapter or as a full book), music, and sound effects with the framechord CLI, using optional reusable character identities and local YAML and Markdown briefs.
---

# FrameChord

Use the `framechord` binary from any agent that can run a process. If it is missing, read [installation](references/installation.md) for public installers, user-local installation and explicit updates, or explain `cargo install --path /path/to/framechord` for a source checkout. Do not switch media tools or make provider API calls inside the skill.

Existing `mediaagent/` projects are supported in place. Read [renaming and legacy projects](references/renaming.md) before upgrading an older installation; use returned `data_directory` paths rather than assuming the folder name.

Read [commands](references/commands.md) for every command and flag in this binary version; [workflow](references/workflow.md) for outputs, limits, recovery, and review; [authoring](references/authoring.md) for character and infographic templates. Machine-readable schemas are in `references/schemas/`. Exported templates are in `templates/`.

Read [character continuity](references/consistency.md) when reusing the same characters across scenes. Use selected identity references, supported fidelity controls, and comparison against the actual original images; API success does not establish likeness.

Read [connection checks](references/connections.md) when checking setup or provider/key failures: `doctor` checks presence offline; `doctor --live` explicitly authenticates with read-only requests. It supports provider filtering, timeouts, and JSON. Read status/actions per provider; successful authentication does not establish generation/model/billing readiness.

Read [voices](references/voices.md) when discovering voices, designing previews, accepting authorized recordings, or explicitly creating/cloning/binding a selected provider voice. Preview generation, provider-side creation, canonical binding, and acceptance of recordings are separate operations.

Read [setup and current models](references/setup.md) for the looping terminal wizard, Google-first defaults and an explicit OpenAI alternative, preserving project .env setup, masked credentials and project skill installation, noninteractive setup readiness, model filters and explicit catalog refresh. For a single asset, use explicit current provider/model/input/output flags without initialization. For repeated work, agents use `init PATH --preset recommended --json --non-interactive`, inspect `config status --json`, and supply exact configuration with `config setup --file` where needed. Human terminal users run `init` to resume setup. No provider calls occur during initialization. Existing character model locks are preserved; retired-model identities need a new character version and human-accepted references before generating again.

Choose the media type from what the user asked for. An audiobook, a book or story read aloud, or long text narrated chapter by chapter is `media: audiobook`: one narrator, the exact source text, one file per chapter and optionally the full book (read [audiobooks](references/audiobooks.md)). A podcast is a conversation or multi-speaker show; never turn a book into a podcast unless the user asks for that adaptation. Short single-voice narration is `speech`.

Preserve requested dialogue, lyrics, numbers, and labels exactly; audiobook text is never summarized, adapted or rewritten. Use a separate explicit planning operation when the user asks to adapt an article into a podcast. Supply article text in a local file. Characters are optional; when selected, reuse their accepted references and pinned voice/provider/model. Derived identity assets must use the original asset’s recorded provider/model; conflicting overrides fail. Preserve origin metadata instead of copying the project default into derived job YAML. Describe stable identity separately from pose, scene, delivery, or temporary clothing. Candidate references need the user's selection before acceptance.

Author files using templates, then `validate`. Inspect `generate --job ... --dry-run --json` before execution when inputs are new. Call generation only when it is part of the user's request. Use `--json --non-interactive` for automation. Report artifact paths returned by the binary and incomplete jobs honestly.

Use `runs status`, `runs wait`, `runs reconcile`, and `runs resume` for recorded runs. Retrieve supported provider results before considering a paid retry. Never silently retry an unknown paid request; `--retry-unknown` requires explicit user authorization. Reviews record quality separately from generation success. Review does not automatically revise, and revisions do not replace canonical character references.

Read [remediation workflows](references/remediation.md) for v2 casts, selected planner inputs, bounded verification, reference storage, recovered-result attachment, usage, and migrations. Jobs/config/runs accept v1 and v2; new casts and podcast assembly require v2. Other manifest versions remain unchanged.

Use `plan --character MANIFEST --asset FILE` to select dependencies explicitly; the binary copies them into `_inputs/`. For a scene with multiple saved characters, use `cast` entries and job-specific direction files. Require accepted references and compatible original model lineages; never rewrite profiles to make a conflict disappear.

For acceptance tests, preview `verify --suite SUITE --json` and obtain approval for the bounded paid batch before `--execute --preview-hash HASH`. Missing credentials are skipped, not passed. Voice creation/cloning use their separate explicit persona commands and approvals. `review report` compares generated artifacts and saved references; only actual user selections authorize canonical acceptance.

`models --live` reads provider catalogs; visibility is not generation entitlement. `usage` reports returned units, labelled list-price estimates from the dated bundled table (`--price-table FILE` overrides it) and unknown costs honestly; estimates never replace a provider-reported cost. `storage plan --output cleanup.yaml` prepares a conservative cleanup preview outside framechord/; `storage clean --plan cleanup.yaml` only removes unreferenced blobs after revalidation. `migrate --kind job|config|run --file FILE` previews; `--apply` preserves a v1 backup. For uncertain jobs, `runs recover` requires the recorded fingerprint and reconciliation evidence, then `runs resume` publishes the supplied response without regeneration.
