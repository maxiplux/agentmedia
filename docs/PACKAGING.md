# Native packages and downloads

Download native binaries from the [Native packages workflow](https://github.com/maxiplux/agentmedia/actions/workflows/package.yml). Open a successful run for the version you want, then use its **Artifacts** section:

| Artifact | Included platform archive |
| --- | --- |
| `mediaagent-windows-latest` | Windows x86-64 ZIP containing `mediaagent.exe` |
| `mediaagent-macos-latest` | macOS Apple Silicon tar.gz containing `mediaagent` |
| `mediaagent-ubuntu-latest` | Linux x86-64 tar.gz containing `mediaagent` |

GitHub may require sign-in to download workflow artifacts. The downloaded artifact is a wrapper ZIP: extract it, then extract the platform archive inside. Check the archive against its `.sha256` sidecar. These are CI build artifacts; this workflow does not create a tagged GitHub Release. Signing and notarization are not provided.

Put the executable on PATH or invoke its absolute path. Rust and Python are not needed to run it. On Windows PowerShell, an executable in the current folder can be started with `./mediaagent.exe`.

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

The packager copies the executable outside the source checkout and verifies 13 isolated offline workflows: preserving credentials on repeat initialization; embedded Codex/Claude skills; recommended defaults and setup readiness; portable skill export; generation dry-run; strict schemas; optional initial profiles; explicit defaults and profile seeding; offline provider diagnostics; bounded verification previews; preserving migration previews; usage/storage previews; and embedded v2 cast/recovery guidance. Subprocesses receive no live API keys and make no provider requests.

Each archive contains the executable, a usage README, a secret-free `.env.example`, and `verification.json` recording the binary hash and executed checks. macOS/Linux use tar.gz to preserve executable permissions; Windows uses ZIP. Build products stay ignored under `target/` and contain no generated media or project characters.

For 0.3.0, Windows x86-64, macOS Apple Silicon and Linux x86-64 each pass 140 offline tests, formatting and strict Clippy. All three archives pass the 13 isolated package checks in [this successful downloadable run](https://github.com/maxiplux/agentmedia/actions/runs/38670199579), built from code revision `3395a58`. Downloaded archive/executable hashes were verified; the Windows executable also retains its 8 MiB stack reserve. See [implementation status](STATUS.md). Native execution checks do not prove provider entitlement, media quality, character consistency, or automatic skill discovery in a fresh agent session.

To replace a local installation, stop its workers, keep the previous executable, copy the new one to the same PATH directory, and check `mediaagent --version`. Existing project files remain readable. Refresh installed skills/templates explicitly when desired; older pinned character models require an explicit new character version before new generation.
