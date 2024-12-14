FROM ubuntu:24.04 AS builder

WORKDIR /tmp

ENV CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN apt-get update && \
    apt-get install -y \
        --no-install-recommends \
        clang \
        curl \
        pkg-config \
        xz-utils \
        build-essential \
        curl \
        ca-certificates \
        meson \
        ninja-build \
        pkg-config \
        cmake \
        nasm \
        yasm \
        protobuf-compiler \
        libxv-dev \
        libc++-dev \
        libdrm2 \
        libasound2t64 \
        patchelf && \
    curl -L https://sourceforge.net/projects/avbuild/files/linux/ffmpeg-7.1-linux-clang-default.tar.xz/download -o ffmpeg.tar.xz && \
    tar -xvf ffmpeg.tar.xz && \
    mv ffmpeg-7.1-linux-clang-default/include/* /usr/include && \
    mv ffmpeg-7.1-linux-clang-default/lib/amd64/* /usr/local/lib && \
    mv ffmpeg-7.1-linux-clang-default/bin/amd64/* /usr/local/bin && \
    ldconfig && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

COPY . .

ARG PROFILE=release

RUN cargo build --profile ${PROFILE} --bin scuffle-image-processor -p scuffle-image-processor --locked

RUN mkdir /out && \
    mv target/${PROFILE}/scuffle-image-processor /out/image-processor && \
    ldd /out/image-processor | grep -o '/[^ ]*' | xargs -I '{}' cp {} /out && \
    patchelf --set-rpath '$ORIGIN' /out/*

FROM ubuntu:24.04

LABEL org.opencontainers.image.source=https://github.com/scufflecloud/scuffle
LABEL org.opencontainers.image.description="Scuffle Image Processor" 
LABEL org.opencontainers.image.licenses=AGPL-3.0

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates && \
    update-ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /out /app

STOPSIGNAL SIGTERM

USER 1000

ENTRYPOINT ["/app/image-processor"]
