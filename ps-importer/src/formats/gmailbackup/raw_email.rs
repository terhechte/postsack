use flate2::read::GzDecoder;
use ps_core::eyre::{eyre, Result};
use ps_core::tracing;

use std::borrow::Cow;
use std::io::Read;
use std::path::{Path, PathBuf};

use super::super::shared::parse::ParseableEmail;
use ps_core::EmailMeta;

/// Raw representation of an email.
/// Contains the paths to the relevant files as well
/// as the name of the folder the email was in.
#[derive(Debug)]
pub struct RawEmailEntry {
    #[allow(unused)]
    folder_name: String,
    eml_path: PathBuf,
    gmail_meta_path: Option<PathBuf>,
    is_compressed: bool,
    #[allow(unused)]
    size: u64,
}

impl RawEmailEntry {
    pub fn path(&self) -> &Path {
        self.eml_path.as_path()
    }

    pub fn read(&self) -> Result<Vec<u8>> {
        if self.is_compressed {
            let reader = std::fs::File::open(&self.eml_path)?;
            let mut decoder = GzDecoder::new(reader);
            let mut buffer = Vec::new();
            decoder.read_to_end(&mut buffer)?;
            Ok(buffer)
        } else {
            std::fs::read(&self.eml_path).map_err(|e| eyre!("IO Error: {}", &e))
        }
    }

    pub fn has_gmail_meta(&self) -> bool {
        self.gmail_meta_path.is_some()
    }

    pub fn read_gmail_meta(&self) -> Option<Result<Vec<u8>>> {
        // Just using map here returns a `&Option` whereas we want `Option`
        #[allow(clippy::manual_map)]
        match &self.gmail_meta_path {
            Some(p) => Some(std::fs::read(p).map_err(|e| eyre!("IO Error: {}", &e))),
            None => None,
        }
    }
}

impl RawEmailEntry {
    pub(super) fn new<P: AsRef<std::path::Path>>(path: P) -> Option<RawEmailEntry> {
        let path = path.as_ref();
        let stem = path.file_stem()?.to_str()?;
        let name = path.file_name()?.to_str()?;
        let is_eml_gz = name.ends_with(".eml.gz");
        let is_eml = name.ends_with(".eml");
        if !is_eml_gz && !is_eml {
            return None;
        }
        let is_compressed = is_eml_gz;
        let folder_name = path.parent()?.file_name()?.to_str()?.to_owned();
        let eml_path = path.to_path_buf();

        let file_metadata = path.metadata().ok()?;

        // Build a meta path
        let meta_path = path
            .parent()?
            .join(format!("{}.meta", stem.replace(".eml", "")));

        // Only embed it, if it exists
        let gmail_meta_path = if meta_path.exists() {
            Some(meta_path)
        } else {
            None
        };
        tracing::trace!(
            "Email [c?: {}] {} {:?}",
            is_compressed,
            eml_path.display(),
            gmail_meta_path
        );
        Some(RawEmailEntry {
            folder_name,
            eml_path,
            gmail_meta_path,
            is_compressed,
            size: file_metadata.len(),
        })
    }
}

impl ParseableEmail for RawEmailEntry {
    fn prepare(&mut self) -> Result<()> {
        Ok(())
    }
    fn message(&self) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::Owned(self.read()?))
    }

    fn path(&self) -> &Path {
        self.eml_path.as_path()
    }

    fn meta(&self) -> Result<Option<EmailMeta>> {
        if self.has_gmail_meta() {
            Ok(Some(super::meta::parse_meta(self)?.into()))
        } else {
            Ok(None)
        }
    }
}
