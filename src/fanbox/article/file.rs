use super::super::check::CheckUnknown;
use super::super::error::FanboxAPIError;
use super::super::post::FanboxFile;
use crate::fanbox_api::FanboxClientInternal;
use json::JsonValue;
use std::fmt::Debug;
use std::sync::Arc;

pub struct FanboxArticleFileMap {
    pub data: JsonValue,
    client: Arc<FanboxClientInternal>,
}

impl FanboxArticleFileMap {
    #[inline]
    /// Create a new instance
    pub fn new(data: &JsonValue, client: Arc<FanboxClientInternal>) -> Self {
        Self {
            data: data.clone(),
            client,
        }
    }

    pub fn get_file<S: AsRef<str> + ?Sized>(&self, id: &S) -> Option<FanboxFile> {
        let id = id.as_ref();
        let file = &self.data[id];
        if file.is_object() {
            Some(FanboxFile::new(file, Arc::clone(&self.client)))
        } else {
            None
        }
    }
}

impl CheckUnknown for FanboxArticleFileMap {
    fn check_unknown(&self) -> Result<(), FanboxAPIError> {
        for (key, _) in self.data.entries() {
            match self.get_file(key) {
                Some(i) => {
                    i.check_unknown()?;
                }
                None => {}
            }
        }
        Ok(())
    }
}

impl Debug for FanboxArticleFileMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("FanboxArticleFileMap");
        for (key, _) in self.data.entries() {
            s.field(key, &self.get_file(key));
        }
        s.finish_non_exhaustive()
    }
}
