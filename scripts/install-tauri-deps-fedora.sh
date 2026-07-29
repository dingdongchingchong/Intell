#!/usr/bin/env bash
# Install system packages required to build the CaseFlow Tauri desktop app on Fedora.
# Usage: sudo bash scripts/install-tauri-deps-fedora.sh

set -euo pipefail

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Run with sudo: sudo bash scripts/install-tauri-deps-fedora.sh" >&2
  exit 1
fi

dnf install -y \
  webkit2gtk4.1-devel \
  gtk3-devel \
  librsvg2-devel \
  openssl-devel \
  curl \
  wget \
  file \
  gcc \
  gcc-c++ \
  make \
  pkgconf-pkg-config

echo "Tauri system dependencies installed."
echo "Next: cd $(dirname "$0")/.. && npm install && npm run tauri:dev"
