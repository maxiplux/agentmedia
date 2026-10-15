#!/usr/bin/env python3
"""Opt-in audio assessment of a verified cached short speech clip; no intentional new speech call."""
import argparse
import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary',default='target/debug/framechord')
parser.add_argument('--project',default='.')
parser.add_argument('--source-run',required=True)
parser.add_argument('--model',required=True,choices=['gemini-3.8-flash','gemini-2.5-flash'])
parser.add_argument('--live',action='store_true')
args=parser.parse_args()
project=pathlib.Path(args.project).resolve()
base=[str(pathlib.Path(args.binary).resolve()),'--project',str(project),'--json','--non-interactive']
def call(argv):
    result=subprocess.run(base+argv,capture_output=True,text=True)
    data=json.loads(result.stdout)
    if result.returncode:
        print(json.dumps({'status':'failed','result':data}));sys.exit(result.returncode)
    return data['data']
if not args.live:
    print(json.dumps({'status':'skipped','passed':False,'reason':'live flag required'}));sys.exit(0)
configured=next(c['configured'] for c in call(['doctor'])['credentials'] if c['provider']=='google')
if not configured:
    print(json.dumps({'status':'skipped','passed':False,'reason':'Google credentials unavailable'}));sys.exit(0)
source=call(['runs','status',args.source_run])
if source['state']!='succeeded' or len(source['jobs'])!=1:
    parser.error('Source must be one successful short speech job')
job=source['jobs'][0]
effective=job['input']
if effective['media']!='speech' or effective['turns'] or effective['references'] or effective['character_hash']:
    parser.error('Use a short speech job without character/references for this bounded smoke test')
if len(effective['text'])>100 or any(a['duration_seconds']>10 for a in job['artifacts']):
    parser.error('Source exceeds 100 characters or 10 seconds')
for artifact in job['artifacts']:
    if hashlib.sha256(pathlib.Path(artifact['path']).read_bytes()).hexdigest()!=artifact['sha256']:
        parser.error('Source artifact is corrupt; no generation submitted')
with tempfile.TemporaryDirectory(prefix='framechord-audio-review-') as temp:
    limits=pathlib.Path(temp)/'limits.yaml'
    limits.write_text('requests: 1\nimages: 1\naudio_seconds: 10\nspeech_chars: 100\nrevisions: 1\nattempts: 1\nconcurrency: 1\nper_provider: 1\n')
    argv=['generate','speech','--provider',effective['selection']['provider'],'--model',effective['selection']['model'],'--text',effective['text'],'--voice',effective['settings']['voice'],'--limits-file',str(limits)]
    if effective['prompt']:argv+=['--prompt',effective['prompt']]
    if effective['locale']:argv+=['--locale',effective['locale']]
    dry=call(argv+['--dry-run'])
    if dry['inputs']!=[effective]:parser.error('Effective source changed; no generation submitted')
    copied=call(argv)
    if not copied['jobs'][0]['cached']:parser.error('Expected cached result; review was not submitted')
    artifact=copied['jobs'][0]['artifacts'][-1]
    review=call(['review','check','--artifact',artifact['id'],'--ai','--provider','google','--model',args.model])
    receipts=call(['review','status','--artifact',artifact['id']])
    print(json.dumps({'status':'passed','run_id':copied['id'],'artifact':artifact,'review':review,'receipts':receipts,'speech_reused':True}))
