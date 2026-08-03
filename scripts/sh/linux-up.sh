#!/usr/bin/env bash
# Build image + start persistent wshot container (Xvfb/openbox/probe via boot.sh).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

NAME="${WSHOT_NAME:-wshot}"
IMAGE="${WSHOT_IMAGE:-widgets-linux}"

echo "linux-up: docker build ${IMAGE} (native platform)..."
docker build -t "${IMAGE}" tests/linux

# Recreate so Dockerfile/env changes always apply (binary still cached on /work mount).
if docker ps -aq -f "name=^/${NAME}$" | grep -q .; then
  echo "linux-up: recreating ${NAME}..."
  docker rm -f "${NAME}" >/dev/null
fi

mkdir -p out/inbox out/png/linux-x11 out/props out/linux

# Cargo registry cache volume speeds rebuilds inside the container.
docker volume create widgets-cargo >/dev/null 2>&1 || true
docker volume create widgets-cargo-git >/dev/null 2>&1 || true

echo "linux-up: starting ${NAME}..."
docker run -d --name "${NAME}" \
  -v "$ROOT:/work" \
  -v widgets-cargo:/root/.cargo/registry \
  -v widgets-cargo-git:/root/.cargo/git \
  -e DISPLAY=:99 \
  -e LIBGL_ALWAYS_SOFTWARE=1 \
  -e WEBKIT_DISABLE_COMPOSITING_MODE=1 \
  -e WEBKIT_DISABLE_DMABUF_RENDERER=1 \
  -e GSK_RENDERER=cairo \
  -e CARGO_BUILD_JOBS=2 \
  -e CARGO_INCREMENTAL=0 \
  -e CARGO_TARGET_DIR=/tmp/widget-probe-target \
  -e RUSTFLAGS="-C link-arg=-fuse-ld=lld" \
  "${IMAGE}" bash tests/linux/boot.sh

echo "linux-up: waiting for probe build (first boot is slow)..."
for i in $(seq 1 120); do
  if docker logs "${NAME}" 2>&1 | grep -qE 'widget-probe watch|starting widget-probe --watch'; then
    echo "linux-up: OK (${NAME})"
    exit 0
  fi
  if ! docker ps -q -f "name=^/${NAME}$" | grep -q .; then
    echo "linux-up: container exited"
    docker logs "${NAME}" 2>&1 | tail -40
    exit 1
  fi
  sleep 5
done

echo "linux-up: timeout waiting for probe — check: docker logs ${NAME}"
docker logs "${NAME}" 2>&1 | tail -60
exit 1
