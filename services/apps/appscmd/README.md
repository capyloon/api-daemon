# Command Line Interface to the Apps Service

This tool talks to apps service of the api-daemon to perform actions on the apps registry:
- app installation.
- app uninstallation.

# Building

Windows
# sudo apt-get install gcc-mingw-w64-x86-64 g++-mingw-w64-x86-64 wine64
# rustup self update
# rustup target add x86_64-pc-windows-gnu
# rustup toolchain install stable-x86_64-pc-windows-gnu

macOS:
Pre-built osx toolchain is in ftp.kaiostech.com: kaios_all/Tools/osxcross.tgz
# tar zxvf osxcross.tgz // Extract osxcross.tgz in “/opt” directory
# rustup target add x86_64-apple-darwin
# rustup toolchain install stable-x86_64-apple-darwin

Ubuntu:
# rustup target add x86_64-unknown-linux-gnu
# rustup toolchain install stable-x86_64-unknown-linux-gnu

Then create cargo config file:
# vi ~/.cargo/config
===============================
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
[target.x86_64-apple-darwin]
linker = "x86_64-apple-darwin15-clang"
ar = "x86_64-apple-darwin15-ar"
================================


# cargo build --release --target x86_64-unknown-linux-gnu //Ubuntu
# cargo build --release --target x86_64-pc-windows-gnu //Windows
# cargo build --release --target x86_64-apple-darwin //macOS

# Running
The current set of parameters is documented by running `./target/$PLATFORM/release/appscmd --help`.
