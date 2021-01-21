FROM ubuntu:18.04

LABEL maintainer="viga.li@kaiostech.com" \
# Update that value for automation to pick up the new tag.
      RELEASETAG="1.49.0_2"

WORKDIR /home

# common packages
RUN apt-get update && \
    apt-get install --no-install-recommends -y \
    ca-certificates curl clang libclang-dev build-essential \
    git cmake gnupg2 gpg-agent wget xvfb zlib1g-dev \
    pkg-config zip unzip libssl-dev openssh-server \
    autoconf automake autotools-dev libtool xutils-dev \
    libfreetype6 libfontconfig1 libxrender1 libasound-dev \
    libgtk-3-0 libdbus-glib-1-dev libgtk2.0-0 libxt6 python-pip \
    gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 wine64 \
    apt-transport-https libselinux1-dev

#install node
ENV NODE_VERSION 9.11.2
RUN wget https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz && \
    tar -xJf "node-v${NODE_VERSION}-linux-x64.tar.xz" -C /usr/local --strip-components=1 --no-same-owner && \
    ln -s /usr/local/bin/node /usr/local/bin/nodejs && \
    rm "node-v${NODE_VERSION}-linux-x64.tar.xz"

#install yarn
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add && \
    echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list && \
    apt-get update && \
    apt-get install -y yarn && \
    rm -rf /var/lib/apt/lists/*

# install rust toolchain
RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y

ENV PATH=/root/.cargo/bin:$PATH

# Install the binding generator
RUN cargo install --version 0.56.0 bindgen && \
    cargo install cargo-deny && \
    rustup target add armv7-linux-androideabi && \
    rustup target add aarch64-linux-android && \
    rustup target add x86_64-linux-android && \
    rustup target add x86_64-pc-windows-gnu && \
    rustup toolchain install stable-x86_64-pc-windows-gnu

ENV FX_VERSION 84.0.2
RUN wget https://ftp.mozilla.org/pub/firefox/releases/${FX_VERSION}/linux-x86_64/en-US/firefox-${FX_VERSION}.tar.bz2 && \
    tar -xjf firefox-${FX_VERSION}.tar.bz2 && \
    mv firefox /opt/firefox${FX_VERSION} && \
    ln -s /opt/firefox${FX_VERSION}/firefox /usr/bin/firefox && \
    rm -f firefox-${FX_VERSION}.tar.bz2

# Install geckodriver to a location in $PATH
ENV GECKODRIVER v0.29.0
RUN wget https://github.com/mozilla/geckodriver/releases/download/${GECKODRIVER}/geckodriver-${GECKODRIVER}-linux64.tar.gz && \
    tar xzf geckodriver-${GECKODRIVER}-linux64.tar.gz && \
    mv geckodriver /usr/bin/geckodriver && \
    rm -f geckodriver-${GECKODRIVER}-linux64.tar.gz

# Add the vhosts mapping that are expected by tests.
RUN echo "127.0.0.1       valid.local" >> /etc/hosts && \
    echo "127.0.0.1       unknown.local" >> /etc/hosts && \
    echo "127.0.0.1       missing-zip.local" >> /etc/hosts
