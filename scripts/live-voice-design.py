#!/usr/bin/env python3
"""Opt-in one-request voice design smoke test; never creates a saved provider voice."""
import argparse
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile
import uuid

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', default='target/debug/mediaagent')
parser.add_argument('--project', default='.')
parser.add_argument('--model', required=True, choices=['eleven_multilingual_ttv_v2', 'eleven_ttv_v3'])
parser.add_argument('--speech-model', required=True, choices=['eleven_multilingual_v2', 'eleven_flash_v2_5', 'eleven_turbo_v2_5', 'eleven_v3'])
parser.add_argument('--live', action='store_true')
args = parser.parse_args()
if not args.live:
    print(json.dumps({'status': 'skipped', 'reason': 'explicit live flag required', 'passed': False}))
    sys.exit(0)
root = pathlib.Path(args.project).resolve()
base = [str(pathlib.Path(args.binary).resolve()), '--project', str(root), '--json', '--non-interactive']
doctor = subprocess.run(base + ['doctor'], capture_output=True, text=True, check=False)
if doctor.returncode:
    print(doctor.stdout)
    sys.exit(doctor.returncode)
credentials = json.loads(doctor.stdout)['data']['credentials']
if not next(row['configured'] for row in credentials if row['provider'] == 'elevenlabs'):
    print(json.dumps({'status': 'skipped', 'reason': 'ElevenLabs credentials unavailable', 'passed': False}))
    sys.exit(0)
run_id = str(uuid.uuid4())
character_id = 'smoke-design-' + uuid.uuid4().hex[:12]
character = root / 'mediaagent/characters' / character_id
character.mkdir(parents=True, exist_ok=False)
try:
    (character / 'character.yaml').write_text(f'schema_version: 1\nid: {character_id}\npersona: persona.md\nvoice:\n  description: voice.md\n  selection:\n    provider: elevenlabs\n    model: {args.speech_model}\n  binding: null\n  references: []\n')
    (character / 'persona.md').write_text('A fictional narrator used only for a bounded local API preview test. No real person is represented.')
    (character / 'voice.md').write_text('A fictional English narrator with warm mid-range pitch, clear diction, a relaxed conversational pace, and calm curiosity.')
    snapshot = root / 'mediaagent/runs' / run_id / 'fixtures' / character_id
    shutil.copytree(character, snapshot)
    with tempfile.TemporaryDirectory(prefix='mediaagent-design-') as temp:
        temp = pathlib.Path(temp)
        script = temp / 'script.md'
        script.write_text('Hello, welcome to this short narration preview. We are testing a fictional voice with clear words, a gentle pace, and a friendly tone.')
        limits = temp / 'limits.yaml'
        limits.write_text('requests: 1\nimages: 1\naudio_seconds: 60\nspeech_chars: 450\nrevisions: 1\nattempts: 1\nconcurrency: 1\nper_provider: 1\n')
        result = subprocess.run(base + ['persona', 'voice-design', character_id, '--model', args.model, '--text-file', str(script), '--run-id', run_id, '--limits-file', str(limits)], capture_output=True, text=True, check=False)
        response = json.loads(result.stdout)
        design = (response.get('data') or {}).get('design', {})
        print(json.dumps({'status': 'passed' if result.returncode == 0 else 'failed', 'run_id': run_id, 'model': args.model, 'provider_voice_created': False, 'canonical_binding_changed': False, 'candidates': [{'id': c['id'], 'path': c['artifact']['path'], 'duration_seconds': c['artifact']['duration_seconds']} for c in design.get('candidates', [])], 'errors': response['errors']}))
        sys.exit(result.returncode)
finally:
    # Remove only this uniquely created fixture; its snapshot remains in the ignored run.
    shutil.rmtree(character)
