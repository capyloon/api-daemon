FROM ubuntu:18.04

ARG LAST_EDIT_COMMIT

LABEL maintainer="viga.li@kaiostech.com" \
# Update that value for automation to pick up the new tag.
      RELEASE_TAG="1.49.0_3"

ENV DEBIAN_FRONTEND=noninteractive \
    LAST_EDIT_COMMIT=${LAST_EDIT_COMMIT}

# common packages
RUN apt-get update && \
    apt-get install --no-install-recommends -qq -y \
    ca-certificates curl clang libclang-dev build-essential \
    git cmake gnupg2 gpg-agent wget xvfb zlib1g-dev \
    pkg-config zip unzip libssl-dev openssh-server \
    autoconf automake autotools-dev libtool xutils-dev \
    libfreetype6 libfontconfig1 libxrender1 libasound-dev \
    libgtk-3-0 libdbus-glib-1-dev libgtk2.0-0 libxt6 python-pip \
    gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 wine64 \
    apt-transport-https libselinux1-dev iproute2

## define the versions across essentials for test
ENV NODE_VERSION="9.11.2" \
    RUSTUP_TOOLCHAIN="stable" \
    FX_VERSION="84.0.2" \
    GECKODRIVER_VERSION="0.29.0" \
    BINDGEN_VERSION="0.56.0" \
    PATH="/root/.cargo/bin:$PATH"

# separate RUN into multiple layers to utilize local cache in development
# node
RUN wget -nv https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-x64.tar.xz && \
    tar -xJf "node-v${NODE_VERSION}-linux-x64.tar.xz" -C /usr/local --strip-components=1 --no-same-owner && \
    ln -s /usr/local/bin/node /usr/local/bin/nodejs

# yarn
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add && \
    echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list && \
    apt-get update && \
    apt-get install --no-install-recommends -y yarn

# rust toolchain and bindgen
RUN curl -sSf https://sh.rustup.rs | sh -s -- --default-toolchain ${RUSTUP_TOOLCHAIN} -y && \
    cargo install --version ${BINDGEN_VERSION} bindgen && \
    cargo install cargo-deny && \
    rustup target add armv7-linux-androideabi && \
    rustup target add aarch64-linux-android && \
    rustup target add x86_64-linux-android && \
    rustup target add x86_64-pc-windows-gnu && \
    rustup toolchain install ${RUSTUP_TOOLCHAIN}-x86_64-pc-windows-gnu

# firefox
RUN wget -nv https://ftp.mozilla.org/pub/firefox/releases/${FX_VERSION}/linux-x86_64/en-US/firefox-${FX_VERSION}.tar.bz2 && \
    tar -xjf firefox-${FX_VERSION}.tar.bz2 && \
    mv firefox /opt/firefox${FX_VERSION} && \
    ln -s /opt/firefox${FX_VERSION}/firefox /usr/bin/firefox

# geckodriver
RUN wget -nv https://github.com/mozilla/geckodriver/releases/download/v${GECKODRIVER_VERSION}/geckodriver-v${GECKODRIVER_VERSION}-linux64.tar.gz && \
    tar xzf geckodriver-v${GECKODRIVER_VERSION}-linux64.tar.gz -C /usr/bin/

# clean tmp files and cache of apt-get
RUN apt-get clean && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* \
    firefox-${FX_VERSION}.tar.bz2 \
    geckodriver-v${GECKODRIVER_VERSION}-linux64.tar.gz \
    node-v${NODE_VERSION}-linux-x64.tar.xz && \
    # add the vhosts mapping that are expected by tests.
    echo "127.0.0.1       valid.local" >> /etc/hosts && \
    echo "127.0.0.1       unknown.local" >> /etc/hosts && \
    echo "127.0.0.1       missing-zip.local" >> /etc/hosts
