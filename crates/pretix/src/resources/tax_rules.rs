//! Tax rules: how tax is calculated for products.
//!
//! <https://docs.pretix.eu/dev/api/resources/taxrules.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::types::{Decimal, LocalizedString, string_enum};
use crate::{Client, EventScope, ListParams, Page, Request};

/// A tax rule of an event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct TaxRule {
    /// Internal ID of the tax rule.
    pub id: u64,
    /// The tax rule's name.
    pub name: LocalizedString,
    /// An optional name that is only used in the backend.
    pub internal_name: Option<String>,
    /// Tax rate in percent.
    pub rate: Decimal,
    /// Codified reason for the tax rate.
    pub code: Option<TaxCode>,
    /// Whether tax is assumed to be included in the product price.
    pub price_includes_tax: bool,
    /// Whether this is the default tax rule of the event (there can only be one).
    #[serde(default)]
    pub default: bool,
    /// **Deprecated**: whether EU reverse charge rules are applied. Use `custom_rules`.
    pub eu_reverse_charge: bool,
    /// Merchant country (required for reverse charge), can be empty.
    pub home_country: Option<String>,
    /// Whether changes of the tax rate based on custom rules keep the gross price constant.
    pub keep_gross_if_rate_changes: bool,
    /// Custom rules, processed in order. See the
    /// [schema](https://github.com/pretix/pretix/blob/master/src/pretix/static/schema/tax-rules-custom.schema.json).
    pub custom_rules: Option<serde_json::Value>,
}

string_enum! {
    /// Codified reason for a tax rate, modeled after EN16931 and DSFinV-K.
    ///
    /// Exemption codes with a reason (`E/<VATEX code>`) are represented as
    /// [`TaxCode::Unrecognized`].
    pub enum TaxCode {
        /// Standard VAT rate in the merchant country.
        Standard = "S/standard",
        /// Reduced VAT rate in the merchant country.
        Reduced = "S/reduced",
        /// Averaged VAT rate in the merchant country.
        Averaged = "S/averaged",
        /// Reverse charge.
        ReverseCharge = "AE",
        /// Services outside of scope of tax.
        OutsideScope = "O",
        /// Exempt from tax (no reason given).
        Exempt = "E",
        /// Zero-rated goods.
        ZeroRated = "Z",
        /// Free export item, VAT not charged.
        FreeExport = "G",
        /// VAT exempt for EEA intra-community supply of goods and services.
        IntraCommunity = "K",
        /// Canary Islands general indirect tax.
        CanaryIslands = "L",
        /// Tax for production, services and importation in Ceuta and Melilla.
        CeutaMelilla = "M",
        /// Transferred (VAT), only in Italy.
        Transferred = "B",
    }
}

/// Body of [`TaxRules::create`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateTaxRule {
    /// The tax rule's name.
    #[builder(into)]
    pub name: LocalizedString,
    /// Tax rate in percent.
    pub rate: Decimal,
    /// An optional name that is only used in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub internal_name: Option<String>,
    /// Codified reason for the tax rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub code: Option<TaxCode>,
    /// Whether tax is assumed to be included in the product price (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_includes_tax: Option<bool>,
    /// Whether this is the default tax rule of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    /// **Deprecated**: whether EU reverse charge rules are applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eu_reverse_charge: Option<bool>,
    /// Merchant country.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub home_country: Option<String>,
    /// Whether changes of the tax rate based on custom rules keep the gross price constant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_gross_if_rate_changes: Option<bool>,
    /// Custom rules, processed in order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_rules: Option<serde_json::Value>,
}

/// Body of [`TaxRules::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateTaxRule {
    /// The tax rule's name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<LocalizedString>,
    /// Tax rate in percent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<Decimal>,
    /// An optional name that is only used in the backend.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub internal_name: Option<Option<String>>,
    /// Codified reason for the tax rate.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub code: Option<Option<TaxCode>>,
    /// Whether tax is assumed to be included in the product price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_includes_tax: Option<bool>,
    /// Whether this is the default tax rule of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
    /// **Deprecated**: whether EU reverse charge rules are applied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eu_reverse_charge: Option<bool>,
    /// Merchant country.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub home_country: Option<Option<String>>,
    /// Whether changes of the tax rate based on custom rules keep the gross price constant.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_gross_if_rate_changes: Option<bool>,
    /// Custom rules, processed in order.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub custom_rules: Option<Option<serde_json::Value>>,
}

/// Endpoints for the tax rules of an event. Created with [`EventScope::tax_rules`].
#[derive(Debug, Clone)]
pub struct TaxRules {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Tax rules of this event.
    #[must_use]
    pub fn tax_rules(&self) -> TaxRules {
        TaxRules {
            client: self.client(),
            path: self.path().join("taxrules"),
        }
    }
}

impl TaxRules {
    /// Lists tax rules.
    pub fn list(&self, params: &ListParams) -> Request<Page<TaxRule>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one tax rule.
    pub fn get(&self, id: u64) -> Request<TaxRule> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a tax rule.
    pub fn create(&self, body: &CreateTaxRule) -> Request<TaxRule> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a tax rule.
    pub fn update(&self, id: u64, body: &UpdateTaxRule) -> Request<TaxRule> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a tax rule. Tax rules in use by products, settings or orders cannot be
    /// deleted (`403`).
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
