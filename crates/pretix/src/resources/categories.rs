//! Item categories: grouping for items (products).
//!
//! <https://docs.pretix.eu/dev/api/resources/categories.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::types::{LocalizedString, string_enum};
use crate::{Client, EventScope, Page, Request};

/// An item category.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Category {
    /// Internal ID of the category.
    pub id: u64,
    /// The category's visible name.
    pub name: LocalizedString,
    /// An optional name that is only used in the backend.
    pub internal_name: Option<String>,
    /// A public description (might include Markdown).
    pub description: Option<LocalizedString>,
    /// Used for sorting the categories.
    pub position: i64,
    /// Whether items within this category are only sold as add-ons to other products.
    pub is_addon: bool,
    /// Whether and where the category is shown for cross-selling (`None` = disabled).
    /// Only available if `is_addon` is `false`.
    #[serde(default)]
    pub cross_selling_mode: Option<CrossSellingMode>,
    /// When the category is shown in the cross-selling step. Only relevant if
    /// `cross_selling_mode` is set.
    #[serde(default)]
    pub cross_selling_condition: Option<CrossSellingCondition>,
    /// IDs of the items of which at least one needs to be in the cart for this category to
    /// be shown. Only relevant if `cross_selling_condition` is `products`.
    #[serde(default)]
    pub cross_selling_match_products: Vec<u64>,
}

string_enum! {
    /// Where a category is shown for cross-selling.
    pub enum CrossSellingMode {
        /// Only visible in the cross-selling step.
        Only = "only",
        /// Visible in the cross-selling step and on the normal index page.
        Both = "both",
    }
}

string_enum! {
    /// When a category is shown in the cross-selling step.
    pub enum CrossSellingCondition {
        /// Always.
        Always = "always",
        /// Only if the cart contains one of `cross_selling_match_products`.
        Products = "products",
        /// Only products that qualify for a discount according to discount rules.
        Discounts = "discounts",
    }
}

string_enum! {
    /// Fields categories can be sorted by.
    pub enum CategoryOrdering {
        /// By ID.
        Id = "id",
        /// By position (default).
        Position = "position",
    }
}

/// Query parameters for [`Categories::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct ListCategories {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<CategoryOrdering>>,
    /// Only categories with this `is_addon` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_addon: Option<bool>,
}

/// Body of [`Categories::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateCategory {
    /// The category's visible name.
    #[builder(into)]
    pub name: LocalizedString,
    /// An optional name that is only used in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub internal_name: Option<String>,
    /// A public description (might include Markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub description: Option<LocalizedString>,
    /// Used for sorting the categories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Whether items within this category are only sold as add-ons to other products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_addon: Option<bool>,
    /// Whether and where the category is shown for cross-selling.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub cross_selling_mode: Option<CrossSellingMode>,
    /// When the category is shown in the cross-selling step.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub cross_selling_condition: Option<CrossSellingCondition>,
    /// IDs of the items of which at least one needs to be in the cart for this category to
    /// be shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_selling_match_products: Option<Vec<u64>>,
}

/// Body of [`Categories::update`]. Unset fields are left unchanged.
///
/// For nullable fields, `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateCategory {
    /// The category's visible name.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<LocalizedString>,
    /// An optional name that is only used in the backend.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub internal_name: Option<Option<String>>,
    /// A public description (might include Markdown).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub description: Option<Option<LocalizedString>>,
    /// Used for sorting the categories.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// Whether items within this category are only sold as add-ons to other products.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_addon: Option<bool>,
    /// Whether and where the category is shown for cross-selling (`Some(None)` disables
    /// cross-selling).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub cross_selling_mode: Option<Option<CrossSellingMode>>,
    /// When the category is shown in the cross-selling step.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub cross_selling_condition: Option<Option<CrossSellingCondition>>,
    /// IDs of the items of which at least one needs to be in the cart for this category to
    /// be shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_selling_match_products: Option<Vec<u64>>,
}

/// Endpoints for the item categories of an event. Created with [`EventScope::categories`].
#[derive(Debug, Clone)]
pub struct Categories {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Item categories of this event.
    #[must_use]
    pub fn categories(&self) -> Categories {
        Categories {
            client: self.client(),
            path: self.path().join("categories"),
        }
    }
}

impl Categories {
    /// Lists categories.
    pub fn list(&self, params: &ListCategories) -> Request<Page<Category>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one category.
    pub fn get(&self, id: u64) -> Request<Category> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a category.
    pub fn create(&self, body: &CreateCategory) -> Request<Category> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a category.
    pub fn update(&self, id: u64, body: &UpdateCategory) -> Request<Category> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a category.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
