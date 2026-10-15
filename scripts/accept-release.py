#!/usr/bin/env python3
"""Opt-in public release acceptance in an isolated directory; no media/provider calls."""
import argparse, hashlib, json, os, pathlib, subprocess, tempfile, urllib.request
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--version', required=True)
args = parser.parse_args()
version = args.version.removeprefix('v')
with tempfile.TemporaryDirectory(prefix='mediaagent-public-acceptance-') as temporary:
    root = pathlib.Path(temporary)
    bin_dir = root / 'installed bin'
    environment = {k: v for k, v in os.environ.items() if k not in ['OPENAI_API_KEY','GEMINI_API_KEY','FISH_API_KEY','ELEVENLABS_API_KEY','OPENROUTER_API_KEY']}
    script_name = 'install.ps1' if os.name == 'nt' else 'install.sh'
    request = urllib.request.Request('https://maxiplux.github.io/agentmedia/' + script_name, headers={'User-Agent': 'mediaagent-release-acceptance'})
    with urllib.request.urlopen(request, timeout=60) as response: script = response.read(1024 * 1024)
    local_script = root / script_name; local_script.write_bytes(script)
    if os.name == 'nt':
        code = "& ([scriptblock]::Create((Get-Content -LiteralPath $env:MEDIAAGENT_ACCEPT_SCRIPT -Raw))) -Version $env:MEDIAAGENT_ACCEPT_VERSION -BinDir $env:MEDIAAGENT_ACCEPT_BIN -NoModifyPath"
        environment.update(MEDIAAGENT_ACCEPT_SCRIPT=str(local_script), MEDIAAGENT_ACCEPT_VERSION=version, MEDIAAGENT_ACCEPT_BIN=str(bin_dir))
        command = ['powershell.exe','-NoProfile','-NonInteractive','-Command',code]
    else:
        command = ['sh', str(local_script), '--version', version, '--bin-dir', str(bin_dir), '--no-modify-path']
    subprocess.run(command, env=environment, cwd=root, check=True, timeout=300)
    binary = bin_dir / ('mediaagent.exe' if os.name == 'nt' else 'mediaagent')
    assert subprocess.run([str(binary),'--version'], env=environment, capture_output=True, text=True, check=True).stdout.strip() == 'mediaagent ' + version
    original = hashlib.sha256(binary.read_bytes()).hexdigest()
    (root / '.env').write_text('LOCAL_ACCEPTANCE_FIXTURE=preserve\n')
    (root / 'keep-project.md').write_text('Preserve user-authored content.')
    def cli(arguments):
        response = subprocess.run([str(binary), '--json', '--non-interactive'] + arguments, env=environment, cwd=root, capture_output=True, text=True, check=True, timeout=300)
        result = json.loads(response.stdout); assert result['ok'], result
        return result['data']
    check = cli(['update','--check','--version',version])
    assert check['available_version'] == version and check['replaced'] is False
    assert cli(['update','--version',version,'--force'])['replaced'] is True
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == original
    restored = cli(['update','--rollback']); assert restored['version'] == version and restored['network_requested'] is False
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == original
    assert (root / '.env').read_text() == 'LOCAL_ACCEPTANCE_FIXTURE=preserve\n'
    assert (root / 'keep-project.md').read_text() == 'Preserve user-authored content.'
    assert not (root / 'mediaagent').exists()
    print(json.dumps({'version':version,'public_installer':True,'public_check_and_reinstall':True,'offline_rollback':True,'project_preservation':True,'binary_sha256':original,'paid_provider_calls':0}))
