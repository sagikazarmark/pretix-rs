use bytes::Bytes;
use http::Method;
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::request::{RawResponse, RequestBuilder};
use crate::{Client, Request, Result};

/// A downloaded file (ticket PDF, invoice, export result, …).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Download {
    /// The `Content-Type` of the response.
    pub content_type: Option<String>,
    /// The file name from the `Content-Disposition` header.
    pub filename: Option<String>,
    /// The file content (base64 encoded when serialized).
    #[serde(with = "base64_bytes")]
    pub data: Bytes,
}

/// Serializes bytes as a base64 string, which is much more compact than serde's
/// default sequence of numbers.
mod base64_bytes {
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serializer, de};

    pub(super) fn serialize<S: Serializer>(data: &Bytes, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(data))
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Bytes, D::Error> {
        let encoded = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        STANDARD
            .decode(encoded.as_bytes())
            .map(Bytes::from)
            .map_err(de::Error::custom)
    }
}

impl Download {
    pub(crate) fn from_response(raw: RawResponse) -> Result<Self> {
        let raw = raw.ensure_success()?;
        Ok(Self {
            content_type: raw.header_str(CONTENT_TYPE).map(str::to_owned),
            filename: raw.header_str(CONTENT_DISPOSITION).and_then(parse_filename),
            data: raw.body,
        })
    }
}

impl RequestBuilder {
    /// Expects a binary file.
    pub fn download(self) -> Request<Download> {
        self.decode_with(Download::from_response)
    }
}

impl Client {
    /// Downloads a file from a link returned by the API (e.g. the `url` of an order's
    /// `downloads`).
    ///
    /// The link is resolved against the configured base URL's origin, so credentials
    /// are never sent to a different host.
    pub fn download(&self, url: &Url) -> Request<Download> {
        self.request_absolute(Method::GET, url).download()
    }
}

/// Extracts the file name from a `Content-Disposition` header value.
///
/// Prefers the RFC 5987 `filename*` parameter (UTF-8 only) over `filename`.
fn parse_filename(header: &str) -> Option<String> {
    let mut plain = None;
    let mut extended = None;
    for part in header.split(';').map(str::trim) {
        let Some((name, value)) = part.split_once('=') else {
            continue;
        };
        match name.trim().to_ascii_lowercase().as_str() {
            "filename*" => {
                // charset'language'percent-encoded-value
                let mut pieces = value.trim().splitn(3, '\'');
                let (Some(charset), Some(_), Some(encoded)) =
                    (pieces.next(), pieces.next(), pieces.next())
                else {
                    continue;
                };
                if charset.eq_ignore_ascii_case("utf-8") {
                    extended = percent_encoding::percent_decode_str(encoded)
                        .decode_utf8()
                        .ok()
                        .map(std::borrow::Cow::into_owned);
                }
            }
            "filename" => plain = Some(value.trim().trim_matches('"').to_owned()),
            _ => {}
        }
    }
    extended.or(plain)
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::{Download, parse_filename};

    #[test]
    fn serializes_data_as_base64() {
        let download = Download {
            content_type: Some("text/plain".into()),
            filename: None,
            data: Bytes::from_static(b"hello"),
        };
        let json = serde_json::to_value(&download).unwrap();
        assert_eq!(json["data"], "aGVsbG8=");
        assert_eq!(serde_json::from_value::<Download>(json).unwrap(), download);
    }

    #[test]
    fn parses_filenames() {
        assert_eq!(
            parse_filename(r#"attachment; filename="ABC12.pdf""#).as_deref(),
            Some("ABC12.pdf")
        );
        assert_eq!(
            parse_filename("attachment; filename=plain.zip").as_deref(),
            Some("plain.zip")
        );
        assert_eq!(
            parse_filename("attachment; filename=\"a.pdf\"; filename*=UTF-8''%C3%A4.pdf")
                .as_deref(),
            Some("ä.pdf")
        );
        assert_eq!(parse_filename("inline"), None);
        // `+` is not a space in percent-encoding.
        assert_eq!(
            parse_filename("attachment; FILENAME*=utf-8''a+b%20c.pdf").as_deref(),
            Some("a+b c.pdf")
        );
        // Unsupported charsets fall back to `filename`.
        assert_eq!(
            parse_filename("attachment; filename=\"x.pdf\"; filename*=iso-8859-1''%E4.pdf")
                .as_deref(),
            Some("x.pdf")
        );
    }
}
