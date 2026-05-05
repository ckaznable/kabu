#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

TARGET_TRIPLE="${TARGET_TRIPLE:-aarch64-unknown-linux-gnu}"
PACKAGE_NAME="${1:-kabu-server}"
IMAGE_TAG="${2:-${PACKAGE_NAME}:aarch64}"
BIN_DIR="${BIN_DIR:-container-bin}"
BIN_PATH="${BIN_DIR}/${PACKAGE_NAME}"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

resolve_containerfile() {
  case "${PACKAGE_NAME}" in
    kabu-server)
      echo "${CONTAINERFILE:-Containerfile.server.aarch64}"
      ;;
    kabu-updater)
      echo "${CONTAINERFILE:-Containerfile.updater.aarch64}"
      ;;
    *)
      echo "unsupported PACKAGE_NAME: ${PACKAGE_NAME}" >&2
      echo "supported values: kabu-server, kabu-updater" >&2
      exit 1
      ;;
  esac
}

CONTAINERFILE="$(resolve_containerfile)"

require_cmd cargo
require_cmd rustup
require_cmd podman
require_cmd zig

if [[ "${PACKAGE_NAME}" == "kabu-server" ]]; then
  require_cmd npm
fi

if ! cargo zigbuild -h >/dev/null 2>&1; then
  echo "missing cargo-zigbuild. install it with: cargo install cargo-zigbuild" >&2
  exit 1
fi

if ! rustup target list --installed | grep -qx "${TARGET_TRIPLE}"; then
  echo "installing rust target: ${TARGET_TRIPLE}"
  rustup target add "${TARGET_TRIPLE}"
fi

mkdir -p "${BIN_DIR}"

if [[ "${PACKAGE_NAME}" == "kabu-server" ]]; then
  echo "building frontend assets"
  npm ci --prefix frontend
  npm run build --prefix frontend
fi

echo "building ${PACKAGE_NAME} for ${TARGET_TRIPLE} with cargo zigbuild"
cargo zigbuild -p "${PACKAGE_NAME}" --release --target "${TARGET_TRIPLE}"

cp "target/${TARGET_TRIPLE}/release/${PACKAGE_NAME}" "${BIN_PATH}"
chmod +x "${BIN_PATH}"

echo "building container image ${IMAGE_TAG}"
podman build \
  --platform linux/arm64 \
  --build-arg "BIN_PATH=${BIN_PATH}" \
  -t "${IMAGE_TAG}" \
  -f "${CONTAINERFILE}" \
  .

echo "done: ${IMAGE_TAG}"
