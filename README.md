# FrameChord

A local Rust CLI that lets coding agents, and the people working with them, generate the media a software project needs: images, infographics with exact copy, narration, podcasts, audiobooks, music and sound effects. It runs where the agent runs, keeps inputs and receipts as files in the repository, and bills nothing itself. Characters are optional: save a visual identity and a provider voice binding when you need reuse.

## Why FrameChord

- **Provider list prices per asset, no subscription.** A 1K illustration on the default Google model is $0.0336, ten seconds of narration $0.00225, a music bed $0.08 (official rates read on 2026-10-15). A month of 40 images, 30 minutes of narration, 4 beds and 20 effects costs about $2 to $4 at list price, against $19 to $20 a month for a Higgsfield or Magnific plan. See [what it costs](docs/COST.md), including when a subscription tool is the better choice.
- **It runs where your agent runs.** Codex, Claude Code, Cursor, Gemini CLI, OpenCode and GitHub Copilot discover the installed skill; any other process calls the binary with `--json`. See [using it in your project](docs/INTEGRATION.md).
- **Files, not a service.** YAML and Markdown inputs, durable run records, exact receipts and labelled `usage` estimates live in the repository; credentials stay in `.env` or the process environment.
- **Honest limits.** No video, no upscaling, and character likeness still needs human review.

Open the [web guide](https://framechord.com/) for copyable examples for every media
workflow, [prompts to hand your agent](https://framechord.com/#delegate), downloadable
YAML/Markdown inputs, and a searchable command reference. Agents can read the same guide
as [Markdown](https://framechord.com/index.md), indexed by
[llms.txt](https://framechord.com/llms.txt). It is a single HTML file with embedded JavaScript, compiled Tailwind CSS, command
help and example downloads. You can also [open the HTML locally](docs/site/index.html);
see [publishing and preview instructions](docs/WEB_GUIDE.md).

The first-version workflows are implemented; visual character consistency has not passed user acceptance. Offline contracts and mocked
execution tests pass, alongside selected bounded live checks across all five providers. It includes
voice design/cloning, authorized Fish recording synthesis, audio/image AI review,
Google request retrieval, interactive configuration, and native local packaging.
Provider voice creation/cloning and recording synthesis have offline coverage;
live testing still needs selected recordings and actual speaker authorization.
Human character consistency and media quality checks remain separate. See
[implementation status](docs/STATUS.md) for exact boundaries.

## What you can create

| Use case | Inputs | Outputs |
| --- | --- | --- |
| Article or story illustration | Image brief; optional accepted character references | PNG/JPEG/WebP supported by the selected model |
| Infographic | Structured YAML with exact copy/data plus a visual brief | Raster infographic requiring typography/data review |
| Read an article aloud | Local text file and a selected voice | Narration file; original clips retained for chunked speech |
| Podcast | YAML with ordered scripts and voices per turn | Individual clips and a combined WAV |
| Audiobook, by chapter or the full book | A Markdown manuscript split at headings, or a chapter list, read by one narrator | One WAV or MP3 per chapter, optionally the full book with ID3 chapter markers, a chapter index and a .cue sheet |
| Background music or a vocal jingle | Music brief, duration, instrumental/vocal choice, exact lyrics when supplied | Music file |
| App or scene sound effects | Event/texture/timing brief and duration | Sound-effect file |
| Recurring characters | Portable persona, visual/voice profiles and accepted references | Reusable inputs across jobs and projects |

Google is the default for supported image, speech, podcast, audiobook, music and text planning
workflows. OpenAI is an explicit alternative for images, speech and planning.
Fish supports speech and authorized voice references; ElevenLabs supports speech,
music, sound effects and explicit voice workflows. OpenRouter provides compatible
text planning and image-review models. Use `framechord models` to inspect the
implemented capability registry; account access is checked separately. Video is
outside the current release.

## Quick start: from installation to your first asset

```sh
mkdir my-media-project
cd my-media-project
framechord init
framechord config status --json
framechord doctor
framechord generate image --prompt 'A friendly woodland library, warm watercolor' \
  --output library.png --dry-run --json
```

Choose providers, voices, limits and optional keys in the setup checklist, then
review and save. Alternatively, edit the project `.env` after initialization and
run `init` again. The image preview above makes no network call. Remove
`--dry-run` when ready to submit a potentially billable provider request. A
successful result reports the final artifact paths and durable run ID.

Credential names are `GEMINI_API_KEY`, `OPENAI_API_KEY`, `FISH_API_KEY`,
`OPENROUTER_API_KEY` and `ELEVENLABS_API_KEY`. Only configure the providers you
intend to use. Process environment values take precedence over the project's
`.env`; unrelated ancestor projects are never searched for credentials.

## Hand it to your agent

Once the binary is installed, you do not need to learn the flags. `framechord init`
installs the project skill that Claude Code, Codex, Cursor, Gemini CLI, OpenCode and
GitHub Copilot discover. Open your agent in the same folder and describe the media in
plain words. The agent writes the YAML job and Markdown brief, runs `validate` and a
free `--dry-run` that shows the provider, model and outputs, and should generate only
after you say go. A dry run does not price the request; compare it with the
[dated rates](docs/COST.md), and `framechord usage` reports labelled list-price
estimates after a run. Start with a check that generates nothing:

> Use the framechord skill. Check this project's FrameChord setup: run framechord config
> status and framechord doctor, tell me which providers and media types are ready, and
> what I still need to configure. Do not generate anything.

Then ask for real work, naming your files, destinations and anything that must stay exact:

> Read posts/launch.md. Use FrameChord to create a 16:9 cover image and one inline
> illustration that match the post. Write the briefs, validate them and show me the dry
> run. Generate only after I say go, save the images to public/images/launch/ and add
> them to the post.

> Turn manuscript/my-novel.md into an audiobook with FrameChord, one chapter per
> `#` heading, read by a single warm narrator. Do not change any words. Validate it,
> show me the chapter list, the totals and whether it fits my limits, then generate
> chapter 1 only to audio/my-novel/ as MP3 so I can listen before the rest.

> Write a 90-second, two-host podcast script from the 2.3 section of CHANGELOG.md and
> show it to me before any audio is made. After I approve the script, author a FrameChord
> podcast job with two distinct voices, preview it, and generate it to
> media/release-2.3.mp3 when I say go.

Every image, voice and note in the [web guide](https://framechord.com/#showcase) came from
one such request to a coding agent. The guide's
[Hand it to your agent](https://framechord.com/#delegate) section has thirteen copyable
prompts by use case, from app sound effects to a consistent mascot, and the case study of
how the guide's own media was made. Agents can read the whole guide as Markdown at
<https://framechord.com/index.md>; <https://framechord.com/llms.txt> indexes it.

## Use it in your own project

Run `framechord init` at the root of an existing repository, or `init ./apps/site`
inside a monorepo. Commit `framechord/config.yaml`, the templates, characters and
jobs; `init` appends ignore rules for `.env`, runs, cache, blobs and the lock file to
the root `.gitignore` and leaves every other line alone. Call the binary from any
script with `--json --non-interactive`: the result is a JSON envelope on stdout,
progress goes to stderr, and the exit code says what happened (0 success, 2 invalid
input, 4 partial batch, 6 limit reached, 7 outcome unknown). The
[integration guide](docs/INTEGRATION.md) has the full exit-code table, Python and
Node examples, and the discovery paths of every supported coding agent.

## Guided setup and current models

Run `framechord init` for a looping, offline terminal setup. Use arrows, Enter and
search to choose options, and Escape to return. New projects default to Google:
Nano Banana 2.1 for images/infographics, Gemini 3.8 Flash TTS with built-in `Kore`
for speech/podcasts, Lyria 3.5 for music, and Gemini 3.8 Flash for planning.
Set `GEMINI_API_KEY` to make these defaults locally ready. OpenAI is the second
welcome option (`--preset openai`), with built-in `coral` for speech; music and
effects are disabled in that preset. Standalone sound effects remain an optional
ElevenLabs feature. Provider changes are always explicit.

Init installs Codex and Claude Code skills inside the project by default; those two
directories are also read by Cursor, Gemini CLI, OpenCode and GitHub Copilot.
`--agents cursor,gemini,opencode,copilot` installs native directories instead and
`--agents none` installs no skill. It reads
the chosen project's `.env`, creates it if absent, and appends missing API key
variable names with empty values. Existing keys, comments and unrelated content
are preserved. Optional masked key entry saves only on review. Shell credentials
take precedence and are never copied into `.env`. With no destination, setup uses
the directory where the command is called; `init PATH` or `--project PATH` selects
another project and its own `.env`.

Finish requires every enabled feature to be locally ready. Save unfinished setup
and run `init` again to resume. Existing character identities are preserved.
`config status --json` reports missing steps without network calls or secrets.

```sh
framechord init
framechord init ./project --preset recommended --json --non-interactive
framechord init ./openai-project --preset openai --json --non-interactive
framechord --project ./project config status --json
framechord models --media image --recommended
framechord models --provider google --media speech
framechord models --all
```

If the downloaded executable is in the current folder, run `./framechord init ./my-project`.
The executable and the required `framechord/` configuration directory cannot share
the same path. Setup offers another folder before collecting choices when this
conflict exists. `framechord --project ./my-project init` also selects that project;
an explicit `init PATH` takes precedence.

`--preset empty` creates an empty scaffold; `--defaults-file FILE` uses that exact
configuration. Only reviewed current stable model families can submit new provider
requests. Superseded/preview models remain readable in old projects, but cannot
create new generations or retries. A pinned old character needs a new character
version and human-accepted references; models are never silently substituted.
See [setup and model policy](assets/skill/references/setup.md) for the complete
human and agent workflow, optional live model discovery and credential handling.

## Install and update

macOS/Linux:

```sh
curl -fsSL https://framechord.com/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://framechord.com/install.ps1 | iex
```

Already downloaded a 0.6.1+ archive? Extract it, then run `./framechord install`
(Windows: `./framechord.exe install`). If you downloaded a raw executable with a
target suffix, run that exact filename followed by `install`; installation gives
it the standard `framechord` name. Installation uses a user bin directory, verifies
release bytes, and needs no administrator or Rust installation. Open a new terminal,
then run `framechord init` inside your project folder, or `framechord init ./my-project` to create another project.

```sh
framechord update --check
framechord update
framechord update --rollback
```

On shared networks, update checks can use optional process-only `GH_TOKEN` or
`GITHUB_TOKEN` authentication for release metadata. CI uses a read-only workflow
token; binary downloads never receive it.

Updates preserve project keys, configuration, characters, generated media and
installed skills. See [installation options](docs/INSTALLATION.md) for exact
versions, custom directories, PATH controls and agent-readable results. Public
binaries for macOS Apple Silicon, Linux x86-64 and Windows x86-64 are on
[GitHub Releases](https://github.com/maxiplux/framechord/releases/latest).

## Build and use

```sh
cargo build --locked
./target/debug/framechord --help
./target/debug/framechord models --json
```

Install the executable separately when wanted: `cargo install --path . --locked`. It embeds its templates, skill instructions, generated command reference, and schemas; an installed binary does not need this checkout to initialize another project.

Quick mode needs no initialization. Choose an explicit model and output:

```sh
framechord generate image --provider openai --model gpt-image-2.5-flare \
  --prompt 'A teal leaf on a cream background' --output leaf.png --dry-run --json
```

Remove `--dry-run` for an authorized generation request. Put credentials in the target project's `.env` or process environment. `.env.example` lists names without values. Process values take precedence; ancestor project files are never loaded.

```sh
framechord generate speech --provider openai --model gpt-4o-mini-tts \
  --voice alloy --text-file article.txt --output narration.wav
framechord generate music --provider elevenlabs --model music_v2_5 \
  --brief-file music.md --duration-seconds 10 --instrumental --output intro.mp3
framechord generate sfx --provider elevenlabs --model eleven_text_to_sound_v2 \
  --brief-file sfx.md --duration-seconds 3 --output bell.mp3
```

Speech retains exact file content. Long OpenAI and ElevenLabs scripts split at documented model character limits and character-safe boundaries, preserve every character, and assemble into a mono 24kHz PCM16 WAV. Original provider clips remain in the run. Articles must be local text files; adapting an article into a conversation is an explicit authoring/planning step.

## Examples for every media workflow

The following examples run inside an initialized project and use its saved
Google defaults unless a provider/model is explicit. Create the input files
shown or copy the linked templates and edit them first. Every generation example
uses `--dry-run`: remove it to execute the reviewed request. Generation, text
planning, AI review and voice creation can incur provider charges.

### Images

```sh
framechord generate image \
  --prompt 'A watercolor woodland library, wide composition, no text' \
  --output library.png --dry-run --json
```

For detailed art direction, use `--brief-file image.md` with the
[image template](assets/templates/image.md). Use `--reference` for general
content and `--identity-reference` for selected character identities with known
original-model provenance. Use `--character` for a saved character profile.

### Infographics with exact copy and data

Export templates, then edit `authoring/infographic.yaml` and its referenced
`authoring/infographic.md`:

```sh
framechord templates export authoring
framechord validate --kind infographic authoring/infographic.yaml --json
framechord generate infographic --spec authoring/infographic.yaml \
  --output writing-habits.png --dry-run --json
```

The [structured spec](assets/templates/infographic.yaml) controls the title,
audience, key message, canvas, layout, ordered sections, exact copy and numeric
strings. The [visual brief](assets/templates/infographic.md) supplies visual
hierarchy and style. Include sources and units for data; inspect the rendered
text and numbers before accepting the image.

### Narration or reading an article

Save only the exact spoken words in `article.txt`; delivery instructions belong
in a separate brief or voice profile.

```sh
framechord generate speech --text-file article.txt \
  --output narration.wav --dry-run --json
```

To choose a provider explicitly:

```sh
framechord generate speech --provider openai --model gpt-4o-mini-tts \
  --voice coral --text-file article.txt --output narration.wav --dry-run --json
```

This reads a local file; it does not fetch article URLs. Long scripts on supported
chunking models retain the exact text and save both chunks and assembled audio.

### Two-speaker podcast

Create `episode.yaml`, `host.txt`, `guest.txt` and `delivery.md` in the same
folder. Put exact dialogue in the text files and delivery guidance in Markdown.
This example inherits the configured Google speech model and explicitly selects
a built-in voice for each turn:

```yaml
schema_version: 2
id: woodland-episode
media: podcast
brief: delivery.md
locale: en-US
turns:
  - id: host-intro
    text_file: host.txt
    settings: {voice: Kore}
  - id: guest-answer
    text_file: guest.txt
    settings: {voice: Puck}
assembly:
  pause_ms: 250
```

```sh
framechord validate --kind job episode.yaml --json
framechord generate --job episode.yaml --output-dir episode-audio --dry-run --json
```

Paths are relative to `episode.yaml`. Original turn clips and the combined mono
24 kHz PCM16 WAV remain available. For saved characters, add a `character` path
per turn; their pinned voice bindings take precedence over unrelated defaults.
The [podcast template](assets/templates/podcast.yaml) also documents optional
turn gains and loudness targets. Music beds are separate assets; automatic music
mixing into the podcast is not part of this command.

### Audiobook, chapter by chapter or the full book

An audiobook is one narrator reading exact text; it is not a podcast. Put the book
in one Markdown file with a `#` heading per chapter (YAML front matter is skipped),
then copy the [audiobook template](assets/templates/audiobook.yaml):

```yaml
schema_version: 2
id: my-novel
media: audiobook
brief: narration.md          # delivery direction, never spoken
manuscript: my-novel.md
split: {heading_level: 1}
locale: en-US
assembly: {pause_ms: 2000, loudness_lufs: -19}
audiobook:
  output_mode: full          # or chapters: one file per chapter only
  format: mp3                # or wav
  announce_titles: true
  title: My Novel
  author: Jane Writer
```

```sh
framechord validate --kind job my-novel.yaml
framechord generate --job my-novel.yaml --chapter chapter-1 --output-dir audio --dry-run --json
framechord generate --job my-novel.yaml --output-dir audio --dry-run --json
```

`validate` prints the chapter table with source lines, characters, request parts and
the first and last words of each chapter, plus warnings for Markdown a speech model
might read aloud. To list chapters explicitly instead, use
[audiobook-chapters.yaml](assets/templates/audiobook-chapters.yaml). Outputs land in
`audio/my-novel/`: `chapter-01-<id>.mp3` and so on, and with `output_mode: full`
`my-novel.mp3` with ID3 chapter markers, `my-novel.chapters.json` and `my-novel.cue`.
`--chapter` produces only the chosen chapters; a later run without it reuses them
from cache and assembles the book. A whole book usually exceeds the default run
limits; generation then stops before any request and names the limits to raise.
`revise --chapter ID` re-reads one chapter and reassembles the book. MP3 encoding
uses the bundled LAME library (LGPL; `framechord notices`). Audio is 24 kHz mono;
WAV books stop at about 24.8 hours.

### Instrumental music or a vocal jingle

Write `music.md` with purpose, genre, tempo, instruments, mood and structure using
the [music template](assets/templates/music.md).

```sh
framechord generate music --brief-file music.md --duration-seconds 60 \
  --instrumental --output background.mp3 --dry-run --json
framechord generate music --brief-file jingle.md --duration-seconds 60 \
  --vocal --output jingle.mp3 --dry-run --json
```

Put requested lyrics verbatim in the vocal brief. Duration and formats depend on
the selected model and are validated before submission. A musical singer is not
guaranteed to match a character's speaking voice.

### Sound effects

Sound effects use an explicit ElevenLabs selection because the Google preset
leaves standalone effects disabled. Write the source, environment, texture and
timing in `sfx.md` using the [sound-effect template](assets/templates/sfx.md).

```sh
framechord generate sfx --provider elevenlabs --model eleven_text_to_sound_v2 \
  --brief-file sfx.md --duration-seconds 3 --output notification.mp3 \
  --dry-run --json
```

### Ask an agent to author the files

After `init`, Codex, Claude Code, Cursor, Gemini CLI, OpenCode and GitHub Copilot
discover the project-local FrameChord skill. Other agents can use
`framechord skills export DIRECTORY` and run the same binary; see the
[integration guide](docs/INTEGRATION.md) and the prompts in
[Hand it to your agent](#hand-it-to-your-agent). For example, ask your agent:

> Use FrameChord to prepare a two-speaker podcast from article.txt, an infographic
> with the article's exact figures, and a 60-second instrumental intro. Use the
> project's selected providers and voices. Author separate YAML/Markdown inputs,
> validate them, and show dry-run results and output destinations for review.

The standalone planner can also author reviewable files using the configured
text model:

```sh
framechord plan --kind infographic \
  --prompt 'Explain three habits for clearer writing; avoid invented statistics' \
  --output-dir planned-infographic --json --non-interactive
```

Planning is a separate, potentially billable text-model call and has no dry-run
flag. It saves files for review and never implicitly generates media or saves a
voice. Validate and preview its generated job before execution.

## Check API connections

`framechord doctor` checks key presence offline. Use `framechord doctor --live` to
check all five providers with read-only authenticated requests, or add
`--provider openai,google --timeout-seconds 10 --json` for selected services and
agent-readable results. Failed checks retain every provider's status and exit nonzero.
No media is generated. See [connection checks](docs/CONNECTIONS.md) for permission
caveats and limits: authentication alone does not prove model or billing readiness.

## Project workflow

```sh
framechord init /path/to/project
# Agents: export/edit project-config.yaml, then initialize without prompts:
# framechord init /path/to/project --defaults-file project-config.yaml --json --non-interactive
framechord --project /path/to/project config show --json
framechord --project /path/to/project config setup --file defaults.yaml
# Terminal-only offline assistance: framechord config wizard
framechord templates export authoring
framechord validate --kind job authoring/job.yaml --json
framechord generate --job authoring/job.yaml --dry-run --json
framechord batch --manifest batch.yaml --output-dir exports --async --json
framechord runs status RUN_ID --json
framechord runs wait RUN_ID
```

Initialization is offline and preserves existing configuration, `.env`, authored files, and modified skills. A fresh interactive terminal opens setup for per-media models, output settings, voices, locales, consistency, and limits; agents supply the complete config with `--defaults-file`. The bundled `project-config.yaml` is an editable example. Without supplied choices, noninteractive init saves the Google preset and reports missing credentials; use `--preset empty` for an unconfigured scaffold. Initial and newly created personas inherit configured image and speech profiles; absent choices leave those profiles disabled. Explicit job flags override compatible settings without rewriting source YAML, while selected original identity assets and pinned characters reject conflicting provider/model changes. Podcast defaults supply unconfigured turns; explicit turn models and characters retain their own bindings. Completed narrator, visual mascot, and combined examples are bundled under `templates/examples/`.

The visible `framechord/` folder holds YAML configuration, Markdown/YAML authoring templates, portable character folders, job briefs, and run records. Relative paths resolve from their containing manifest. Generated run artifacts and credentials are ignored; authored definitions can be versioned.

```text
my-media-project/
  .env                         # Ignored credentials
  .env.example                 # Names only
  framechord/
    config.yaml                # Models, voices, settings and limits
    templates/                 # Editable YAML/Markdown scaffolds
    characters/<id>/           # Identity, profiles and local references
    jobs/<id>/                 # Authored jobs and briefs
    runs/<run-id>/              # Durable status, receipts and outputs
  .agents/skills/framechord/    # Codex project skill
  .claude/skills/framechord/    # Claude Code project skill
```

Infographics use `infographic.yaml` plus a visual Markdown brief. Exact copy, numeric strings, units, source notes, ordered sections, canvas, and layout compile into the saved prompt. Reference objects specify `path` and `role` (`identity`, `style`, `layout`, or `content`). Rendered typography and data accuracy still need review.

## Parallel jobs, progress and recovery

Save individual YAML jobs first. For example, a `batch.yaml` beside the job folders
can contain:

```yaml
schema_version: 1
jobs:
  - framechord/jobs/header/job.yaml
  - framechord/jobs/narration/job.yaml
```

```sh
framechord validate --kind batch batch.yaml --json
framechord batch --manifest batch.yaml --output-dir exports --dry-run --json
# Submit the reviewed batch and return a durable run ID:
framechord batch --manifest batch.yaml --output-dir exports --async --json
framechord runs status RUN_ID --json
framechord runs wait RUN_ID
framechord runs resume RUN_ID --json
```

Replace `RUN_ID` with the ID returned by submission. Independent jobs run
concurrently within global, provider and shared host ceilings. Dependencies wait
for their prerequisites. In an interactive terminal, generation shows colors and
progress; agent JSON output contains no animation. Successful artifacts survive a
partial failure and are reused on resume. Inspect an uncertain submission before
explicitly choosing `--retry-unknown`: it may already have incurred charges.

## Identity, review, and revisions

Derived identity images keep the original asset’s provider and exact model, resolved from hash-verified local receipts or declared origin metadata. Project defaults cannot replace that selection. Common original settings carry forward, with explicit output-setting changes allowed. Accepted character references retain portable model/settings/hash provenance. New projects require known original identity models; identical models alone still do not guarantee likeness.

```sh
framechord persona add mira
framechord persona show mira --json
# Edit persona.md, visual.md, voice.md, and enable explicit profiles in character.yaml.
framechord persona bootstrap mira --output-dir candidates
framechord persona accept mira --kind image --reference candidates/mira-candidate.png
framechord persona export mira --destination mira-bundle
framechord persona import mira-bundle
framechord review check --artifact ARTIFACT_ID --json
framechord review record --file review.yaml --json
framechord review status --artifact ARTIFACT_ID --json
framechord revise --artifact ARTIFACT_ID --instructions-file revision.md --async --json
```

For the same characters in another scene, use selected identity references:

```sh
framechord generate image --provider openai --model gpt-image-2.5-flare \
  --identity-reference mira.png --identity-reference kito.png \
  --brief-file forest.md --output forest.png --dry-run --json
```

Current GPT Image 2.5 models omit input fidelity. Historical supported models
recorded high input fidelity for identity references and rejected low-only editing.
Output quality controls cannot guarantee likeness. Compare faces/body proportions
with the original references and record drift as `needs_revision`. See the
[continuity workflow](assets/skill/references/consistency.md).

Import into a different project or use `--id` to resolve a conflict. Provider voice IDs are account-specific bindings, not universal portable voices. Accepted references and pinned identity settings are reused; generation never promotes new references automatically. Singing follows its music brief and is not promised to match a speaking voice.

Use `framechord voices --provider elevenlabs --account --json` to inspect bounded pages of account voices. Add `--search`, `--language`, `--page-size`, and `--max-pages` for discovery; continuation tokens/page numbers are returned. Google and Fish account catalogs are also supported; `--public` explicitly selects Fish licensed public voices. Catalog lookup does not create or clone a voice.

For a character with an explicit ElevenLabs speech profile, descriptive voice design uses a separate selected model and exact preview script:

```sh
framechord persona voice-design narrator --model eleven_ttv_v3 \
  --text-file preview.md --run-id narrator-design-1 --dry-run --json
framechord persona voice-status narrator-design-1 --json
framechord persona voice-create narrator --design-run narrator-design-1 \
  --candidate SELECTED_CANDIDATE_ID --name Narrator --bind --dry-run --json
```

Remove `--dry-run` for authorized preview generation or provider-side voice creation. `voice-design` keeps up to three local candidates; it does not create a saved voice or change the character. `voice-create` creates exactly the selected candidate; `--bind` is separate, and replacing an existing binding requires `--replace-binding`. Canonical reference recordings require separate acceptance. Changed identity or unconfirmed provider verification prevents binding. After satisfying provider requirements, `--refresh-verification --bind` retrieves current metadata for the already-created voice. `voice-status` reports voice operation records; media `runs resume` is separate. See [voice workflow](assets/skill/references/voices.md).

Recorded-voice cloning is explicit through `persona voice-clone --request clone.yaml`;
validate and dry-run first. The bundled `voice-clone.yaml` and `voice-consent.yaml`
require selected recordings and actual granted `saved_voice_clone` permission.
Verification-aware binding stays separate from creation and reference acceptance.
For Fish synthesis without a saved voice, use `--voice-sample`,
`--sample-transcript`, and `--consent-file`, or accepted `voice.samples` in a
character. `persona accept-sample` copies the recording, transcript, permission,
and evidence into its portable folder. See the [voice workflow](assets/skill/references/voices.md).

Generation success and semantic quality are separate. Local artifact integrity is checked automatically. Explicit `review check --ai --provider ... --model ...` supports image review using OpenAI/OpenRouter multimodal text models; Google audio review supports `gemini-3.8-flash` against exact scripts, briefs, and accepted voice recordings. Inline requests must fit 20 MiB; review individual clips for larger jobs. Completed AI reviews are saved with provider receipts and reused on identical commands; unknown review outcomes require explicit `--retry-unknown`. `review reconcile --artifact ID` retrieves uncertain Google audio assessments without resubmission. Human or agent assessments require provenance. Revisions create child runs, preserve originals, and inherit budgets. `--source-job` enables explicit updated scripts/copy; `--turn` targets a podcast turn and reuses unchanged clips; `--chapter` targets an audiobook chapter and reassembles the book.

## Agent integration and limits

`init` installs project-local skills in `.agents/skills/framechord/` and `.claude/skills/framechord/`, directories that Cursor, Gemini CLI, OpenCode and GitHub Copilot also read. `--agents cursor,gemini,opencode,copilot` selects native directories (`.cursor/skills`, `.gemini/skills`, `.opencode/skills`, `.github/skills`), `--agents none` writes no skill, and `config status --json` reports `skills`, `skill_targets` and `skills_shared_by`. `skills export DIRECTORY` creates a portable package for other agents. `skills install --scope user --agents codex,claude` is a separate explicit user installation. No global agent settings or permissions are rewritten. Existing bundled files are preserved by default; `skills install --overwrite` or `templates export DIRECTORY --overwrite` explicitly refreshes bundled files, including modified ones, while preserving unrelated files.

The exported skill includes all current commands/flags generated from the same CLI definitions, schemas, templates, authoring examples, lifecycle, review, progress, and limit guidance. Use `--json --non-interactive` for process callers. JSON stays on stdout; progress and diagnostics use stderr. Result views show reference metadata/hashes, while complete embedded bytes stay in run YAML. Interactive terminals get a colored job-count bar and a spinner for unknown provider completion. `--progress auto|plain|off`, `--color auto|always|never`, and `NO_COLOR` control presentation.

Default budgets per run lineage: 20 creation requests, 8 images, 1,800 reserved audio seconds, 60,000 speech characters, 3 revisions, 3 attempts per job. Default host concurrency: 4 globally, 2 per provider; per-run limits can be configured up to 32/16, while shared host ceilings also apply. `config host show` reads ceilings; explicit `config host setup --file host-limits.yaml` changes only framechord user configuration. `--limits-file` supplies explicit per-run limits. Speech duration is an estimate; overrun artifacts are retained and further requests stop when the budget is exhausted. Audiobooks are checked as a whole before the run starts, so a book that does not fit makes no request at all. These are volume controls, not a dollar cap. `framechord usage --json` reports provider-returned units beside labelled list-price estimates from the dated bundled table (`--price-table FILE` overrides it); estimates never replace a provider-reported cost, `currency_total` stays null, and unpriceable work is reported as not estimable. See [what it costs](docs/COST.md).

Requests do not silently switch providers. Matching fingerprints reuse only verified local artifacts; `--fresh` bypasses reuse. `--request-id` provides submission idempotency. Safe rejected jobs can be resumed, with provider retry guidance honored. Use `runs reconcile RUN_ID` to retrieve uncertain Google interactions by saved ID, then resume local publication. Unknown submissions without retrievable results require explicit `--retry-unknown`; they may already have been charged. Successful jobs remain intact when a batch fails.

## More documentation

- [What it costs, with dated provider rates and plan comparisons](docs/COST.md)
- [Using FrameChord in your own project and from other agents](docs/INTEGRATION.md)
- [Installation, update and rollback](docs/INSTALLATION.md)
- [Every command and flag](https://framechord.com/#commands), also exported by `framechord skills export DIRECTORY` as `references/commands.md`
- [Authoring templates and file rules](assets/skill/references/authoring.md)
- [Character continuity](assets/skill/references/consistency.md) and [voice workflows](assets/skill/references/voices.md)
- [Contracts, JSON results and lifecycle](docs/CONTRACTS.md)
- [Implementation and native release status](docs/STATUS.md), [live evidence](docs/LIVE_TESTS.md), and [acceptance boundaries](docs/ACCEPTANCE.md)

## Project history

FrameChord was MediaAgent before 0.6.1, its first release under the new name. Existing `mediaagent/` projects stay supported
in place; see [upgrade guidance](assets/skill/references/renaming.md). The documentation
domain is [framechord.com](https://framechord.com/); [DNS and hosting details](docs/DOMAIN.md)
are documented separately.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
```

Normal tests mock providers and run offline even if credentials exist. Native release CI verifies Windows x86-64, Linux x86-64 and macOS Apple Silicon, including public installation, update and offline rollback. See [version-specific results](docs/STATUS.md) for executed checks. Intel Mac release binaries are not produced.

`python3 scripts/live-smoke.py --provider openai --model gpt-image-2.5-flare --media image` reports skipped without generating. Add `--live` to explicitly enable at most one request using this project's credentials and strict limits. Speech also requires `--voice`. Historical Google music evidence used a preview preset; new requests must use a currently selectable model shown by `framechord models`. Returned duration is measured separately and can differ from the requested duration. Skipped credentials or unavailable account capabilities are never represented as passed live tests.

`python3 scripts/live-voice-design.py --model eleven_ttv_v3 --speech-model eleven_flash_v2_5 --live` makes one bounded request for up to three fictional narrator previews. It snapshots its temporary character inside the ignored run, preserves `.env`, and never creates or binds a provider voice. Omit `--live` to report skipped.

`python3 scripts/live-recording.py --mode synthesis` or `--mode clone` reports skipped by default. Live recording tests require explicitly selected local audio, actual permission/evidence, and `--live`; cloning creates an account voice without canonical binding or automatic reference acceptance.

`python3 scripts/live-audio-review.py --source-run RUN_ID --model gemini-3.8-flash --live` reviews a verified cached short speech clip with one assessment request; it preserves the source generation.

A local native binary can be packaged with `python3 scripts/package-release.py --offline`; the packager verifies it outside this checkout without provider calls. See [packaging](docs/PACKAGING.md) for archive contents, release verification and supported native targets.

See [live test evidence](docs/LIVE_TESTS.md) for the exact models, runs, and verification limits.

See [contracts](docs/CONTRACTS.md) for lifecycle and result guarantees and `framechord schema KIND` for actual JSON Schemas.


## Remediation workflows

Use `verify --suite suite.yaml --json` to preview bounded acceptance tests without
provider calls. Execute only an approved preview hash. New v2 jobs support a cast
of saved characters and optional podcast assembly. `plan --character ... --asset ...`
copies selected dependencies into a contained bundle. `models --live` reads catalogs;
`usage`, `storage plan`, `migrate`, `runs recover`, and `review report` support local
operation and review. See [workflow details](docs/REMEDIATION.md) and
[acceptance gates and prepared batches](docs/ACCEPTANCE.md).
