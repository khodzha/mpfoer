## -----------------------------------------------------------------------------
## Build
## -----------------------------------------------------------------------------
FROM rust:1.53.0-slim-buster as build-stage

RUN apt update && apt install -y --no-install-recommends pkg-config libssl-dev

WORKDIR "/build"

COPY Cargo.* /build/
RUN mkdir /build/src
COPY src/ /build/src/
RUN cargo build --release

FROM debian:buster

RUN apt update && apt install -y --no-install-recommends pkg-config libssl-dev wget ca-certificates xz-utils
RUN mkdir /ffmpeg/
WORKDIR /ffmpeg
RUN wget https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz
RUN wget https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz.md5
RUN md5sum -c ffmpeg-release-amd64-static.tar.xz.md5
RUN ls && tar -xvf ffmpeg-release-amd64-static.tar.xz
RUN cp /ffmpeg/ffmpeg-4.4-amd64-static/ffmpeg /bin/ffmpeg

COPY --from=build-stage "/build/target/release/mpfoer" "/app/mpfoer"

WORKDIR "/app"
ENTRYPOINT ["/app/mpfoer"]
