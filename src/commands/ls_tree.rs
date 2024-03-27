use std::{
    ffi::CStr,
    io::{BufRead, Read, Write},
};

use anyhow::Context;

use crate::objects::{self, Kind};

pub(crate) fn invoke(name_only: bool, tree_hash: &str) -> anyhow::Result<()> {
    let mut object = objects::Object::read_object(tree_hash).context("parse our object file")?;

    match object.kind {
        Kind::Tree => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let mut hash_buf = [0u8; 20];
            let mut buf = Vec::new();
            loop {
                buf.clear();
                let n = object
                    .reader
                    .read_until(b'\0', &mut buf)
                    .context("Read mode and name")?;
                if n == 0 {
                    break;
                }
                object
                    .reader
                    .read_exact(&mut hash_buf)
                    .context("Read hash")?;
                let mode_and_name =
                    CStr::from_bytes_with_nul(&buf).context("parse mode and name")?;
                let mut bits = mode_and_name.to_bytes().splitn(2, |&b| b == b' ');
                let mode = bits.next().context("parse mode")?;
                let name = bits.next().context("parse name")?;

                if name_only {
                    stdout.write_all(name).context("Write name to stdout")?;
                } else {
                    let hash = hex::encode(&hash_buf);
                    stdout.write_all(mode).context("Write mode to stdout")?;
                    let kind = "tree";
                    write!(stdout, " {kind} {hash} ").context("Write kind and hash to stdout")?;
                    stdout.write_all(name).context("Write name to stdout")?;
                }
                writeln!(stdout, "").context("Write newline to stdout")?;
            }
        }
        _ => anyhow::bail!("Unknown object type - '{}'", object.kind),
    }
    Ok(())
}
