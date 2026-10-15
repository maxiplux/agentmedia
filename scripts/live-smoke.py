#!/usr/bin/env python3
"""Explicit one-request smoke test. Never reads or prints credential values."""
import argparse
import json
import pathlib
import subprocess
import tempfile
import sys

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', default='target/debug/framechord')
parser.add_argument('--project', default='.')
parser.add_argument('--provider', required=True, choices=['openai', 'google', 'fish', 'openrouter', 'elevenlabs'])
parser.add_argument('--model', required=True)
parser.add_argument('--media', required=True, choices=['image', 'speech', 'music', 'sfx'])
parser.add_argument('--voice', help='Explicit voice name/ID for speech')
parser.add_argument('--live', action='store_true', help='Authorize at most one paid generation request')
args = parser.parse_args()
binary = str(pathlib.Path(args.binary).resolve())
project = str(pathlib.Path(args.project).resolve())
base = [binary, '--project', project, '--json', '--non-interactive']
doctor = subprocess.run(base + ['doctor'], capture_output=True, text=True, check=False)
if doctor.returncode:
    print(doctor.stdout)
    sys.exit(doctor.returncode)
configured = next(row['configured'] for row in json.loads(doctor.stdout)['data']['credentials'] if row['provider'] == args.provider)
if not args.live or not configured:
    print(json.dumps({'status': 'skipped', 'provider': args.provider, 'model': args.model, 'passed': False,
                      'reason': 'live flag required' if not args.live else 'credentials unavailable'}))
    sys.exit(0)
if args.media == 'speech' and not args.voice:
    print(json.dumps({'status':'skipped','provider':args.provider,'passed':False,'reason':'explicit voice binding required'}))
    sys.exit(0)
if args.voice in ('saved-voice-id', 'VOICE_ID', 'YOUR_VOICE_ID'):
    parser.error('Use a real provider voice ID, not a placeholder; inspect framechord voices --account')
duration = 3
if args.provider == 'google' and args.media == 'music':
    if args.model != 'lyria-3-clip-preview':
        parser.error('Use lyria-3-clip-preview for bounded Google music smoke testing')
    duration = 30
with tempfile.TemporaryDirectory(prefix='framechord-smoke-') as temp:
    limits = pathlib.Path(temp) / 'limits.yaml'
    limits.write_text(f'requests: 1\nimages: 1\naudio_seconds: {max(10, duration + 2)}\nspeech_chars: 100\nrevisions: 1\nattempts: 1\nconcurrency: 1\nper_provider: 1\n')
    argv = base + ['generate', args.media, '--provider', args.provider, '--model', args.model,
                   '--fresh', '--limits-file', str(limits)]
    if args.media == 'speech':
        argv += ['--text', 'Hello. This is a short local test.', '--voice', args.voice]
    else:
        prompt = {'image': 'A small teal leaf on a plain cream background.',
                  'music': 'A gentle instrumental piano introduction, no vocals.',
                  'sfx': 'A single quiet glass bell with a soft decay.'}[args.media]
        argv += ['--prompt', prompt]
        if args.media == 'image' and args.provider == 'openai':
            argv += ['--size', '1024x1024', '--image-quality', 'low']
        if args.media in ('music', 'sfx'):
            argv += ['--duration-seconds', str(duration)]
        if args.media == 'music':
            argv += ['--instrumental']
    result = subprocess.run(argv, capture_output=True, text=True, check=False)
    response = json.loads(result.stdout)
    print(json.dumps({'status': 'passed' if result.returncode == 0 else 'failed',
                      'provider': args.provider, 'model': args.model, 'result': response}))
    sys.exit(result.returncode)
