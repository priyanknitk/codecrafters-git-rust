use anyhow::Ok;
#[allow(unused_imports)]
use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub(crate) mod commands;
pub(crate) mod objects;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,

        tree_hash: String,
    },
    WriteTree,
    CommitTree {
        #[clap(short = 'm')]
        message: String,

        #[clap(short = 'p')]
        parent_commit: Option<String>,

        tree_hash: String,
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => init(),
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, &object_hash),
        Command::HashObject { write, file } => commands::hash_object::invoke(&file, write),
        Command::LsTree { name_only, tree_hash } => commands::ls_tree::invoke(name_only, &tree_hash),
        Command::WriteTree => commands::write_tree::invoke(),
        Command::CommitTree {
            message,
            mut parent_commit,
            tree_hash,
        } => commands::commit_tree::invoke(&message, parent_commit.take(), &tree_hash),
    }
}

fn init() -> anyhow::Result<()> {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    Ok(())
}
