use std::fmt::Display;

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use url::Url;

/// A relative API path, built from segments that are percent-encoded individually.
///
/// pretix requires trailing slashes on almost every endpoint, so they are added by
/// default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Path {
    segments: Vec<String>,
    trailing_slash: bool,
}

impl Path {
    pub(crate) fn root() -> Self {
        Self {
            segments: Vec::new(),
            trailing_slash: true,
        }
    }

    /// Returns a new path with `segment` appended. `/` inside a segment is escaped.
    pub(crate) fn join(&self, segment: impl Display) -> Self {
        let mut path = self.clone();
        path.segments.push(segment.to_string());
        path
    }

    pub(crate) fn without_trailing_slash(mut self) -> Self {
        self.trailing_slash = false;
        self
    }

    pub(crate) fn to_url(&self, base: &Url) -> Result<Url, String> {
        let mut path = base.path().trim_end_matches('/').to_owned();
        for segment in &self.segments {
            // URL parsers normalize `.` and `..` (even percent-encoded) away, which
            // would silently address a different resource.
            if matches!(segment.as_str(), "." | ".." | "") {
                return Err(format!("invalid path segment `{segment}`"));
            }
            path.push('/');
            path.extend(utf8_percent_encode(segment, SEGMENT));
        }
        if self.trailing_slash {
            path.push('/');
        }
        let mut url = base.clone();
        url.set_path(&path);
        Ok(url)
    }
}

/// Characters escaped inside a path segment (including `/` and `%`).
const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'/')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Moves an absolute URL returned by the API (pagination links, download links) onto
/// the configured base URL's origin.
///
/// This guarantees credentials are never sent to a different host, and keeps working
/// behind reverse proxies that report an internal origin.
pub(crate) fn rebase(base: &Url, url: &Url) -> Url {
    const API_ROOT: &str = "/api/v1/";
    let mut rebased = base.clone();
    let path = url.path();
    // A proxy may report a different path prefix (e.g. `/api/v1/…` for a base URL of
    // `/pretix/api/v1/`); keep the configured prefix in that case.
    match (base.path().strip_suffix(API_ROOT), path.find(API_ROOT)) {
        (Some(prefix), Some(index)) if !path.starts_with(base.path()) => {
            rebased.set_path(&format!("{prefix}{}", &path[index..]));
        }
        _ => rebased.set_path(path),
    }
    rebased.set_query(url.query());
    rebased.set_fragment(None);
    rebased
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Url {
        Url::parse("https://pretix.example/api/v1/").unwrap()
    }

    #[test]
    fn joins_with_trailing_slash() {
        let path = Path::root().join("organizers").join("big events").join(42);
        assert_eq!(
            path.to_url(&base()).unwrap().as_str(),
            "https://pretix.example/api/v1/organizers/big%20events/42/"
        );
    }

    #[test]
    fn escapes_special_characters() {
        let url = Path::root()
            .join("a/b")
            .join("%")
            .join("x?y#z")
            .to_url(&base())
            .unwrap();
        assert_eq!(
            url.as_str(),
            "https://pretix.example/api/v1/a%2Fb/%25/x%3Fy%23z/"
        );
    }

    #[test]
    fn rejects_dot_segments() {
        for segment in [".", "..", ""] {
            assert!(Path::root().join(segment).to_url(&base()).is_err());
        }
    }

    #[test]
    fn without_trailing_slash() {
        let url = Path::root()
            .join("upload")
            .without_trailing_slash()
            .to_url(&base())
            .unwrap();
        assert_eq!(url.as_str(), "https://pretix.example/api/v1/upload");
    }

    #[test]
    fn rebases_keeping_path_prefix() {
        let base = Url::parse("https://h.example/pretix/api/v1/").unwrap();
        let next = Url::parse("http://internal/api/v1/organizers/?page=2").unwrap();
        assert_eq!(
            rebase(&base, &next).as_str(),
            "https://h.example/pretix/api/v1/organizers/?page=2"
        );
        let same = Url::parse("http://internal/pretix/api/v1/organizers/").unwrap();
        assert_eq!(
            rebase(&base, &same).as_str(),
            "https://h.example/pretix/api/v1/organizers/"
        );
    }

    #[test]
    fn rebases_foreign_origin() {
        let next = Url::parse("http://internal:8000/api/v1/organizers/?page=2").unwrap();
        assert_eq!(
            rebase(&base(), &next).as_str(),
            "https://pretix.example/api/v1/organizers/?page=2"
        );
    }
}
