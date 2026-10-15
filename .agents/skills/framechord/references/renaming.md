# FrameChord and older MediaAgent projects

FrameChord 0.6.1 renames the executable, Rust package and skill to `framechord`.
The GitHub repository is `maxiplux/framechord`. The owned domain is
`framechord.com`; domain registration and a working DNS/HTTPS deployment are
separate steps. The documentation remains reachable through its published
GitHub Pages address while custom-domain setup is pending.

## Install the renamed executable

Install FrameChord using the current installation instructions. Older
`mediaagent update` commands cannot perform the product rename: release asset
names, executable identity and installation receipts have changed. Install the
new binary explicitly, check `framechord --version`, then use `framechord` for
subsequent updates. Existing `mediaagent` executables and their rollback receipts
are preserved; do not rename them into managed FrameChord installations.

## Keep existing projects in place

New projects initialize a visible `framechord/` directory. When an existing
`mediaagent/` directory is present, FrameChord uses it in place for configuration,
characters, templates, jobs, runs, caches and reference blobs. It does not move
the directory, rewrite original receipts, upgrade pinned models or accept media.
Relative dependencies and historical absolute artifact paths remain intact.
Do not rename old project folders or replace paths inside saved manifests.

Run `framechord init --json --non-interactive` from the project root. The result
reports `data_directory` and `legacy_layout`; agents must use those paths for
existing projects. Initialization installs the new project-local skills at
`.agents/skills/framechord/` and `.claude/skills/framechord/`. Existing modified
skills are preserved. Old `skills/mediaagent/` directories also remain untouched;
after reviewing the new skill, the user can remove obsolete old skill folders
to prevent duplicate discovery. User-level skills require a separate explicit
installation with `skills install --scope user`.

A root containing both project directories is rejected with
`project_layout_conflict`. Inspect them and place each project under a separate
root before continuing; do not merge or select one automatically. Credentials
remain in the selected root's `.env`, with the same process-environment priority.

The development checkout deliberately retains its historical `mediaagent/`
authored data and ignored evidence. This is compatibility data, not an incomplete
branding change. Historical test and acceptance reports retain their original
commands, release names and artifact paths.

## Preserve host ceilings

FrameChord also reuses `~/.config/mediaagent/host.yaml` when it exists. New user
settings use `~/.config/framechord/host.yaml`. Conflicting files are rejected
instead of silently replacing a selected limit with a higher default. Stop
workers before explicitly reconciling duplicate host settings. Old and new
binaries share the existing host slot locks.
