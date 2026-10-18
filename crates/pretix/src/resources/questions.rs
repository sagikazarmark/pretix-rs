//! Questions: additional fields customers fill out during checkout, and the options of
//! choice questions.
//!
//! <https://docs.pretix.eu/dev/api/resources/questions.html>
//! <https://docs.pretix.eu/dev/api/resources/question_options.html>

use jiff::Timestamp;
use jiff::civil::Date;
use serde::{Deserialize, Serialize};

use crate::path::Path;
use crate::query::Ordering;
use crate::types::{Decimal, LocalizedString, string_enum};
use crate::{Client, EventScope, Page, Request};

/// A question asked during checkout (or check-in).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Question {
    /// Internal ID of the question.
    pub id: u64,
    /// The field label shown to the customer.
    pub question: LocalizedString,
    /// The help text shown to the customer.
    pub help_text: Option<LocalizedString>,
    /// The expected type of answer.
    pub r#type: QuestionType,
    /// Whether the question needs to be filled out.
    pub required: bool,
    /// Used for sorting.
    pub position: i64,
    /// IDs of the items this question is assigned to.
    #[serde(default)]
    pub items: Vec<u64>,
    /// An arbitrary string that can be used for matching with other sources.
    pub identifier: String,
    /// Whether the question is asked when redeeming the ticket instead of while buying
    /// it.
    pub ask_during_checkin: bool,
    /// Whether the answer is shown during check-in (if the check-in client supports it).
    pub show_during_checkin: bool,
    /// Whether the question is only shown in the backend.
    pub hidden: bool,
    /// Whether the answer is shown on invoices.
    pub print_on_invoice: bool,
    /// The available options of choice questions (`C` and `M`).
    #[serde(default)]
    pub options: Vec<QuestionOption>,
    /// Minimum value for number questions.
    pub valid_number_min: Option<Decimal>,
    /// Maximum value for number questions.
    pub valid_number_max: Option<Decimal>,
    /// Minimum value for date questions.
    pub valid_date_min: Option<Date>,
    /// Maximum value for date questions.
    pub valid_date_max: Option<Date>,
    /// Minimum value for date and time questions.
    pub valid_datetime_min: Option<Timestamp>,
    /// Maximum value for date and time questions.
    pub valid_datetime_max: Option<Timestamp>,
    /// Whether file uploads are validated as portrait photos.
    pub valid_file_portrait: bool,
    /// Minimum length for string questions.
    pub valid_string_length_min: Option<u32>,
    /// Maximum length for string questions.
    pub valid_string_length_max: Option<u32>,
    /// ID of a different question that must be answered with one of
    /// `dependency_values` for this question to be shown.
    pub dependency_question: Option<u64>,
    /// `["True"]` or `["False"]` for boolean dependency questions, otherwise
    /// `identifier`s of options of the dependency question.
    #[serde(default)]
    pub dependency_values: Vec<String>,
    /// **Deprecated**: use `dependency_values`.
    pub dependency_value: Option<String>,
}

/// An option of a choice question.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct QuestionOption {
    /// Internal ID of the option.
    pub id: u64,
    /// Used for sorting.
    pub position: i64,
    /// An arbitrary string that can be used for matching with other sources.
    pub identifier: String,
    /// The displayed value of this option.
    pub answer: LocalizedString,
}

string_enum! {
    /// The expected type of answer to a question.
    pub enum QuestionType {
        /// Number.
        Number = "N",
        /// One-line string.
        String = "S",
        /// Multi-line string.
        Text = "T",
        /// Boolean.
        Boolean = "B",
        /// Choice from a list.
        Choice = "C",
        /// Multiple choice from a list.
        MultipleChoice = "M",
        /// File upload.
        File = "F",
        /// Date.
        Date = "D",
        /// Time.
        Time = "H",
        /// Date and time.
        DateTime = "W",
        /// Country code (ISO 3166-1 alpha-2).
        CountryCode = "CC",
        /// Telephone number.
        PhoneNumber = "TEL",
    }
}

string_enum! {
    /// Fields questions can be sorted by.
    pub enum QuestionOrdering {
        /// By ID.
        Id = "id",
        /// By position (default).
        Position = "position",
    }
}

/// Query parameters for [`Questions::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListQuestions {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Sort order.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub ordering: Option<Ordering<QuestionOrdering>>,
    /// Only questions with this identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub identifier: Option<String>,
    /// Only questions that are (`true`) or are not (`false`) asked during check-in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_during_checkin: Option<bool>,
    /// Only questions that are (`true`) or are not (`false`) required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Query parameters for [`QuestionOptions::list`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct ListQuestionOptions {
    /// The page to fetch (1-based).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Results per page (at most 50).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    /// Only options with this `active` value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
}

/// Body of [`Questions::create`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateQuestion {
    /// The field label shown to the customer.
    #[builder(into)]
    pub question: LocalizedString,
    /// The expected type of answer.
    pub r#type: QuestionType,
    /// The help text shown to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub help_text: Option<LocalizedString>,
    /// Whether the question needs to be filled out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// IDs of the items this question is assigned to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<u64>>,
    /// An arbitrary string that can be used for matching with other sources. pretix
    /// requires the field but accepts `null` (sent when unset), in which case it
    /// generates an identifier.
    #[builder(into)]
    pub identifier: Option<String>,
    /// Whether the question is asked when redeeming the ticket instead of while buying
    /// it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_during_checkin: Option<bool>,
    /// Whether the answer is shown during check-in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_during_checkin: Option<bool>,
    /// Whether the question is only shown in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// Whether the answer is shown on invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_on_invoice: Option<bool>,
    /// The options of choice questions. Can only be set during creation; use
    /// [`Questions::options`] to change them later.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<CreateQuestionOption>>,
    /// Minimum value for number questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_number_min: Option<Decimal>,
    /// Maximum value for number questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_number_max: Option<Decimal>,
    /// Minimum value for date questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_date_min: Option<Date>,
    /// Maximum value for date questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_date_max: Option<Date>,
    /// Minimum value for date and time questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_datetime_min: Option<Timestamp>,
    /// Maximum value for date and time questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_datetime_max: Option<Timestamp>,
    /// Whether file uploads are validated as portrait photos.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_file_portrait: Option<bool>,
    /// Minimum length for string questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_string_length_min: Option<u32>,
    /// Maximum length for string questions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_string_length_max: Option<u32>,
    /// ID of a different question this question depends on. Cannot be combined with
    /// `ask_during_checkin`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_question: Option<u64>,
    /// Values of the dependency question for which this question is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_values: Option<Vec<String>>,
    /// **Deprecated**: use `dependency_values`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub dependency_value: Option<String>,
}

/// Body of [`Questions::update`]. Unset fields are left unchanged.
///
/// Options cannot be changed here; use [`Questions::options`]. For nullable fields,
/// `Some(None)` clears the value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateQuestion {
    /// The field label shown to the customer.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub question: Option<LocalizedString>,
    /// The help text shown to the customer.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub help_text: Option<Option<LocalizedString>>,
    /// The expected type of answer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<QuestionType>,
    /// Whether the question needs to be filled out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
    /// IDs of the items this question is assigned to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<u64>>,
    /// An arbitrary string that can be used for matching with other sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub identifier: Option<String>,
    /// Whether the question is asked when redeeming the ticket instead of while buying
    /// it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_during_checkin: Option<bool>,
    /// Whether the answer is shown during check-in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_during_checkin: Option<bool>,
    /// Whether the question is only shown in the backend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    /// Whether the answer is shown on invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_on_invoice: Option<bool>,
    /// Minimum value for number questions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_number_min: Option<Option<Decimal>>,
    /// Maximum value for number questions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_number_max: Option<Option<Decimal>>,
    /// Minimum value for date questions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_date_min: Option<Option<Date>>,
    /// Maximum value for date questions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_date_max: Option<Option<Date>>,
    /// Minimum value for date and time questions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_datetime_min: Option<Option<Timestamp>>,
    /// Maximum value for date and time questions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_datetime_max: Option<Option<Timestamp>>,
    /// Whether file uploads are validated as portrait photos.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_file_portrait: Option<bool>,
    /// Minimum length for string questions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_string_length_min: Option<Option<u32>>,
    /// Maximum length for string questions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub valid_string_length_max: Option<Option<u32>>,
    /// ID of a different question this question depends on.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub dependency_question: Option<Option<u64>>,
    /// Values of the dependency question for which this question is shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_values: Option<Vec<String>>,
    /// **Deprecated**: use `dependency_values`.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::serde_util::double_option"
    )]
    pub dependency_value: Option<Option<String>>,
}

/// Body of [`QuestionOptions::create`], also used for the options in
/// [`CreateQuestion`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct CreateQuestionOption {
    /// The displayed value of this option.
    #[builder(into)]
    pub answer: LocalizedString,
    /// An arbitrary string that can be used for matching with other sources. pretix
    /// requires the field but accepts `null` (sent when unset), in which case it
    /// generates an identifier.
    #[builder(into)]
    pub identifier: Option<String>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
}

/// Body of [`QuestionOptions::update`]. Unset fields are left unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, bon::Builder)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct UpdateQuestionOption {
    /// The displayed value of this option.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub answer: Option<LocalizedString>,
    /// An arbitrary string that can be used for matching with other sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(into)]
    pub identifier: Option<String>,
    /// Used for sorting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i64>,
}

/// Endpoints for the questions of an event. Created with [`EventScope::questions`].
#[derive(Debug, Clone)]
pub struct Questions {
    client: Client,
    path: Path,
}

impl EventScope {
    /// Questions of this event.
    #[must_use]
    pub fn questions(&self) -> Questions {
        Questions {
            client: self.client(),
            path: self.path().join("questions"),
        }
    }
}

impl Questions {
    /// Lists questions.
    pub fn list(&self, params: &ListQuestions) -> Request<Page<Question>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one question.
    pub fn get(&self, id: u64) -> Request<Question> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates a question.
    pub fn create(&self, body: &CreateQuestion) -> Request<Question> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates a question.
    pub fn update(&self, id: u64, body: &UpdateQuestion) -> Request<Question> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes a question.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }

    /// Options of the choice question `question_id`.
    #[must_use]
    pub fn options(&self, question_id: u64) -> QuestionOptions {
        QuestionOptions {
            client: self.client.clone(),
            path: self.path.join(question_id).join("options"),
        }
    }
}

/// Endpoints for the options of a choice question. Created with
/// [`Questions::options`].
#[derive(Debug, Clone)]
pub struct QuestionOptions {
    client: Client,
    path: Path,
}

impl QuestionOptions {
    /// Lists options.
    pub fn list(&self, params: &ListQuestionOptions) -> Request<Page<QuestionOption>> {
        self.client.get(&self.path).query(params).page()
    }

    /// Fetches one option.
    pub fn get(&self, id: u64) -> Request<QuestionOption> {
        self.client.get(&self.path.join(id)).json()
    }

    /// Creates an option.
    pub fn create(&self, body: &CreateQuestionOption) -> Request<QuestionOption> {
        self.client.post(&self.path).json_body(body).json()
    }

    /// Updates an option.
    pub fn update(&self, id: u64, body: &UpdateQuestionOption) -> Request<QuestionOption> {
        self.client
            .patch(&self.path.join(id))
            .json_body(body)
            .json()
    }

    /// Deletes an option.
    pub fn delete(&self, id: u64) -> Request<()> {
        self.client.delete(&self.path.join(id)).empty()
    }
}
