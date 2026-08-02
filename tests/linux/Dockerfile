# Native arm64 on Apple Silicon — do NOT add --platform linux/amd64 (qemu is 5–10× slower).
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV PATH=/root/.cargo/bin:$PATH
# WebKit under Xvfb: force software path (set again in boot/run scripts).
ENV LIBGL_ALWAYS_SOFTWARE=1 \
    WEBKIT_DISABLE_COMPOSITING_MODE=1 \
    WEBKIT_DISABLE_DMABUF_RENDERER=1 \
    GSK_RENDERER=cairo

RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential curl pkg-config ca-certificates jq \
      libwebkit2gtk-4.1-dev libgtk-3-dev librsvg2-dev libgtk-layer-shell-dev \
      libxdo-dev \
      xvfb openbox x11-utils xdotool imagemagick \
      dbus-x11 at-spi2-core \
      libgl1-mesa-dri mesa-utils \
      fonts-noto-core fonts-noto-color-emoji \
      sway grim \
      patchelf file lld \
 && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
 && rustup default stable

WORKDIR /work

CMD ["sleep", "infinity"]
