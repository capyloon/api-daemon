// Copyright (c) 2016, 2018 vergen developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! Output types
use crate::constants::{
    ConstantsFlags, BRANCH_COMMENT, BRANCH_NAME, BUILD_DATE_COMMENT, BUILD_DATE_NAME,
    BUILD_TIMESTAMP_COMMENT, BUILD_TIMESTAMP_NAME, COMMIT_DATE_COMMENT, COMMIT_DATE_NAME,
    HOST_TRIPLE_COMMENT, HOST_TRIPLE_NAME, RUSTC_CHANNEL_COMMENT, RUSTC_CHANNEL_NAME,
    RUSTC_SEMVER_COMMENT, RUSTC_SEMVER_NAME, SEMVER_COMMENT, SEMVER_NAME, SEMVER_TAGS_COMMENT,
    SEMVER_TAGS_NAME, SHA_COMMENT, SHA_NAME, SHA_SHORT_COMMENT, SHA_SHORT_NAME,
    TARGET_TRIPLE_COMMENT, TARGET_TRIPLE_NAME,
};
use chrono::Utc;
use rustc_version::Channel;
use std::collections::HashMap;
use std::env;
use std::process::Command;

pub(crate) mod codegen;
pub(crate) mod envvar;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub(crate) fn generate_build_info(flags: ConstantsFlags) -> Result<HashMap<VergenKey, String>> {
    let mut build_info = HashMap::new();
    let now = Utc::now();

    if flags.contains(ConstantsFlags::BUILD_TIMESTAMP) {
        let _ = build_info.insert(VergenKey::BuildTimestamp, now.to_rfc3339());
    }

    if flags.contains(ConstantsFlags::BUILD_DATE) {
        let _ = build_info.insert(VergenKey::BuildDate, now.format("%Y-%m-%d").to_string());
    }

    if flags.contains(ConstantsFlags::SHA) {
        let sha = run_command(Command::new("git").args(&["rev-parse", "HEAD"]));
        let _ = build_info.insert(VergenKey::Sha, sha);
    }

    if flags.contains(ConstantsFlags::SHA_SHORT) {
        let sha = run_command(Command::new("git").args(&["rev-parse", "--short", "HEAD"]));
        let _ = build_info.insert(VergenKey::ShortSha, sha);
    }

    if flags.contains(ConstantsFlags::COMMIT_DATE) {
        let commit_date = run_command(Command::new("git").args(&[
            "log",
            "--pretty=format:'%ad'",
            "-n1",
            "--date=short",
        ]));
        let _ = build_info.insert(
            VergenKey::CommitDate,
            commit_date.trim_matches('\'').to_string(),
        );
    }

    if flags.contains(ConstantsFlags::TARGET_TRIPLE) {
        let target_triple = env::var("TARGET").unwrap_or_else(|_| "UNKNOWN".to_string());
        let _ = build_info.insert(VergenKey::TargetTriple, target_triple);
    }

    if flags.contains(ConstantsFlags::SEMVER) {
        let describe = run_command(Command::new("git").args(&["describe"]));

        let semver = if describe.is_empty() {
            env::var("CARGO_PKG_VERSION")?
        } else {
            describe
        };
        let _ = build_info.insert(VergenKey::Semver, semver);
    } else if flags.contains(ConstantsFlags::SEMVER_FROM_CARGO_PKG) {
        let _ = build_info.insert(VergenKey::Semver, env::var("CARGO_PKG_VERSION")?);
    }

    if flags.contains(ConstantsFlags::SEMVER_LIGHTWEIGHT) {
        let describe = run_command(Command::new("git").args(&["describe", "--tags"]));

        let semver = if describe.is_empty() {
            env::var("CARGO_PKG_VERSION")?
        } else {
            describe
        };
        let _ = build_info.insert(VergenKey::SemverLightweight, semver);
    }

    if flags.intersects(
        ConstantsFlags::RUSTC_SEMVER | ConstantsFlags::RUSTC_CHANNEL | ConstantsFlags::HOST_TRIPLE,
    ) {
        let rustc = rustc_version::version_meta()?;

        if flags.contains(ConstantsFlags::RUSTC_SEMVER) {
            let _ = build_info.insert(VergenKey::RustcSemver, format!("{}", rustc.semver));
        }

        if flags.contains(ConstantsFlags::RUSTC_CHANNEL) {
            let channel = match rustc.channel {
                Channel::Dev => "dev",
                Channel::Nightly => "nightly",
                Channel::Beta => "beta",
                Channel::Stable => "stable",
            }
            .to_string();

            let _ = build_info.insert(VergenKey::RustcChannel, channel);
        }

        if flags.contains(ConstantsFlags::HOST_TRIPLE) {
            let _ = build_info.insert(VergenKey::HostTriple, rustc.host);
        }
    }

    if flags.contains(ConstantsFlags::BRANCH) {
        let branch = run_command(Command::new("git").args(&["rev-parse", "--abbrev-ref", "HEAD"]));
        let _ = build_info.insert(VergenKey::Branch, branch);
    }

    Ok(build_info)
}

fn run_command(command: &mut Command) -> String {
    if let Ok(o) = command.output() {
        if o.status.success() {
            return String::from_utf8_lossy(&o.stdout).trim().to_owned();
        }
    }
    "UNKNOWN".to_owned()
}

/// Build information keys.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub(crate) enum VergenKey {
    /// The build timestamp. (VERGEN_BUILD_TIMESTAMP)
    BuildTimestamp,
    /// The build date. (VERGEN_BUILD_DATE)
    BuildDate,
    /// The latest commit SHA. (VERGEN_SHA)
    Sha,
    /// The latest commit short SHA. (VERGEN_SHA_SHORT)
    ShortSha,
    /// The commit date. (VERGEN_COMMIT_DATE).
    CommitDate,
    /// The target triple. (VERGEN_TARGET_TRIPLE)
    TargetTriple,
    /// The semver version from the last git tag. (VERGEN_SEMVER)
    Semver,
    /// The semver version from the last git tag, including lightweight.
    /// (VERGEN_SEMVER_LIGHTWEIGHT)
    SemverLightweight,
    /// The version information of the rust compiler. (VERGEN_RUSTC_SEMVER)
    RustcSemver,
    /// The release channel of the rust compiler. (VERGEN_RUSTC_CHANNEL)
    RustcChannel,
    /// The host triple. (VERGEN_HOST_TRIPLE)
    HostTriple,
    /// The current working branch name (VERGEN_BRANCH)
    Branch,
}

impl VergenKey {
    /// Get the comment string for the given key.
    pub(crate) fn comment(self) -> &'static str {
        match self {
            VergenKey::BuildTimestamp => BUILD_TIMESTAMP_COMMENT,
            VergenKey::BuildDate => BUILD_DATE_COMMENT,
            VergenKey::Sha => SHA_COMMENT,
            VergenKey::ShortSha => SHA_SHORT_COMMENT,
            VergenKey::CommitDate => COMMIT_DATE_COMMENT,
            VergenKey::TargetTriple => TARGET_TRIPLE_COMMENT,
            VergenKey::Semver => SEMVER_COMMENT,
            VergenKey::SemverLightweight => SEMVER_TAGS_COMMENT,
            VergenKey::RustcSemver => RUSTC_SEMVER_COMMENT,
            VergenKey::RustcChannel => RUSTC_CHANNEL_COMMENT,
            VergenKey::HostTriple => HOST_TRIPLE_COMMENT,
            VergenKey::Branch => BRANCH_COMMENT,
        }
    }

    /// Get the name for the given key.
    pub(crate) fn name(self) -> &'static str {
        match self {
            VergenKey::BuildTimestamp => BUILD_TIMESTAMP_NAME,
            VergenKey::BuildDate => BUILD_DATE_NAME,
            VergenKey::Sha => SHA_NAME,
            VergenKey::ShortSha => SHA_SHORT_NAME,
            VergenKey::CommitDate => COMMIT_DATE_NAME,
            VergenKey::TargetTriple => TARGET_TRIPLE_NAME,
            VergenKey::Semver => SEMVER_NAME,
            VergenKey::SemverLightweight => SEMVER_TAGS_NAME,
            VergenKey::RustcSemver => RUSTC_SEMVER_NAME,
            VergenKey::RustcChannel => RUSTC_CHANNEL_NAME,
            VergenKey::HostTriple => HOST_TRIPLE_NAME,
            VergenKey::Branch => BRANCH_NAME,
        }
    }
}

#[cfg(test)]
mod test {}
