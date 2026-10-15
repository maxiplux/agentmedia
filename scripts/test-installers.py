#!/usr/bin/env python3
"""Offline native bootstrap tests: intercept downloads, install a real fixture binary."""
import argparse, hashlib, os, pathlib, shutil, subprocess, sys, tempfile
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=pathlib.Path, required=True)
parser.add_argument('--target', required=True)
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parents[1]
binary = args.binary.resolve()
version = subprocess.run([str(binary), '--version'], capture_output=True, text=True, check=True).stdout.strip().split()[1]
with tempfile.TemporaryDirectory(prefix='mediaagent-bootstrap-') as temporary:
    work = pathlib.Path(temporary)
    environment = {k: v for k, v in os.environ.items() if k not in ['OPENAI_API_KEY','GEMINI_API_KEY','FISH_API_KEY','ELEVENLABS_API_KEY','OPENROUTER_API_KEY']}
    if os.name == 'nt':
        # CI starts in PowerShell 7; Windows PowerShell 5.1 must load its own modules.
        environment.pop('PSModulePath', None)
    environment.update(MEDIAAGENT_TEST_FIXTURE_BINARY=str(binary), MEDIAAGENT_TEST_FIXTURE_VERSION=version, MEDIAAGENT_TEST_FIXTURE_TARGET=args.target)
    if os.name == 'nt':
        script = root / 'docs/site/install.ps1'
        command = r'''$ErrorActionPreference='Stop'; function Invoke-WebRequest { param([string]$Uri,[string]$OutFile,[switch]$UseBasicParsing)
        if(-not $Uri.StartsWith('https://github.com/maxiplux/agentmedia/releases/')) { throw 'Unexpected download destination' }
        $name=[IO.Path]::GetFileName(([Uri]$Uri).AbsolutePath); $asset='mediaagent-'+$env:MEDIAAGENT_TEST_FIXTURE_TARGET+'.exe'
        if($name -eq 'version.txt') { [IO.File]::WriteAllText($OutFile,$env:MEDIAAGENT_TEST_FIXTURE_VERSION) }
        elseif($name -eq ($asset+'.sha256')) { $hash=(Get-FileHash -LiteralPath $env:MEDIAAGENT_TEST_FIXTURE_BINARY -Algorithm SHA256).Hash.ToLower(); if($env:MEDIAAGENT_TEST_BAD_CHECKSUM -eq '1'){$hash='0'*64}; [IO.File]::WriteAllText($OutFile,$hash+'  '+$asset) }
        elseif($name -eq $asset) { Copy-Item -LiteralPath $env:MEDIAAGENT_TEST_FIXTURE_BINARY -Destination $OutFile }
        else { throw 'Unexpected release asset' } }
        & ([scriptblock]::Create((Get-Content -LiteralPath $env:MEDIAAGENT_TEST_INSTALL_SCRIPT -Raw))) -BinDir $env:MEDIAAGENT_TEST_BIN_DIR -NoModifyPath'''
        environment['MEDIAAGENT_TEST_INSTALL_SCRIPT'] = str(script)
        def run(bin_dir, bad=False):
            env = dict(environment, MEDIAAGENT_TEST_BIN_DIR=str(bin_dir), MEDIAAGENT_TEST_BAD_CHECKSUM='1' if bad else '0')
            return subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',command], env=env, capture_output=True, text=True)
    else:
        mock = work / 'mock-tools'; mock.mkdir()
        curl = mock / 'curl'
        curl.write_text(f'#!{sys.executable}\n' + r'''
import sys, os, pathlib, hashlib
arguments=sys.argv[1:]; url=next(a for a in arguments if a.startswith('https://')); destination=pathlib.Path(arguments[arguments.index('-o')+1])
assert url.startswith('https://github.com/maxiplux/agentmedia/releases/')
name=url.rsplit('/',1)[-1]; asset='mediaagent-'+os.environ['MEDIAAGENT_TEST_FIXTURE_TARGET']; binary=pathlib.Path(os.environ['MEDIAAGENT_TEST_FIXTURE_BINARY'])
if name=='version.txt': destination.write_text(os.environ['MEDIAAGENT_TEST_FIXTURE_VERSION']+'\n')
elif name==asset+'.sha256': destination.write_text(('0'*64 if os.environ.get('MEDIAAGENT_TEST_BAD_CHECKSUM')=='1' else hashlib.sha256(binary.read_bytes()).hexdigest())+'  '+asset+'\n')
elif name==asset: destination.write_bytes(binary.read_bytes())
else: raise SystemExit('Unexpected release asset')
''')
        curl.chmod(0o755)
        environment['PATH'] = str(mock) + os.pathsep + environment.get('PATH', '')
        def run(bin_dir, bad=False):
            env = dict(environment, MEDIAAGENT_TEST_BAD_CHECKSUM='1' if bad else '0')
            return subprocess.run(['sh',str(root / 'docs/site/install.sh'),'--bin-dir',str(bin_dir),'--no-modify-path'], env=env, capture_output=True, text=True)
    destination = work / 'installed bin with spaces'
    first = run(destination)
    assert first.returncode == 0, first.stdout + first.stderr
    installed = destination / ('mediaagent.exe' if os.name == 'nt' else 'mediaagent')
    assert hashlib.sha256(installed.read_bytes()).hexdigest() == hashlib.sha256(binary.read_bytes()).hexdigest()
    assert subprocess.run([str(installed),'--version'], capture_output=True, text=True, check=True).stdout.strip() == 'mediaagent ' + version
    second = run(destination)
    assert second.returncode == 0, second.stdout + second.stderr
    # The Windows user-PATH branch is exercised only on a disposable CI runner,
    # with its original registry value restored even when assertions fail.
    if os.name == 'nt' and os.environ.get('GITHUB_ACTIONS') == 'true':
        path_test = r'''$ErrorActionPreference='Stop'; $before=[Environment]::GetEnvironmentVariable('Path','User'); try {
        $exe=Join-Path $env:MEDIAAGENT_TEST_BIN_DIR 'mediaagent.exe'
        & $exe install --bin-dir $env:MEDIAAGENT_TEST_BIN_DIR --json | Out-Null
        if($LASTEXITCODE -ne 0){throw 'User PATH installation failed'}
        $once=[Environment]::GetEnvironmentVariable('Path','User')
        if($once -ne ($env:MEDIAAGENT_TEST_BIN_DIR+';'+$before)){throw 'User PATH did not preserve the prior value'}
        & $exe install --bin-dir $env:MEDIAAGENT_TEST_BIN_DIR --json | Out-Null
        if($LASTEXITCODE -ne 0 -or [Environment]::GetEnvironmentVariable('Path','User') -ne $once){throw 'Repeated install duplicated the PATH entry'}
        } finally { [Environment]::SetEnvironmentVariable('Path',$before,'User') }'''
        tested = subprocess.run(['powershell.exe','-NoProfile','-NonInteractive','-Command',path_test], env=dict(environment, MEDIAAGENT_TEST_BIN_DIR=str(destination)), capture_output=True, text=True)
        assert tested.returncode == 0, tested.stdout + tested.stderr
    rejected = work / 'must not be installed'
    bad = run(rejected, bad=True)
    assert bad.returncode != 0 and not rejected.exists(), bad.stdout + bad.stderr
print('Native bootstrap: real install, repeat install, checksum rejection, paths with spaces; zero network requests.')
