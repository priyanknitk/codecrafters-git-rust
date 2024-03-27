use anyhow::Context;

use crate::objects;

pub(crate) fn invoke(name_only: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(name_only, "Only name-only mode is supported for now");
    let mut object = objects::Object::read_object(object_hash).context("parse our object file")?;

    Ok(())
}