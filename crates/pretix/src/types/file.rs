use std::fmt;

use serde::{Deserialize, Serialize};

/// The ID of an uploaded file (`file:<uuid>`), returned by
/// [`Client::upload`](crate::Client::upload).
///
/// Pass it in place of a file URL when creating or updating resources.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FileId(String);

impl FileId {
    /// Wraps an existing file ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The ID as a string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<FileId> for String {
    fn from(id: FileId) -> Self {
        id.0
    }
}
