use std::{
    fs,
    path::Path,
};

use anyhow::Context;
use std::os::unix::fs::PermissionsExt;

use crate::objects::{Kind, Object};

pub(crate) fn invoke() -> anyhow::Result<()> {
    let tree_hash = write_tree_for(Path::new("./"))?;
    let hash = tree_hash.context("write tree")?;
    println!("{}", hex::encode(hash));
    Ok(())
}

fn write_tree_for(path: &Path) -> anyhow::Result<Option<[u8; 20]>> {
    let mut dir =
        fs::read_dir(path).with_context(|| format!("open directory {}", path.display()))?;
    let mut tree_object = Vec::new();
    while let Some(entry) = dir.next() {
        let entry = entry.with_context(|| format!("read directory {}", path.display()))?;
        let file_name = path.file_name().context("get file name")?;
        let meta = entry.metadata().context("metadata for directory entry")?;
        let mode = if meta.is_dir() {
            "40000"
        } else if meta.is_symlink() {
            "120000"
        } else if meta.permissions().mode() & 0o111 != 0 {
            // has at least one executable bit set
            "100755"
        } else {
            "100644"
        };
        let path = entry.path();
        let hash = if meta.is_dir() {
            let Some(hash) = write_tree_for(&path)? else {
                continue;
            };
            Some(hash)
        } else {
            let tmp = "tmp";
            let hash = Object::blob_from_file(&path)?
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
            Some(hash)
        };
        tree_object.extend(mode.as_bytes());
        tree_object.push(b' ');
        tree_object.extend(file_name.as_encoded_bytes());
        tree_object.push(0);
        tree_object.extend(&hash.unwrap());
    }

    if tree_object.is_empty() {
        Ok(None)
    } else {
        let tmp = "tmp";
        let hash = Object {
            kind: Kind::Tree,
            expected_size: tree_object.len() as u64,
            reader: tree_object.as_slice(),
        }
        .write(fs::File::create(tmp).context("create temp file")?)
        .context("write object")?;

        let hash_hex = hex::encode(hash);
        fs::create_dir_all(format!(".git/objects/{}", &hash_hex[..2])).context("create object dir")?;
        fs::rename(tmp, format!(".git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]))
            .context("rename object file")?;
        Ok(Some(hash))
    }
}
