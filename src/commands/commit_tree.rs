use std::io::Cursor;

use anyhow::Context;

use crate::objects::{Kind, Object};

pub(crate) fn invoke(message: &str, parent_commit: Option<String>, tree_hash: &str) -> anyhow::Result<()> {
    let mut commit_object: Vec<u8> = Vec::new();
    commit_object.extend("tree ".as_bytes());
    commit_object.extend(tree_hash.as_bytes());
    commit_object.push(b'\n');
    if let Some(parent_commit) = parent_commit {
        commit_object.extend("parent ".as_bytes());
        commit_object.extend(parent_commit.as_bytes());
        commit_object.push(b'\n');
    }
    commit_object.extend("author ".as_bytes());
    commit_object.extend("Priyank ".as_bytes());
    commit_object.extend("<priyank@localhost.com> ".as_bytes());
    commit_object.extend("1620000000 +0000\n".as_bytes());
    commit_object.extend("committer ".as_bytes());
    commit_object.extend("Priyank ".as_bytes());
    commit_object.extend("<priyank@localhost.com> ".as_bytes());
    commit_object.extend("1620000000 +0000\n".as_bytes());
    commit_object.push(b'\n');
    commit_object.extend(message.as_bytes());
    commit_object.push(b'\n');

    let commit_object = Object {
        kind: Kind::Commit,
        expected_size: commit_object.len() as u64,
        reader: Cursor::new(commit_object),
    };
    let hash = commit_object.write_to_object().context("write commit")?;
    println!("{}", hex::encode(hash));
    Ok(())
}