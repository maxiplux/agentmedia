#!/bin/sh
# Install a stable public release. No sudo, Rust, Python, or project initialization.
set -eu
version=""
bin_dir=""
modify_path=1
force=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --version|--bin-dir) [ "$#" -ge 2 ] || { echo "Missing value for $1" >&2; exit 2; }; key=$1; shift; if [ "$key" = --version ]; then version=$1; else bin_dir=$1; fi ;;
    --no-modify-path) modify_path=0 ;;
    --force) force=1 ;;
    --help) echo 'Usage: install.sh [--version 0.6.1] [--bin-dir PATH] [--no-modify-path] [--force]'; exit 0 ;;
    *) echo "Unknown installer option: $1" >&2; exit 2 ;;
  esac
  shift
done
command -v curl >/dev/null || { echo 'curl is required to download framechord' >&2; exit 1; }
case "$(uname -s):$(uname -m)" in
  Darwin:arm64) target=aarch64-apple-darwin ;;
  Linux:x86_64) target=x86_64-unknown-linux-gnu ;;
  *) echo 'No published native binary for this platform. Build from source or use a supported Apple Silicon macOS, Linux x86-64 or Windows x86-64 release.' >&2; exit 1 ;;
esac
umask 077
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT HUP INT TERM
base=https://github.com/maxiplux/framechord/releases
if [ -z "$version" ]; then
  curl --proto '=https' --tlsv1.2 -fsSL "$base/latest/download/version.txt" -o "$temporary/version.txt"
  version=$(cat "$temporary/version.txt")
fi
version=${version#v}
case "$version" in ''|*[!0-9.]*) echo 'Invalid stable release version' >&2; exit 1 ;; esac
# Stable three-component versions only; never evaluate downloaded text as shell code.
if ! printf '%s\n' "$version" | LC_ALL=C grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then echo 'Invalid stable release version' >&2; exit 1; fi
asset=framechord-$target
url=$base/download/v$version
curl --proto '=https' --tlsv1.2 -fsSL "$url/$asset" -o "$temporary/framechord"
curl --proto '=https' --tlsv1.2 -fsSL "$url/$asset.sha256" -o "$temporary/checksum"
expected=$(awk 'NR==1 {print $1}' "$temporary/checksum")
if ! printf '%s\n' "$expected" | LC_ALL=C grep -Eq '^[a-fA-F0-9]{64}$'; then echo 'Invalid release checksum' >&2; exit 1; fi
if command -v sha256sum >/dev/null; then
  actual=$(sha256sum "$temporary/framechord" | awk '{print $1}')
else
  actual=$(shasum -a 256 "$temporary/framechord" | awk '{print $1}')
fi
[ "$actual" = "$expected" ] || { echo 'Checksum mismatch; nothing was installed' >&2; exit 1; }
chmod 755 "$temporary/framechord"
[ "$("$temporary/framechord" --version)" = "framechord $version" ] || { echo 'Executable version check failed; nothing was installed' >&2; exit 1; }
set -- install
[ -z "$bin_dir" ] || set -- "$@" --bin-dir "$bin_dir"
[ "$modify_path" = 1 ] || set -- "$@" --no-modify-path
[ "$force" = 0 ] || set -- "$@" --force
"$temporary/framechord" "$@"
