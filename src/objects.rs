use core::fmt;
use std::{
    ffi::CStr, fs, io::{BufRead, BufReader, Read}
};

use anyhow::Context;
use flate2::read::ZlibDecoder;

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
            Kind::Tag => write!(f, "tag")
        }
    }
}

pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) expected_size: u64,
    pub(crate) reader: R,
}

impl Object<()> {
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
