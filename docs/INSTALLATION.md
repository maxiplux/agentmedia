# Install and update mediaagent

Install without Rust or administrator access. Public binaries support macOS Apple Silicon, macOS Intel, Linux x86-64 and Windows x86-64. Other targets can build from source; installers reject unsupported architectures rather than select a different binary.

## macOS and Linux

```sh
curl -fsSL https://maxiplux.github.io/agentmedia/install.sh | sh
```

The installer downloads the latest stable version, verifies its checksum and version, and installs to `~/.local/bin`. It adds this directory to the relevant zsh/bash/login profile when needed. Open a new terminal, then run:

```sh
mediaagent --version
mediaagent init ./my-project
```

For an exact version or a custom directory, download the script first:

```sh
curl -fsSL https://maxiplux.github.io/agentmedia/install.sh -o install-mediaagent.sh
sh install-mediaagent.sh --version 0.4.0 --bin-dir "$HOME/tools/bin" --no-modify-path
```

`curl` and a SHA-256 tool (`sha256sum` or macOS `shasum`) are required. Fish users receive a manual `fish_add_path` instruction. No shell profile is changed with `--no-modify-path`.

## Windows PowerShell

```powershell
irm https://maxiplux.github.io/agentmedia/install.ps1 | iex
```

The installer verifies the Windows x86-64 executable and installs it to `%LOCALAPPDATA%\mediaagent\bin`, adding only that directory to your user PATH. Restart your terminal if it was already open. It does not require an administrator or change the machine PATH or execution policy.

To select a version or custom directory:

```powershell
& ([scriptblock]::Create((irm https://maxiplux.github.io/agentmedia/install.ps1))) -Version 0.4.0 -BinDir "$HOME\tools\bin" -NoModifyPath
```

## Already downloaded the binary?

From the download folder:

```sh
./mediaagent install
```

Windows: `./mediaagent.exe install`. No network is needed for this operation. The binary is copied into your user bin directory, so subsequent `init` commands can create the `mediaagent/` project folder without colliding with the downloaded executable. Manual downloads are available from [the latest public release](https://github.com/maxiplux/agentmedia/releases/latest); GitHub sign-in is not required for public release assets.

`--bin-dir PATH` chooses another executable directory. `--no-modify-path` preserves your PATH and profiles. Existing unmanaged files are refused unless you explicitly add `--force`. For source builds, `cargo install --path . --locked` remains available.

## Updates and rollback

```sh
mediaagent update --check
mediaagent update
mediaagent update --rollback
```

`--check` reads bounded public GitHub metadata and reports the installed and available versions without downloading a binary or changing files. `update` replaces the executable you invoked after checking GitHub's asset digest and the new executable's version. One preceding executable is saved for offline rollback. An unmanaged file replaced with `install --force` is preserved with an unknown version and restored by its recorded hash; it is never executed just to identify its version. Stop generation workers before updating; other copies on PATH are not replaced.

An exact stable version can be selected with `--version 0.4.0`. Reinstall the same version with `--force`. An intentional downgrade additionally requires `--allow-downgrade`. For scripts and agents, all commands support `--json --non-interactive`.

Binary installation and updates preserve project `.env`, configuration, characters, references, generated media and installed skills. Initialize a project with `init` separately. Refresh a project's installed skills explicitly with `mediaagent --project PATH skills install --overwrite` when desired. Original character model locks are never rewritten by updating the binary.

Public releases include raw executables, native archives, SHA-256 sidecars, installers and a release manifest identifying the exact commit and hashes. Downloads use HTTPS and checksums; binaries are not signed or notarized. No provider API keys are used and no paid media calls occur during installation or updates.

## Maintainer release workflow

Bump Cargo's version, update the lock file, merge tested changes, then push a matching stable tag such as `v0.4.0`. The `Publish release` workflow builds/tests all four native targets, runs isolated package and bootstrap tests, verifies archives and raw binary hashes, uploads to a draft GitHub Release and checks GitHub's stored digests before publishing it as latest. Existing published release bytes cannot be silently replaced; a changed build needs a new version.
