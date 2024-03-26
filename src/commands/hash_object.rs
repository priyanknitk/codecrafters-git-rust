use anyhow::{Context, Ok};
use flate2::write::ZlibEncoder;
use sha1::Digest;
#[allow(unused_imports)]
use std::fs;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

pub(crate) fn invoke(file_path: &PathBuf, write: bool) -> anyhow::Result<()> {
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