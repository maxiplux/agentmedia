# Acceptance execution record

FrameChord is the project name from 0.6.0 onward. Earlier evidence below keeps its original MediaAgent commands, release names and artifact paths. Existing `mediaagent/` projects remain supported in place. See [rename and upgrade guidance](../assets/skill/references/renaming.md).

The approved original-character-sheet batch has executed: two successful OpenAI
requests, one attempt each. Both outputs fail agent visual review and are recorded
as `needs_revision`; human acceptance remains pending. No new references or voices
have been accepted. Existing Mira/Kito forest scenes remain rejected for identity
drift. Other batches below remain unexecuted and require separate approval.

## Original-character-sheet results

Run: `aa44c69e-844a-406c-83d2-8d7a524afbd0`.
Approved preview: `e8131c5a14f9f24cdce5d131c939ca1e4fb94c7e629a3eb8f0a8c8d38511325f`.
Both images are PNG, 1536×1024, medium quality, with high input fidelity and the
original `openai/gpt-image-1` selection. Neither result was cached or retried.

| Candidate | Agent visual findings | Quality |
| --- | --- | --- |
| Mira, artifact `b131875f-3e58-4953-a671-8248cb73ea40` | Changed eyes, face and hair; added ear stud; front view clipped at left edge; missing clear three-quarter view | needs_revision |
| Kito, artifact `e77146ce-ae87-44f5-a3e9-7784a6b4d255` | Narrower head and chest disk, longer legs; left view almost entirely clipped and right close-up clipped | needs_revision |

Outputs are under `mediaagent/runs/aa44c69e-844a-406c-83d2-8d7a524afbd0/outputs/`.
Original/output comparisons and agent findings are exported to
`target/acceptance/mira-sheet-comparison.md` and
`target/acceptance/kito-sheet-comparison.md`. The original input images were
visually inspected before generation and compared directly with both results.
These findings are agent assessments, not a claimed user decision.

OpenAI reported 4,583 input / 1,568 output tokens for Mira and 4,584 input / 1,568
output tokens for Kito. No monetary cost was returned or inferred. No further paid
requests were made. Matching the original model and high input fidelity did not
establish identity preservation; both candidates remain unsuitable as accepted
canonical references without further review and explicit selection.

## Bounded batches and completion gates

| Batch | Prepared input | Calls and limits | Completion gate |
| --- | --- | --- | --- |
| Original character sheets — executed | `mediaagent/verification/character-baseline/suite.yaml` | Two OpenAI gpt-image-1 image edits, one attempt each, medium quality, 1536×1024, high input fidelity | Both need revision; no references accepted |
| Designed voice preview | `mediaagent/verification/voice/preview.md` and `limits.yaml` | One ElevenLabs eleven_ttv_v3 design request, at most three candidate previews, 60 seconds budget, no saved voice creation | User selects one voice candidate |
| Complete media | `mediaagent/verification/media/suite.yaml` | 13 resolved requests: two narration chunks, six podcast turns, two music clips, one effect, two infographics; one attempt each; 600 seconds total audio budget | Listen/read against exact scripts, lyrics, numbers, and readability criteria |
| Repeated scenes | `mediaagent/verification/character-scenes/suite.yaml` | Eight images; validation remains blocked until canonical references exist | Three solo scenes per character and two paired interactions pass human identity review |
| Saved voice reuse | `mediaagent/verification/voice/suite.yaml` | Three speech clips; validation remains blocked until explicit creation/binding | Human voice consistency and portable binding reuse |

Costs are unknown where providers do not supply reliable estimates. Request/media
limits do not promise a dollar cap. Recompute previews after any change; their hash
binds the approval to effective scripts, references, settings and limits.

Preview commands:

```sh
mediaagent verify --suite mediaagent/verification/character-baseline/suite.yaml --json
mediaagent verify --suite mediaagent/verification/media/suite.yaml --json
mediaagent persona voice-design acceptance-narrator --model eleven_ttv_v3 \
  --text-file mediaagent/verification/voice/preview.md --run-id voice-acceptance-design \
  --limits-file mediaagent/verification/voice/limits.yaml --dry-run --json
```

After explicit selection of a preview candidate, prepare the concrete `persona
voice-create` operation with its actual candidate ID, original design run and target
character. Its provider-side creation and binding need separate approval. Use three
short clips, then export/import the character into an isolated project and verify
binding accessibility before repeating a clip. Missing access must not create a
replacement voice. Real recording cloning and Fish reference synthesis remain
blocked until the speaker's selected recordings, exact transcripts and permission
records are supplied. Use existing `live-recording.py` with one explicit operation
per approved batch; never populate a consent fixture with invented permission.

## Follow-up acceptance scenarios

- Revise only `guest-answer` in the podcast. Confirm five successful clips are reused,
  only the selected turn incurs generation, and the final assembly preserves order.
- Plan jobs through Google and OpenRouter using their explicit configured text models.
  Approve one planning request per provider, include a selected character/asset, and
  validate the returned bundle without media generation. Pin model choices in that
  preview; do not choose an unrecorded latest model.
- Review both infographics at 100% output size. Check all day/session names, values,
  units, source notes, contrast, clipping and reading order. Any wrong value fails.
- Compare character scenes with `review report`. Record criterion-level findings;
  request a bounded revision batch rather than looping paid generation automatically.

## Agent and platform gates

The packager tests a copied binary outside the checkout, both installed skill
locations, all schemas, v2 job/verification previews, preserved credentials,
paths with spaces, migration preview, and usage/storage reporting. This is an
actual generic process-caller test, not automatic skill-discovery evidence.

For each Codex and Claude clean-session acceptance, use a new project and the
packaged binary. Ask the agent to discover the skill, inspect configuration, author
and validate a bounded job, explain its inputs/output destination, generate only an
approved batch, report partial failures, and use review/revision/recovery accurately.
Record agent version, binary hash, commands, artifacts and reviewer decision.
Those two clean-agent sessions remain unexecuted; installation alone is not a pass.

Native macOS ARM64 and Linux x86-64 container execution are tracked separately from
Windows x86-64. Linux executes under Docker Desktop emulation on the Mac; it is not
physical x86 hardware certification. The Windows workflow is prepared but needs a
Windows runner. No Git remote, public publishing, signing or notarization is added.

## Evidence discipline

Record every capability with four independent fields: implemented, offline_verified,
live_verified, human_accepted. Use pending/failed/skipped/inconclusive honestly.
Approval enables only the listed bounded batch, not retries, voice creation, or
canonical acceptance of generated candidates. Store runtime receipts and media in
ignored run directories; keep sanitized acceptance summaries in documentation.
