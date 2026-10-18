//! Reusable media: physical tokens such as plastic cards or NFC wristbands that stand
//! in for tickets or gift cards.
//!
//! <https://docs.pretix.eu/dev/api/resources/reusablemedia.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Expand;
use crate::resources::customers::Customer;
use crate::resources::gift_cards::GiftCard;
use crate::types::{Expandable, string_enum};
use crate::{Client, OrganizerScope, Page, Request};

/// A reusable medium.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ReusableMedium {
    /// Internal ID of the medium.
    pub id: u64,
    /// Type of medium.
    pub r#type: MediumType,
    /// Slug of the organizer who owns this medium.
    pub organizer: String,
    /// Unique identifier of the medium. The format depends on the type.
    pub identifier: String,
    /// Secret token to claim ownership of the medium.
    #[serde(default)]
    pub claim_token: Option<String>,
    /// Human-readable label to identify the medium.
    #[serde(default)]
    pub label: Option<String>,
    /// Whether this medium may be used.
    pub active: bool,
    /// Creation time.
    pub created: Timestamp,
    /// Time of last modification.
    pub updated: Timestamp,
    /// Expiry date.
    pub expires: Option<Timestamp>,
    /// The customer account this medium belongs to.
    pub customer: Option<Expandable<String, Customer>>,
    /// Tickets this medium is linked to.
    ///
    /// When expanded, positions are returned as JSON: they differ from
    /// [`OrderPosition`](crate::resources::orders::OrderPosition) by an `order` object
    /// (`{"code": "ABCDE", "event": "eventslug"}`).
    #[serde(default)]
    pub linked_orderpositions: Vec<Expandable<u64, serde_json::Value>>,
    /// **Deprecated**: the ticket the medium is linked to, if it is linked to exactly
    /// one ticket. Use `linked_orderpositions`.
    #[serde(default)]
    pub linked_orderposition: Option<Expandable<u64, serde_json::Value>>,
    /// The gift card this medium is linked to.
    pub linked_giftcard: Option<Expandable<u64, GiftCard>>,
    /// Additional data, content depends on the type. Internal to pretix.
    #[serde(default)]
    pub info: serde_json::Map<String, serde_json::Value>,
    /// Internal notes and comments.
    pub notes: Option<String>,
}

string_enum! {
    /// Type of a reusable medium (also used as the source type of scans).
    pub enum MediumType {
        /// A barcode.
        Barcode = "barcode",
        /// The UID of an NFC chip.
        NfcUid = "nfc_uid",
        /// An NFC MIFARE Ultralight AES chip.
        NfcMf0aes = "nfc_mf0aes",
    }
}

string_enum! {
    /// Fields of reusable media that can be expanded into full objects.
    pub enum ReusableMediumExpand {
        /// The linked gift card.
        LinkedGiftcard = "linked_giftcard",
        /// The owner ticket of the linked gift card.
        LinkedGiftcardOwnerTicket = "linked_giftcard.owner_ticket",
        /// The linked tickets.
        LinkedOrderpositions = "linked_orderpositions",
        /// **Deprecated**: the single linked ticket.
        LinkedOrderposition = "linked_orderposition",
        /// The customer.
        Customer = "customer",
    }
}

/// Query parameters for [`ReusableMedia::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListReusableMedia {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only media with this identifier. Prefer [`ReusableMedia::lookup`].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub identifier: Option<String>,
    /// Only media of this type.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub r#type: Option<MediumType>,
    /// Only (in)active media.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Only media linked to this customer identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub customer: Option<String>,
    /// Only media created since this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_since: Option<Timestamp>,
    /// Only media updated since this time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_since: Option<Timestamp>,
    /// Only media linked to any of these tickets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub linked_orderpositions: Vec<u64>,
    /// Only media linked to this ticket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_orderposition: Option<u64>,
    /// Only media linked to this gift card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_giftcard: Option<u64>,
    /// Fields to expand into full objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub expand: Vec<ReusableMediumExpand>,
}

/// Body of [`ReusableMedia::lookup`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct LookupMedium {
    /// Identifier of the medium.
    #[builder(into)]
    pub identifier: String,
    /// Type of the medium.
    #[builder(into)]
    pub r#type: MediumType,
}

/// Body of [`ReusableMedia::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateReusableMedium {
    /// Unique identifier of the medium.
    #[builder(into)]
    pub identifier: String,
    /// Type of the medium.
    #[builder(into)]
    pub r#type: MediumType,
    /// Human-readable label.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub label: Option<String>,
    /// Secret token to claim ownership of the medium.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub claim_token: Option<String>,
    /// Whether this medium may be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Expiry date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<Timestamp>,
    /// Identifier of the customer account.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub customer: Option<String>,
    /// IDs of the tickets to link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_orderpositions: Option<Vec<u64>>,
    /// **Deprecated**: ID of the single ticket to link. Use `linked_orderpositions`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_orderposition: Option<u64>,
    /// ID of the gift card to link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_giftcard: Option<u64>,
    /// Additional data, content depends on the type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<serde_json::Map<String, serde_json::Value>>,
    /// Internal notes and comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub notes: Option<String>,
}

/// Body of [`ReusableMedia::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateReusableMedium {
    /// Human-readable label.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub label: Option<Option<String>>,
    /// Secret token to claim ownership of the medium.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub claim_token: Option<Option<String>>,
    /// Whether this medium may be used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Expiry date.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub expires: Option<Option<Timestamp>>,
    /// Identifier of the customer account.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub customer: Option<Option<String>>,
    /// IDs of the linked tickets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_orderpositions: Option<Vec<u64>>,
    /// **Deprecated**: ID of the single linked ticket. Use `linked_orderpositions`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub linked_orderposition: Option<Option<u64>>,
    /// ID of the linked gift card.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub linked_giftcard: Option<Option<u64>>,
    /// Additional data, content depends on the type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<serde_json::Map<String, serde_json::Value>>,
    /// Internal notes and comments.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub notes: Option<Option<String>>,
}

/// Endpoints for the reusable media of an organizer. Created with
/// [`OrganizerScope::reusable_media`].
#[derive(Debug, Clone)]
pub struct ReusableMedia {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Reusable media of this organizer.
    #[must_use]
    pub fn reusable_media(&self) -> ReusableMedia {
        ReusableMedia {
            client: self.client(),
            path: self.path().join("reusablemedia"),
        }
    }
}

impl ReusableMedia {
    /// Lists reusable media.
    pub fn list(&self, params: &ListReusableMedia) -> Request<Page<ReusableMedium>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one medium.
    pub fn get(&self, id: u64) -> Request<ReusableMedium> {
        self.get_with(id, &Expand::default())
    }

    /// Fetches one medium, expanding linked objects.
    pub fn get_with(
        &self,
        id: u64,
        params: &Expand<ReusableMediumExpand>,
    ) -> Request<ReusableMedium> {
        self.client.get(&self.path.join(id)).query(params).json()
    }

    /// Looks up a medium by identifier and type, possibly creating it automatically.
    ///
    /// May return media of other organizers with a cross-acceptance agreement (with
    /// only linked gift cards).
    pub fn lookup(&self, body: &LookupMedium) -> Request<ReusableMedium> {
        self.lookup_with(body, &Expand::default())
    }

    /// Like [`ReusableMedia::lookup`], expanding linked objects.
    pub fn lookup_with(
        &self,
        body: &LookupMedium,
        params: &Expand<ReusableMediumExpand>,
    ) -> Request<ReusableMedium> {
        self.client
            .post(&self.path.join("lookup"))
            .query(params)
            .json_body(body)
            .json()
    }

    /// Creates a medium.
    pub fn create(&self, body: &CreateReusableMedium) -> Request<ReusableMedium> {
        self.create_with(body, &Expand::default())
    }

    /// Like [`ReusableMedia::create`], expanding linked objects.
    pub fn create_with(
        &self,
        body: &CreateReusableMedium,
        params: &Expand<ReusableMediumExpand>,
    ) -> Request<ReusableMedium> {
        self.client
            .post(&self.path)
            .query(params)
            .json_body(body)
            .json()
    }

    /// Updates a medium.
    pub fn update(&self, id: u64, body: &UpdateReusableMedium) -> Request<ReusableMedium> {
        self.update_with(id, body, &Expand::default())
    }

    /// Like [`ReusableMedia::update`], expanding linked objects.
    pub fn update_with(
        &self,
        id: u64,
        body: &UpdateReusableMedium,
        params: &Expand<ReusableMediumExpand>,
    ) -> Request<ReusableMedium> {
        self.client
            .patch(&self.path.join(id))
            .query(params)
            .json_body(body)
            .json()
    }
}
