# Command Line Interface to `app-service`

This tool performs the following actions on apps registry via `apps-service` of `api-daemon`:
- app list
- app installation
- app uninstallation

## Table of Contents ##
1. [Setup](#1-setup-build-environment)
2. [Build](#2-build)
3. [Run](#3-run)

# 1. Setup Build Environment

## 1.1 Setup toolchain via `rustup`

**For Ubuntu target**
```bash
rustup target add x86_64-unknown-linux-gnu
rustup toolchain install stable-x86_64-unknown-linux-gnu

```

**For Windows target**
```bash
sudo apt-get install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 wine64
rustup self update
rustup target add x86_64-pc-windows-gnu
rustup toolchain install stable-x86_64-pc-windows-gnu
```

**For macOS target**

```bash
rustup target add x86_64-apple-darwin
rustup toolchain install stable-x86_64-apple-darwin
```
OSXCross is required to build the macOS target. A pre-built toolchain for OSX cross is available at `ftp.kaiostech.com/kaios_all/Tools/osxcross.tgz`
```
tar zxvf osxcross.tgz -C /opt
# Update the enironment variable for osxcross.
export PATH=/opt/osxcross/bin:$PATH
export LD_LIBRARY_PATH=/opt/osxcross/lib
```

## 1.2 Create `cargo` configuration

Edit `~/.cargo/config` and add the following:
```
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin15-clang"
ar = "x86_64-apple-darwin15-ar"
```

# 2. Build

**Build Ubuntu target**
```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

**Build Windows target**
```bash
cargo build --release --target x86_64-pc-windows-gnu 
```

**Build macOS target**
```bash
CC=x86_64-apple-darwin15-cc cargo build --release --target x86_64-apple-darwin
```

# 3. Run
Check the supported parameters by running `./target/$PLATFORM/release/appscmd --help`.
