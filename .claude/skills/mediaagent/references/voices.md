# Reusable voice workflows

Use `voices --provider PROVIDER` for built-in names and `--account` for a bounded Google/Fish/ElevenLabs catalog. A saved binding belongs to that provider/account. Imported characters need an accessible destination binding; do not choose a replacement silently.

For descriptive ElevenLabs voice design, enable a supported explicit ElevenLabs speech profile in the character YAML. Author `voice.md` as the stable description, 20–1000 characters. Supply a separate exact 100–1000 character preview script, such as `templates/voice-preview.md`. Design uses an explicitly chosen `eleven_multilingual_ttv_v2` or `eleven_ttv_v3` model; the character's speech model is independent and remains pinned.

```sh
mediaagent persona voice-design narrator --model eleven_ttv_v3 \
  --text-file preview.md --run-id narrator-design-1 --dry-run --json
mediaagent persona voice-design narrator --model eleven_ttv_v3 \
  --text-file preview.md --run-id narrator-design-1 --json
mediaagent persona voice-status narrator-design-1 --json
```

One request returns up to three preview candidates. They stay in the recorded run and do not create a saved voice or change canonical character files. `--limits-file` supplies explicit volume limits; reservations cover three previews of the exact script. Decoded durations are recorded, and overrun candidates remain available while further calls stop. Saved responses recover without another design request. To retry an uncertain design, use the same `--run-id` with explicit `--retry-unknown`; it may already have incurred charges.

After the user selects a candidate and requests creating a provider-side voice, use its exact candidate ID:

```sh
mediaagent persona voice-create narrator --design-run narrator-design-1 \
  --candidate SELECTED_CANDIDATE_ID --name Narrator --dry-run --json
mediaagent persona voice-create narrator --design-run narrator-design-1 \
  --candidate SELECTED_CANDIDATE_ID --name Narrator --bind --json
```

`voice-create` is an explicit account operation. Without `--bind`, it returns the new ID without changing the character. Binding requires confirmed provider verification status; unknown or incomplete required verification leaves it unbound. After completing provider requirements, add `--refresh-verification --bind` to retrieve current metadata and bind the already-created voice without creating another. `voices --provider elevenlabs --account --voice-id VOICE_ID --json` can also inspect an existing account binding. `--replace-binding` explicitly permits replacement of an existing binding. Changed persona, voice description, or speech model requires new previews. Repeat identical create commands reuse their saved result; they do not create duplicate provider voices. An uncertain create requires explicit `--retry-unknown` after account reconciliation. Per-attempt IDs and outcomes remain in operation history.

Binding does not accept the preview as a canonical recording. If the user also selects that recording, use `persona accept ... --kind audio --reference FILE` separately. Use the authorized recording workflows below for reference synthesis or recorded cloning; descriptive design does not reproduce an imported recording. Listen to repeated speech clips before claiming voice quality or consistency.

## Authorized recordings and cloning

Audio reference synthesis is supported by Fish speech models in `mediaagent models`.
A sample requires an actual recording, its exact transcript Markdown, and a granted
`voice_consent` YAML with `reference_synthesis` purpose plus permission evidence.
Templates `voice-consent.yaml`, `voice-permission.md`, `voice-sample.yaml`, and
`voice-transcript.md` explain these fields. Templates start with permission denied;
never invent the speaker/rightsholder's authorization.

```sh
mediaagent validate --kind voice_consent consent.yaml --json
mediaagent persona accept-sample narrator --audio selected.wav \
  --transcript selected-transcript.md --consent consent.yaml --json
mediaagent generate speech --provider fish --model s2.1-pro-free \
  --text-file article.md --voice-sample selected.wav \
  --sample-transcript selected-transcript.md --consent-file consent.yaml \
  --output narration.wav --dry-run --json
```

`accept-sample` is explicit canonical acceptance. It copies the audio, transcript,
consent, and evidence into the portable character folder and avoids duplicate
samples. Character `voice.samples` and speech job/turn `voice_samples` use the same
contract. Without a saved binding, Fish receives authorized audio samples as binary
MessagePack with exact transcripts; it does not create a provider voice. With a
saved binding, synthesis uses the binding, and recordings remain review evidence.
Other providers require saved voices; they can retain recordings for audio review.
Do not mix raw audio references lacking transcripts into Fish reference synthesis.

Recorded-voice cloning uses `voice-clone.yaml`, a supported ElevenLabs character
speech profile, one to five selected recordings, and a consent record whose purposes
include `saved_voice_clone`. The name is 1–100 characters; combined recordings must
fit the 48 MiB encoded budget. Creating a provider account voice is explicit:

```sh
mediaagent validate --kind voice_clone clone.yaml --json
mediaagent persona voice-clone --request clone.yaml --dry-run --json
mediaagent persona voice-clone --request clone.yaml --json
mediaagent persona voice-clone-status narrator-clone-1 --json
```

`bind: false` creates without changing the canonical voice. `bind: true` binds only
when required provider verification is confirmed; `replace_binding: true` explicitly
permits replacement. When provider verification is required, complete it through the
provider's supported account flow, then repeat with `--refresh-verification`. This
reads current verification status and reuses the already-created ID. It does not
submit biometric/professional verification on the speaker's behalf. Repeat identical
requests reuse their durable result; changed identity/recordings/permission with the
same operation ID is rejected. Unknown cloning requires account reconciliation and
explicit `--retry-unknown`; do not recreate a possibly charged voice automatically.
Cloning does not accept the source recordings as canonical references.

Catalogs support `--search`, `--language`, `--page-size 1..100`, and
`--max-pages 1..20`, defaulting to one page of 20. Google/ElevenLabs use the returned
`--page-token` with unchanged filters; Fish uses `--page-number` and optionally
`--public` for licensed public voices. `--category` filters Google
prebuilt/replicated/prompted or ElevenLabs premade/cloned/generated/professional.
A catalog binding still needs account access and explicit selection before reuse.
