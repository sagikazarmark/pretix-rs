//! Gift cards and their transactions.
//!
//! <https://docs.pretix.eu/dev/api/resources/giftcards.html>

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Expand;
use crate::types::{Decimal, Expandable, string_enum};
use crate::{Client, ListParams, OrganizerScope, Page, Request};

/// A gift card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct GiftCard {
    /// Internal ID of the gift card.
    pub id: u64,
    /// Gift card code. Cannot be changed.
    pub secret: String,
    /// Time of issuance. Not in the pretix docs.
    #[serde(default)]
    pub issuance: Option<Timestamp>,
    /// Current value.
    pub value: Decimal,
    /// Currency of the value. Cannot be changed.
    pub currency: String,
    /// Whether this is a test gift card.
    pub testmode: bool,
    /// Expiry date.
    pub expires: Option<Timestamp>,
    /// Special terms and conditions for this card.
    pub conditions: Option<String>,
    /// The order position that "owns" this gift card and can view all transactions.
    ///
    /// When expanded, the position is returned as JSON: it differs from
    /// [`OrderPosition`](crate::resources::orders::OrderPosition) by an `order` object
    /// (`{"code": "ABCDE", "event": "eventslug"}`).
    pub owner_ticket: Option<Expandable<u64, serde_json::Value>>,
    /// Slug of the organizer who created this gift card and is responsible for it.
    pub issuer: String,
}

/// A transaction changing the value of a gift card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct GiftCardTransaction {
    /// Internal ID of the transaction.
    pub id: u64,
    /// Creation time.
    pub datetime: Timestamp,
    /// Transaction amount.
    pub value: Decimal,
    /// Event slug, if the gift card was used in the web shop.
    pub event: Option<String>,
    /// Order code, if the gift card was used in the web shop.
    pub order: Option<String>,
    /// Custom text of the transaction.
    pub text: Option<String>,
    /// Additional data about the transaction.
    pub info: Option<serde_json::Map<String, serde_json::Value>>,
    /// Slug of the organizer who created this transaction (`None` for old
    /// transactions).
    pub acceptor: Option<String>,
}

string_enum! {
    /// Fields of gift cards that can be expanded into full objects.
    pub enum GiftCardExpand {
        /// The owner ticket.
        OwnerTicket = "owner_ticket",
    }
}

/// Query parameters for [`GiftCards::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListGiftCards {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only gift cards with this secret.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub secret: Option<String>,
    /// Only gift cards with this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Decimal>,
    /// Only (not) expired gift cards.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired: Option<bool>,
    /// Only gift cards (not) in test mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// Also include gift cards of other organizers that are accepted by this organizer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_accepted: Option<bool>,
    /// Fields to expand into full objects.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[builder(default)]
    pub expand: Vec<GiftCardExpand>,
}

/// A reference to an order position when setting the owner ticket of a gift card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(untagged)]
#[non_exhaustive]
pub enum TicketRef {
    /// The order position ID.
    Id(u64),
    /// The ticket secret of the order position.
    Secret(String),
}

/// Body of [`GiftCards::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateGiftCard {
    /// Gift card code. Generated if not set.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub secret: Option<String>,
    /// Currency of the value.
    #[builder(into)]
    pub currency: String,
    /// Initial value.
    #[builder(into)]
    pub value: Decimal,
    /// Whether this is a test gift card.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub testmode: Option<bool>,
    /// Expiry date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<Timestamp>,
    /// Special terms and conditions for this card.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub conditions: Option<String>,
    /// The owner ticket (ID or secret).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_ticket: Option<TicketRef>,
}

/// Body of [`GiftCards::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value. Prefer
/// [`GiftCards::transact`] over setting `value` to avoid race conditions.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateGiftCard {
    /// The value.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub value: Option<Decimal>,
    /// Expiry date.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub expires: Option<Option<Timestamp>>,
    /// Special terms and conditions for this card.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub conditions: Option<Option<String>>,
    /// The owner ticket (ID or secret).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub owner_ticket: Option<Option<TicketRef>>,
}

/// Body of [`GiftCards::transact`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct TransactGiftCard {
    /// Amount to add (positive) or remove (negative).
    #[builder(into)]
    pub value: Decimal,
    /// Text explaining the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub text: Option<String>,
}

/// Query parameters of [`GiftCards::get_with`] and [`GiftCards::transact_with`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct IncludeAccepted {
    /// Also find gift cards issued by other organizers whose gift cards this organizer
    /// accepts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_accepted: Option<bool>,
}

/// Endpoints for the gift cards of an organizer. Created with
/// [`OrganizerScope::gift_cards`].
#[derive(Debug, Clone)]
pub struct GiftCards {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Gift cards of this organizer.
    #[must_use]
    pub fn gift_cards(&self) -> GiftCards {
        GiftCards {
            client: self.client(),
            path: self.path().join("giftcards"),
        }
    }
}

impl GiftCards {
    /// Lists gift cards.
    pub fn list(&self, params: &ListGiftCards) -> Request<Page<GiftCard>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one gift card issued by this organizer.
    pub fn get(&self, id: u64) -> Request<GiftCard> {
        self.get_with(id, &IncludeAccepted::default())
    }

    /// Fetches one gift card, optionally also among gift cards issued by other
    /// organizers whose gift cards this organizer accepts.
    pub fn get_with(&self, id: u64, params: &IncludeAccepted) -> Request<GiftCard> {
        self.client.get(&self.path.join(id)).query(params).json()
    }

    /// Creates a gift card.
    pub fn create(&self, body: &CreateGiftCard) -> Request<GiftCard> {
        self.create_with(body, &Expand::default())
    }

    /// Like [`GiftCards::create`], expanding fields of the returned gift card.
    pub fn create_with(
        &self,
        body: &CreateGiftCard,
        params: &Expand<GiftCardExpand>,
    ) -> Request<GiftCard> {
        self.client
            .post(&self.path)
            .query(params)
            .json_body(body)
            .json()
    }

    /// Updates a gift card.
    pub fn update(&self, id: u64, body: &UpdateGiftCard) -> Request<GiftCard> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Atomically changes the value of a gift card issued by this organizer.
    ///
    /// Fails with `409 Conflict` if there is not sufficient credit.
    pub fn transact(&self, id: u64, body: &TransactGiftCard) -> Request<GiftCard> {
        self.transact_with(id, body, &IncludeAccepted::default())
    }

    /// Like [`GiftCards::transact`], optionally also for gift cards issued by other
    /// organizers whose gift cards this organizer accepts.
    pub fn transact_with(
        &self,
        id: u64,
        body: &TransactGiftCard,
        params: &IncludeAccepted,
    ) -> Request<GiftCard> {
        self.client
            .post(&self.path.join(id).join("transact"))
            .query(params)
            .json_body(body)
            .json()
    }

    /// Lists the transactions of a gift card.
    pub fn transactions(&self, id: u64, params: &ListParams) -> Request<Page<GiftCardTransaction>> {
        self.client
            .get(&self.path.join(id).join("transactions"))
            .query(params)
            .page()
    }
}
