# Authoring

YAML is structured state; Markdown holds exact scripts and readable creative direction. Paths are relative to the manifest containing them. Schemas reject unknown fields. Jobs, configuration and runs support v2 with v1 readers retained; character, infographic and review contracts remain v1. The binary treats both formats as data.

Use `templates list`, `templates show NAME`, or `templates export DIRECTORY`. Edit exported scaffolds before submission. Every template filename is available through these commands; skill exports include identical copies.

Characters: `persona add ID` creates persona.md, visual.md, voice.md, and character.yaml. Fill identity first (motivation, values, background); visual second (age range, proportions, face, hair, colors, distinguishing features, stable wardrobe); voice third (language/accent, pace, pitch, timbre, pronunciation, emotion baseline). Enable only the profiles needed. Choose an explicit provider/model per enabled profile. Save a provider voice binding for speech. Use `voices --provider PROVIDER` for built-in names or `--account` for bounded, filterable pages from Google, Fish, or ElevenLabs. Fish `--account --public` selects licensed public voices explicitly. Account lookup does not design or clone a voice. For explicit ElevenLabs descriptive preview design and selected voice creation/binding, use the [voice workflow](voices.md). Recorded-voice cloning is implemented for ElevenLabs, and synthesis from authorized recordings with exact transcripts is implemented for Fish. Their selection, consent, verification, and canonical acceptance steps are documented in the voice workflow. Visual/voice-only characters work independently.

Bootstrap candidate images with `persona bootstrap ...`; review them, then `persona accept ... --reference FILE --kind image`. Existing reference recordings can be accepted explicitly with kind audio, but an adapter may require a provider voice ID instead. Routine outputs never become canonical references. `persona export` creates a portable directory; `persona import` detects conflicts and validates every bundled dependency. Accessible provider bindings still need verification in the destination account.

Image briefs: goal, scene, subject, framing, lighting, palette, intended use, exclusions. Reusing selected characters requires identity references (`--identity-reference` or `{path, role: identity}`); see [character continuity](consistency.md). Bare paths retain a generic content role. Put job-specific pose/expression/outfit here; do not silently edit canonical identity.

Infographics: fill infographic.yaml plus infographic.md. Supply exact title, audience, key_message, canvas, layout, ordered sections, exact copy, and optional values with units and source notes. Values are strings to retain formatting. Use provided facts; flag missing facts instead of inventing them. The compiler preserves this structured source and emits explicit typography, hierarchy, reading-order, and copy constraints. Raster text still needs visual review.

Speech: script.md contains only exact spoken words. Voice/delivery direction belongs in the brief/profile, not embedded commentary in the script. For articles, read the supplied local text; there is no URL fetching. Podcast jobs contain ordered turns, each with text_file, optional character, selection, and settings. Original turn clips and a mono 24kHz PCM16 combined WAV are retained.

Audiobooks: one narrator reads exact source text. Either point `manuscript` at one Markdown file and set `split.heading_level` (chapters start at `#` headings at or above that level, outside code fences; YAML front matter is skipped), or list `chapters` with an `id`, optional `title`, and `text_file` each. Use the `audiobook.yaml` or `audiobook-chapters.yaml` template. Headings are not read unless `audiobook.announce_titles` is true. Put delivery direction in `brief` (never spoken). The narrator comes from `character`, `selection`, or the audiobook/speech default; chapters cannot change it. See [audiobooks](audiobooks.md).

Music: define use, genre, tempo, instruments, mood, duration, instrumental/vocal choice; quote requested lyrics exactly. The singer is not guaranteed to match a character's speaking voice. SFX: source/event, texture, environment, timing, duration, and exclusions. Set explicit bounded duration for both. Choose `--instrumental` or `--vocal` for music; they are mutually exclusive. Long OpenAI and supported ElevenLabs speech is split at documented model limits without changing script content and assembled natively into WAV.

Initialization can configure the complete authoring baseline before creating the
initial persona: `init PATH --interactive` for a terminal, or
`init PATH --defaults-file project-config.yaml --json --non-interactive` for an
agent. Copy/edit `templates/project-config.yaml` first: its explicit model IDs are
examples, not automatic account entitlement. It covers per-media `defaults`,
`default_settings`, `default_locales`, `consistency`, and `limits`. The wizard offers
shared image/infographic choices while allowing separate configuration. The
initial persona takes the configured image model/settings and speech model/voice;
unconfigured profiles remain disabled. Future persona creation uses saved defaults.
Repeated init preserves existing profiles, config, credentials, and authored files;
conflicting supplied config is rejected. Use `config setup --file` or `config wizard`
for an explicit later defaults update; existing characters retain their bindings.
