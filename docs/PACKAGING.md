# Native packages and downloads

Use the [installation guide](INSTALLATION.md) for one-command installers, user-local installation and `framechord update`. Public binaries are available from [GitHub Releases](https://github.com/maxiplux/framechord/releases/latest), without requiring GitHub sign-in. Releases include raw executables and archives for macOS Apple Silicon, Linux x86-64 and Windows x86-64, with SHA-256 sidecars and a manifest recording the exact source commit and hashes.

Development builds remain available from the [Native packages workflow](https://github.com/maxiplux/framechord/actions/workflows/package.yml). Open a successful run and its Artifacts section. The wrapper ZIP contains a native archive, raw executable and their checksum files. CI artifacts are separate from stable releases and may require GitHub sign-in.

Run `./framechord install` (Windows `./framechord.exe install`) or invoke its absolute path. Rust and Python are not needed to run it. On Windows PowerShell, an executable in the current folder can be started with `./framechord.exe`.

```sh
framechord --version
framechord --help
framechord init /path/to/project
```

On macOS/Linux, do not initialize in the folder containing the downloaded file
named `framechord`: setup needs to create a directory with that same name. From
that folder, use `./framechord init ./my-project`, or put the binary on PATH and
initialize in your project folder. Version 0.3.1 detects the conflict before the
wizard and offers a different destination. In noninteractive mode it reports
`init_path_conflict` without writing setup files. `--project PATH init` is also
supported; an explicit `init PATH` takes precedence.

Initialization opens a looping terminal checklist with Google image, speech, music and planning defaults, plus an explicit OpenAI alternative. Add missing keys through masked input or the target project's root `.env`; process environment values take precedence. Review and finish, or save unfinished setup and run `init` again. No development credentials are bundled or copied. Project-local skills are included; user-level skill installation remains explicit. See the [web guide](https://framechord.com/) and [setup reference](../assets/skill/references/setup.md).

## Building and verifying a package

`python3 scripts/package-release.py --offline` builds an optimized native binary with locked, cached dependencies and creates an archive plus SHA-256 sidecar under `target/package-dist/`. Omit `--offline` when dependencies need downloading. Python 3.11 or newer (CI uses 3.13) and Rust are development dependencies for this script.

The packager copies the executable outside the source checkout and verifies 18 isolated offline workflows: detecting the executable/configuration name collision without writes; preserving credentials on repeat initialization; embedded Codex/Claude skills; native Copilot/Gemini skill targets; recommended defaults and setup readiness; portable skill export; generation dry-run; strict schemas; optional initial profiles; explicit defaults and profile seeding; offline provider diagnostics; bounded verification previews; preserving migration previews; usage/storage previews; embedded v2 cast/recovery guidance; offline user installation/repeat checks; native bootstrap installation/repeat/checksum-rejection tests with mocked downloads; and preservation of legacy project paths, identities and configuration. Subprocesses receive no live API keys and make no provider requests.

Each archive contains the executable, a usage README, a secret-free `.env.example`, `THIRD_PARTY_NOTICES.md`, and `verification.json` recording the binary hash and executed checks. macOS/Linux use tar.gz to preserve executable permissions; Windows uses ZIP. Build products stay ignored under `target/` and contain no generated media or project characters.

For 0.3.1, Windows x86-64, macOS Apple Silicon and Linux x86-64 each pass 144 offline tests, formatting and strict Clippy. The [0.3.1 native package run and downloads](https://github.com/maxiplux/framechord/actions/runs/38673081221) record the 14 isolated checks per platform, built from code revision `5318faa`; subsequent changes only update documentation. The local macOS package and downloaded macOS/Linux archives have also passed integrity/hash verification. See [implementation status](STATUS.md). Native execution checks do not prove provider entitlement, media quality, character consistency, or automatic skill discovery in a fresh agent session.

To replace a local installation, stop its workers, keep the previous executable, copy the new one to the same PATH directory, and check `framechord --version`. Existing project files remain readable. Refresh installed skills/templates explicitly when desired; older pinned character models require an explicit new character version before new generation.


Current releases target Windows x86-64, Linux x86-64 and macOS Apple Silicon.
Publication requires all three targets to pass tests and package/bootstrap checks.
Historical releases retain their original assets, including Intel Mac builds.
`verification.json` records `source_commit` and `source_dirty`; release publication
requires the exact clean tag checkout. See [release maintenance](INSTALLATION.md#maintainer-release-workflow).

## MP3 encoder and the LGPL

Audiobook MP3 output uses LAME 3.100, built from the copy bundled in the
`mp3lame-sys` crate and statically linked into the default build (the `mp3` Cargo
feature). LAME is licensed under the GNU LGPL; the `mp3lame-sys` and `mp3lame-encoder`
bindings are LGPL-3.0. Every archive includes `THIRD_PARTY_NOTICES.md` with the license
texts, source location and relinking instructions, and every binary prints the same
file with `framechord notices`, because raw executables are also published without an
archive. `cargo build --no-default-features` produces a binary without the encoder;
audiobook `format: mp3` then fails validation. On Unix the LAME build uses its
autotools scripts (a C compiler and `make`); on Windows it is compiled with MSVC
through the `cc` crate. The obligations that follow from static LGPL linking have not
been reviewed by counsel.
