#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$repo_root/target/debug/velatura"
launcher="$repo_root/scripts/velatura-native"
destination="${HOME}/.local/bin/velatura"
legacy_dir="${HOME}/.local/share/velatura/legacy-cli"
legacy_backup="$legacy_dir/velatura"

mkdir -p "$(dirname -- "$destination")" "$legacy_dir"

# Preserve the previous Python CLI outside PATH for rollback/reference.
if [[ -f "$destination" && ! -L "$destination" ]]; then
  if [[ -e "$legacy_backup" ]]; then
    printf 'Refusing to overwrite existing legacy backup: %s\n' "$legacy_backup" >&2
    exit 1
  fi
  mv -- "$destination" "$legacy_backup"
fi

if [[ -L "$destination" ]]; then
  current_target="$(readlink -f -- "$destination" || true)"
  if [[ "$current_target" != "$binary" && "$current_target" != "$launcher" ]]; then
    printf 'Refusing to replace unexpected symlink: %s -> %s\n' "$destination" "$(readlink -- "$destination")" >&2
    exit 1
  fi
fi

# Keep a stable symlink to the launcher: it selects the current compiled
# binary when present and falls back to Cargo after a clean checkout/build.
target="$launcher"
ln -sfn "$target" "$destination"
chmod +x "$launcher"
printf 'Installed native command: %s -> %s\n' "$destination" "$target"
if [[ -e "$legacy_backup" ]]; then
  printf 'Previous Python CLI preserved outside PATH: %s\n' "$legacy_backup"
fi
printf 'Use: velatura run | help | stop | restart | ipc ...\n'
