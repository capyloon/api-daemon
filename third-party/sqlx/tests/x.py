#!/usr/bin/env python3

import subprocess
import os
import sys
import time
import argparse
import platform
import urllib.request
from glob import glob
from docker import start_database

parser = argparse.ArgumentParser()
parser.add_argument("-t", "--target")
parser.add_argument("-e", "--target-exact")
parser.add_argument("-l", "--list-targets", action="store_true")
parser.add_argument("--test")

argv, unknown = parser.parse_known_args()

# base dir of sqlx workspace
dir_workspace = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))

# dir of tests
dir_tests = os.path.join(dir_workspace, "tests")


def maybe_fetch_sqlite_extension():
    """
    For supported platforms, if we're testing SQLite and the file isn't
    already present, grab a simple extension for testing.

    Returns the extension name if it was downloaded successfully or `None` if not.
    """
    BASE_URL = "https://github.com/nalgeon/sqlean/releases/download/0.15.2/"
    if platform.system() == "Darwin":
        if platform.machine() == "arm64":
            download_url = BASE_URL + "/ipaddr.arm64.dylib"
            filename = "ipaddr.dylib"
        else:
            download_url = BASE_URL + "/ipaddr.dylib"
            filename = "ipaddr.dylib"
    elif platform.system() == "Linux":
        download_url = BASE_URL + "/ipaddr.so"
        filename = "ipaddr.so"
    else:
        # Unsupported OS
        return None

    if not os.path.exists(filename):
        content = urllib.request.urlopen(download_url).read()
        with open(filename, "wb") as fd:
            fd.write(content)

    return filename.split(".")[0]


def run(command, comment=None, env=None, service=None, tag=None, args=None, database_url_args=None):
    if argv.list_targets:
        if tag:
            print(f"{tag}")

        return

    if argv.target and not tag.startswith(argv.target):
        return

    if argv.target_exact and tag != argv.target_exact:
        return

    if comment is not None:
        print(f"\x1b[2m # {comment}\x1b[0m")

    environ = env or {}

    if service == "sqlite":
        if maybe_fetch_sqlite_extension() is not None:
            if environ.get("RUSTFLAGS"):
                environ["RUSTFLAGS"] += " --cfg sqlite_ipaddr"
            else:
                environ["RUSTFLAGS"] = "--cfg sqlite_ipaddr"

    if service is not None:
        database_url = start_database(service, database="sqlite/sqlite.db" if service == "sqlite" else "sqlx", cwd=dir_tests)

        if database_url_args:
            database_url += "?" + database_url_args

        environ["DATABASE_URL"] = database_url

        # show the database url
        print(f"\x1b[94m @ {database_url}\x1b[0m")

    command_args = []

    if argv.test:
        command_args.extend(["--test", argv.test])

    if unknown:
        command_args.extend(["--", *unknown])

        if args is not None:
            command_args.extend(args)

    print(f"\x1b[93m $ {command} {' '.join(command_args)}\x1b[0m")

    cwd = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))
    res = subprocess.run(
        [
            *command.split(" "),
            *command_args
        ],
        env=dict(**os.environ, **environ),
        cwd=cwd,
    )

    if res.returncode != 0:
        sys.exit(res.returncode)


# before we start, we clean previous profile data
# keeping these around can cause weird errors
for path in glob(os.path.join(os.path.dirname(__file__), "target/**/*.gc*"), recursive=True):
    os.remove(path)

#
# check
#

for runtime in ["async-std", "tokio", "actix"]:
    for tls in ["native-tls", "rustls"]:
        run(
            f"cargo c --no-default-features --features all-databases,all-types,offline,macros,runtime-{runtime}-{tls}",
            comment="check with async-std",
            tag=f"check_{runtime}_{tls}"
        )

#
# unit test
#

for runtime in ["async-std", "tokio", "actix"]:
    for tls in ["native-tls", "rustls"]:
        run(
            f"cargo test --no-default-features --manifest-path sqlx-core/Cargo.toml --features all-databases,all-types,runtime-{runtime}-{tls}",
            comment="unit test core",
            tag=f"unit_{runtime}_{tls}"
        )

#
# integration tests
#

for runtime in ["async-std", "tokio", "actix"]:
    for tls in ["native-tls", "rustls"]:

        #
        # sqlite
        #

        run(
            f"cargo test --no-default-features --features macros,offline,any,all-types,sqlite,runtime-{runtime}-{tls}",
            comment=f"test sqlite",
            service="sqlite",
            tag=f"sqlite" if runtime == "async-std" else f"sqlite_{runtime}",
        )

        #
        # postgres
        #

        for version in ["14", "13", "12", "11", "10"]:
            run(
                f"cargo test --no-default-features --features macros,offline,any,all-types,postgres,runtime-{runtime}-{tls}",
                comment=f"test postgres {version}",
                service=f"postgres_{version}",
                tag=f"postgres_{version}" if runtime == "async-std" else f"postgres_{version}_{runtime}",
            )

        ## +ssl
        for version in ["14", "13", "12", "11", "10"]:
            run(
                f"cargo test --no-default-features --features macros,offline,any,all-types,postgres,runtime-{runtime}-{tls}",
                comment=f"test postgres {version} ssl",
                database_url_args="sslmode=verify-ca&sslrootcert=.%2Ftests%2Fcerts%2Fca.crt",
                service=f"postgres_{version}",
                tag=f"postgres_{version}_ssl" if runtime == "async-std" else f"postgres_{version}_ssl_{runtime}",
            )

        #
        # mysql
        #

        for version in ["8", "5_7"]:
            run(
                f"cargo test --no-default-features --features macros,offline,any,all-types,mysql,runtime-{runtime}-{tls}",
                comment=f"test mysql {version}",
                service=f"mysql_{version}",
                tag=f"mysql_{version}" if runtime == "async-std" else f"mysql_{version}_{runtime}",
            )

        #
        # mariadb
        #

        for version in ["10_6", "10_5", "10_4", "10_3", "10_2"]:
            run(
                f"cargo test --no-default-features --features macros,offline,any,all-types,mysql,runtime-{runtime}-{tls}",
                comment=f"test mariadb {version}",
                service=f"mariadb_{version}",
                tag=f"mariadb_{version}" if runtime == "async-std" else f"mariadb_{version}_{runtime}",
            )

        #
        # mssql
        #

        for version in ["2019", "2017"]:
            run(
                f"cargo test --no-default-features --features macros,offline,any,all-types,mssql,runtime-{runtime}-{tls}",
                comment=f"test mssql {version}",
                service=f"mssql_{version}",
                tag=f"mssql_{version}" if runtime == "async-std" else f"mssql_{version}_{runtime}",
            )

# TODO: Use [grcov] if available
# ~/.cargo/bin/grcov tests/.cache/target/debug -s sqlx-core/ -t html --llvm --branch -o ./target/debug/coverage
