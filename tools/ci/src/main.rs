//! Runs a set of checks on the codebase, allowing for easy local testing of CI runs.
//!
//! Heavily modified from [Bevy's CI runner](https://github.com/bevyengine/bevy/tree/main/tools/ci/src)
//! When run locally, results may differ from actual CI runs triggered by
//! .github/workflows/ci.yml
//! - Official CI runs latest stable
//! - Local runs use whatever the default Rust is locally

use std::process;

use bevy::utils::HashSet;
use xshell::{cmd, Shell};

/// The checks that can be run in CI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Check {
    Format,
    Clippy,
    Test,
    DocTest,
    DocCheck,
    CompileCheck,
}

impl Check {
    /// Returns the complete set of checks.
    fn all() -> HashSet<Check> {
        [
            Check::Format,
            Check::Clippy,
            Check::Test,
            Check::DocTest,
            Check::DocCheck,
            Check::CompileCheck,
        ]
        .iter()
        .copied()
        .collect()
    }

    /// Returns the argument that corresponds to this check.
    fn argument(&self) -> &'static str {
        match self {
            Check::Format => "format",
            Check::Clippy => "clippy",
            Check::Test => "test",
            Check::DocTest => "doctest",
            Check::DocCheck => "doccheck",
            Check::CompileCheck => "compilecheck",
        }
    }

    /// Returns the [`Check`] that corresponds to the given argument.
    fn from_argument(argument: &str) -> Option<Check> {
        match argument {
            "format" => Some(Check::Format),
            "clippy" => Some(Check::Clippy),
            "test" => Some(Check::Test),
            "doctest" => Some(Check::DocTest),
            "doccheck" => Some(Check::DocCheck),
            "compilecheck" => Some(Check::CompileCheck),
            _ => None,
        }
    }
}

/// Controls how clippy is run.
const CLIPPY_FLAGS: [&str; 3] = [
    "-Aclippy::type_complexity",
    "-Wclippy::doc_markdown",
    "-Dwarnings",
];

fn main() {
    let what_to_run = if let Some(arg) = std::env::args().nth(1).as_deref() {
        if let Some(check) = Check::from_argument(arg) {
            let mut set = HashSet::default();
            set.insert(check);
            set
        } else {
            println!(
                "Invalid argument: {arg}.\nEnter one of: {}.",
                Check::all()
                    .iter()
                    .map(|check| check.argument())
                    .collect::<Vec<&str>>()
                    .join(", "),
            );
            process::exit(1);
        }
    } else {
        Check::all()
    };

    let sh = Shell::new().unwrap();

    if what_to_run.contains(&Check::Format) {
        // See if any code needs to be formatted
        cmd!(sh, "cargo fmt --all -- --check")
            .run()
            .expect("Please run 'cargo fmt --all' to format your code.");
    }

    if what_to_run.contains(&Check::Clippy) {
        // See if clippy has any complaints.
        // --all-targets --all-features was removed because Emergence currently has no special
        // targets or features; please add them back as necessary
        cmd!(sh, "cargo clippy --workspace -- {CLIPPY_FLAGS...}")
            .run()
            .expect("Please fix clippy errors in output above.");
    }

    if what_to_run.contains(&Check::Test) {
        // Run tests (except doc tests and without building examples)
        cmd!(sh, "cargo test --workspace --lib --bins --tests --benches")
            .run()
            .expect("Please fix failing tests in output above.");
    }

    if what_to_run.contains(&Check::DocTest) {
        // Run doc tests
        cmd!(sh, "cargo test --workspace --doc")
            .run()
            .expect("Please fix failing doc-tests in output above.");
    }

    if what_to_run.contains(&Check::DocCheck) {
        // Check that building docs work and does not emit warnings
        std::env::set_var("RUSTDOCFLAGS", "-D warnings");
        cmd!(
            sh,
            "cargo doc --workspace --all-features --no-deps --document-private-items"
        )
        .run()
        .expect("Please fix doc warnings in output above.");
    }

    if what_to_run.contains(&Check::CompileCheck) {
        cmd!(sh, "cargo check --workspace")
            .run()
            .expect("Please fix compiler errors in above output.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_from_argument_reverses() {
        for check in Check::all() {
            assert_eq!(Check::from_argument(check.argument()), Some(check));
        }
        assert_eq!(Check::from_argument("invalid"), None);
    }
}
