---
name: mediaagent
description: Generate images, infographics, speech, podcasts, music, and sound effects with the mediaagent CLI, using optional reusable character identities and local YAML and Markdown briefs.
---

# Mediaagent

Use the `mediaagent` binary from any agent that can run a process. If it is missing, explain `cargo install --path /path/to/mediaagent` or use a release binary. Do not switch media tools or make provider API calls inside the skill.

Read [commands](references/commands.md) for every command and flag in this binary version; [workflow](references/workflow.md) for outputs, limits, recovery, and review; [authoring](references/authoring.md) for character and infographic templates. Machine-readable schemas are in `references/schemas/`. Exported templates are in `templates/`.

Read [character continuity](references/consistency.md) when reusing the same characters across scenes. Use selected identity references, supported fidelity controls, and comparison against the actual original images; API success does not establish likeness.

Read [connection checks](references/connections.md) when checking setup or provider/key failures: `doctor` checks presence offline; `doctor --live` explicitly authenticates with read-only requests. It supports provider filtering, timeouts, and JSON. Read status/actions per provider; successful authentication does not establish generation/model/billing readiness.

Read [voices](references/voices.md) when discovering voices, designing previews, accepting authorized recordings, or explicitly creating/cloning/binding a selected provider voice. Preview generation, provider-side creation, canonical binding, and acceptance of recordings are separate operations.

For a single asset, use explicit provider/model/input/output flags without initialization. For repeated work, use `init PATH --interactive` or `init PATH --defaults-file templates/project-config.yaml --json --non-interactive` to save models, settings, voices, locales, consistency, and limits; inspect `mediaagent config show --json`, and use saved defaults. `mediaagent models` reports the capability allowlist, not account entitlement. Inspect built-in voices with `voices --provider PROVIDER`; `--account` performs an explicit bounded catalog lookup where supported.

Preserve requested dialogue, lyrics, numbers, and labels exactly. Use a separate explicit planning operation when adapting an article into a podcast. Supply article text in a local file. Characters are optional; when selected, reuse their accepted references and pinned voice/provider/model. Derived identity assets must use the original asset’s recorded provider/model; conflicting overrides fail. Preserve origin metadata instead of copying the project default into derived job YAML. Describe stable identity separately from pose, scene, delivery, or temporary clothing. Candidate references need the user's selection before acceptance.

Author files using templates, then `validate`. Inspect `generate --job ... --dry-run --json` before execution when inputs are new. Call generation only when it is part of the user's request. Use `--json --non-interactive` for automation. Report artifact paths returned by the binary and incomplete jobs honestly.

Use `runs status`, `runs wait`, `runs reconcile`, and `runs resume` for recorded runs. Retrieve supported provider results before considering a paid retry. Never silently retry an unknown paid request; `--retry-unknown` requires explicit user authorization. Reviews record quality separately from generation success. Review does not automatically revise, and revisions do not replace canonical character references.
