#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$repo_root/target/debug/velatura"
launcher="$repo_root/scripts/velatura-native"
destination="${HOME}/.local/bin/velatura-native"

mkdir -p "$(dirname -- "$destination")"
if [[ -e "$destination" && ! -L "$destination" ]]; then
  printf 'Refusing to replace existing non-symlink: %s\n' "$destination" >&2
  exit 1
fi

# Install the compiled executable when available; otherwise install a
# development launcher that builds it on first use.
if [[ -x "$binary" ]]; then
  target="$binary"
else
  target="$launcher"
fi
ln -sfn "$target" "$destination"
printf 'Installed development command: %s -> %s\n' "$destination" "$target"
printf 'PATH already includes ~/.local/bin on this machine.\n'
