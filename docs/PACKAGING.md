# Native local packages

`python3 scripts/package-release.py --offline` builds an optimized native binary with locked, cached dependencies and creates an archive plus SHA-256 sidecar under `target/package-dist/`. Omit `--offline` when dependencies need downloading. Python 3.9+ and the Rust toolchain are development dependencies for this script; the packaged binary does not require either at runtime.

The packager copies the executable into an isolated temporary directory outside the source checkout and verifies twelve offline behaviors: preserving credentials on repeated initialization, embedding both agent skills, portable skill export, generation dry-run, strict schema output, initially disabled unconfigured profiles, complete initialization settings seeding configured profiles and generation defaults, and offline provider diagnostics with its embedded contract. Its subprocesses receive no live API keys. It never invokes media generation or creates provider account resources.

Each package contains the native executable, a standalone usage README, a secret-free generated `.env.example`, and `verification.json` with the binary hash and executed checks. macOS/Linux archives preserve executable permissions using tar.gz; Windows uses zip. Archives and checksums are ignored by Git through the existing `target/` rule. They contain no development `.env`, generated media, or project characters.

On October 14, 2026, the local Apple Silicon build for `aarch64-apple-darwin` passed all eight isolated checks and archive integrity/hash verification. Its local archive is `target/package-dist/mediaagent-0.1.0-aarch64-apple-darwin.tar.gz`. The `.sha256` sidecar identifies the exact archive; `verification.json` inside identifies the exact executable. This local package has not been published or notarized. A compiled, locally verified package is distinct from a public release.

After extraction, put the executable on PATH or invoke its absolute path. Run `mediaagent init /path/to/project` to create the visible configuration folder and project-local skills. Fill credentials in that target project's root `.env` or process environment; the development repository's keys are never bundled or copied. User-level skill installation remains a separate explicit command.

The manually dispatched `Native packages` workflow builds and verifies native archives on Linux, macOS, and Windows and uploads workflow artifacts. It does not create a GitHub release or publish packages. That workflow and the Linux/Windows packages have not been executed during local development; platform support remains subject to those native checks.


The 0.2.0 packager additionally exercises bounded verification and migration previews,
usage/storage reports, and embedded v2 cast/recovery guidance. Its generic process
caller uses a project path containing spaces and receives no API keys. This does not
prove automatic discovery in a fresh Codex/Claude session. Replace a local installation
by stopping its workers, preserving the previous executable, copying the new binary
to the same PATH directory, and checking `mediaagent --version`. Existing project
files are preserved; explicitly refresh bundled skills/templates when desired.

Windows checks remain pending a Windows runner. The configured workflow builds local
archives only and does not publish a release. Linux container evidence identifies
Docker Desktop x86-64 emulation rather than physical x86 hardware.


Verified 0.2.0 artifacts: `mediaagent-0.2.0-aarch64-apple-darwin.tar.gz` and
`mediaagent-0.2.0-x86_64-unknown-linux-gnu.tar.gz` under `target/package-dist/`, each
with a SHA-256 sidecar and 12 passed isolated checks. Both platforms pass all 119
offline tests. Linux x86-64 executes under Docker Desktop emulation on Apple Silicon.
The exact current archive hashes are in `target/remediation-verification.json`.
