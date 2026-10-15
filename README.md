# FrameChord

Formerly MediaAgent. The first FrameChord release is 0.6.0. Existing projects stay supported in place; see [upgrade guidance](assets/skill/references/renaming.md). The owned domain is **framechord.com**; [custom-domain setup](docs/DOMAIN.md) is tracked separately from the published guide.

A local Rust CLI for any agent with process access, and for terminal users. Generate images, raster infographics, exact narration, podcasts, music, and sound effects. Characters are optional: save visual identity and a provider voice binding when you need reuse.

Open the [web guide](https://maxiplux.github.io/framechord/) for copyable examples for every media
workflow, agent prompts, downloadable YAML/Markdown inputs, and a searchable command
reference. It is a single HTML file with embedded JavaScript, compiled Tailwind CSS, command
help and example downloads. You can also [open the HTML locally](docs/site/index.html);
see [publishing and preview instructions](docs/WEB_GUIDE.md).

The first-version workflows are implemented; visual character consistency has not passed user acceptance. Offline contracts and mocked
execution tests plus bounded live checks across all five providers. It includes
voice design/cloning, authorized Fish recording synthesis, audio/image AI review,
Google request retrieval, interactive configuration, and native local packaging.
Provider voice creation/cloning and recording synthesis have offline coverage;
live testing still needs selected recordings and actual speaker authorization.
Human character consistency and media quality checks remain separate. See
[implementation status](docs/STATUS.md) for exact boundaries.

## Guided setup and current models

Run `framechord init` for a looping, offline terminal setup. Use arrows, Enter and
search to choose options, and Escape to return. New projects default to Google:
Nano Banana 2.1 for images/infographics, Gemini 3.8 Flash TTS with built-in `Kore`
for speech/podcasts, Lyria 3.5 for music, and Gemini 3.8 Flash for planning.
Set `GEMINI_API_KEY` to make these defaults locally ready. OpenAI is the second
welcome option (`--preset openai`), with built-in `coral` for speech; music and
effects are disabled in that preset. Standalone sound effects remain an optional
ElevenLabs feature. Provider changes are always explicit.

Init installs Codex and Claude Code skills inside the project by default. It reads
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
curl -fsSL https://maxiplux.github.io/framechord/install.sh | sh
```

Windows PowerShell:

```powershell
irm https://maxiplux.github.io/framechord/install.ps1 | iex
```

Already downloaded a 0.6.0+ binary? Run `./framechord install` (Windows:
`./framechord.exe install`). Installation uses a user bin directory, verifies
release bytes, and needs no administrator or Rust installation. Open a new terminal,
then use `framechord init ./my-project` from your project folder.

```sh
framechord update --check
framechord update
framechord update --rollback
```

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
framechord generate --job authoring/job.yaml --image-quality low --dry-run --json
framechord batch --manifest batch.yaml --output-dir exports --async --json
framechord runs status RUN_ID --json
framechord runs wait RUN_ID
```

Initialization is offline and preserves existing configuration, `.env`, authored files, and modified skills. A fresh interactive terminal opens setup for per-media models, output settings, voices, locales, consistency, and limits; agents supply the complete config with `--defaults-file`. The bundled `project-config.yaml` is an editable example. Without supplied choices, noninteractive init creates an unconfigured scaffold and reports setup is required. Initial and newly created personas inherit configured image and speech profiles; absent choices leave those profiles disabled. Explicit job flags override compatible settings without rewriting source YAML, while selected original identity assets and pinned characters reject conflicting provider/model changes. Podcast defaults supply unconfigured turns; explicit turn models and characters retain their own bindings. Completed narrator, visual mascot, and combined examples are bundled under `templates/examples/`.

The visible `framechord/` folder holds YAML configuration, Markdown/YAML authoring templates, portable character folders, job briefs, and run records. Relative paths resolve from their containing manifest. Generated run artifacts and credentials are ignored; authored definitions can be versioned.

Infographics use `infographic.yaml` plus a visual Markdown brief. Exact copy, numeric strings, units, source notes, ordered sections, canvas, and layout compile into the saved prompt. Reference objects specify `path` and `role` (`identity`, `style`, `layout`, or `content`). Rendered typography and data accuracy still need review.

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

Generation success and semantic quality are separate. Local artifact integrity is checked automatically. Explicit `review check --ai --provider ... --model ...` supports image review using OpenAI/OpenRouter multimodal text models; Google audio review supports `gemini-3.8-flash` against exact scripts, briefs, and accepted voice recordings. Inline requests must fit 20 MiB; review individual clips for larger jobs. Completed AI reviews are saved with provider receipts and reused on identical commands; unknown review outcomes require explicit `--retry-unknown`. `review reconcile --artifact ID` retrieves uncertain Google audio assessments without resubmission. Human or agent assessments require provenance. Revisions create child runs, preserve originals, and inherit budgets. `--source-job` enables explicit updated scripts/copy; `--turn` targets a podcast turn and reuses unchanged clips.

## Agent integration and limits

`init` installs project-local skills in `.agents/skills/framechord/` and `.claude/skills/framechord/`. `skills export DIRECTORY` creates a portable package for other agents. `skills install --scope user --agents codex,claude` is a separate explicit user installation. No global agent settings or permissions are rewritten. Existing bundled files are preserved by default; `skills install --overwrite` or `templates export DIRECTORY --overwrite` explicitly refreshes bundled files, including modified ones, while preserving unrelated files.

The exported skill includes all current commands/flags generated from the same CLI definitions, schemas, templates, authoring examples, lifecycle, review, progress, and limit guidance. Use `--json --non-interactive` for process callers. JSON stays on stdout; progress and diagnostics use stderr. Result views show reference metadata/hashes, while complete embedded bytes stay in run YAML. Interactive terminals get a colored job-count bar and a spinner for unknown provider completion. `--progress auto|plain|off`, `--color auto|always|never`, and `NO_COLOR` control presentation.

Default budgets per run lineage: 20 creation requests, 8 images, 1,800 reserved audio seconds, 60,000 speech characters, 3 revisions, 3 attempts per job. Default host concurrency: 4 globally, 2 per provider; per-run limits can be configured up to 32/16, while shared host ceilings also apply. `config host show` reads ceilings; explicit `config host setup --file host-limits.yaml` changes only framechord user configuration. `--limits-file` supplies explicit per-run limits. Speech duration is an estimate; overrun artifacts are retained and further requests stop when the budget is exhausted. These are volume controls, not a dollar cap.

Requests do not silently switch providers. Matching fingerprints reuse only verified local artifacts; `--fresh` bypasses reuse. `--request-id` provides submission idempotency. Safe rejected jobs can be resumed, with provider retry guidance honored. Use `runs reconcile RUN_ID` to retrieve uncertain Google interactions by saved ID, then resume local publication. Unknown submissions without retrievable results require explicit `--retry-unknown`; they may already have been charged. Successful jobs remain intact when a batch fails.

## Verification

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
```

Normal tests mock providers and run offline even if credentials exist. The CI matrix checks macOS, Linux, and Windows; only local macOS has been executed during this implementation.

`python3 scripts/live-smoke.py --provider openai --model gpt-image-2.5-flare --media image` reports skipped without generating. Add `--live` to explicitly enable at most one request using this project's credentials and strict limits. Speech also requires `--voice`. Google music smoke tests request the 30-second `lyria-3-clip-preview` preset; the returned duration is measured separately and can differ. Skipped credentials or unavailable account capabilities are never represented as passed live tests.

`python3 scripts/live-voice-design.py --model eleven_ttv_v3 --speech-model eleven_flash_v2_5 --live` makes one bounded request for up to three fictional narrator previews. It snapshots its temporary character inside the ignored run, preserves `.env`, and never creates or binds a provider voice. Omit `--live` to report skipped.

`python3 scripts/live-recording.py --mode synthesis` or `--mode clone` reports skipped by default. Live recording tests require explicitly selected local audio, actual permission/evidence, and `--live`; cloning creates an account voice without canonical binding or automatic reference acceptance.

`python3 scripts/live-audio-review.py --source-run RUN_ID --model gemini-3.8-flash --live` reviews a verified cached short speech clip with one assessment request; it preserves the source generation.

A local native binary can be packaged with `python3 scripts/package-release.py --offline`; the packager verifies it outside this checkout without provider calls. See [packaging](docs/PACKAGING.md) for the current Apple Silicon archive and the unexecuted cross-platform workflow.

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
