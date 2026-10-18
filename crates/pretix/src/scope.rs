use crate::Client;
use crate::path::Path;

/// Resources belonging to one organizer account.
///
/// Created with [`Client::organizer`]. Resource handles such as `events()`,
/// `customers()` or `orders()` are available as methods.
#[derive(Debug, Clone)]
pub struct OrganizerScope {
    client: Client,
    slug: String,
    path: Path,
}

impl Client {
    /// Scopes requests to the organizer with the given slug.
    pub fn organizer(&self, slug: impl Into<String>) -> OrganizerScope {
        let slug = slug.into();
        let path = Path::root().join("organizers").join(&slug);
        OrganizerScope {
            client: self.clone(),
            slug,
            path,
        }
    }
}

impl OrganizerScope {
    /// The organizer slug.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Scopes requests to one event of this organizer.
    pub fn event(&self, slug: impl Into<String>) -> EventScope {
        let slug = slug.into();
        let path = self.path.join("events").join(&slug);
        EventScope {
            organizer: self.clone(),
            slug,
            path,
        }
    }

    pub(crate) fn client(&self) -> Client {
        self.client.clone()
    }

    /// `organizers/{organizer}/`
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

/// Resources belonging to one event.
///
/// Created with [`OrganizerScope::event`]. Resource handles such as `orders()`,
/// `items()` or `quotas()` are available as methods.
#[derive(Debug, Clone)]
pub struct EventScope {
    organizer: OrganizerScope,
    slug: String,
    path: Path,
}

impl EventScope {
    /// The event slug.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// The organizer this event belongs to.
    #[must_use]
    pub fn organizer(&self) -> &OrganizerScope {
        &self.organizer
    }

    pub(crate) fn client(&self) -> Client {
        self.organizer.client.clone()
    }

    /// `organizers/{organizer}/events/{event}/`
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}
