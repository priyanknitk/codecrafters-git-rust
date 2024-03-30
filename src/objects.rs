use core::fmt;
use std::{
    ffi::CStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
};

use anyhow::Context;
use flate2::{read::ZlibDecoder, write::ZlibEncoder};
use sha1::Digest;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
            Kind::Tag => write!(f, "tag"),
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
    pub(crate) fn blob_from_file(file: impl AsRef<Path>) -> anyhow::Result<Object<impl Read>> {
        let file = file.as_ref();
        let stat = fs::metadata(file).context("stat file")?;
        let file = fs::File::open(file).context("open file")?;
        Ok(Object {
            kind: Kind::Blob,
            expected_size: stat.len(),
            reader: file,
        })
    }

    pub(crate) fn read_object(object_hash: &str) -> anyhow::Result<Object<impl BufRead>> {
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

        let decoded_data_reader = decoded_data_reader.take(size as u64);

        Ok(Object {
            kind,
            expected_size: size as u64,
            reader: decoded_data_reader,
        })
    }
}

impl<R> Object<R>
where
    R: Read,
{
    pub(crate) fn write(mut self, writer: impl Write) -> anyhow::Result<[u8; 20]> {
        let writer = ZlibEncoder::new(writer, flate2::Compression::default());
        let mut writer = HashWriter {
            hasher: sha1::Sha1::new(),
            writer,
        };
        write!(writer, "{} {}\0", self.kind, self.expected_size).context("write header")?;
        std::io::copy(&mut self.reader, &mut writer).context("copy file to writer")?;
        let _ = writer.writer.finish().context("finish writing")?;
        let hash = writer.hasher.finalize();
        Ok(hash.into())
    }

    pub(crate) fn write_to_object(self) -> anyhow::Result<[u8; 20]> {
        let tmp = "tmp";
        let hash = self
            .write(fs::File::create(tmp).context("create temp file")?)
            .context("write object")?;
        let hash_hex = hex::encode(hash);
        fs::create_dir_all(format!(".git/objects/{}", &hash_hex[..2]))
            .context("create object dir")?;
        fs::rename(
            tmp,
            format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
        )
        .context("rename object file")?;
        Ok(hash)
    }
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
