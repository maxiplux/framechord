# framechord 0.7.0 command reference

Generated from the executable's command definitions. Global flags are accepted before or after the command.

## framechord

```text
Generate images, infographics, exact speech, podcasts, music, and sound effects. Use direct flags for one asset or versioned YAML jobs for repeated work. Run --help on any command for inputs, output destinations, and examples. Initialization and validation are offline.

Usage: framechord [OPTIONS] <COMMAND>

Commands:
  install    Install this downloaded binary in your user bin directory, offline and without administrator access
  update     Update this executable from a verified public GitHub Release; optional GH_TOKEN/GITHUB_TOKEN authenticates metadata checks
  init       Initialize offline, preserving existing files and credentials
  config     Inspect or explicitly replace validated configuration
  verify     Preview a bounded live verification suite; execution requires the exact approved preview hash
  usage      Report provider-returned usage separately from unknown costs
  storage    Inspect storage or apply a previously reviewed conservative cleanup plan
  migrate    Preview migration of a v1 job, config or run; --apply creates a non-overwriting backup
  doctor     Inspect credentials offline, or explicitly check all provider endpoints and keys with --live
  notices    Print third-party license notices for components bundled in this binary
  schema     Print a JSON Schema for a versioned manifest
  models     List implemented provider/model capabilities (not account entitlements)
  voices     List built-in voices or search bounded pages of provider account voices
  templates  Inspect or export embedded authoring templates
  persona    Create, inspect, bootstrap, accept, or port character definitions
  validate   Validate local YAML and all referenced files; never generate
  generate   Generate one media job. Example: generate image --prompt 'A forest' --provider openai --model gpt-image-2.5-flare --output forest.png
  batch      Execute independent jobs concurrently using a YAML batch manifest
  runs       Inspect, wait, cancel, or resume durable runs
  resume     Continue a recorded run, preserving successes; ambiguous retries are explicit
  review     Check artifact integrity, request explicit image/audio AI review, or record a human/agent review
  revise     Create a revision child run with inherited identity and limits
  plan       Explicit text-model planning; saves reviewable files without generating media
  skills     Export a portable skill or install it in agent discovery locations
  help       Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

```

## framechord install

```text
Install this downloaded binary in your user bin directory, offline and without administrator access

Usage: framechord install [OPTIONS]

Options:
      --bin-dir <BIN_DIR>
          Executable destination directory (default: ~/.local/bin; Windows: LOCALAPPDATA/framechord/bin)

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --no-modify-path
          Do not add the directory to your user PATH or shell profile

      --force
          Explicitly replace an existing executable not managed by framechord

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord update

```text
Update this executable from a verified public GitHub Release; optional GH_TOKEN/GITHUB_TOKEN authenticates metadata checks

Usage: framechord update [OPTIONS]

Options:
      --check
          Only check release metadata; do not download or replace the executable

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --version <VERSION>
          Select an exact stable version, for example 0.4.0; defaults to the latest stable release

      --force
          Reinstall the selected version even when already current

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --allow-downgrade
          Explicitly allow installing an older stable release

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --rollback
          Restore the verified backup saved by the preceding install or update; offline

  -h, --help
          Print help

```

## framechord init

```text
Initialize offline, preserving existing files and credentials

Usage: framechord init [OPTIONS] [PATH]

Arguments:
  [PATH]
          Project folder (defaults to --project or the current directory); must allow a framechord/ subfolder

Options:
      --agents <AGENTS>
          Agent targets: codex, claude, cursor, gemini, opencode, copilot, or none (comma-separated). The default pair is already discovered by Cursor, Gemini CLI, OpenCode and Copilot

          [default: codex,claude]

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --defaults-file <DEFAULTS_FILE>
          Complete configuration YAML: provider/models, settings, locales, consistency, and limits

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --preset <PRESET>
          New-project preset: Google first, OpenAI alternative; existing choices are preserved

          Possible values:
          - recommended: Google images, speech, podcasts, music and planning
          - google:      Same as recommended: Google for supported media
          - openai:      OpenAI images, speech, podcasts and planning; music/effects optional
          - empty:       Empty scaffold without enabled generation features

      --interactive
          Configure explicit provider/model, generation settings, limits, and identity policy offline

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help (see a summary with '-h')

```

## framechord config

```text
Inspect or explicitly replace validated configuration

Usage: framechord config [OPTIONS] <COMMAND>

Commands:
  wizard  Terminal-only guided defaults; agents use setup --file instead
  host    Explicit user-scope host concurrency ceilings, independent of agent settings
  show
  status  Inspect local setup readiness without network calls or revealing credentials
  setup   Replace config only after validating the supplied file
  limits  Inspect finite generation limits
  help    Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config wizard

```text
Terminal-only guided defaults; agents use setup --file instead

Usage: framechord config wizard [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config host

```text
Explicit user-scope host concurrency ceilings, independent of agent settings

Usage: framechord config host [OPTIONS] <COMMAND>

Commands:
  show
  setup
  help   Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config host show

```text
Usage: framechord config host show [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config host setup

```text
Usage: framechord config host setup [OPTIONS] --file <FILE>

Options:
      --file <FILE>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config host help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord config host help [COMMAND]

Commands:
  show
  setup
  help   Print this message or the help of the given subcommand(s)

```

## framechord config host help show

```text
Usage: framechord config host help show

```

## framechord config host help setup

```text
Usage: framechord config host help setup

```

## framechord config host help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord config host help help

```

## framechord config show

```text
Usage: framechord config show [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config status

```text
Inspect local setup readiness without network calls or revealing credentials

Usage: framechord config status [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config setup

```text
Replace config only after validating the supplied file

Usage: framechord config setup [OPTIONS] --file <FILE>

Options:
      --file <FILE>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config limits

```text
Inspect finite generation limits

Usage: framechord config limits [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord config help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord config help [COMMAND]

Commands:
  wizard  Terminal-only guided defaults; agents use setup --file instead
  host    Explicit user-scope host concurrency ceilings, independent of agent settings
  show
  status  Inspect local setup readiness without network calls or revealing credentials
  setup   Replace config only after validating the supplied file
  limits  Inspect finite generation limits
  help    Print this message or the help of the given subcommand(s)

```

## framechord config help wizard

```text
Terminal-only guided defaults; agents use setup --file instead

Usage: framechord config help wizard

```

## framechord config help host

```text
Explicit user-scope host concurrency ceilings, independent of agent settings

Usage: framechord config help host [COMMAND]

Commands:
  show
  setup

```

## framechord config help host show

```text
Usage: framechord config help host show

```

## framechord config help host setup

```text
Usage: framechord config help host setup

```

## framechord config help show

```text
Usage: framechord config help show

```

## framechord config help status

```text
Inspect local setup readiness without network calls or revealing credentials

Usage: framechord config help status

```

## framechord config help setup

```text
Replace config only after validating the supplied file

Usage: framechord config help setup

```

## framechord config help limits

```text
Inspect finite generation limits

Usage: framechord config help limits

```

## framechord config help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord config help help

```

## framechord verify

```text
Preview a bounded live verification suite; execution requires the exact approved preview hash

Usage: framechord verify [OPTIONS] --suite <SUITE>

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --suite <SUITE>


      --execute


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --preview-hash <PREVIEW_HASH>


      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord usage

```text
Report provider-returned usage separately from unknown costs

Usage: framechord usage [OPTIONS]

Options:
      --price-table <FILE>
          Explicit price table (JSON or YAML) replacing the bundled dated table; estimates stay labelled and never replace provider-reported cost

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord storage

```text
Inspect storage or apply a previously reviewed conservative cleanup plan

Usage: framechord storage [OPTIONS] <COMMAND>

Commands:
  inspect
  plan     Save a cleanup preview; only unreferenced blobs can be removed
  clean    Revalidate and apply a saved cleanup preview; fails if stale or workers active
  help     Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord storage inspect

```text
Usage: framechord storage inspect [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord storage plan

```text
Save a cleanup preview; only unreferenced blobs can be removed

Usage: framechord storage plan [OPTIONS] --output <OUTPUT>

Options:
      --output <OUTPUT>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord storage clean

```text
Revalidate and apply a saved cleanup preview; fails if stale or workers active

Usage: framechord storage clean [OPTIONS] --plan <PLAN>

Options:
      --plan <PLAN>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord storage help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord storage help [COMMAND]

Commands:
  inspect
  plan     Save a cleanup preview; only unreferenced blobs can be removed
  clean    Revalidate and apply a saved cleanup preview; fails if stale or workers active
  help     Print this message or the help of the given subcommand(s)

```

## framechord storage help inspect

```text
Usage: framechord storage help inspect

```

## framechord storage help plan

```text
Save a cleanup preview; only unreferenced blobs can be removed

Usage: framechord storage help plan

```

## framechord storage help clean

```text
Revalidate and apply a saved cleanup preview; fails if stale or workers active

Usage: framechord storage help clean

```

## framechord storage help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord storage help help

```

## framechord migrate

```text
Preview migration of a v1 job, config or run; --apply creates a non-overwriting backup

Usage: framechord migrate [OPTIONS] --kind <KIND> --file <FILE>

Options:
      --kind <KIND>
          [possible values: job, config, run]

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --file <FILE>


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --apply


      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord doctor

```text
Inspect credentials offline, or explicitly check all provider endpoints and keys with --live

Usage: framechord doctor [OPTIONS]

Options:
      --live
          Make one read-only authenticated GET per selected provider; never generate media

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --provider <PROVIDER>
          Selected providers (repeat or comma-separate); default all five, including missing keys

          [possible values: openai, google, fish, openrouter, elevenlabs]

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --timeout-seconds <TIMEOUT_SECONDS>
          Timeout per provider, including response download; 1–60 seconds, no retries

          [default: 10]

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord notices

```text
Print third-party license notices for components bundled in this binary

Usage: framechord notices [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord schema

```text
Print a JSON Schema for a versioned manifest

Usage: framechord schema [OPTIONS] <KIND>

Arguments:
  <KIND>
          [possible values: config, character, infographic, job, batch, run, artifact, review, effective, budget, revision, operation, voice_design, voice_consent, voice_clone, host_limits, diagnostics, verification, chapters]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord models

```text
List implemented provider/model capabilities (not account entitlements)

Usage: framechord models [OPTIONS]

Options:
      --live
          Explicit read-only provider catalog lookup; not a generation entitlement check

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --provider <PROVIDER>
          [possible values: openai, google, fish, openrouter, elevenlabs]

      --media <MEDIA>
          Filter by generation purpose (infographic uses image; podcast uses speech)

          [possible values: image, infographic, speech, podcast, audiobook, music, sfx, text]

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --search <SEARCH>
          Search model names, IDs and descriptions

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --recommended
          Show only recommended current supported models

      --all
          Include nonselectable historical and unsupported provider catalog entries

      --sort <SORT>
          Popularity requires --live --provider openrouter; no rankings are invented

          [default: recommended]
          [possible values: recommended, name, popular]

  -h, --help
          Print help

```

## framechord voices

```text
List built-in voices or search bounded pages of provider account voices

Usage: framechord voices [OPTIONS] --provider <PROVIDER>

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --provider <PROVIDER>
          [possible values: openai, google, fish, openrouter, elevenlabs]

      --account
          Read account voices from Google, Fish, or ElevenLabs; default one page

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --public
          Fish only: explicitly query licensed public voices instead of owned voices

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --voice-id <VOICE_ID>
          ElevenLabs only: inspect one voice binding and provider verification status

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --page-size <PAGE_SIZE>
          Maximum voices per page, 1–100

          [default: 20]

      --max-pages <MAX_PAGES>
          Maximum catalog requests, 1–20; default one

          [default: 1]

      --page-token <PAGE_TOKEN>
          Resume a Google or ElevenLabs catalog with the same filters

      --page-number <PAGE_NUMBER>
          Fish catalog page, starting at one

          [default: 1]

      --search <SEARCH>


      --language <LANGUAGE>


      --category <CATEGORY>
          Google: prebuilt/replicated/prompted; ElevenLabs: premade/cloned/generated/professional

  -h, --help
          Print help

```

## framechord templates

```text
Inspect or export embedded authoring templates

Usage: framechord templates [OPTIONS] <COMMAND>

Commands:
  list
  show
  export
  help    Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord templates list

```text
Usage: framechord templates list [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord templates show

```text
Usage: framechord templates show [OPTIONS] <NAME>

Arguments:
  <NAME>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord templates export

```text
Usage: framechord templates export [OPTIONS] <DIRECTORY>

Arguments:
  <DIRECTORY>


Options:
      --overwrite
          Explicitly replace bundled template files, preserving unrelated files

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord templates help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord templates help [COMMAND]

Commands:
  list
  show
  export
  help    Print this message or the help of the given subcommand(s)

```

## framechord templates help list

```text
Usage: framechord templates help list

```

## framechord templates help show

```text
Usage: framechord templates help show

```

## framechord templates help export

```text
Usage: framechord templates help export

```

## framechord templates help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord templates help help

```

## framechord persona

```text
Create, inspect, bootstrap, accept, or port character definitions

Usage: framechord persona [OPTIONS] <COMMAND>

Commands:
  voice-clone         Explicitly create an ElevenLabs clone from selected authorized recordings
  voice-clone-status  Inspect cloned-voice receipts and unresolved creation attempts
  voice-design        Generate ElevenLabs voice preview candidates without creating or binding a saved voice
  voice-create        Create a provider voice from one selected candidate; canonical binding is explicit
  voice-status        Inspect saved voice candidates, creation results, and uncertain operations
  add
  list
  show
  bootstrap           Generate visual candidates; this never promotes references
  accept-sample       Accept a selected recording, exact transcript, and granted consent as a portable voice sample
  accept              Accept a user-selected image or recording into the canonical profile
  export
  import
  help                Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona voice-clone

```text
Explicitly create an ElevenLabs clone from selected authorized recordings

Usage: framechord persona voice-clone [OPTIONS] --request <REQUEST>

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --request <REQUEST>


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --limits-file <LIMITS_FILE>


      --dry-run


      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --retry-unknown


      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --refresh-verification


  -h, --help
          Print help

```

## framechord persona voice-clone-status

```text
Inspect cloned-voice receipts and unresolved creation attempts

Usage: framechord persona voice-clone-status [OPTIONS] <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona voice-design

```text
Generate ElevenLabs voice preview candidates without creating or binding a saved voice

Usage: framechord persona voice-design [OPTIONS] --model <MODEL> --text-file <TEXT_FILE> <ID>

Arguments:
  <ID>


Options:
      --model <MODEL>
          Use eleven_ttv_v3 for new previews; the historical v2 value remains readable but cannot submit

          [possible values: eleven_multilingual_ttv_v2, eleven_ttv_v3]

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --text-file <TEXT_FILE>
          Exact 100–1000 character preview script

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --run-id <RUN_ID>
          Stable design run ID for recovery; omit for a new run

      --limits-file <LIMITS_FILE>


      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --dry-run


      --retry-unknown
          Explicit retry of an uncertain paid design request

  -h, --help
          Print help

```

## framechord persona voice-create

```text
Create a provider voice from one selected candidate; canonical binding is explicit

Usage: framechord persona voice-create [OPTIONS] --design-run <DESIGN_RUN> --candidate <CANDIDATE> --name <NAME> <ID>

Arguments:
  <ID>


Options:
      --design-run <DESIGN_RUN>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --candidate <CANDIDATE>


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --name <NAME>


      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --bind
          Bind the created voice ID to this character after confirmed provider verification status

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --replace-binding


      --refresh-verification
          Refresh provider verification with one metadata lookup before binding

      --dry-run


      --retry-unknown


  -h, --help
          Print help

```

## framechord persona voice-status

```text
Inspect saved voice candidates, creation results, and uncertain operations

Usage: framechord persona voice-status [OPTIONS] <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona add

```text
Usage: framechord persona add [OPTIONS] <ID>

Arguments:
  <ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona list

```text
Usage: framechord persona list [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona show

```text
Usage: framechord persona show [OPTIONS] <ID>

Arguments:
  <ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona bootstrap

```text
Generate visual candidates; this never promotes references

Usage: framechord persona bootstrap [OPTIONS] --output-dir <OUTPUT_DIR> <ID>

Arguments:
  <ID>


Options:
      --output-dir <OUTPUT_DIR>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --async


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona accept-sample

```text
Accept a selected recording, exact transcript, and granted consent as a portable voice sample

Usage: framechord persona accept-sample [OPTIONS] --audio <AUDIO> --transcript <TRANSCRIPT> --consent <CONSENT> <ID>

Arguments:
  <ID>


Options:
      --audio <AUDIO>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --transcript <TRANSCRIPT>


      --consent <CONSENT>


      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona accept

```text
Accept a user-selected image or recording into the canonical profile

Usage: framechord persona accept [OPTIONS] --reference <REFERENCE> --kind <KIND> <ID>

Arguments:
  <ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --reference <REFERENCE>


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --kind <KIND>
          [possible values: image, audio]

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona export

```text
Usage: framechord persona export [OPTIONS] --destination <DESTINATION> <ID>

Arguments:
  <ID>


Options:
      --destination <DESTINATION>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona import

```text
Usage: framechord persona import [OPTIONS] <DIRECTORY>

Arguments:
  <DIRECTORY>


Options:
      --id <ID>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord persona help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord persona help [COMMAND]

Commands:
  voice-clone         Explicitly create an ElevenLabs clone from selected authorized recordings
  voice-clone-status  Inspect cloned-voice receipts and unresolved creation attempts
  voice-design        Generate ElevenLabs voice preview candidates without creating or binding a saved voice
  voice-create        Create a provider voice from one selected candidate; canonical binding is explicit
  voice-status        Inspect saved voice candidates, creation results, and uncertain operations
  add
  list
  show
  bootstrap           Generate visual candidates; this never promotes references
  accept-sample       Accept a selected recording, exact transcript, and granted consent as a portable voice sample
  accept              Accept a user-selected image or recording into the canonical profile
  export
  import
  help                Print this message or the help of the given subcommand(s)

```

## framechord persona help voice-clone

```text
Explicitly create an ElevenLabs clone from selected authorized recordings

Usage: framechord persona help voice-clone

```

## framechord persona help voice-clone-status

```text
Inspect cloned-voice receipts and unresolved creation attempts

Usage: framechord persona help voice-clone-status

```

## framechord persona help voice-design

```text
Generate ElevenLabs voice preview candidates without creating or binding a saved voice

Usage: framechord persona help voice-design

```

## framechord persona help voice-create

```text
Create a provider voice from one selected candidate; canonical binding is explicit

Usage: framechord persona help voice-create

```

## framechord persona help voice-status

```text
Inspect saved voice candidates, creation results, and uncertain operations

Usage: framechord persona help voice-status

```

## framechord persona help add

```text
Usage: framechord persona help add

```

## framechord persona help list

```text
Usage: framechord persona help list

```

## framechord persona help show

```text
Usage: framechord persona help show

```

## framechord persona help bootstrap

```text
Generate visual candidates; this never promotes references

Usage: framechord persona help bootstrap

```

## framechord persona help accept-sample

```text
Accept a selected recording, exact transcript, and granted consent as a portable voice sample

Usage: framechord persona help accept-sample

```

## framechord persona help accept

```text
Accept a user-selected image or recording into the canonical profile

Usage: framechord persona help accept

```

## framechord persona help export

```text
Usage: framechord persona help export

```

## framechord persona help import

```text
Usage: framechord persona help import

```

## framechord persona help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord persona help help

```

## framechord validate

```text
Validate local YAML and all referenced files; never generate

Usage: framechord validate [OPTIONS] --kind <KIND> <PATH>

Arguments:
  <PATH>


Options:
      --kind <KIND>
          [possible values: config, character, job, batch, infographic, review, voice_consent, voice_clone, host_limits]

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord generate

```text
Generate one media job. Example: generate image --prompt 'A forest' --provider openai --model gpt-image-2.5-flare --output forest.png

Usage: framechord generate [OPTIONS] [MEDIA]

Arguments:
  [MEDIA]
          [possible values: image, infographic, speech, podcast, audiobook, music, sfx, text]

Options:
      --job <JOB>
          Reusable YAML job; settings/provider overrides are explicit, authoring flags conflict

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --chapter <CHAPTERS>
          Audiobook only: generate just these chapter IDs (repeatable); a later run without it assembles the book

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --identity-reference <IDENTITY_REFERENCE>
          Preserve referenced character identities; repeat for multiple selected characters

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --input-fidelity <INPUT_FIDELITY>
          Historical image-edit fidelity field; omit for current GPT Image 2.5 models

          [possible values: high, low]

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --voice-sample <VOICE_SAMPLE>
          Authorized audio sample for Fish reference synthesis or voice review evidence

      --sample-transcript <SAMPLE_TRANSCRIPT>
          Exact transcript of --voice-sample

      --consent-file <CONSENT_FILE>
          Versioned permission record and referenced evidence for --voice-sample

      --image-quality <IMAGE_QUALITY>
          OpenAI image quality: low/medium/high/auto; current 2.5 models also support xhigh/max

          [possible values: low, medium, high, auto, xhigh, max]

      --provider <PROVIDER>
          [possible values: openai, google, fish, openrouter, elevenlabs]

      --model <MODEL>


      --prompt <PROMPT>
          Creative direction (not speech script); conflicts with --brief-file

      --brief-file <BRIEF_FILE>


      --text <TEXT>
          Exact spoken words; conflicts with --text-file

      --text-file <TEXT_FILE>


      --spec <SPEC>
          Structured infographic YAML

      --character <CHARACTER>
          Character YAML; pins identity, provider/model, and accepted references

      --reference <REFERENCE>


      --voice <VOICE>


      --locale <LOCALE>


      --format <FORMAT>


      --size <SIZE>


      --duration-seconds <DURATION_SECONDS>


      --instrumental


      --vocal
          Request vocal music; conflicts with --instrumental

      --limits-file <LIMITS_FILE>
          Explicit limits YAML for this run (otherwise project limits apply)

      --output <OUTPUT>
          Final filename for a single job; mutually exclusive with --output-dir

      --output-dir <OUTPUT_DIR>
          Directory for final artifacts and podcast clips

      --overwrite
          Explicitly allow replacing a requested destination

      --dry-run
          Validate and print effective inputs without network calls or run creation

      --fresh
          Bypass verified local cache

      --async
          Start a local worker and return the durable run ID

      --request-id <REQUEST_ID>
          Repeated submissions with this ID must have identical effective inputs

  -h, --help
          Print help

```

## framechord batch

```text
Execute independent jobs concurrently using a YAML batch manifest

Usage: framechord batch [OPTIONS] --manifest <MANIFEST>

Options:
      --manifest <MANIFEST>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --limits-file <LIMITS_FILE>
          Explicit limits YAML for this run (otherwise project limits apply)

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --output <OUTPUT>
          Final filename for a single job; mutually exclusive with --output-dir

      --output-dir <OUTPUT_DIR>
          Directory for final artifacts and podcast clips

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --overwrite
          Explicitly allow replacing a requested destination

      --dry-run
          Validate and print effective inputs without network calls or run creation

      --fresh
          Bypass verified local cache

      --async
          Start a local worker and return the durable run ID

      --request-id <REQUEST_ID>
          Repeated submissions with this ID must have identical effective inputs

  -h, --help
          Print help

```

## framechord runs

```text
Inspect, wait, cancel, or resume durable runs

Usage: framechord runs [OPTIONS] <COMMAND>

Commands:
  recover    Stage a manually recovered response for an uncertain job; resume publishes without a new paid request
  reconcile  Retrieve saved Google interactions without a new generation; resume publishes recovered output
  list
  status
  wait
  cancel
  resume
  help       Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord runs recover

```text
Stage a manually recovered response for an uncertain job; resume publishes without a new paid request

Usage: framechord runs recover [OPTIONS] --job <JOB> --file <FILE> --format <FORMAT> --fingerprint <FINGERPRINT> --evidence <EVIDENCE> <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --job <JOB>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --file <FILE>


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --format <FORMAT>


      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --fingerprint <FINGERPRINT>


      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --evidence <EVIDENCE>
          Markdown reconciliation evidence, without account information or credentials

  -h, --help
          Print help

```

## framechord runs reconcile

```text
Retrieve saved Google interactions without a new generation; resume publishes recovered output

Usage: framechord runs reconcile [OPTIONS] <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord runs list

```text
Usage: framechord runs list [OPTIONS]

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord runs status

```text
Usage: framechord runs status [OPTIONS] <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord runs wait

```text
Usage: framechord runs wait [OPTIONS] <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --timeout-seconds <TIMEOUT_SECONDS>


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord runs cancel

```text
Usage: framechord runs cancel [OPTIONS] <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord runs resume

```text
Usage: framechord runs resume [OPTIONS] <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --retry-unknown


      --async


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord runs help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord runs help [COMMAND]

Commands:
  recover    Stage a manually recovered response for an uncertain job; resume publishes without a new paid request
  reconcile  Retrieve saved Google interactions without a new generation; resume publishes recovered output
  list
  status
  wait
  cancel
  resume
  help       Print this message or the help of the given subcommand(s)

```

## framechord runs help recover

```text
Stage a manually recovered response for an uncertain job; resume publishes without a new paid request

Usage: framechord runs help recover

```

## framechord runs help reconcile

```text
Retrieve saved Google interactions without a new generation; resume publishes recovered output

Usage: framechord runs help reconcile

```

## framechord runs help list

```text
Usage: framechord runs help list

```

## framechord runs help status

```text
Usage: framechord runs help status

```

## framechord runs help wait

```text
Usage: framechord runs help wait

```

## framechord runs help cancel

```text
Usage: framechord runs help cancel

```

## framechord runs help resume

```text
Usage: framechord runs help resume

```

## framechord runs help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord runs help help

```

## framechord resume

```text
Continue a recorded run, preserving successes; ambiguous retries are explicit

Usage: framechord resume [OPTIONS] <RUN_ID>

Arguments:
  <RUN_ID>


Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --retry-unknown


      --async


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord review

```text
Check artifact integrity, request explicit image/audio AI review, or record a human/agent review

Usage: framechord review [OPTIONS] <COMMAND>

Commands:
  report     Export a comparison report and immutable references without making acceptance decisions
  reconcile  Retrieve uncertain Google audio reviews without submitting another assessment
  status     Inspect recorded AI review attempts, provider receipts, and uncertain outcomes
  check      Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio
  record     Record a schema-validated assessment with human/agent provenance
  help       Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord review report

```text
Export a comparison report and immutable references without making acceptance decisions

Usage: framechord review report [OPTIONS] --artifact <ARTIFACT> --output <OUTPUT>

Options:
      --artifact <ARTIFACT>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --output <OUTPUT>


      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord review reconcile

```text
Retrieve uncertain Google audio reviews without submitting another assessment

Usage: framechord review reconcile [OPTIONS] --artifact <ARTIFACT>

Options:
      --artifact <ARTIFACT>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord review status

```text
Inspect recorded AI review attempts, provider receipts, and uncertain outcomes

Usage: framechord review status [OPTIONS] --artifact <ARTIFACT>

Options:
      --artifact <ARTIFACT>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord review check

```text
Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio

Usage: framechord review check [OPTIONS] --artifact <ARTIFACT>

Options:
      --artifact <ARTIFACT>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --ai


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --provider <PROVIDER>
          [possible values: openai, google, fish, openrouter, elevenlabs]

      --model <MODEL>


      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --retry-unknown
          Explicitly retry an uncertain AI review; it may already have incurred a charge

  -h, --help
          Print help

```

## framechord review record

```text
Record a schema-validated assessment with human/agent provenance

Usage: framechord review record [OPTIONS] --file <FILE>

Options:
      --file <FILE>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord review help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord review help [COMMAND]

Commands:
  report     Export a comparison report and immutable references without making acceptance decisions
  reconcile  Retrieve uncertain Google audio reviews without submitting another assessment
  status     Inspect recorded AI review attempts, provider receipts, and uncertain outcomes
  check      Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio
  record     Record a schema-validated assessment with human/agent provenance
  help       Print this message or the help of the given subcommand(s)

```

## framechord review help report

```text
Export a comparison report and immutable references without making acceptance decisions

Usage: framechord review help report

```

## framechord review help reconcile

```text
Retrieve uncertain Google audio reviews without submitting another assessment

Usage: framechord review help reconcile

```

## framechord review help status

```text
Inspect recorded AI review attempts, provider receipts, and uncertain outcomes

Usage: framechord review help status

```

## framechord review help check

```text
Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio

Usage: framechord review help check

```

## framechord review help record

```text
Record a schema-validated assessment with human/agent provenance

Usage: framechord review help record

```

## framechord review help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord review help help

```

## framechord revise

```text
Create a revision child run with inherited identity and limits

Usage: framechord revise [OPTIONS] --artifact <ARTIFACT> --instructions-file <INSTRUCTIONS_FILE>

Options:
      --artifact <ARTIFACT>


      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --instructions-file <INSTRUCTIONS_FILE>


      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --source-job <SOURCE_JOB>
          Updated source job for explicit script or infographic copy changes

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --turn <TURN>
          Revise one podcast turn and reuse verified unchanged clips

      --chapter <CHAPTER>
          Revise one audiobook chapter, reuse unchanged chapters, and reassemble the book

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --async


  -h, --help
          Print help

```

## framechord plan

```text
Explicit text-model planning; saves reviewable files without generating media

Usage: framechord plan [OPTIONS] --kind <KIND> --prompt <PROMPT> --output-dir <OUTPUT_DIR>

Options:
      --kind <KIND>
          [possible values: job, character, infographic]

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --prompt <PROMPT>


      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --output-dir <OUTPUT_DIR>


      --character <CHARACTERS>
          Existing character manifests copied into the contained authoring bundle

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --asset <SELECTED_ASSETS>
          Selected image/audio assets copied by the binary

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

      --source <SOURCES>
          Exact source text (Markdown or plain text, such as a book manuscript) copied into the bundle; only its path, size and hash are sent to the planner

      --provider <PROVIDER>
          [possible values: openai, google, fish, openrouter, elevenlabs]

      --model <MODEL>


  -h, --help
          Print help

```

## framechord skills

```text
Export a portable skill or install it in agent discovery locations

Usage: framechord skills [OPTIONS] <COMMAND>

Commands:
  export
  install
  help     Print this message or the help of the given subcommand(s)

Options:
      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord skills export

```text
Usage: framechord skills export [OPTIONS] <DIRECTORY>

Arguments:
  <DIRECTORY>


Options:
      --overwrite
          Explicitly replace bundled skill files, preserving unrelated files

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord skills install

```text
Usage: framechord skills install [OPTIONS]

Options:
      --agents <AGENTS>
          Agent targets: codex, claude, cursor, gemini, opencode, copilot, or none (comma-separated). The default pair is already discovered by Cursor, Gemini CLI, OpenCode and Copilot

          [default: codex,claude]

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --scope <SCOPE>
          [default: project]
          [possible values: project, user]

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --overwrite
          Explicitly replace bundled skill files, including modified bundled assets

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## framechord skills help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord skills help [COMMAND]

Commands:
  export
  install
  help     Print this message or the help of the given subcommand(s)

```

## framechord skills help export

```text
Usage: framechord skills help export

```

## framechord skills help install

```text
Usage: framechord skills help install

```

## framechord skills help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord skills help help

```

## framechord help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord help [COMMAND]

Commands:
  install    Install this downloaded binary in your user bin directory, offline and without administrator access
  update     Update this executable from a verified public GitHub Release; optional GH_TOKEN/GITHUB_TOKEN authenticates metadata checks
  init       Initialize offline, preserving existing files and credentials
  config     Inspect or explicitly replace validated configuration
  verify     Preview a bounded live verification suite; execution requires the exact approved preview hash
  usage      Report provider-returned usage separately from unknown costs
  storage    Inspect storage or apply a previously reviewed conservative cleanup plan
  migrate    Preview migration of a v1 job, config or run; --apply creates a non-overwriting backup
  doctor     Inspect credentials offline, or explicitly check all provider endpoints and keys with --live
  notices    Print third-party license notices for components bundled in this binary
  schema     Print a JSON Schema for a versioned manifest
  models     List implemented provider/model capabilities (not account entitlements)
  voices     List built-in voices or search bounded pages of provider account voices
  templates  Inspect or export embedded authoring templates
  persona    Create, inspect, bootstrap, accept, or port character definitions
  validate   Validate local YAML and all referenced files; never generate
  generate   Generate one media job. Example: generate image --prompt 'A forest' --provider openai --model gpt-image-2.5-flare --output forest.png
  batch      Execute independent jobs concurrently using a YAML batch manifest
  runs       Inspect, wait, cancel, or resume durable runs
  resume     Continue a recorded run, preserving successes; ambiguous retries are explicit
  review     Check artifact integrity, request explicit image/audio AI review, or record a human/agent review
  revise     Create a revision child run with inherited identity and limits
  plan       Explicit text-model planning; saves reviewable files without generating media
  skills     Export a portable skill or install it in agent discovery locations
  help       Print this message or the help of the given subcommand(s)

```

## framechord help install

```text
Install this downloaded binary in your user bin directory, offline and without administrator access

Usage: framechord help install

```

## framechord help update

```text
Update this executable from a verified public GitHub Release; optional GH_TOKEN/GITHUB_TOKEN authenticates metadata checks

Usage: framechord help update

```

## framechord help init

```text
Initialize offline, preserving existing files and credentials

Usage: framechord help init

```

## framechord help config

```text
Inspect or explicitly replace validated configuration

Usage: framechord help config [COMMAND]

Commands:
  wizard  Terminal-only guided defaults; agents use setup --file instead
  host    Explicit user-scope host concurrency ceilings, independent of agent settings
  show
  status  Inspect local setup readiness without network calls or revealing credentials
  setup   Replace config only after validating the supplied file
  limits  Inspect finite generation limits

```

## framechord help config wizard

```text
Terminal-only guided defaults; agents use setup --file instead

Usage: framechord help config wizard

```

## framechord help config host

```text
Explicit user-scope host concurrency ceilings, independent of agent settings

Usage: framechord help config host [COMMAND]

Commands:
  show
  setup

```

## framechord help config host show

```text
Usage: framechord help config host show

```

## framechord help config host setup

```text
Usage: framechord help config host setup

```

## framechord help config show

```text
Usage: framechord help config show

```

## framechord help config status

```text
Inspect local setup readiness without network calls or revealing credentials

Usage: framechord help config status

```

## framechord help config setup

```text
Replace config only after validating the supplied file

Usage: framechord help config setup

```

## framechord help config limits

```text
Inspect finite generation limits

Usage: framechord help config limits

```

## framechord help verify

```text
Preview a bounded live verification suite; execution requires the exact approved preview hash

Usage: framechord help verify

```

## framechord help usage

```text
Report provider-returned usage separately from unknown costs

Usage: framechord help usage

```

## framechord help storage

```text
Inspect storage or apply a previously reviewed conservative cleanup plan

Usage: framechord help storage [COMMAND]

Commands:
  inspect
  plan     Save a cleanup preview; only unreferenced blobs can be removed
  clean    Revalidate and apply a saved cleanup preview; fails if stale or workers active

```

## framechord help storage inspect

```text
Usage: framechord help storage inspect

```

## framechord help storage plan

```text
Save a cleanup preview; only unreferenced blobs can be removed

Usage: framechord help storage plan

```

## framechord help storage clean

```text
Revalidate and apply a saved cleanup preview; fails if stale or workers active

Usage: framechord help storage clean

```

## framechord help migrate

```text
Preview migration of a v1 job, config or run; --apply creates a non-overwriting backup

Usage: framechord help migrate

```

## framechord help doctor

```text
Inspect credentials offline, or explicitly check all provider endpoints and keys with --live

Usage: framechord help doctor

```

## framechord help notices

```text
Print third-party license notices for components bundled in this binary

Usage: framechord help notices

```

## framechord help schema

```text
Print a JSON Schema for a versioned manifest

Usage: framechord help schema

```

## framechord help models

```text
List implemented provider/model capabilities (not account entitlements)

Usage: framechord help models

```

## framechord help voices

```text
List built-in voices or search bounded pages of provider account voices

Usage: framechord help voices

```

## framechord help templates

```text
Inspect or export embedded authoring templates

Usage: framechord help templates [COMMAND]

Commands:
  list
  show
  export

```

## framechord help templates list

```text
Usage: framechord help templates list

```

## framechord help templates show

```text
Usage: framechord help templates show

```

## framechord help templates export

```text
Usage: framechord help templates export

```

## framechord help persona

```text
Create, inspect, bootstrap, accept, or port character definitions

Usage: framechord help persona [COMMAND]

Commands:
  voice-clone         Explicitly create an ElevenLabs clone from selected authorized recordings
  voice-clone-status  Inspect cloned-voice receipts and unresolved creation attempts
  voice-design        Generate ElevenLabs voice preview candidates without creating or binding a saved voice
  voice-create        Create a provider voice from one selected candidate; canonical binding is explicit
  voice-status        Inspect saved voice candidates, creation results, and uncertain operations
  add
  list
  show
  bootstrap           Generate visual candidates; this never promotes references
  accept-sample       Accept a selected recording, exact transcript, and granted consent as a portable voice sample
  accept              Accept a user-selected image or recording into the canonical profile
  export
  import

```

## framechord help persona voice-clone

```text
Explicitly create an ElevenLabs clone from selected authorized recordings

Usage: framechord help persona voice-clone

```

## framechord help persona voice-clone-status

```text
Inspect cloned-voice receipts and unresolved creation attempts

Usage: framechord help persona voice-clone-status

```

## framechord help persona voice-design

```text
Generate ElevenLabs voice preview candidates without creating or binding a saved voice

Usage: framechord help persona voice-design

```

## framechord help persona voice-create

```text
Create a provider voice from one selected candidate; canonical binding is explicit

Usage: framechord help persona voice-create

```

## framechord help persona voice-status

```text
Inspect saved voice candidates, creation results, and uncertain operations

Usage: framechord help persona voice-status

```

## framechord help persona add

```text
Usage: framechord help persona add

```

## framechord help persona list

```text
Usage: framechord help persona list

```

## framechord help persona show

```text
Usage: framechord help persona show

```

## framechord help persona bootstrap

```text
Generate visual candidates; this never promotes references

Usage: framechord help persona bootstrap

```

## framechord help persona accept-sample

```text
Accept a selected recording, exact transcript, and granted consent as a portable voice sample

Usage: framechord help persona accept-sample

```

## framechord help persona accept

```text
Accept a user-selected image or recording into the canonical profile

Usage: framechord help persona accept

```

## framechord help persona export

```text
Usage: framechord help persona export

```

## framechord help persona import

```text
Usage: framechord help persona import

```

## framechord help validate

```text
Validate local YAML and all referenced files; never generate

Usage: framechord help validate

```

## framechord help generate

```text
Generate one media job. Example: generate image --prompt 'A forest' --provider openai --model gpt-image-2.5-flare --output forest.png

Usage: framechord help generate

```

## framechord help batch

```text
Execute independent jobs concurrently using a YAML batch manifest

Usage: framechord help batch

```

## framechord help runs

```text
Inspect, wait, cancel, or resume durable runs

Usage: framechord help runs [COMMAND]

Commands:
  recover    Stage a manually recovered response for an uncertain job; resume publishes without a new paid request
  reconcile  Retrieve saved Google interactions without a new generation; resume publishes recovered output
  list
  status
  wait
  cancel
  resume

```

## framechord help runs recover

```text
Stage a manually recovered response for an uncertain job; resume publishes without a new paid request

Usage: framechord help runs recover

```

## framechord help runs reconcile

```text
Retrieve saved Google interactions without a new generation; resume publishes recovered output

Usage: framechord help runs reconcile

```

## framechord help runs list

```text
Usage: framechord help runs list

```

## framechord help runs status

```text
Usage: framechord help runs status

```

## framechord help runs wait

```text
Usage: framechord help runs wait

```

## framechord help runs cancel

```text
Usage: framechord help runs cancel

```

## framechord help runs resume

```text
Usage: framechord help runs resume

```

## framechord help resume

```text
Continue a recorded run, preserving successes; ambiguous retries are explicit

Usage: framechord help resume

```

## framechord help review

```text
Check artifact integrity, request explicit image/audio AI review, or record a human/agent review

Usage: framechord help review [COMMAND]

Commands:
  report     Export a comparison report and immutable references without making acceptance decisions
  reconcile  Retrieve uncertain Google audio reviews without submitting another assessment
  status     Inspect recorded AI review attempts, provider receipts, and uncertain outcomes
  check      Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio
  record     Record a schema-validated assessment with human/agent provenance

```

## framechord help review report

```text
Export a comparison report and immutable references without making acceptance decisions

Usage: framechord help review report

```

## framechord help review reconcile

```text
Retrieve uncertain Google audio reviews without submitting another assessment

Usage: framechord help review reconcile

```

## framechord help review status

```text
Inspect recorded AI review attempts, provider receipts, and uncertain outcomes

Usage: framechord help review status

```

## framechord help review check

```text
Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio

Usage: framechord help review check

```

## framechord help review record

```text
Record a schema-validated assessment with human/agent provenance

Usage: framechord help review record

```

## framechord help revise

```text
Create a revision child run with inherited identity and limits

Usage: framechord help revise

```

## framechord help plan

```text
Explicit text-model planning; saves reviewable files without generating media

Usage: framechord help plan

```

## framechord help skills

```text
Export a portable skill or install it in agent discovery locations

Usage: framechord help skills [COMMAND]

Commands:
  export
  install

```

## framechord help skills export

```text
Usage: framechord help skills export

```

## framechord help skills install

```text
Usage: framechord help skills install

```

## framechord help help

```text
Print this message or the help of the given subcommand(s)

Usage: framechord help help

```
