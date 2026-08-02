#!/usr/bin/env bash
# Persistent Wayland/sway container for layer-shell shots (optional second host).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

NAME="${WSHOT_WL_NAME:-wshot-wl}"
IMAGE="${WSHOT_IMAGE:-widgets-linux}"

if ! docker image inspect "${IMAGE}" >/dev/null 2>&1; then
  docker build -t "${IMAGE}" tests/linux
fi

if docker ps -q -f "name=^/${NAME}$" | grep -q .; then
  echo "linux-up-wl: ${NAME} already running"
  exit 0
fi
if docker ps -aq -f "name=^/${NAME}$" | grep -q .; then
  docker rm -f "${NAME}" >/dev/null
fi

mkdir -p out/png/linux-wayland

docker volume create widgets-cargo >/dev/null 2>&1 || true

# Keep container idle; tests/linux/run-wayland.sh is invoked via docker exec.
docker run -d --name "${NAME}" \
  -v "$ROOT:/work" \
  -v widgets-cargo:/root/.cargo/registry \
  -v widgets-cargo-git:/root/.cargo/git \
  "${IMAGE}" sleep infinity

echo "linux-up-wl: OK (${NAME}) — run: docker exec ${NAME} bash tests/linux/run-wayland.sh"
