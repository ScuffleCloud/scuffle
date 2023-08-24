FROM ubuntu:latest

LABEL org.opencontainers.image.source=https://github.com/scuffletv/scuffle
LABEL org.opencontainers.image.description="Video Ingest Container for ScuffleTV"
LABEL org.opencontainers.image.licenses=BSD-4-Clause

WORKDIR /app

RUN --mount=type=bind,src=docker/cve.sh,dst=/cve.sh --mount=type=bind,src=target/x86_64-unknown-linux-gnu/release/video-ingest,dst=/mount/ingest /cve.sh && \
    cp /mount/ingest /app/ingest && \
    chmod +x /app/ingest

STOPSIGNAL SIGTERM

USER 1000

ENTRYPOINT ["/app/ingest"]
