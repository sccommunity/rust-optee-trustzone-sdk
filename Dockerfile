FROM ubuntu:16.04
MAINTAINER Teaclave Contributors <dev@teaclave.apache.org>

RUN dpkg --add-architecture i386
RUN apt-get update && \
  apt-get install -y -q android-tools-adb android-tools-fastboot autoconf \
  automake bc bison build-essential cscope curl device-tree-compiler \
  expect flex ftp-upload gdisk iasl libattr1-dev libc6:i386 libcap-dev \
  libfdt-dev libftdi-dev libglib2.0-dev libhidapi-dev libncurses5-dev \
  libpixman-1-dev libssl-dev libstdc++6:i386 libtool libz1:i386 make \
  mtools netcat python-crypto python-serial python-wand unzip uuid-dev \
  xdg-utils xterm xz-utils zlib1g-dev git wget cpio libssl-dev iasl \
  screen libbrlapi-dev libaio-dev libcurl3 libbluetooth-dev libsdl2-2.0 \
  python3 python3-pip python3-pyelftools

RUN pip3 install pycryptodome

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
  . $HOME/.cargo/env && \
  rustup default nightly-2019-07-08 && \
  rustup component add rust-src && \
  rustup target install aarch64-unknown-linux-gnu && \
  rustup default 1.44.0 && cargo +1.44.0 install xargo && \
  rustup default nightly-2019-07-08

ENV PATH="/root/.cargo/bin:$PATH"
