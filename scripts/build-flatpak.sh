#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"
APP_ID="io.github.tonyboysuper.RenovaPlan"
mkdir -p target

python3 scripts/generate-cargo-sources.py

if command -v flatpak-builder >/dev/null; then
  BUILDER=(flatpak-builder)
else
  BUILDER=(flatpak run org.flatpak.Builder)
fi

"${BUILDER[@]}" --user --force-clean --install-deps-from=flathub \
  --repo=target/flatpak-repo target/flatpak-build "flatpak/$APP_ID.json"

flatpak build-bundle --runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo \
  target/flatpak-repo renova-plan.flatpak "$APP_ID"

if [[ "${FLATPAK_SKIP_INSTALL:-0}" != 1 ]]; then
  flatpak --user install -y --or-update --bundle renova-plan.flatpak
fi

printf 'Paquet Flatpak généré : %s/renova-plan.flatpak\n' "$ROOT_DIR"
