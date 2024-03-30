use std::{fs, io::Cursor, path::Path};

use anyhow::Context;
use std::os::unix::fs::PermissionsExt;

use crate::objects::{Kind, Object};

pub(crate) fn invoke() -> anyhow::Result<()> {
    let tree_hash = write_tree_for(Path::new("."))?;
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
        let file_name = entry.file_name();
        if file_name == ".git" {
            continue;
        }
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
            let hash = Object::blob_from_file(&path)?
                .write_to_object()
                .context("write blob")?;
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
        let hash = Object {
            kind: Kind::Tree,
            expected_size: tree_object.len() as u64,
            reader: Cursor::new(tree_object),
        }
        .write_to_object()
        .context("write tree")?;
        Ok(Some(hash))
    }
}
