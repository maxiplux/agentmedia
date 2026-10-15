# Setup and current models

`mediaagent init PATH` starts the offline terminal wizard when stdin/stderr are terminals. Use arrows and Enter, type to search, and Escape to go back. The checklist loops through features/models, credentials, voices, project-local skills, optional character scaffolds and advanced settings. Finish requires all enabled features to be locally ready. Save unfinished setup to continue later by running init again. No provider requests occur during setup.

`PATH` is the project folder. If omitted, init uses `--project` or the current directory. A downloaded executable named `mediaagent` conflicts with creating `mediaagent/` in that same folder: use `mediaagent init ./my-project` or install the binary on PATH and initialize elsewhere. The wizard detects this before collecting choices and offers another folder; noninteractive callers receive `init_path_conflict` with corrective arguments. Existing files and credentials are preserved. The full destination is shown before setup and during save review.

New projects default to OpenAI GPT Image 2.5 Flare for images/infographics, Fish s2.1-pro for speech/podcasts, and OpenAI GPT-6 Luna for planning. Music and effects stay disabled until selected. Set an actual Fish voice ID; the wizard can reuse it for the default podcast speaker. Custom podcast casts remain separate.

Agents never open the wizard:

```sh
mediaagent init ./project --preset recommended --non-interactive --json
mediaagent --project ./project config status --json
mediaagent --project ./project config show --json
mediaagent models --media image --recommended --json
mediaagent models --provider fish --media speech --json
mediaagent models --all --search image --json
```

`init --preset empty` preserves the old empty scaffold behavior. `init --defaults-file FILE` uses that exact configuration instead of merging a preset. Repeated noninteractive init preserves existing choices. Use `config setup --file FILE` for an explicit validated change. Config status returns `setup_complete`, `pending_steps`, feature readiness and local skill presence. Successful scaffold creation may still report incomplete setup; agents must inspect these fields.

The wizard offers masked entry for missing project keys. Existing environment keys take precedence, including an empty environment value that must be corrected there first. Keys are written only on explicit save to the target project's ignored .env. Do not print keys, insert them into YAML/Markdown, or copy development credentials between projects. Missing/invalid keys, unsupported models and absent voice IDs block Finish. Local readiness does not prove authentication, billing, voice access or media quality.

Only reviewed current stable model families may submit new generation, review, planning or voice-design requests. Older files/results remain inspectable. Unknown, retired and preview models are blocked before provider submission, including retries. Existing successful caches and response recovery remain available. A pinned older character needs a new character version, bootstrap and explicit human acceptance on a current model; never relabel its references or silently upgrade its model.

The bundled catalog records source URLs and verification date; it is an offline snapshot, not a guarantee of account entitlement or knowledge of future releases. `models --all` includes nonselectable historical and unsupported entries. Explicit live discovery is read-only at the provider and writes sanitized metadata to mediaagent/cache/models.yaml:

```sh
mediaagent models --live --provider openai --media image --json
mediaagent models --live --provider openrouter --media image --sort popular --json
```

Discovery uses one bounded page per selected provider and reports truncation. Fish uses a clearly labeled bundled TTS-model catalog because live TTS-model discovery is not implemented. New discoveries cannot authorize execution or replace pinned models. Provider-reported popularity is scoped to OpenRouter; other providers use documented recommendations. Initialization itself never refreshes catalogs or verifies keys online. For explicit authentication checks, use doctor --live with selected providers after setup.
