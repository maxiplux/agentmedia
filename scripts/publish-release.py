#!/usr/bin/env python3
"""Validate all native packages, then atomically expose a complete public release."""
import argparse, hashlib, json, pathlib, re, subprocess, tarfile, zipfile, tomllib

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--tag', required=True)
parser.add_argument('--artifacts', type=pathlib.Path, required=True)
parser.add_argument('--dry-run', action='store_true')
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parents[1]
version = tomllib.loads((root / 'Cargo.toml').read_text())['package']['version']
assert re.fullmatch(r'\d+\.\d+\.\d+', version) and args.tag == 'v' + version, 'Tag must match the stable Cargo package version'
commit = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=root, capture_output=True, text=True, check=True).stdout.strip()
if not args.dry_run:
    tag_commit = subprocess.run(['git', 'rev-parse', args.tag + '^{commit}'], cwd=root, capture_output=True, text=True, check=True).stdout.strip()
    assert tag_commit == commit, 'Release checkout must match the exact tag commit'
targets = ['aarch64-apple-darwin', 'x86_64-unknown-linux-gnu', 'x86_64-pc-windows-msvc']
files = []
for target in targets:
    binary_name = 'framechord-' + target + ('.exe' if 'windows' in target else '')
    archive_name = f'framechord-{version}-{target}' + ('.zip' if 'windows' in target else '.tar.gz')
    found = {}
    for name in [binary_name, binary_name + '.sha256', archive_name, archive_name + '.sha256']:
        matches = list(args.artifacts.rglob(name)); assert len(matches) == 1, f'Missing or duplicate native asset: {name}'
        found[name] = matches[0]
    binary = found[binary_name].read_bytes()
    for name in [binary_name, archive_name]:
        digest = hashlib.sha256(found[name].read_bytes()).hexdigest()
        assert found[name + '.sha256'].read_text().split()[0] == digest, f'Checksum failed: {name}'
    archive = found[archive_name]
    if 'windows' in target:
        with zipfile.ZipFile(archive) as package:
            assert package.testzip() is None
            metadata = json.loads(package.read(next(n for n in package.namelist() if n.endswith('/verification.json'))))
            archived_binary = package.read(next(n for n in package.namelist() if n.endswith('/framechord.exe')))
    else:
        with tarfile.open(archive) as package:
            metadata = json.load(package.extractfile(next(m for m in package.getmembers() if m.name.endswith('/verification.json'))))
            archived_binary = package.extractfile(next(m for m in package.getmembers() if m.name.endswith('/framechord'))).read()
    assert archived_binary == binary and metadata['binary_sha256'] == hashlib.sha256(binary).hexdigest()
    assert metadata['target'] == target and metadata['version'] == version
    assert metadata['source_commit'] == commit and metadata['source_dirty'] is False, 'Release artifacts must come from the exact clean tag checkout'
    assert len(metadata['native_offline_checks']) >= 17 and metadata['live_provider_testing_performed_by_packager'] is False
    assert 'legacy_project_paths_identity_and_configuration_preserved' in metadata['native_offline_checks']
    files.extend(found.values())
staged = root / 'target/release-publication'
staged.mkdir(parents=True, exist_ok=True)
for filename in ['install.sh', 'install.ps1']:
    path = staged / filename; path.write_bytes((root / 'docs/site' / filename).read_bytes()); files.append(path)
path = staged / 'version.txt'; path.write_text(version + '\n'); files.append(path)
manifest = {'schema_version': 1, 'version': version, 'commit': commit, 'assets': [{'name': p.name, 'size': p.stat().st_size, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]}
manifest_path = staged / 'release-manifest.json'; manifest_path.write_text(json.dumps(manifest, indent=2) + '\n'); files.append(manifest_path)
assert len({p.name for p in files}) == len(files)
if args.dry_run:
    print(json.dumps({'validated': True, 'version': version, 'assets': len(files), 'published': False})); raise SystemExit
repo = 'maxiplux/framechord'
def remote_release():
    # The by-tag REST endpoint excludes drafts. Resolve the numeric API URL with
    # gh release view, which can see this repository's authenticated draft.
    info = json.loads(subprocess.run(['gh', 'release', 'view', args.tag, '--repo', repo, '--json', 'apiUrl'], capture_output=True, text=True, check=True).stdout)
    return json.loads(subprocess.run(['gh', 'api', info['apiUrl']], capture_output=True, text=True, check=True).stdout)

existing = subprocess.run(['gh', 'release', 'view', args.tag, '--repo', repo, '--json', 'isDraft'], capture_output=True, text=True)
if existing.returncode == 0 and not json.loads(existing.stdout)['isDraft']:
    remote = remote_release()
    assets = {a['name']: a for a in remote['assets']}
    for path in files:
        assert assets.get(path.name, {}).get('digest') == 'sha256:' + hashlib.sha256(path.read_bytes()).hexdigest(), 'Published release differs; never overwrite stable assets'
    print('Published release already matches the validated assets'); raise SystemExit
if existing.returncode != 0:
    notes = staged / 'notes.md'
    notes.write_text(f'''Install a user-local binary without Rust or administrator access.\n\nmacOS/Linux:\n```sh\ncurl -fsSL https://maxiplux.github.io/framechord/install.sh | sh\n```\n\nWindows PowerShell:\n```powershell\nirm https://maxiplux.github.io/framechord/install.ps1 | iex\n```\n\nAlready downloaded? Run `framechord install`. Later use `framechord update --check`, `framechord update`, or `framechord update --rollback`.\n\nNative packages were built and tested on macOS Apple Silicon, Linux x86-64 and Windows x86-64 from `{commit}`. Raw executables and archives include SHA-256 sidecars; the release manifest records their exact hashes. Installation/update preserves project configuration, keys, characters, media and installed skills. Project initialization and skill refresh remain separate.\n\nSee the [installation guide](https://github.com/{repo}/blob/{args.tag}/docs/INSTALLATION.md). These binaries are not signed or notarized; checksum checks are not code signing. No paid provider calls are performed by packaging.\n''')
    subprocess.run(['gh', 'release', 'create', args.tag, '--repo', repo, '--verify-tag', '--draft', '--title', f'framechord {version}', '--notes-file', str(notes)], check=True)
subprocess.run(['gh', 'release', 'upload', args.tag, '--repo', repo, '--clobber'] + [str(p) for p in files], check=True)
# Verify GitHub's stored digests before making latest/download usable.
remote = remote_release()
assets = {a['name']: a for a in remote['assets']}
for path in files:
    assert assets.get(path.name, {}).get('digest') == 'sha256:' + hashlib.sha256(path.read_bytes()).hexdigest(), 'Uploaded asset digest mismatch'
subprocess.run(['gh', 'release', 'edit', args.tag, '--repo', repo, '--draft=false', '--latest'], check=True)
print(json.dumps({'published': True, 'version': version, 'commit': commit, 'assets': len(files)}))
