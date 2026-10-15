#!/usr/bin/env python3
"""Opt-in one-request recording synthesis or account cloning; requires real permission files."""
import argparse
import json
import pathlib
import subprocess
import sys
import tempfile

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary',default='target/debug/framechord')
parser.add_argument('--project',default='.')
parser.add_argument('--mode',required=True,choices=['synthesis','clone'])
parser.add_argument('--request',help='Authored voice-clone manifest; bind must be false')
parser.add_argument('--sample')
parser.add_argument('--transcript')
parser.add_argument('--consent')
parser.add_argument('--model',help='Explicit supported Fish speech model for synthesis')
parser.add_argument('--live',action='store_true')
args=parser.parse_args()
base=[str(pathlib.Path(args.binary).resolve()),'--project',str(pathlib.Path(args.project).resolve()),'--json','--non-interactive']
if not args.live:
    print(json.dumps({'status':'skipped','passed':False,'reason':'live flag and selected authorized recordings required'}));sys.exit(0)
provider='fish' if args.mode=='synthesis' else 'elevenlabs'
def call(argv):
    result=subprocess.run(base+argv,capture_output=True,text=True)
    response=json.loads(result.stdout)
    if result.returncode:
        print(json.dumps({'status':'failed','result':response}));sys.exit(result.returncode)
    return response['data']
configured=next(c['configured'] for c in call(['doctor'])['credentials'] if c['provider']==provider)
if not configured:
    print(json.dumps({'status':'skipped','passed':False,'reason':'provider credentials unavailable'}));sys.exit(0)
with tempfile.TemporaryDirectory(prefix='framechord-recording-smoke-') as temp:
    limits=pathlib.Path(temp)/'limits.yaml'
    limits.write_text('requests: 1\nimages: 1\naudio_seconds: 10\nspeech_chars: 100\nrevisions: 1\nattempts: 1\nconcurrency: 1\nper_provider: 1\n')
    if args.mode=='clone':
        if not args.request:parser.error('Clone requires an explicit request manifest and real saved_voice_clone permission')
        manifest=call(['validate','--kind','voice_clone',args.request])['effective']['request']
        if manifest['bind']:parser.error('Bounded clone smoke testing requires bind: false; bind later after reviewing the voice')
        if len(manifest['recordings'])!=1:parser.error('Use exactly one selected recording for a bounded clone smoke test')
        argv=['persona','voice-clone','--request',args.request,'--limits-file',str(limits)]
    else:
        if not all([args.sample,args.transcript,args.consent,args.model]):parser.error('Synthesis requires sample, exact transcript, consent, and explicit Fish model')
        argv=['generate','speech','--provider','fish','--model',args.model,'--text','Hello. This is a short local test.',
              '--voice-sample',args.sample,'--sample-transcript',args.transcript,'--consent-file',args.consent,'--limits-file',str(limits)]
    dry=call(argv+['--dry-run'])
    references=dry['input']['recordings'] if args.mode=='clone' else dry['inputs'][0]['references']
    # Small selected files only; size checked before any provider operation.
    for reference in references:
        if pathlib.Path(reference['name']).suffix.lower() not in ('.wav','.mp3','.flac'):parser.error('Audio recording required')
    if args.mode=='synthesis' and pathlib.Path(args.sample).stat().st_size>1024*1024:parser.error('Use a recording at most 1 MiB for bounded synthesis testing')
    if args.mode=='clone':
        source=pathlib.Path(args.request).resolve().parent/pathlib.Path(manifest['recordings'][0])
        if source.stat().st_size>1024*1024:parser.error('Use a recording at most 1 MiB for bounded cloning testing')
    result=call(argv)
    print(json.dumps({'status':'passed','provider':provider,'mode':args.mode,'result':result,'canonical_acceptance_performed':False}))
