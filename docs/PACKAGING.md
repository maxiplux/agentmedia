# Native packages and downloads

Use the [installation guide](INSTALLATION.md) for one-command installers, user-local installation and `mediaagent update`. Public binaries are available from [GitHub Releases](https://github.com/maxiplux/agentmedia/releases/latest), without requiring GitHub sign-in. Releases include raw executables and archives for macOS Apple Silicon, macOS Intel, Linux x86-64 and Windows x86-64, with SHA-256 sidecars and a manifest recording the exact source commit and hashes.

Development builds remain available from the [Native packages workflow](https://github.com/maxiplux/agentmedia/actions/workflows/package.yml). Open a successful run and its Artifacts section. The wrapper ZIP contains a native archive, raw executable and their checksum files. CI artifacts are separate from stable releases and may require GitHub sign-in.

Run `./mediaagent install` (Windows `./mediaagent.exe install`) or invoke its absolute path. Rust and Python are not needed to run it. On Windows PowerShell, an executable in the current folder can be started with `./mediaagent.exe`.

```sh
mediaagent --version
mediaagent --help
mediaagent init /path/to/project
```

On macOS/Linux, do not initialize in the folder containing the downloaded file
named `mediaagent`: setup needs to create a directory with that same name. From
that folder, use `./mediaagent init ./my-project`, or put the binary on PATH and
initialize in your project folder. Version 0.3.1 detects the conflict before the
wizard and offers a different destination. In noninteractive mode it reports
`init_path_conflict` without writing setup files. `--project PATH init` is also
supported; an explicit `init PATH` takes precedence.

Initialization opens a looping terminal checklist with OpenAI image/planning and Fish speech defaults. Add missing keys through masked input or the target project's root `.env`; process environment values take precedence. Review and finish, or save unfinished setup and run `init` again. No development credentials are bundled or copied. Project-local skills are included; user-level skill installation remains explicit. See the [web guide](https://maxiplux.github.io/agentmedia/) and [setup reference](../assets/skill/references/setup.md).

## Building and verifying a package

`python3 scripts/package-release.py --offline` builds an optimized native binary with locked, cached dependencies and creates an archive plus SHA-256 sidecar under `target/package-dist/`. Omit `--offline` when dependencies need downloading. Python 3.9+ and Rust are development dependencies for this script.

The packager copies the executable outside the source checkout and verifies 16 isolated offline workflows: detecting the executable/configuration name collision without writes; preserving credentials on repeat initialization; embedded Codex/Claude skills; recommended defaults and setup readiness; portable skill export; generation dry-run; strict schemas; optional initial profiles; explicit defaults and profile seeding; offline provider diagnostics; bounded verification previews; preserving migration previews; usage/storage previews; embedded v2 cast/recovery guidance; offline user installation/repeat checks; and native bootstrap installation/repeat/checksum-rejection tests with mocked downloads. Subprocesses receive no live API keys and make no provider requests.

Each archive contains the executable, a usage README, a secret-free `.env.example`, and `verification.json` recording the binary hash and executed checks. macOS/Linux use tar.gz to preserve executable permissions; Windows uses ZIP. Build products stay ignored under `target/` and contain no generated media or project characters.

For 0.3.1, Windows x86-64, macOS Apple Silicon and Linux x86-64 each pass 144 offline tests, formatting and strict Clippy. The [0.3.1 native package run and downloads](https://github.com/maxiplux/agentmedia/actions/runs/38673081221) record the 14 isolated checks per platform, built from code revision `5318faa`; subsequent changes only update documentation. The local macOS package and downloaded macOS/Linux archives have also passed integrity/hash verification. See [implementation status](STATUS.md). Native execution checks do not prove provider entitlement, media quality, character consistency, or automatic skill discovery in a fresh agent session.

To replace a local installation, stop its workers, keep the previous executable, copy the new one to the same PATH directory, and check `mediaagent --version`. Existing project files remain readable. Refresh installed skills/templates explicitly when desired; older pinned character models require an explicit new character version before new generation.


The 0.4.0 distribution workflow adds native Intel Mac builds and publishes a stable
release only after all four targets pass tests and package/bootstrap checks.
`verification.json` records `source_commit` and `source_dirty`; release publication
requires the exact clean tag checkout. See [release maintenance](INSTALLATION.md#maintainer-release-workflow).
