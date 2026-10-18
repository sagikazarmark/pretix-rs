//! Teams of an organizer, with their members, invites and API tokens.
//!
//! Unlike the pretix web interface, the API allows you to lock yourself out by deleting
//! or modifying the team your credentials belong to.
//!
//! <https://docs.pretix.eu/dev/api/resources/teams.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::string_enum;
use crate::{Client, ListParams, OrganizerScope, Page, Request};

/// A team: a group of users and API tokens sharing a set of permissions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Team {
    /// Internal ID of the team.
    pub id: u64,
    /// Team name.
    pub name: String,
    /// Whether this team has access to all events.
    pub all_events: bool,
    /// Slugs of the events this team has access to, if not all.
    #[serde(default)]
    pub limit_events: Vec<String>,
    /// Whether members are required to use two-factor authentication.
    pub require_2fa: bool,
    /// Whether members are granted all event-level permissions, including future
    /// additions.
    pub all_event_permissions: bool,
    /// The event-level permissions members are granted, if not all.
    #[serde(default)]
    pub limit_event_permissions: Vec<EventPermission>,
    /// Whether members are granted all organizer-level permissions, including future
    /// additions.
    pub all_organizer_permissions: bool,
    /// The organizer-level permissions members are granted, if not all.
    #[serde(default)]
    pub limit_organizer_permissions: Vec<OrganizerPermission>,
}

/// A user who is a member of a team.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct TeamMember {
    /// Internal ID of the user.
    pub id: u64,
    /// The user's email address.
    pub email: String,
    /// The user's full name.
    pub fullname: Option<String>,
    /// Whether this user uses two-factor authentication.
    pub require_2fa: bool,
}

/// A pending invitation to a team.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct TeamInvite {
    /// Internal ID of the invite.
    ///
    /// `None` in the response of [`TeamInvites::create`] if the user already had a
    /// pretix account and was added to the team directly.
    #[serde(default)]
    pub id: Option<u64>,
    /// The invitee's email address.
    pub email: String,
}

/// An API token of a team.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct TeamToken {
    /// Internal ID of the token.
    pub id: u64,
    /// Name of the token.
    pub name: String,
    /// Whether the token is active. Can never be set to `true` again once `false`.
    pub active: bool,
    /// The actual API token. Only included in the response of [`TeamTokens::create`].
    #[serde(default)]
    pub token: Option<String>,
}

string_enum! {
    /// Organizer-level permissions of a team. Plugins may add more.
    pub enum OrganizerPermission {
        /// Create events.
        EventsCreate = "organizer.events:create",
        /// Change general organizer settings.
        SettingsGeneralWrite = "organizer.settings.general:write",
        /// Change teams.
        TeamsWrite = "organizer.teams:write",
        /// Change seating plans.
        SeatingPlansWrite = "organizer.seatingplans:write",
        /// View gift cards.
        GiftCardsRead = "organizer.giftcards:read",
        /// Change gift cards.
        GiftCardsWrite = "organizer.giftcards:write",
        /// View customer accounts.
        CustomersRead = "organizer.customers:read",
        /// Change customer accounts.
        CustomersWrite = "organizer.customers:write",
        /// View reusable media.
        ReusableMediaRead = "organizer.reusablemedia:read",
        /// Change reusable media.
        ReusableMediaWrite = "organizer.reusablemedia:write",
        /// View devices.
        DevicesRead = "organizer.devices:read",
        /// Change devices.
        DevicesWrite = "organizer.devices:write",
        /// View outgoing emails.
        OutgoingMailsRead = "organizer.outgoingmails:read",
    }
}

string_enum! {
    /// Event-level permissions of a team. Plugins may add more.
    pub enum EventPermission {
        /// Change general event settings.
        SettingsGeneralWrite = "event.settings.general:write",
        /// Change payment settings.
        SettingsPaymentWrite = "event.settings.payment:write",
        /// Change tax settings.
        SettingsTaxWrite = "event.settings.tax:write",
        /// Change invoicing settings.
        SettingsInvoicingWrite = "event.settings.invoicing:write",
        /// Change dates of event series.
        SubeventsWrite = "event.subevents:write",
        /// Change products.
        ItemsWrite = "event.items:write",
        /// View orders.
        OrdersRead = "event.orders:read",
        /// Change orders.
        OrdersWrite = "event.orders:write",
        /// Check in orders.
        OrdersCheckin = "event.orders:checkin",
        /// View vouchers.
        VouchersRead = "event.vouchers:read",
        /// Change vouchers.
        VouchersWrite = "event.vouchers:write",
        /// Cancel the event.
        Cancel = "event:cancel",
    }
}

/// Body of [`Teams::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateTeam {
    /// Team name.
    #[builder(into)]
    pub name: String,
    /// Whether this team has access to all events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_events: Option<bool>,
    /// Slugs of the events this team has access to, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_events: Option<Vec<String>>,
    /// Whether members are required to use two-factor authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_2fa: Option<bool>,
    /// Whether members are granted all event-level permissions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_event_permissions: Option<bool>,
    /// The event-level permissions members are granted, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_event_permissions: Option<Vec<EventPermission>>,
    /// Whether members are granted all organizer-level permissions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_organizer_permissions: Option<bool>,
    /// The organizer-level permissions members are granted, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_organizer_permissions: Option<Vec<OrganizerPermission>>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_create_events: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_teams: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_organizer_settings: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_manage_customers: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_manage_reusable_media: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_manage_gift_cards: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_event_settings: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_items: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_view_orders: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_orders: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_view_vouchers: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_vouchers: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_checkin_orders: Option<bool>,
}

/// Body of [`Teams::update`]. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateTeam {
    /// Team name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// Whether this team has access to all events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_events: Option<bool>,
    /// Slugs of the events this team has access to, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_events: Option<Vec<String>>,
    /// Whether members are required to use two-factor authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_2fa: Option<bool>,
    /// Whether members are granted all event-level permissions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_event_permissions: Option<bool>,
    /// The event-level permissions members are granted, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_event_permissions: Option<Vec<EventPermission>>,
    /// Whether members are granted all organizer-level permissions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_organizer_permissions: Option<bool>,
    /// The organizer-level permissions members are granted, if not all.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_organizer_permissions: Option<Vec<OrganizerPermission>>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_create_events: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_teams: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_organizer_settings: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_manage_customers: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_manage_reusable_media: Option<bool>,
    /// **Deprecated**: use `limit_organizer_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_manage_gift_cards: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_event_settings: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_items: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_view_orders: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_orders: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_view_vouchers: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_change_vouchers: Option<bool>,
    /// **Deprecated**: use `limit_event_permissions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_checkin_orders: Option<bool>,
}

/// Body of [`TeamInvites::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateTeamInvite {
    /// The invitee's email address.
    #[builder(into)]
    pub email: String,
}

/// Body of [`TeamTokens::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateTeamToken {
    /// Name of the token.
    #[builder(into)]
    pub name: String,
}

/// Endpoints for the teams of an organizer. Created with [`OrganizerScope::teams`].
#[derive(Debug, Clone)]
pub struct Teams {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Teams of this organizer.
    #[must_use]
    pub fn teams(&self) -> Teams {
        Teams {
            client: self.client(),
            path: self.path().join("teams"),
        }
    }
}

impl Teams {
    /// Lists teams.
    pub fn list(&self, params: &ListParams) -> Request<Page<Team>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one team.
    pub fn get(&self, id: u64) -> Request<Team> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a team.
    pub fn create(&self, body: &CreateTeam) -> Request<Team> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a team.
    pub fn update(&self, id: u64, body: &UpdateTeam) -> Request<Team> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a team.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }

    /// Members of the team `team_id`.
    #[must_use]
    pub fn members(&self, team_id: u64) -> TeamMembers {
        TeamMembers {
            client: self.client.clone(),
            path: self.path.join(team_id).join("members"),
        }
    }

    /// Pending invites of the team `team_id`.
    #[must_use]
    pub fn invites(&self, team_id: u64) -> TeamInvites {
        TeamInvites {
            client: self.client.clone(),
            path: self.path.join(team_id).join("invites"),
        }
    }

    /// API tokens of the team `team_id`.
    #[must_use]
    pub fn tokens(&self, team_id: u64) -> TeamTokens {
        TeamTokens {
            client: self.client.clone(),
            path: self.path.join(team_id).join("tokens"),
        }
    }
}

/// Endpoints for the members of a team. Created with [`Teams::members`].
#[derive(Debug, Clone)]
pub struct TeamMembers {
    client: Client,
    path: Path,
}

impl TeamMembers {
    /// Lists team members.
    pub fn list(&self, params: &ListParams) -> Request<Page<TeamMember>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one team member.
    pub fn get(&self, id: u64) -> Request<TeamMember> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Removes a member from the team.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}

/// Endpoints for the invites of a team. Created with [`Teams::invites`].
#[derive(Debug, Clone)]
pub struct TeamInvites {
    client: Client,
    path: Path,
}

impl TeamInvites {
    /// Lists pending invites.
    pub fn list(&self, params: &ListParams) -> Request<Page<TeamInvite>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one invite.
    pub fn get(&self, id: u64) -> Request<TeamInvite> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Invites someone into the team.
    ///
    /// If the user already has a pretix account, they are added to the team directly
    /// and the response has no `id`.
    pub fn create(&self, body: &CreateTeamInvite) -> Request<TeamInvite> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Revokes an invite.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}

/// Endpoints for the API tokens of a team. Created with [`Teams::tokens`].
#[derive(Debug, Clone)]
pub struct TeamTokens {
    client: Client,
    path: Path,
}

impl TeamTokens {
    /// Lists API tokens.
    pub fn list(&self, params: &ListParams) -> Request<Page<TeamToken>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one API token.
    pub fn get(&self, id: u64) -> Request<TeamToken> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an API token. The response contains the secret `token`.
    pub fn create(&self, body: &CreateTeamToken) -> Request<TeamToken> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Disables an API token. Returns the disabled token.
    pub fn delete(&self, id: u64) -> Request<TeamToken> {
        self.client.delete(&self.path.join(id)).json()
    }
}
