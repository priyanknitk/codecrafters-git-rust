use anyhow::{Context, Ok};
#[allow(unused_imports)]
use std::fs;
use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crate::objects::Object;

pub(crate) fn invoke(file_path: &PathBuf, write: bool) -> anyhow::Result<()> {
    fn write_blob<W>(file_path: &Path, writer: W) -> anyhow::Result<String>
    where
        W: Write,
    {
        let mut hash = Object::blob_from_file(file_path)
            .context("create object")?
            .write(writer)
            .context("write object")?;
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