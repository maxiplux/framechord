# Use FrameChord in your own project

FrameChord is a single binary that reads and writes files inside the repository it is
pointed at. This page covers adding it to an existing codebase, calling it from scripts
and application code, and letting coding agents other than Codex and Claude Code use it.
Install it first with the [one-command installers](INSTALLATION.md); every command below
is offline unless it says otherwise.

## Add it to an existing repository

Run initialization at the repository root, or at a sub-project in a monorepo:

```sh
cd my-app
framechord init
framechord init ./apps/site --preset recommended --json --non-interactive
```

Initialization creates, and on repeat runs preserves:

| Path | Purpose | Commit it? |
| --- | --- | --- |
| `framechord/config.yaml` | Providers, models, voices, settings and limits | Yes |
| `framechord/templates/` | Editable YAML and Markdown scaffolds | Yes |
| `framechord/characters/<id>/` | Portable characters and accepted references | Yes |
| `framechord/jobs/<id>/` | Authored jobs and briefs | Yes |
| `framechord/runs/<run-id>/` | Durable run records, receipts and default outputs | No (ignored) |
| `framechord/cache/`, `framechord/blobs/`, `framechord/setup.lock` | Cache, reference blobs, lock | No (ignored) |
| `.env` | Credentials; names appended with empty values, existing values kept | No (ignored) |
| `.env.example` | Credential names only | Yes |
| `.agents/skills/framechord/`, `.claude/skills/framechord/` | Project skills (default targets) | Yes |

`init` appends these rules to the repository's root `.gitignore` when they are missing,
and leaves every other line untouched:

```gitignore
.env
.env.*
!.env.example
/framechord/runs/
/framechord/cache/
/framechord/blobs/
/framechord/setup.lock
/mediaagent/runs/
/mediaagent/cache/
/mediaagent/blobs/
/mediaagent/setup.lock
```

It also writes `framechord/.gitignore` with `runs/`, `cache/`, `blobs/` and `setup.lock`.
One rule is not added: `framechord/characters/*/character.lock` appears once you accept a
character reference. Add it yourself if you do not want lock files in version control.
A sub-project initialized with `init ./apps/site` gets the same rules relative to that
folder's own `.gitignore`.

**Credentials.** The process environment takes precedence over the project `.env`;
parent directories are never searched. In continuous integration, provide the key
variables (`GEMINI_API_KEY`, `OPENAI_API_KEY`, `FISH_API_KEY`, `OPENROUTER_API_KEY`,
`ELEVENLABS_API_KEY`) as secrets in the environment and commit nothing else. Only the
provider needed by a command is read.

**Outputs.** `--output FILE` writes one final artifact where you ask (an existing file
needs `--overwrite`); `--output-dir DIR` is for batches, podcast clips and audiobooks (`DIR/<book>/`). Without either,
artifacts land under `framechord/runs/<run-id>/outputs/`, which is ignored. Paths in
JSON results are absolute. Generated media you want to ship belongs in your own asset
folder, named by `--output`.

**Repeat runs are safe.** `init` reports existing content instead of overwriting it;
`skills install --overwrite` is the only way to replace a modified bundled skill file.

## Call it from scripts and code

Every command accepts `--json --non-interactive`. The result envelope on stdout is:

```json
{"schema_version": 1, "command": "generate.image", "ok": true, "data": {}, "errors": [], "warnings": []}
```

`command` is the parsed command path joined with dots. Progress bars, spinners and
diagnostics go to stderr, never into the JSON. Each error
carries `code`, `message`, `field`, `retryable`, `corrective_argv` and, when known,
`request_id` and `retry_after_seconds`. When a command completes but exits non-zero,
`data` is still filled and an `operation_incomplete` error explains why.

| Exit code | Meaning | What to do |
| --- | --- | --- |
| 0 | Success, or accepted/pending for `--async` | Read `data.jobs[].artifacts[].path` |
| 1 | Internal error | Report the message; retrying is unlikely to help |
| 2 | Invalid input or configuration | Fix the arguments or files named in `errors[].field` |
| 3 | Provider or local operation failed | Check `retryable` and `retry_after_seconds`; `runs resume` honors provider guidance |
| 4 | Partial batch | Keep the succeeded artifacts; inspect the failed jobs; `runs resume RUN_ID` |
| 5 | Review found the result needs revision | Read the findings; `revise` is a separate paid step |
| 6 | A limit stopped further work | Raise the budget with `--limits-file` only if intended |
| 7 | Outcome unknown | Do not resubmit blindly; `runs reconcile`, then `runs resume --retry-unknown` only with approval |
| 130 | Wait interrupted | Generation continues; `runs wait RUN_ID` again |

Preview with `--dry-run` first: it validates the request and returns the effective inputs
and limits without a provider call or a run record. Remove it to submit, and keep the
returned run ID.

**Shell with `jq`:**

```sh
framechord --project . generate image \
  --prompt 'A teal leaf on a cream background' --output assets/leaf.png \
  --json --non-interactive > result.json
status=$?
jq -r '.data.jobs[].artifacts[].path' result.json
case "$status" in
  0) ;;
  4) echo "partial batch; see .errors" >&2 ;;
  7) echo "outcome unknown; inspect before any retry" >&2; exit 7 ;;
  *) jq -r '.errors[].message' result.json >&2; exit "$status" ;;
esac
```

**Python:**

```python
import json
import subprocess


def framechord(*arguments, project="."):
    result = subprocess.run(
        ["framechord", "--project", project, "--json", "--non-interactive", *arguments],
        capture_output=True, text=True,
    )
    response = json.loads(result.stdout)  # stderr holds progress and diagnostics
    return result.returncode, response


code, response = framechord(
    "generate", "speech", "--text-file", "article.txt", "--output", "narration.wav",
    "--limits-file", "limits.yaml",
)
if code == 0:
    paths = [a["path"] for job in response["data"]["jobs"] for a in job["artifacts"]]
elif code == 7:
    raise RuntimeError("outcome unknown; reconcile before retrying: %s" % response["errors"])
else:
    raise RuntimeError(response["errors"])
```

**Node.js:**

```js
const { spawnSync } = require("node:child_process");

const run = spawnSync(
  "framechord",
  ["--project", ".", "--json", "--non-interactive",
   "generate", "music", "--brief-file", "music.md", "--duration-seconds", "30",
   "--instrumental", "--output", "intro.mp3"],
  { encoding: "utf8" },
);
const response = JSON.parse(run.stdout);
if (run.status !== 0) {
  console.error(response.errors);
  process.exit(run.status);
}
const paths = response.data.jobs.flatMap((job) => job.artifacts.map((a) => a.path));
```

For longer batches, submit with `--async` to get a run ID at once, then poll
`runs status RUN_ID --json` or block on `runs wait RUN_ID`. Pass `--request-id` for
submission idempotency when your own retry logic might call twice. Use
`framechord schema run` for the exact JSON Schema of `data`.

## Other coding agents

The bundled skill is a standard `SKILL.md` folder. Initialization installs it for Codex
(`.agents/skills/framechord/`) and Claude Code (`.claude/skills/framechord/`), and those
two directories are already read by the other major agents:

| Agent | Project directories it reads | User directories | `--agents` target | Documentation |
| --- | --- | --- | --- | --- |
| Codex | `.agents/skills/` | `~/.agents/skills/` | `codex` (default) | [Codex skills](https://developers.openai.com/codex/skills) |
| Claude Code | `.claude/skills/` | `~/.claude/skills/` | `claude` (default) | [Claude Code skills](https://code.claude.com/docs/en/skills) |
| Cursor | `.cursor/skills/`, `.agents/skills/`, `.claude/skills/`, `.codex/skills/` | the same four under `~` | `cursor` | [Cursor skills](https://cursor.com/docs/skills) |
| Gemini CLI | `.gemini/skills/`, `.agents/skills/` (trusted workspace) | `~/.gemini/skills/` | `gemini` | [Gemini CLI skills](https://geminicli.com/docs/cli/tutorials/skills-getting-started/) |
| OpenCode | `.opencode/skills/`, `.claude/skills/`, `.agents/skills/` | `~/.config/opencode/skills/`, `~/.claude/skills/`, `~/.agents/skills/` | `opencode` | [OpenCode skills](https://opencode.ai/docs/skills/) |
| GitHub Copilot | `.github/skills/`, `.claude/skills/`, `.agents/skills/` | `~/.copilot/skills/`, `~/.claude/skills/`, `~/.agents/skills/` | `copilot` | [Copilot agent skills](https://docs.github.com/en/copilot/concepts/agents/about-agent-skills) |

Paths verified against each vendor's documentation on 2026-10-15. So the default
`framechord init` already serves Cursor, Gemini CLI, OpenCode and Copilot. Pick a native
directory only when you want it, and `none` when no skill folder should be written:

```sh
framechord init --agents codex,claude          # default; discovered by all six
framechord skills install --agents cursor,copilot
framechord skills install --scope user --agents gemini,opencode
framechord init ./tool-only --agents none
framechord --project . config status --json    # skills, skill_targets, skills_shared_by
```

`config status --json` reports, per agent, whether a FrameChord skill is discoverable and
through which directory. Installing both a shared and a native directory gives that agent
two skills with the same name; keep one. That case sets `duplicate_discovery: true` on
the agent's `skill_targets` entry, and the human output of `init` and `config status`
prints one warning line naming the copies. `none` must be given alone.

Once the skill is installed, start a new agent session in the project and describe the
media you want in plain words. Ask for a dry run and say you will approve generation, for
example: "Use FrameChord to narrate docs/getting-started.md exactly as written. Preview the
request first, then generate when I approve and tell me the output path." The web guide's
[Hand it to your agent](https://framechord.com/#delegate) section has a first readiness
prompt, twelve prompts by use case, and the request that produced the guide's own media.

An agent without the skill can learn FrameChord from the web:
[llms.txt](https://framechord.com/llms.txt) lists the install commands, the working rules
and every section of the guide, and [index.md](https://framechord.com/index.md) is the
whole guide in Markdown, regenerated with the HTML.

For any other process-based agent, export the skill and point the agent's instruction
file at it:

```sh
framechord skills export tools/framechord-skill
```

Then add a short block to `AGENTS.md`, `.cursor/rules`, or whatever file your agent
reads:

```markdown
## Media assets

Use the `framechord` CLI for images, infographics, narration, podcasts, audiobooks, music and
sound effects. Read `tools/framechord-skill/SKILL.md` and its references first.
Author inputs from the templates, run `framechord validate`, preview with
`framechord generate ... --dry-run --json`, and generate only when asked. Always pass
`--json --non-interactive`, keep requested scripts and copy exact, and report the
artifact paths the binary returns. If the binary is missing, say so instead of using
another media tool.
```

The exported folder carries the same `SKILL.md`, references, JSON Schemas and templates
as a project installation, generated from the installed binary's own command
definitions, so an agent reading it sees the flags of the version you run.
