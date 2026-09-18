use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Parser;
use globset::Glob;
use sitediff::allow::AllowList;
use sitediff::compare::{self, Options, Status};

/// Compare a generated site tree against a reference snapshot.
#[derive(Parser)]
struct Args {
    /// Reference tree (e.g. archive/reference-site)
    reference: PathBuf,
    /// Candidate tree (e.g. _site)
    candidate: PathBuf,
    /// sha256 manifest of reference files not stored in the tree [default: <reference>.manifest if present]
    #[arg(long)]
    manifest: Option<PathBuf>,
    /// Do not use a manifest even if the default one exists
    #[arg(long)]
    no_manifest: bool,
    /// Allowlist file [default: sitediff.allow.toml if present]
    #[arg(long)]
    allow: Option<PathBuf>,
    /// Only compare paths matching this glob (e.g. 'rust/opengl/**')
    #[arg(long)]
    only: Option<String>,
    /// Print only the summary, no diff hunks
    #[arg(long, short)]
    quiet: bool,
    /// Print diff hunks for at most this many files (the rest are listed without hunks)
    #[arg(long, default_value_t = 5)]
    limit: usize,
    /// Also list files that are identical
    #[arg(long, short)]
    verbose: bool,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();

    let manifest = if args.no_manifest {
        None
    } else {
        args.manifest.or_else(|| {
            let mut default = args.reference.clone().into_os_string();
            default.push(".manifest");
            Some(PathBuf::from(default)).filter(|p| p.is_file())
        })
    };
    let allow = match args.allow.or_else(|| Some(PathBuf::from("sitediff.allow.toml")).filter(|p| p.is_file())) {
        Some(path) => AllowList::load(&path)?,
        None => AllowList::default(),
    };
    let only = match &args.only {
        Some(glob) => Some(Glob::new(glob).context("bad --only glob")?.compile_matcher()),
        None => None,
    };

    // With --only most rules legitimately match nothing; do not nag about them.
    let filtered = only.is_some();
    let report = compare::run(&Options { reference: args.reference, candidate: args.candidate, manifest, only, allow })?;

    let mut counts = std::collections::BTreeMap::<&str, usize>::new();
    let mut hunks_printed = 0;
    for (path, status) in &report.files {
        let label = match status {
            Status::Same => "same",
            Status::Allowed { .. } => "allowed",
            Status::Ignored => "ignored",
            Status::Different { .. } => "DIFFERENT",
            Status::HashMismatch => "HASH-MISMATCH",
            Status::Missing => "MISSING",
            Status::Extra => "EXTRA",
        };
        *counts.entry(label).or_default() += 1;
        match status {
            Status::Same | Status::Ignored if !args.verbose => {}
            Status::Allowed { lines } => println!("{label:<14} {path}  ({lines} changed lines, all allowlisted)"),
            Status::Different { hunks, lines, allowed_lines } => {
                println!("{label:<14} {path}  ({lines} changed lines, {allowed_lines} allowlisted)");
                if !args.quiet && hunks_printed < args.limit {
                    print!("{hunks}");
                    hunks_printed += 1;
                }
            }
            _ => println!("{label:<14} {path}"),
        }
    }

    println!();
    for rule in report.unused_rules.iter().filter(|_| !filtered) {
        println!("warning: unused allowlist rule {rule}");
    }
    let summary: Vec<String> = counts.iter().map(|(label, n)| format!("{n} {label}")).collect();
    println!("{} files: {}", report.files.len(), summary.join(", "));

    Ok(if report.failures() == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE })
}
