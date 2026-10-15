#!/usr/bin/env python3
"""Offline publisher regression: draft lookup, integrity rejection and immutable releases."""
import contextlib
import hashlib
import io
import json
import pathlib
import runpy
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest.mock
import zipfile

source = pathlib.Path(__file__).resolve().parents[1]
commit = '1' * 40
api_url = 'https://api.github.com/repos/maxiplux/agentmedia/releases/123'
with tempfile.TemporaryDirectory(prefix='mediaagent-publisher-test-') as temporary:
    root = pathlib.Path(temporary)
    (root / 'scripts').mkdir()
    script = root / 'scripts/publish-release.py'
    shutil.copyfile(source / 'scripts/publish-release.py', script)
    (root / 'Cargo.toml').write_text('[package]\nversion = "0.4.0"\n')
    (root / 'docs/site').mkdir(parents=True)
    for name in ['install.sh', 'install.ps1']:
        (root / 'docs/site' / name).write_text('offline installer fixture\n')
    artifacts = root / 'artifacts'
    artifacts.mkdir()
    targets = ['aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc']
    for target in targets:
        windows = 'windows' in target
        data = ('native fixture: ' + target).encode()
        binary = artifacts / ('mediaagent-' + target + ('.exe' if windows else ''))
        binary.write_bytes(data)
        metadata = json.dumps({'binary_sha256': hashlib.sha256(data).hexdigest(), 'target': target, 'version': '0.4.0', 'source_commit': commit, 'source_dirty': False, 'native_offline_checks': ['offline fixture'] * 16, 'live_provider_testing_performed_by_packager': False}).encode()
        archive = artifacts / ('mediaagent-0.4.0-' + target + ('.zip' if windows else '.tar.gz'))
        entries = {'package/verification.json': metadata, 'package/mediaagent' + ('.exe' if windows else ''): data}
        if windows:
            with zipfile.ZipFile(archive, 'w') as bundle:
                for name, content in entries.items():
                    bundle.writestr(name, content)
        else:
            with tarfile.open(archive, 'w:gz') as bundle:
                for name, content in entries.items():
                    info = tarfile.TarInfo(name)
                    info.size = len(content)
                    bundle.addfile(info, io.BytesIO(content))
        for path in [binary, archive]:
            path.with_name(path.name + '.sha256').write_text(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + path.name + '\n')

    state = {'exists': False, 'draft': True, 'assets': {}, 'uploads': 0, 'corrupt_response': False}
    def execute(command, **kwargs):
        if command[:2] == ['git', 'rev-parse']:
            output = commit
        elif command[:3] == ['gh', 'release', 'view']:
            if not state['exists']:
                return subprocess.CompletedProcess(command, 1, '', 'Not found')
            output = json.dumps({'isDraft': state['draft']} if command[-1] == 'isDraft' else {'apiUrl': api_url})
        elif command[:3] == ['gh', 'release', 'create']:
            assert '--draft' in command and '--verify-tag' in command
            state['exists'] = True
            output = ''
        elif command[:3] == ['gh', 'release', 'upload']:
            assert state['draft'], 'Never overwrite published release assets'
            state['uploads'] += 1
            for path in command[command.index('--clobber') + 1:]:
                path = pathlib.Path(path)
                state['assets'][path.name] = {'name': path.name, 'digest': 'sha256:' + hashlib.sha256(path.read_bytes()).hexdigest()}
            output = ''
        elif command[:2] == ['gh', 'api']:
            assert command[2] == api_url, 'Drafts require the numeric release endpoint'
            assets = list(state['assets'].values())
            if state['corrupt_response']:
                assets = [dict(a, digest='sha256:' + '0' * 64) for a in assets]
            output = json.dumps({'assets': assets})
        elif command[:3] == ['gh', 'release', 'edit']:
            assert len(state['assets']) == 20 and '--draft=false' in command
            state['draft'] = False
            output = ''
        else:
            raise AssertionError(command)
        return subprocess.CompletedProcess(command, 0, output, '')

    def publish():
        with unittest.mock.patch.object(sys, 'argv', [str(script), '--tag', 'v0.4.0', '--artifacts', str(artifacts)]), unittest.mock.patch('subprocess.run', side_effect=execute), contextlib.redirect_stdout(io.StringIO()):
            runpy.run_path(str(script), run_name='__main__')

    state['corrupt_response'] = True
    try:
        publish()
        raise RuntimeError('Corrupt remote digests must fail closed')
    except AssertionError as error:
        assert 'digest mismatch' in str(error)
    assert state['draft'] is True
    state['corrupt_response'] = False
    publish()  # Resume the draft and verify every uploaded asset before publication.
    assert state['draft'] is False
    uploads = state['uploads']
    try:
        publish()  # An identical published release is read-only and idempotent.
    except SystemExit as error:
        assert error.code is None
    assert state['uploads'] == uploads
    state['corrupt_response'] = True
    try:
        publish()
        raise RuntimeError('Changed published assets must be refused')
    except AssertionError as error:
        assert 'never overwrite stable assets' in str(error)
    assert state['uploads'] == uploads
print('Offline publication: numeric draft lookup, corrupt digest refusal, draft resume, immutable published assets; zero network requests.')
