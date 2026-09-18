mod build;
mod config;
mod filters;
mod front_matter;
mod highlight;
mod markdown;
mod site;
mod templates;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Static site generator for this blog.
#[derive(Parser)]
struct Args {
    /// Repository root (where site.toml lives)
    #[arg(long, default_value = ".", global = true)]
    root: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate the site
    Build {
        /// Output directory, relative to the root
        #[arg(long, default_value = "_site")]
        out: PathBuf,
        /// Also render `_drafts`
        #[arg(long)]
        drafts: bool,
    },
    /// List every output URL and its source
    Urls {
        #[arg(long)]
        drafts: bool,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Build { out, drafts } => {
            let site = site::Site::load(&args.root, drafts)?;
            let summary = build::build(&site, &args.root.join(&out))?;
            println!("built {} documents, copied {} static files into {}", summary.documents, summary.statics, out.display());
        }
        Command::Urls { drafts } => {
            let site = site::Site::load(&args.root, drafts)?;
            for document in site.posts.iter().chain(&site.pages) {
                println!("{:<6} {:<90} {}", format!("{:?}", document.kind).to_lowercase(), document.url, document.source);
            }
        }
    }
    Ok(())
}
