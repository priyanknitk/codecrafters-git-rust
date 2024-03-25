use anyhow::{Context, Ok};
use flate2::read::ZlibDecoder;
#[allow(unused_imports)]
use std::fs;
use std::{
    ffi::CStr,
    io::{BufRead, BufReader, Read},
};

use clap::{Parser, Subcommand};

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
}

enum Kind {
    Blob,
    Tree,
    Commit,
    Tag,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Init => init(),
        Command::CatFile {
            pretty_print,
            object_hash,
        } => cat_file(&object_hash, pretty_print),
    }
}

fn init() -> anyhow::Result<()> {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    Ok(())
}

fn cat_file(object_hash: &str, pretty_print: bool) -> anyhow::Result<()> {
    let f = fs::File::open(format!(
        ".git/objects/{}/{}",
        &object_hash[..2],
        &object_hash[2..]
    ))
    .context("open in .git/objects")?;

    //decompress the file
    let decoded_data_reader = ZlibDecoder::new(f);

    let mut decoded_data_reader = BufReader::new(decoded_data_reader);

    let mut buf: Vec<u8> = Vec::new();

    decoded_data_reader
        .read_until(b'\0', &mut buf)
        .context("Read head from object file")?;

    let header = CStr::from_bytes_until_nul(&buf).context("parse head from object file")?;

    let header = header.to_str().context("parse head from object file")?;

    let Some((kind, size)) = header.split_once(' ') else {
        anyhow::bail!("Invalid header - {header}");
    };

    let kind = match kind {
        "blob" => Kind::Blob,
        "tree" => Kind::Tree,
        "commit" => Kind::Commit,
        "tag" => Kind::Tag,
        _ => anyhow::bail!("Unknown object type - {kind}"),
    };

    let size = size.parse::<usize>().context("parse size from header")?;

    let mut decoded_data_reader = decoded_data_reader.take(size as u64);

    match kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut decoded_data_reader, &mut stdout).context("Write data to stdout")?;
            anyhow::ensure!(n == size as u64, "Expected {size} bytes of data in object file");
        }
        _ => anyhow::bail!("Unknown object type"),
    }
    Ok(())
}
