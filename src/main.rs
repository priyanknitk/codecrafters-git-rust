use anyhow::{Context, Ok};
use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use sha1::Digest;
#[allow(unused_imports)]
use std::fs;
use std::{
    ffi::CStr,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
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
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: PathBuf,
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
        Command::HashObject { write, file } => hash_object(&file, write),
    }
}

fn init() -> anyhow::Result<()> {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    Ok(())
}

fn cat_file(object_hash: &str, _pretty_print: bool) -> anyhow::Result<()> {
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
            let n = std::io::copy(&mut decoded_data_reader, &mut stdout)
                .context("Write data to stdout")?;
            anyhow::ensure!(
                n == size as u64,
                "Expected {size} bytes of data in object file"
            );
        }
        _ => anyhow::bail!("Unknown object type"),
    }
    Ok(())
}

fn hash_object(file_path: &PathBuf, write: bool) -> anyhow::Result<()> {
    fn write_blob<W>(file: &Path, writer: W) -> anyhow::Result<String>
    where
        W: Write,
    {
        let stat = fs::metadata(file).context("stat file")?;
        let writer = ZlibEncoder::new(writer, flate2::Compression::default());
        let mut writer = HashWriter {
            hasher: sha1::Sha1::new(),
            writer,
        };
        write!(writer, "blob {}\0", stat.len()).context("write header")?;
        let mut file = fs::File::open(file).context("open file")?;
        std::io::copy(&mut file, &mut writer).context("copy file to writer")?;
        let _ = writer.writer.finish().context("finish writing")?;
        let hash = writer.hasher.finalize();
        Ok(hex::encode(hash))
    }

    let hash = if write {
        let temp = "temporary";
        let hash = write_blob(
            &file_path,
            fs::File::create(temp).context("create object file")?,
        );
        let hash = hash?;

        fs::create_dir_all(format!(".git/objects/{}", &hash[..2])).context("create object dir")?;
        fs::rename(temp, format!(".git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("rename object file")?;
        hash
    } else {
        let hash = write_blob(&file_path, std::io::sink())?;
        hash
    };
    println!("{hash}");
    Ok(())
}

struct HashWriter<W> {
    hasher: sha1::Sha1,
    writer: W,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(&buf)?;
        self.hasher.update(&buf[..n]);
        std::io::Result::Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
