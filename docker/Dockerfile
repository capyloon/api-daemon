FROM debian:stretch

MAINTAINER Viga <viga.li@kaiostech.com>

WORKDIR /home

# common packages
RUN apt-get update && \
    apt-get install --no-install-recommends -y \
    ca-certificates curl file \
    build-essential \
    zlib1g-dev \
    pkg-config \
    libssl-dev openssh-server \
    git cmake gpg wget xvfb \
    zip unzip \
    clang libclang-dev \
    libgtk-3-0 \
    autoconf automake autotools-dev libtool xutils-dev libselinux1-dev && \
    rm -rf /var/lib/apt/lists/*

RUN curl -sL https://deb.nodesource.com/setup_9.x | bash - && \
    apt-get install -y nodejs

RUN apt-get update && apt-get install -y apt-transport-https
RUN curl -sS https://dl.yarnpkg.com/debian/pubkey.gpg | apt-key add -
RUN echo "deb https://dl.yarnpkg.com/debian/ stable main" | tee /etc/apt/sources.list.d/yarn.list
RUN apt-get update && apt-get install yarn

# install toolchain
RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- --default-toolchain stable -y

ENV PATH=/root/.cargo/bin:$PATH

# Install the binding generator
RUN cargo install --version 0.41.0 bindgen

RUN apt-get update
RUN apt-get install -y wget libfreetype6 libfontconfig1 libxrender1 libasound-dev libdbus-glib-1-dev libgtk2.0-0 libxt6 python-pip
RUN pip install selenium==2.48.0 robotframework==3.0 requests robotframework-requests robotframework-selenium2library==1.8.0 pymysql robotframework-databaselibrary robotframework-excellibrary
RUN pip install requests --upgrade
RUN pip install awscli --upgrade --user

RUN wget https://ftp.mozilla.org/pub/firefox/releases/70.0/linux-x86_64/en-US/firefox-70.0.tar.bz2
RUN tar -xjf firefox-70.0.tar.bz2
RUN mv firefox /opt/firefox70
RUN ln -s /opt/firefox70/firefox /usr/bin/firefox

# Install geckodriver to a location in $PATH
RUN wget https://github.com/mozilla/geckodriver/releases/download/v0.26.0/geckodriver-v0.26.0-linux64.tar.gz
RUN tar xzf geckodriver-v0.26.0-linux64.tar.gz
RUN mv geckodriver /usr/bin/geckodriver

RUN firefox --version

# Needed to link the selinux stuff
RUN apt-get install libselinux1-dev

# Add the vhosts mapping that are expected by tests.
RUN echo "127.0.0.1       valid.local" >> /etc/hosts
RUN echo "127.0.0.1       unknown.local" >> /etc/hosts
RUN echo "127.0.0.1       missing-zip.local" >> /etc/hosts
