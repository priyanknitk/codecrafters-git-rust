use anyhow::{Context, Ok};
#[allow(unused_imports)]
use std::fs;
use std::path::PathBuf;

use crate::objects::Object;

pub(crate) fn invoke(file_path: &PathBuf, write: bool) -> anyhow::Result<()> {
    let object = Object::blob_from_file(file_path).context("create blob object")?;
    let hash = if write {
        object.write_to_object().context("write to object file")?
    } else {
        object.write(std::io::sink()).context("write object")?
    };
    let hash = hex::encode(hash);
    println!("{hash}");
    Ok(())
}
