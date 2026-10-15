#!/usr/bin/env python3
"""Regenerate the generated regions of docs/site/index.html (command pages, model catalog,
example archive and version stamps) from the local binary, assets/models.json and
docs/site/examples; --check fails when the committed guide has drifted. Offline: the only
subprocess is the local framechord executable."""
import argparse
import base64
import datetime
import difflib
import hashlib
import html
import io
import json
import pathlib
import re
import subprocess
import sys
import tempfile
import tomllib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
ENTRY = ('<details class="command-entry"><summary>{path}</summary><div class="code-block"><div class="code-bar">'
         '<span>{path}</span><button type="button" class="copy transition-colors motion-reduce:transition-none">Copy</button>'
         '</div><pre><code>{help}</code></pre></div></details>')
MARKERS = {
    'commands': ('<!-- generated:commands -->', '<!-- /generated:commands -->'),
    'catalog': ('<!-- generated:catalog -->', '<!-- /generated:catalog -->'),
}
PROVIDER_ORDER = ['openai', 'fish', 'google', 'elevenlabs', 'openrouter']
ZIP_DATE = (2026, 10, 14, 22, 15, 34)
DATA_URI = re.compile(r'href="data:application/zip;base64,([A-Za-z0-9+/=]+)"')
COMMAND_BLOCK = re.compile(r'\n## (framechord[^\n]*)\n\n```text\n(.*?)\n```\n', re.DOTALL)
STAMPS = {
    'version': (re.compile(r'<span class="version">v[^<]*</span>'), '<span class="version">v{version}</span>'),
    'target': (re.compile(r'This guide targets <strong>FrameChord [^<]*</strong>'), 'This guide targets <strong>FrameChord {version}</strong>'),
    'command pages': (re.compile(r'all \d+ command pages'), 'all {count} command pages'),
    'catalog date': (re.compile(r'The bundled catalog was verified on [^.]*\.'), 'The bundled catalog was verified on {date}.'),
}


def crate_version(root=ROOT):
    return tomllib.loads((root / 'Cargo.toml').read_text(encoding='utf-8'))['package']['version']


def default_binary(root=ROOT):
    name = 'framechord.exe' if sys.platform == 'win32' else 'framechord'
    for profile in ['release', 'debug']:
        candidate = root / 'target' / profile / name
        if candidate.is_file():
            return candidate
    raise SystemExit('No built framechord executable under target/; run cargo build --release --locked or pass --binary')


def export_commands(binary):
    """Exports the bundled skill into a scratch directory and returns commands.md."""
    with tempfile.TemporaryDirectory(prefix='framechord-site-') as scratch:
        target = pathlib.Path(scratch) / 'skill'
        subprocess.run([str(binary), '--json', '--non-interactive', 'skills', 'export', str(target)],
                       capture_output=True, text=True, check=True, timeout=120)
        return (target / 'references' / 'commands.md').read_text(encoding='utf-8')


def binary_version(binary):
    output = subprocess.run([str(binary), '--version'], capture_output=True, text=True, check=True, timeout=60).stdout
    return output.strip().split()[-1]


def parse_commands(markdown):
    """Real command paths in declaration order; clap's `help` pseudo-commands are dropped."""
    entries = []
    for path, body in COMMAND_BLOCK.findall(markdown):
        if 'help' in path.split():
            continue
        # Trailing whitespace is stripped per line; each page ends with exactly one newline.
        entries.append((path, '\n'.join(line.rstrip() for line in body.split('\n')).rstrip('\n') + '\n'))
    return entries


def render_commands(entries):
    return ''.join(ENTRY.format(path=html.escape(path, quote=True), help=html.escape(body, quote=True)) for path, body in entries)


def render_catalog(models):
    current = [m for m in models['models'] if m.get('current') and m.get('supported')]
    parts = []
    for provider in PROVIDER_ORDER:
        items = ''.join(
            f'<li><code>{html.escape(m["model"])}</code> — {m["media"]}: {html.escape(m["description"])}</li>'
            for m in current if m['provider'] == provider)
        parts.append(f'<h3>{provider}</h3><ul>{items}</ul>')
    return ''.join(parts)


def build_zip(examples):
    """Deterministic archive of the example sources; fixed metadata keeps the bytes stable."""
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, 'w', zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(p for p in pathlib.Path(examples).rglob('*') if p.is_file()):
            info = zipfile.ZipInfo('examples/' + path.relative_to(examples).as_posix(), date_time=ZIP_DATE)
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o600 << 16
            info.create_system = 3
            archive.writestr(info, path.read_bytes())
    return buffer.getvalue()


def zip_manifest(payload):
    """Name → bytes, so archives compare by content rather than by compressor output."""
    with zipfile.ZipFile(io.BytesIO(payload)) as archive:
        if archive.testzip() is not None:
            raise ValueError('corrupt example archive')
        return {name: archive.read(name) for name in archive.namelist()}


def splice(text, name, generated):
    opening, closing = MARKERS[name]
    if text.count(opening) != 1 or text.count(closing) != 1:
        raise ValueError(f'expected exactly one {opening} … {closing} pair in the guide')
    start = text.index(opening) + len(opening)
    end = text.index(closing)
    if end < start:
        raise ValueError(f'{opening} appears after its closing marker')
    return text[:start] + generated + text[end:]


def apply_stamps(text, version, count, date):
    for label, (pattern, template) in STAMPS.items():
        matches = pattern.findall(text)
        if len(matches) != 1:
            raise ValueError(f'expected exactly one {label} stamp, found {len(matches)}')
        text = pattern.sub(template.format(version=version, count=count, date=date), text)
    return text


def catalog_date(models):
    date = datetime.date.fromisoformat(models['verified_at'])
    return f'{date:%B} {date.day}, {date.year}'


def verify_anchors(text):
    ids = set(re.findall(r'(?<![-\w])id="([^"]+)"', text))
    missing = sorted({anchor for anchor in re.findall(r'href="#([^"]+)"', text) if anchor not in ids})
    if missing:
        raise ValueError('nav anchors without a matching id: ' + ', '.join('#' + m for m in missing))


def replace_archives(text, payload):
    """Every data:application/zip link on the page is the example pack; all are replaced together."""
    encoded = base64.b64encode(payload).decode('ascii')
    matches = DATA_URI.findall(text)
    if not matches:
        raise ValueError('no embedded example archive found in the guide')
    if all(zip_manifest(base64.b64decode(m)) == zip_manifest(payload) for m in matches) and len(set(matches)) == 1:
        return text  # identical content: keep the committed bytes to avoid churn
    return DATA_URI.sub(lambda _: f'href="data:application/zip;base64,{encoded}"', text)


def regenerate(text, *, commands_markdown, models, examples, version):
    entries = parse_commands(commands_markdown)
    if not entries:
        raise ValueError('no command pages parsed from commands.md')
    text = splice(text, 'commands', render_commands(entries))
    text = splice(text, 'catalog', render_catalog(models))
    text = replace_archives(text, build_zip(examples))
    text = apply_stamps(text, version, len(entries), catalog_date(models))
    if text.count('class="command-entry"') != len(entries):
        raise ValueError('command entry count does not match the stamped page count')
    verify_anchors(text)
    return text


def check(current, regenerated):
    """Byte-exact outside the archives; archives compare by content. Returns (ok, report)."""
    if current == regenerated:
        return True, 'guide is up to date'
    def strip(text):
        return DATA_URI.sub('href="data:application/zip;base64,<archive>"', text)
    report = []
    archives_current = [zip_manifest(base64.b64decode(m)) for m in DATA_URI.findall(current)]
    archives_new = [zip_manifest(base64.b64decode(m)) for m in DATA_URI.findall(regenerated)]
    if archives_current != archives_new:
        report.append('embedded example archives differ from docs/site/examples')
    def lines(text):
        # The guide packs whole sections onto one line; diff at tag boundaries instead.
        return re.sub(r'(?<=>)(?=<)', '\n', strip(text)).splitlines()
    if strip(current) != strip(regenerated):
        diff = difflib.unified_diff(lines(current), lines(regenerated), 'committed', 'regenerated', lineterm='', n=1)
        diff_lines = [line if len(line) < 400 else line[:400] + '…' for line in diff]
        report.extend(diff_lines[:80])
        if len(diff_lines) > 80:
            report.append(f'… {len(diff_lines) - 80} more diff lines')
    return not report, '\n'.join(report) if report else 'guide is up to date'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=pathlib.Path, help='framechord executable (default: target/release, then target/debug)')
    parser.add_argument('--check', action='store_true', help='fail when the committed guide differs from the regenerated one')
    parser.add_argument('--html', type=pathlib.Path, default=ROOT / 'docs/site/index.html')
    parser.add_argument('--examples', type=pathlib.Path, default=ROOT / 'docs/site/examples')
    parser.add_argument('--models', type=pathlib.Path, default=ROOT / 'assets/models.json')
    args = parser.parse_args()
    binary = args.binary or default_binary()
    version = crate_version()
    built = binary_version(binary)
    if built != version:
        raise SystemExit(f'{binary} reports {built} but Cargo.toml says {version}; rebuild before generating')
    if not args.examples.is_dir():
        raise SystemExit(f'missing example sources: {args.examples}')
    current = args.html.read_text(encoding='utf-8')
    models = json.loads(args.models.read_text(encoding='utf-8'))
    regenerated = regenerate(current, commands_markdown=export_commands(binary), models=models,
                             examples=args.examples, version=version)
    ok, report = check(current, regenerated)
    if args.check:
        print(report)
        raise SystemExit(0 if ok else 1)
    if ok:
        print('guide already up to date')
        return
    args.html.write_text(regenerated, encoding='utf-8', newline='\n')
    print(json.dumps({'html': str(args.html), 'bytes': len(regenerated.encode('utf-8')),
                      'sha256': hashlib.sha256(regenerated.encode('utf-8')).hexdigest(),
                      'command_pages': regenerated.count('class="command-entry"'), 'version': version}))


if __name__ == '__main__':
    main()
