use emlx::parse_emlx;
use eyre::Result;

use std::borrow::Cow;
use std::path::{Path, PathBuf};

use ps_core::EmailMeta;

use super::super::shared::parse::ParseableEmail;

pub struct Mail {
    path: PathBuf,
    // This is parsed out of the `emlx` as it is parsed
    is_seen: bool,
    // This is parsed out of the `path`
    label: Option<String>,
    // Maildata
    data: Vec<u8>,
}

impl Mail {
    pub fn new<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = path.as_ref();
        let name = path.file_name()?.to_str()?;
        if !name.ends_with(".emlx") {
            return None;
        }
        // find the folder ending with `.mbox` in the path
        let ext = ".mbox";
        let label = path
            .iter()
            .map(|e| e.to_str())
            .flatten()
            .find(|s| s.ends_with(ext))
            .map(|s| s.replace(ext, ""));
        Some(Self {
            path: path.to_path_buf(),
            is_seen: false,
            label,
            data: Vec::new(),
        })
    }
}

impl ParseableEmail for Mail {
    fn prepare(&mut self) -> Result<()> {
        let data = std::fs::read(self.path.as_path())?;
        let parsed = parse_emlx(&data)?;
        self.is_seen = !parsed.flags.is_read;
        self.data = parsed.message.to_vec();
        Ok(())
    }
    fn message(&self) -> Result<Cow<'_, [u8]>> {
        Ok(Cow::Borrowed(self.data.as_slice()))
    }
    fn path(&self) -> &Path {
        self.path.as_path()
    }
    fn meta(&self) -> Result<Option<EmailMeta>> {
        let tags = match self.label {
            Some(ref n) => vec![n.clone()],
            None => vec![],
        };
        let meta = EmailMeta {
            tags,
            is_seen: self.is_seen,
        };
        Ok(Some(meta))
    }
}
