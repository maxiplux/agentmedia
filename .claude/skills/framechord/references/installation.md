# Installing and updating the binary

Use public GitHub Releases at https://github.com/maxiplux/framechord/releases/latest. For a downloaded 0.6.1+ executable, `./framechord install` (Windows `./framechord.exe install`) copies it to the user's bin directory without Rust, administrator access, provider calls or project initialization. Default destinations: `~/.local/bin` on macOS/Linux and `%LOCALAPPDATA%/framechord/bin` on Windows. It preserves project configuration, credentials, characters, runs and skills.

```sh
./framechord install
./framechord install --bin-dir /chosen/bin --no-modify-path --json
framechord update --check --json
framechord update --json
framechord update --version 0.6.1 --force --json
framechord update --rollback --json
```

Installation adds the directory to the relevant shell profile or Windows user PATH unless `--no-modify-path` is supplied. An unmanaged executable is preserved unless `--force` is explicit. Updates replace the executable actually invoked, not another copy found on PATH. `--check` reads bounded public release metadata without downloading a binary or writing files. A normal update selects the latest stable release, verifies GitHub's SHA-256 digest and executable version before replacement, and saves one preceding executable for offline rollback. Explicit older versions require `--allow-downgrade`. No automatic provider/model/voice migration, project initialization or skill refresh occurs. With multiple installed copies, inspect the returned `executable` path.

Installing/updating software and editing PATH should be part of the user's request. Do not upgrade the binary merely to carry out an unrelated media request. For agent-driven setup use `--json`; use `--no-modify-path` when profile changes are outside the requested scope. A missing binary is an installation issue; do not silently substitute a media tool or put credentials into installer arguments. Use `skills install --overwrite` only when a skill refresh is explicitly wanted.

One-command installers and exact-version/custom-directory options are documented at https://github.com/maxiplux/framechord/blob/master/docs/INSTALLATION.md. Checksums verify release bytes; they are not signing or notarization. Installation/update uses GitHub, not media-provider keys.
