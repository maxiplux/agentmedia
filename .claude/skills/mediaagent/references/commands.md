# mediaagent 0.5.1 command reference

Generated from the executable's command definitions. Global flags are accepted before or after the command.

## mediaagent

```text
Generate images, infographics, exact speech, podcasts, music, and sound effects. Use direct flags for one asset or versioned YAML jobs for repeated work. Run --help on any command for inputs, output destinations, and examples. Initialization and validation are offline.

Usage: mediaagent [OPTIONS] <COMMAND>

Commands:
  install    Install this downloaded binary in your user bin directory, offline and without administrator access
  update     Update this executable from a verified public GitHub Release; project files stay untouched
  init       Initialize offline, preserving existing files and credentials
  config     Inspect or explicitly replace validated configuration
  verify     Preview a bounded live verification suite; execution requires the exact approved preview hash
  usage      Report provider-returned usage separately from unknown costs
  storage    Inspect storage or apply a previously reviewed conservative cleanup plan
  migrate    Preview migration of a v1 job, config or run; --apply creates a non-overwriting backup
  doctor     Inspect credentials offline, or explicitly check all provider endpoints and keys with --live
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

## mediaagent install

```text
Install this downloaded binary in your user bin directory, offline and without administrator access

Usage: mediaagent install [OPTIONS]

Options:
      --bin-dir <BIN_DIR>
          Executable destination directory (default: ~/.local/bin; Windows: LOCALAPPDATA/mediaagent/bin)

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --no-modify-path
          Do not add the directory to your user PATH or shell profile

      --force
          Explicitly replace an existing executable not managed by mediaagent

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

## mediaagent update

```text
Update this executable from a verified public GitHub Release; project files stay untouched

Usage: mediaagent update [OPTIONS]

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

## mediaagent init

```text
Initialize offline, preserving existing files and credentials

Usage: mediaagent init [OPTIONS] [PATH]

Arguments:
  [PATH]
          Project folder (defaults to --project or the current directory); must allow a mediaagent/ subfolder

Options:
      --agents <AGENTS>
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

## mediaagent config

```text
Inspect or explicitly replace validated configuration

Usage: mediaagent config [OPTIONS] <COMMAND>

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

## mediaagent config wizard

```text
Terminal-only guided defaults; agents use setup --file instead

Usage: mediaagent config wizard [OPTIONS]

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

## mediaagent config host

```text
Explicit user-scope host concurrency ceilings, independent of agent settings

Usage: mediaagent config host [OPTIONS] <COMMAND>

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

## mediaagent config host show

```text
Usage: mediaagent config host show [OPTIONS]

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

## mediaagent config host setup

```text
Usage: mediaagent config host setup [OPTIONS] --file <FILE>

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

## mediaagent config host help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent config host help [COMMAND]

Commands:
  show
  setup
  help   Print this message or the help of the given subcommand(s)

```

## mediaagent config host help show

```text
Usage: mediaagent config host help show

```

## mediaagent config host help setup

```text
Usage: mediaagent config host help setup

```

## mediaagent config host help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent config host help help

```

## mediaagent config show

```text
Usage: mediaagent config show [OPTIONS]

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

## mediaagent config status

```text
Inspect local setup readiness without network calls or revealing credentials

Usage: mediaagent config status [OPTIONS]

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

## mediaagent config setup

```text
Replace config only after validating the supplied file

Usage: mediaagent config setup [OPTIONS] --file <FILE>

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

## mediaagent config limits

```text
Inspect finite generation limits

Usage: mediaagent config limits [OPTIONS]

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

## mediaagent config help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent config help [COMMAND]

Commands:
  wizard  Terminal-only guided defaults; agents use setup --file instead
  host    Explicit user-scope host concurrency ceilings, independent of agent settings
  show
  status  Inspect local setup readiness without network calls or revealing credentials
  setup   Replace config only after validating the supplied file
  limits  Inspect finite generation limits
  help    Print this message or the help of the given subcommand(s)

```

## mediaagent config help wizard

```text
Terminal-only guided defaults; agents use setup --file instead

Usage: mediaagent config help wizard

```

## mediaagent config help host

```text
Explicit user-scope host concurrency ceilings, independent of agent settings

Usage: mediaagent config help host [COMMAND]

Commands:
  show
  setup

```

## mediaagent config help host show

```text
Usage: mediaagent config help host show

```

## mediaagent config help host setup

```text
Usage: mediaagent config help host setup

```

## mediaagent config help show

```text
Usage: mediaagent config help show

```

## mediaagent config help status

```text
Inspect local setup readiness without network calls or revealing credentials

Usage: mediaagent config help status

```

## mediaagent config help setup

```text
Replace config only after validating the supplied file

Usage: mediaagent config help setup

```

## mediaagent config help limits

```text
Inspect finite generation limits

Usage: mediaagent config help limits

```

## mediaagent config help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent config help help

```

## mediaagent verify

```text
Preview a bounded live verification suite; execution requires the exact approved preview hash

Usage: mediaagent verify [OPTIONS] --suite <SUITE>

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

## mediaagent usage

```text
Report provider-returned usage separately from unknown costs

Usage: mediaagent usage [OPTIONS]

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

## mediaagent storage

```text
Inspect storage or apply a previously reviewed conservative cleanup plan

Usage: mediaagent storage [OPTIONS] <COMMAND>

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

## mediaagent storage inspect

```text
Usage: mediaagent storage inspect [OPTIONS]

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

## mediaagent storage plan

```text
Save a cleanup preview; only unreferenced blobs can be removed

Usage: mediaagent storage plan [OPTIONS] --output <OUTPUT>

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

## mediaagent storage clean

```text
Revalidate and apply a saved cleanup preview; fails if stale or workers active

Usage: mediaagent storage clean [OPTIONS] --plan <PLAN>

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

## mediaagent storage help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent storage help [COMMAND]

Commands:
  inspect
  plan     Save a cleanup preview; only unreferenced blobs can be removed
  clean    Revalidate and apply a saved cleanup preview; fails if stale or workers active
  help     Print this message or the help of the given subcommand(s)

```

## mediaagent storage help inspect

```text
Usage: mediaagent storage help inspect

```

## mediaagent storage help plan

```text
Save a cleanup preview; only unreferenced blobs can be removed

Usage: mediaagent storage help plan

```

## mediaagent storage help clean

```text
Revalidate and apply a saved cleanup preview; fails if stale or workers active

Usage: mediaagent storage help clean

```

## mediaagent storage help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent storage help help

```

## mediaagent migrate

```text
Preview migration of a v1 job, config or run; --apply creates a non-overwriting backup

Usage: mediaagent migrate [OPTIONS] --kind <KIND> --file <FILE>

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

## mediaagent doctor

```text
Inspect credentials offline, or explicitly check all provider endpoints and keys with --live

Usage: mediaagent doctor [OPTIONS]

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

## mediaagent schema

```text
Print a JSON Schema for a versioned manifest

Usage: mediaagent schema [OPTIONS] <KIND>

Arguments:
  <KIND>
          [possible values: config, character, infographic, job, batch, run, artifact, review, effective, budget, revision, operation, voice_design, voice_consent, voice_clone, host_limits, diagnostics, verification]

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

## mediaagent models

```text
List implemented provider/model capabilities (not account entitlements)

Usage: mediaagent models [OPTIONS]

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

          [possible values: image, infographic, speech, podcast, music, sfx, text]

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

## mediaagent voices

```text
List built-in voices or search bounded pages of provider account voices

Usage: mediaagent voices [OPTIONS] --provider <PROVIDER>

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

## mediaagent templates

```text
Inspect or export embedded authoring templates

Usage: mediaagent templates [OPTIONS] <COMMAND>

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

## mediaagent templates list

```text
Usage: mediaagent templates list [OPTIONS]

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

## mediaagent templates show

```text
Usage: mediaagent templates show [OPTIONS] <NAME>

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

## mediaagent templates export

```text
Usage: mediaagent templates export [OPTIONS] <DIRECTORY>

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

## mediaagent templates help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent templates help [COMMAND]

Commands:
  list
  show
  export
  help    Print this message or the help of the given subcommand(s)

```

## mediaagent templates help list

```text
Usage: mediaagent templates help list

```

## mediaagent templates help show

```text
Usage: mediaagent templates help show

```

## mediaagent templates help export

```text
Usage: mediaagent templates help export

```

## mediaagent templates help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent templates help help

```

## mediaagent persona

```text
Create, inspect, bootstrap, accept, or port character definitions

Usage: mediaagent persona [OPTIONS] <COMMAND>

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

## mediaagent persona voice-clone

```text
Explicitly create an ElevenLabs clone from selected authorized recordings

Usage: mediaagent persona voice-clone [OPTIONS] --request <REQUEST>

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

## mediaagent persona voice-clone-status

```text
Inspect cloned-voice receipts and unresolved creation attempts

Usage: mediaagent persona voice-clone-status [OPTIONS] <RUN_ID>

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

## mediaagent persona voice-design

```text
Generate ElevenLabs voice preview candidates without creating or binding a saved voice

Usage: mediaagent persona voice-design [OPTIONS] --model <MODEL> --text-file <TEXT_FILE> <ID>

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

## mediaagent persona voice-create

```text
Create a provider voice from one selected candidate; canonical binding is explicit

Usage: mediaagent persona voice-create [OPTIONS] --design-run <DESIGN_RUN> --candidate <CANDIDATE> --name <NAME> <ID>

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

## mediaagent persona voice-status

```text
Inspect saved voice candidates, creation results, and uncertain operations

Usage: mediaagent persona voice-status [OPTIONS] <RUN_ID>

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

## mediaagent persona add

```text
Usage: mediaagent persona add [OPTIONS] <ID>

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

## mediaagent persona list

```text
Usage: mediaagent persona list [OPTIONS]

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

## mediaagent persona show

```text
Usage: mediaagent persona show [OPTIONS] <ID>

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

## mediaagent persona bootstrap

```text
Generate visual candidates; this never promotes references

Usage: mediaagent persona bootstrap [OPTIONS] --output-dir <OUTPUT_DIR> <ID>

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

## mediaagent persona accept-sample

```text
Accept a selected recording, exact transcript, and granted consent as a portable voice sample

Usage: mediaagent persona accept-sample [OPTIONS] --audio <AUDIO> --transcript <TRANSCRIPT> --consent <CONSENT> <ID>

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

## mediaagent persona accept

```text
Accept a user-selected image or recording into the canonical profile

Usage: mediaagent persona accept [OPTIONS] --reference <REFERENCE> --kind <KIND> <ID>

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

## mediaagent persona export

```text
Usage: mediaagent persona export [OPTIONS] --destination <DESTINATION> <ID>

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

## mediaagent persona import

```text
Usage: mediaagent persona import [OPTIONS] <DIRECTORY>

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

## mediaagent persona help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent persona help [COMMAND]

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

## mediaagent persona help voice-clone

```text
Explicitly create an ElevenLabs clone from selected authorized recordings

Usage: mediaagent persona help voice-clone

```

## mediaagent persona help voice-clone-status

```text
Inspect cloned-voice receipts and unresolved creation attempts

Usage: mediaagent persona help voice-clone-status

```

## mediaagent persona help voice-design

```text
Generate ElevenLabs voice preview candidates without creating or binding a saved voice

Usage: mediaagent persona help voice-design

```

## mediaagent persona help voice-create

```text
Create a provider voice from one selected candidate; canonical binding is explicit

Usage: mediaagent persona help voice-create

```

## mediaagent persona help voice-status

```text
Inspect saved voice candidates, creation results, and uncertain operations

Usage: mediaagent persona help voice-status

```

## mediaagent persona help add

```text
Usage: mediaagent persona help add

```

## mediaagent persona help list

```text
Usage: mediaagent persona help list

```

## mediaagent persona help show

```text
Usage: mediaagent persona help show

```

## mediaagent persona help bootstrap

```text
Generate visual candidates; this never promotes references

Usage: mediaagent persona help bootstrap

```

## mediaagent persona help accept-sample

```text
Accept a selected recording, exact transcript, and granted consent as a portable voice sample

Usage: mediaagent persona help accept-sample

```

## mediaagent persona help accept

```text
Accept a user-selected image or recording into the canonical profile

Usage: mediaagent persona help accept

```

## mediaagent persona help export

```text
Usage: mediaagent persona help export

```

## mediaagent persona help import

```text
Usage: mediaagent persona help import

```

## mediaagent persona help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent persona help help

```

## mediaagent validate

```text
Validate local YAML and all referenced files; never generate

Usage: mediaagent validate [OPTIONS] --kind <KIND> <PATH>

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

## mediaagent generate

```text
Generate one media job. Example: generate image --prompt 'A forest' --provider openai --model gpt-image-2.5-flare --output forest.png

Usage: mediaagent generate [OPTIONS] [MEDIA]

Arguments:
  [MEDIA]
          [possible values: image, infographic, speech, podcast, music, sfx, text]

Options:
      --job <JOB>
          Reusable YAML job; settings/provider overrides are explicit, authoring flags conflict

      --project <PROJECT>
          Target project root; credentials are loaded only from this root's .env

          [default: .]

      --identity-reference <IDENTITY_REFERENCE>
          Preserve referenced character identities; repeat for multiple selected characters

      --json
          Emit a structured JSON result; diagnostics stay on stderr, with no animation

      --input-fidelity <INPUT_FIDELITY>
          Historical image-edit fidelity field; omit for current GPT Image 2.5 models

          [possible values: high, low]

      --non-interactive
          Never prompt; supply YAML files or explicit flags for automation

      --progress <PROGRESS>
          Terminal progress mode; redirected auto progress uses plain stage updates

          [default: auto]
          [possible values: auto, plain, off]

      --voice-sample <VOICE_SAMPLE>
          Authorized audio sample for Fish reference synthesis or voice review evidence

      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

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

## mediaagent batch

```text
Execute independent jobs concurrently using a YAML batch manifest

Usage: mediaagent batch [OPTIONS] --manifest <MANIFEST>

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

## mediaagent runs

```text
Inspect, wait, cancel, or resume durable runs

Usage: mediaagent runs [OPTIONS] <COMMAND>

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

## mediaagent runs recover

```text
Stage a manually recovered response for an uncertain job; resume publishes without a new paid request

Usage: mediaagent runs recover [OPTIONS] --job <JOB> --file <FILE> --format <FORMAT> --fingerprint <FINGERPRINT> --evidence <EVIDENCE> <RUN_ID>

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

## mediaagent runs reconcile

```text
Retrieve saved Google interactions without a new generation; resume publishes recovered output

Usage: mediaagent runs reconcile [OPTIONS] <RUN_ID>

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

## mediaagent runs list

```text
Usage: mediaagent runs list [OPTIONS]

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

## mediaagent runs status

```text
Usage: mediaagent runs status [OPTIONS] <RUN_ID>

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

## mediaagent runs wait

```text
Usage: mediaagent runs wait [OPTIONS] <RUN_ID>

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

## mediaagent runs cancel

```text
Usage: mediaagent runs cancel [OPTIONS] <RUN_ID>

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

## mediaagent runs resume

```text
Usage: mediaagent runs resume [OPTIONS] <RUN_ID>

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

## mediaagent runs help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent runs help [COMMAND]

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

## mediaagent runs help recover

```text
Stage a manually recovered response for an uncertain job; resume publishes without a new paid request

Usage: mediaagent runs help recover

```

## mediaagent runs help reconcile

```text
Retrieve saved Google interactions without a new generation; resume publishes recovered output

Usage: mediaagent runs help reconcile

```

## mediaagent runs help list

```text
Usage: mediaagent runs help list

```

## mediaagent runs help status

```text
Usage: mediaagent runs help status

```

## mediaagent runs help wait

```text
Usage: mediaagent runs help wait

```

## mediaagent runs help cancel

```text
Usage: mediaagent runs help cancel

```

## mediaagent runs help resume

```text
Usage: mediaagent runs help resume

```

## mediaagent runs help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent runs help help

```

## mediaagent resume

```text
Continue a recorded run, preserving successes; ambiguous retries are explicit

Usage: mediaagent resume [OPTIONS] <RUN_ID>

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

## mediaagent review

```text
Check artifact integrity, request explicit image/audio AI review, or record a human/agent review

Usage: mediaagent review [OPTIONS] <COMMAND>

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

## mediaagent review report

```text
Export a comparison report and immutable references without making acceptance decisions

Usage: mediaagent review report [OPTIONS] --artifact <ARTIFACT> --output <OUTPUT>

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

## mediaagent review reconcile

```text
Retrieve uncertain Google audio reviews without submitting another assessment

Usage: mediaagent review reconcile [OPTIONS] --artifact <ARTIFACT>

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

## mediaagent review status

```text
Inspect recorded AI review attempts, provider receipts, and uncertain outcomes

Usage: mediaagent review status [OPTIONS] --artifact <ARTIFACT>

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

## mediaagent review check

```text
Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio

Usage: mediaagent review check [OPTIONS] --artifact <ARTIFACT>

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

## mediaagent review record

```text
Record a schema-validated assessment with human/agent provenance

Usage: mediaagent review record [OPTIONS] --file <FILE>

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

## mediaagent review help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent review help [COMMAND]

Commands:
  report     Export a comparison report and immutable references without making acceptance decisions
  reconcile  Retrieve uncertain Google audio reviews without submitting another assessment
  status     Inspect recorded AI review attempts, provider receipts, and uncertain outcomes
  check      Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio
  record     Record a schema-validated assessment with human/agent provenance
  help       Print this message or the help of the given subcommand(s)

```

## mediaagent review help report

```text
Export a comparison report and immutable references without making acceptance decisions

Usage: mediaagent review help report

```

## mediaagent review help reconcile

```text
Retrieve uncertain Google audio reviews without submitting another assessment

Usage: mediaagent review help reconcile

```

## mediaagent review help status

```text
Inspect recorded AI review attempts, provider receipts, and uncertain outcomes

Usage: mediaagent review help status

```

## mediaagent review help check

```text
Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio

Usage: mediaagent review help check

```

## mediaagent review help record

```text
Record a schema-validated assessment with human/agent provenance

Usage: mediaagent review help record

```

## mediaagent review help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent review help help

```

## mediaagent revise

```text
Create a revision child run with inherited identity and limits

Usage: mediaagent revise [OPTIONS] --artifact <ARTIFACT> --instructions-file <INSTRUCTIONS_FILE>

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

      --async


      --color <COLOR>
          Terminal colors; auto honors NO_COLOR

          [default: auto]
          [possible values: auto, always, never]

  -h, --help
          Print help

```

## mediaagent plan

```text
Explicit text-model planning; saves reviewable files without generating media

Usage: mediaagent plan [OPTIONS] --kind <KIND> --prompt <PROMPT> --output-dir <OUTPUT_DIR>

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

      --provider <PROVIDER>
          [possible values: openai, google, fish, openrouter, elevenlabs]

      --model <MODEL>


  -h, --help
          Print help

```

## mediaagent skills

```text
Export a portable skill or install it in agent discovery locations

Usage: mediaagent skills [OPTIONS] <COMMAND>

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

## mediaagent skills export

```text
Usage: mediaagent skills export [OPTIONS] <DIRECTORY>

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

## mediaagent skills install

```text
Usage: mediaagent skills install [OPTIONS]

Options:
      --agents <AGENTS>
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

## mediaagent skills help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent skills help [COMMAND]

Commands:
  export
  install
  help     Print this message or the help of the given subcommand(s)

```

## mediaagent skills help export

```text
Usage: mediaagent skills help export

```

## mediaagent skills help install

```text
Usage: mediaagent skills help install

```

## mediaagent skills help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent skills help help

```

## mediaagent help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent help [COMMAND]

Commands:
  install    Install this downloaded binary in your user bin directory, offline and without administrator access
  update     Update this executable from a verified public GitHub Release; project files stay untouched
  init       Initialize offline, preserving existing files and credentials
  config     Inspect or explicitly replace validated configuration
  verify     Preview a bounded live verification suite; execution requires the exact approved preview hash
  usage      Report provider-returned usage separately from unknown costs
  storage    Inspect storage or apply a previously reviewed conservative cleanup plan
  migrate    Preview migration of a v1 job, config or run; --apply creates a non-overwriting backup
  doctor     Inspect credentials offline, or explicitly check all provider endpoints and keys with --live
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

## mediaagent help install

```text
Install this downloaded binary in your user bin directory, offline and without administrator access

Usage: mediaagent help install

```

## mediaagent help update

```text
Update this executable from a verified public GitHub Release; project files stay untouched

Usage: mediaagent help update

```

## mediaagent help init

```text
Initialize offline, preserving existing files and credentials

Usage: mediaagent help init

```

## mediaagent help config

```text
Inspect or explicitly replace validated configuration

Usage: mediaagent help config [COMMAND]

Commands:
  wizard  Terminal-only guided defaults; agents use setup --file instead
  host    Explicit user-scope host concurrency ceilings, independent of agent settings
  show
  status  Inspect local setup readiness without network calls or revealing credentials
  setup   Replace config only after validating the supplied file
  limits  Inspect finite generation limits

```

## mediaagent help config wizard

```text
Terminal-only guided defaults; agents use setup --file instead

Usage: mediaagent help config wizard

```

## mediaagent help config host

```text
Explicit user-scope host concurrency ceilings, independent of agent settings

Usage: mediaagent help config host [COMMAND]

Commands:
  show
  setup

```

## mediaagent help config host show

```text
Usage: mediaagent help config host show

```

## mediaagent help config host setup

```text
Usage: mediaagent help config host setup

```

## mediaagent help config show

```text
Usage: mediaagent help config show

```

## mediaagent help config status

```text
Inspect local setup readiness without network calls or revealing credentials

Usage: mediaagent help config status

```

## mediaagent help config setup

```text
Replace config only after validating the supplied file

Usage: mediaagent help config setup

```

## mediaagent help config limits

```text
Inspect finite generation limits

Usage: mediaagent help config limits

```

## mediaagent help verify

```text
Preview a bounded live verification suite; execution requires the exact approved preview hash

Usage: mediaagent help verify

```

## mediaagent help usage

```text
Report provider-returned usage separately from unknown costs

Usage: mediaagent help usage

```

## mediaagent help storage

```text
Inspect storage or apply a previously reviewed conservative cleanup plan

Usage: mediaagent help storage [COMMAND]

Commands:
  inspect
  plan     Save a cleanup preview; only unreferenced blobs can be removed
  clean    Revalidate and apply a saved cleanup preview; fails if stale or workers active

```

## mediaagent help storage inspect

```text
Usage: mediaagent help storage inspect

```

## mediaagent help storage plan

```text
Save a cleanup preview; only unreferenced blobs can be removed

Usage: mediaagent help storage plan

```

## mediaagent help storage clean

```text
Revalidate and apply a saved cleanup preview; fails if stale or workers active

Usage: mediaagent help storage clean

```

## mediaagent help migrate

```text
Preview migration of a v1 job, config or run; --apply creates a non-overwriting backup

Usage: mediaagent help migrate

```

## mediaagent help doctor

```text
Inspect credentials offline, or explicitly check all provider endpoints and keys with --live

Usage: mediaagent help doctor

```

## mediaagent help schema

```text
Print a JSON Schema for a versioned manifest

Usage: mediaagent help schema

```

## mediaagent help models

```text
List implemented provider/model capabilities (not account entitlements)

Usage: mediaagent help models

```

## mediaagent help voices

```text
List built-in voices or search bounded pages of provider account voices

Usage: mediaagent help voices

```

## mediaagent help templates

```text
Inspect or export embedded authoring templates

Usage: mediaagent help templates [COMMAND]

Commands:
  list
  show
  export

```

## mediaagent help templates list

```text
Usage: mediaagent help templates list

```

## mediaagent help templates show

```text
Usage: mediaagent help templates show

```

## mediaagent help templates export

```text
Usage: mediaagent help templates export

```

## mediaagent help persona

```text
Create, inspect, bootstrap, accept, or port character definitions

Usage: mediaagent help persona [COMMAND]

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

## mediaagent help persona voice-clone

```text
Explicitly create an ElevenLabs clone from selected authorized recordings

Usage: mediaagent help persona voice-clone

```

## mediaagent help persona voice-clone-status

```text
Inspect cloned-voice receipts and unresolved creation attempts

Usage: mediaagent help persona voice-clone-status

```

## mediaagent help persona voice-design

```text
Generate ElevenLabs voice preview candidates without creating or binding a saved voice

Usage: mediaagent help persona voice-design

```

## mediaagent help persona voice-create

```text
Create a provider voice from one selected candidate; canonical binding is explicit

Usage: mediaagent help persona voice-create

```

## mediaagent help persona voice-status

```text
Inspect saved voice candidates, creation results, and uncertain operations

Usage: mediaagent help persona voice-status

```

## mediaagent help persona add

```text
Usage: mediaagent help persona add

```

## mediaagent help persona list

```text
Usage: mediaagent help persona list

```

## mediaagent help persona show

```text
Usage: mediaagent help persona show

```

## mediaagent help persona bootstrap

```text
Generate visual candidates; this never promotes references

Usage: mediaagent help persona bootstrap

```

## mediaagent help persona accept-sample

```text
Accept a selected recording, exact transcript, and granted consent as a portable voice sample

Usage: mediaagent help persona accept-sample

```

## mediaagent help persona accept

```text
Accept a user-selected image or recording into the canonical profile

Usage: mediaagent help persona accept

```

## mediaagent help persona export

```text
Usage: mediaagent help persona export

```

## mediaagent help persona import

```text
Usage: mediaagent help persona import

```

## mediaagent help validate

```text
Validate local YAML and all referenced files; never generate

Usage: mediaagent help validate

```

## mediaagent help generate

```text
Generate one media job. Example: generate image --prompt 'A forest' --provider openai --model gpt-image-2.5-flare --output forest.png

Usage: mediaagent help generate

```

## mediaagent help batch

```text
Execute independent jobs concurrently using a YAML batch manifest

Usage: mediaagent help batch

```

## mediaagent help runs

```text
Inspect, wait, cancel, or resume durable runs

Usage: mediaagent help runs [COMMAND]

Commands:
  recover    Stage a manually recovered response for an uncertain job; resume publishes without a new paid request
  reconcile  Retrieve saved Google interactions without a new generation; resume publishes recovered output
  list
  status
  wait
  cancel
  resume

```

## mediaagent help runs recover

```text
Stage a manually recovered response for an uncertain job; resume publishes without a new paid request

Usage: mediaagent help runs recover

```

## mediaagent help runs reconcile

```text
Retrieve saved Google interactions without a new generation; resume publishes recovered output

Usage: mediaagent help runs reconcile

```

## mediaagent help runs list

```text
Usage: mediaagent help runs list

```

## mediaagent help runs status

```text
Usage: mediaagent help runs status

```

## mediaagent help runs wait

```text
Usage: mediaagent help runs wait

```

## mediaagent help runs cancel

```text
Usage: mediaagent help runs cancel

```

## mediaagent help runs resume

```text
Usage: mediaagent help runs resume

```

## mediaagent help resume

```text
Continue a recorded run, preserving successes; ambiguous retries are explicit

Usage: mediaagent help resume

```

## mediaagent help review

```text
Check artifact integrity, request explicit image/audio AI review, or record a human/agent review

Usage: mediaagent help review [COMMAND]

Commands:
  report     Export a comparison report and immutable references without making acceptance decisions
  reconcile  Retrieve uncertain Google audio reviews without submitting another assessment
  status     Inspect recorded AI review attempts, provider receipts, and uncertain outcomes
  check      Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio
  record     Record a schema-validated assessment with human/agent provenance

```

## mediaagent help review report

```text
Export a comparison report and immutable references without making acceptance decisions

Usage: mediaagent help review report

```

## mediaagent help review reconcile

```text
Retrieve uncertain Google audio reviews without submitting another assessment

Usage: mediaagent help review reconcile

```

## mediaagent help review status

```text
Inspect recorded AI review attempts, provider receipts, and uncertain outcomes

Usage: mediaagent help review status

```

## mediaagent help review check

```text
Verify an artifact; AI review uses OpenAI/OpenRouter for images and Google for audio

Usage: mediaagent help review check

```

## mediaagent help review record

```text
Record a schema-validated assessment with human/agent provenance

Usage: mediaagent help review record

```

## mediaagent help revise

```text
Create a revision child run with inherited identity and limits

Usage: mediaagent help revise

```

## mediaagent help plan

```text
Explicit text-model planning; saves reviewable files without generating media

Usage: mediaagent help plan

```

## mediaagent help skills

```text
Export a portable skill or install it in agent discovery locations

Usage: mediaagent help skills [COMMAND]

Commands:
  export
  install

```

## mediaagent help skills export

```text
Usage: mediaagent help skills export

```

## mediaagent help skills install

```text
Usage: mediaagent help skills install

```

## mediaagent help help

```text
Print this message or the help of the given subcommand(s)

Usage: mediaagent help help

```
