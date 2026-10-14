# Character continuity

Use this workflow when the user wants the same character in another pose, scene,
or group. Similar colors, clothing, or a provider voice ID do not establish
visual identity. Generated candidates require selection before canonical acceptance.
A scene request may select particular images for that job without promoting them.

For a saved character, reuse its selected `character.yaml` and accepted visual
references. For multiple selected characters or job-specific references, use
repeatable `--identity-reference FILE`, or explicit YAML objects:

```yaml
references:
  - path: mira.png
    role: identity
  - path: kito.png
    role: identity
  - path: clearing-layout.png
    role: layout
settings:
  input_fidelity: high
```

Bare reference paths and `--reference` retain their generic content role for
compatibility. They are insufficient to express an identity requirement. Identity
references are ordered before other roles, with filenames included in the compiled
reference guide. Keep character-to-reference associations explicit in the brief.

For OpenAI `gpt-image-1` and `gpt-image-1.5`, resolving an identity reference records
`input_fidelity: high`; explicitly requesting low is rejected. This controls input
preservation separately from output `image_quality`. The mini model's low-only
editing is rejected for identity references. `gpt-image-2` processes inputs at high
fidelity without that parameter, so omit `input_fidelity` for that model. Keep the
selected provider/model pinned; do not switch models to repair identity silently.
Other supported adapters retain identity roles and prompt constraints without
claiming a provider fidelity control that is not implemented.

Preserve the selected reference's facial geometry, eye shape/spacing, nose,
hair silhouette, head/body ratio, limbs, material, and characteristic details.
Put pose/action/setting in the job brief. Avoid conflicting prose, unnecessary
style changes, and extreme expression/angle changes when testing continuity.
A rejected scene must never become an identity reference automatically.

Review the actual result against every selected identity image. Check face and
body geometry independently of costume/palette, including robot limb/head ratios.
Record `needs_revision` with specific findings when those change. API success,
correct file format, high fidelity, and passing offline tests do not prove visual
identity. User review decides acceptance. Do not claim consistency has passed while
the user rejects the likeness; preserve the rejected result and feedback.

## Original model and portable provenance

The original identity asset's provider AND exact model ID take precedence over
project defaults. The CLI resolves selected files against local run receipts by
path and verified hash. A conflicting explicit selection fails before submission;
there is no implicit model migration. References from different models cannot be
combined as identities in one job. Style/layout/content references do not pin the
identity model. Files changed since their recorded generation are rejected.
Previously rejected `needs_revision` scenes cannot serve as the original identity.

For an external original without a local receipt, declare its actual known
provider/model and SHA-256 in the reference YAML; never invent provenance:

```yaml
references:
  - path: selected-original.png
    role: identity
    origin:
      selection: {provider: openai, model: gpt-image-1}
      sha256: REPLACE_WITH_ACTUAL_SHA256_OF_SELECTED_FILE
      settings: {format: png, image_quality: high, size: 1024x1024}
```

New initialization requires known identity metadata. If the original model is
unknown, explain that missing information; do not label a guessed model as known.
Legacy configs still read, but known originals always enforce the model lock.
Accepted character images retain `visual.origins` keyed by media hash, alongside
`visual.selection` and `visual.settings`. Export/import preserves those records.
Character model changes must not relabel an existing asset's origin.

Compatible project defaults apply only when their model matches. Common original
settings then take precedence, followed by pinned visual-profile settings and
explicit job settings. Output size/quality may change explicitly without changing
the model. Scene/action direction stays in Markdown. Seeds and matching models do
not prove identity; compare each output with the actual selected original.
