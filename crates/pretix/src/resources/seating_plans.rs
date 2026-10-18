//! Seating plans of an organizer.
//!
//! <https://docs.pretix.eu/dev/api/resources/seatingplans.html>

use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::{Client, ListParams, OrganizerScope, Page, Request};

/// A seating plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SeatingPlan {
    /// Internal ID of the plan.
    pub id: u64,
    /// Human-readable name of the plan.
    pub name: String,
    /// JSON representation of the seating plan, following pretix'
    /// [seating plan schema](https://github.com/pretix/pretix/blob/master/src/pretix/static/seating/seating-plan.schema.json).
    pub layout: serde_json::Value,
}

/// Body of [`SeatingPlans::create`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct CreateSeatingPlan {
    /// Human-readable name of the plan.
    #[builder(into)]
    pub name: String,
    /// JSON representation of the seating plan.
    pub layout: serde_json::Value,
}

/// Body of [`SeatingPlans::update`]. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, bon::Builder)]
#[non_exhaustive]
pub struct UpdateSeatingPlan {
    /// Human-readable name of the plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub name: Option<String>,
    /// JSON representation of the seating plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<serde_json::Value>,
}

/// Endpoints for the seating plans of an organizer. Created with
/// [`OrganizerScope::seating_plans`].
#[derive(Debug, Clone)]
pub struct SeatingPlans {
    client: Client,
    path: Path,
}

impl OrganizerScope {
    /// Seating plans of this organizer.
    #[must_use]
    pub fn seating_plans(&self) -> SeatingPlans {
        SeatingPlans {
            client: self.client(),
            path: self.path().join("seatingplans"),
        }
    }
}

impl SeatingPlans {
    /// Lists seating plans.
    pub fn list(&self, params: &ListParams) -> Request<Page<SeatingPlan>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one seating plan.
    pub fn get(&self, id: u64) -> Request<SeatingPlan> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a seating plan.
    pub fn create(&self, body: &CreateSeatingPlan) -> Request<SeatingPlan> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a seating plan. Plans in use by an event cannot be changed.
    pub fn update(&self, id: u64, body: &UpdateSeatingPlan) -> Request<SeatingPlan> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a seating plan. Plans in use by an event cannot be deleted.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
