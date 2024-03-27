use anyhow::Context;

use crate::objects::{Object, Kind};

pub(crate) fn invoke(_pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    let mut object = Object::read_object(object_hash).context("parse our object file")?;
    
    match object.kind {
        Kind::Blob => {
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            let n = std::io::copy(&mut object.reader, &mut stdout)
                .context("Write data to stdout")?;
            anyhow::ensure!(
                n == object.expected_size,
                "Expected {} bytes of data in object file",
                object.expected_size
            );
        }
        _ => anyhow::bail!("Unknown object type - '{}'", object.kind),
    }
    Ok(())
}