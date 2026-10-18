use bytes::Bytes;
use http::header::CONTENT_DISPOSITION;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;

use crate::path::Path;
use crate::types::FileId;
use crate::{Client, Request};

#[derive(Deserialize)]
struct UploadResponse {
    id: FileId,
}

impl Client {
    /// Uploads a file and returns its ID (`file:…`).
    ///
    /// The ID can be used in place of a file URL in subsequent requests (e.g. as an
    /// item's `picture`). IDs are valid for 24 hours and only for the same credentials.
    pub fn upload(
        &self,
        filename: &str,
        content_type: &str,
        data: impl Into<Bytes>,
    ) -> Request<FileId> {
        self.post(&Path::root().join("upload").without_trailing_slash())
            .body(data.into(), content_type)
            .header(CONTENT_DISPOSITION, &content_disposition(filename))
            .decode_with(|raw| {
                let response: UploadResponse = raw.ensure_success()?.json()?;
                Ok(response.id)
            })
    }
}

fn content_disposition(filename: &str) -> String {
    let ascii: String = filename
        .chars()
        .map(|c| match c {
            '"' | '\\' => '_',
            c if c.is_ascii_graphic() || c == ' ' => c,
            _ => '_',
        })
        .collect();
    let mut value = format!("attachment; filename=\"{ascii}\"");
    if ascii != filename {
        value.push_str("; filename*=UTF-8''");
        value.extend(utf8_percent_encode(filename, NON_ALPHANUMERIC));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::content_disposition;

    #[test]
    fn encodes_filenames() {
        assert_eq!(
            content_disposition("logo.png"),
            "attachment; filename=\"logo.png\""
        );
        assert_eq!(
            content_disposition("lögo.png"),
            "attachment; filename=\"l_go.png\"; filename*=UTF-8''l%C3%B6go%2Epng"
        );
    }
}
