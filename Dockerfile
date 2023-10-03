FROM rust:1.72-bookworm AS builder

WORKDIR /usr/src/tocas

RUN sudo apt-get update && sudo apt-get install -y libopus-dev ffmpeg python3-pip
RUN pip install yt-dlp

COPY . .

RUN cargo install --path .