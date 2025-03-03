// Copyright (c) 2022 Snowplow Analytics Ltd. All rights reserved.
//
// This program is licensed to you under the Apache License Version 2.0,
// and you may not use this file except in compliance with the Apache License Version 2.0.
// You may obtain a copy of the Apache License Version 2.0 at http://www.apache.org/licenses/LICENSE-2.0.
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the Apache License Version 2.0 is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the Apache License Version 2.0 for the specific language governing permissions and limitations there under.

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use uuid::Uuid;

use crate::timestamp::{ts_milliseconds_string, ts_milliseconds_string_option};
use crate::Error;
use crate::StructuredEvent;
use crate::Subject;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EventType {
    #[serde(rename(serialize = "se"))]
    StructuredEvent,
    #[serde(rename(serialize = "ue"))]
    SelfDescribingEvent,
}

#[derive(Builder, Serialize, Deserialize, Default, Clone, Debug)]
#[builder(field(public))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
#[builder(build_fn(error = "Error"))]
#[builder(derive(Clone))]
/// The final payload that is sent to the collector
///
/// For more information, see the [Snowplow Tracker Protocol](https://docs.snowplow.io/docs/collecting-data/collecting-from-own-applications/snowplow-tracker-protocol)
pub struct Payload {
    p: String,
    tv: String,
    pub(crate) eid: Uuid,
    #[serde(with = "ts_milliseconds_string")]
    dtm: DateTime<Utc>,
    #[serde(with = "ts_milliseconds_string")]
    pub(crate) stm: DateTime<Utc>,

    /// The true timestamp of the event
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "ts_milliseconds_string_option")]
    pub ttm: Option<DateTime<Utc>>,

    #[builder(default)]
    e: Option<EventType>,
    aid: String,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) ue_pr: Option<SelfDescribingEventData>,

    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    co: Option<ContextData>,

    // Structured Event
    #[builder(default)]
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) structured_event: Option<StructuredEvent>,

    // Subject
    #[builder(default)]
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) subject: Option<Subject>,
}

impl Payload {
    pub fn builder() -> PayloadBuilder {
        PayloadBuilder::default()
    }
}

impl PayloadBuilder {
    pub fn finalise_payload(self) -> Result<Payload, Error> {
        self.stm(Utc::now()).build()
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SelfDescribingEventData {
    pub schema: String,
    pub data: SelfDescribingJson,
}

impl SelfDescribingEventData {
    pub fn new(data: SelfDescribingJson) -> SelfDescribingEventData {
        SelfDescribingEventData {
            schema: String::from(
                "iglu:com.snowplowanalytics.snowplow/unstruct_event/jsonschema/1-0-0",
            ),
            data: data,
        }
    }
}

// The collector expects the `data` field of the `SelfDescribingEventData` to be an object,
// but the SelfDescribingEventData to be a string, so we have to manually serialize SelfDescribingEventData
impl Serialize for SelfDescribingEventData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &json!({
                "schema": self.schema,
                "data": self.data,
            })
            .to_string(),
        )
    }
}

/// Self-describing JSON to be used mainly when creating context entities.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelfDescribingJson {
    /// A valid Iglu schema path.
    ///
    /// This must point to the location of the custom event’s schema, of the format: `iglu:{vendor}/{name}/{format}/{version}`.
    pub schema: String,

    /// The custom data for the event.
    ///
    /// This data must conform to the schema specified in the schema argument, or the event will fail validation and land in bad rows.
    pub data: Value,
}

impl SelfDescribingJson {
    pub fn new(schema: &str, data: Value) -> SelfDescribingJson {
        SelfDescribingJson {
            schema: schema.to_string(),
            data: data,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ContextData {
    pub schema: String,
    pub data: Vec<SelfDescribingJson>,
}

impl ContextData {
    pub fn new(data: Vec<SelfDescribingJson>) -> ContextData {
        ContextData {
            schema: String::from("iglu:com.snowplowanalytics.snowplow/contexts/jsonschema/1-0-1"),
            data,
        }
    }
}

// The collector expects the `data` field of the `SelfDescribingEventData` to be an object,
// but the SelfDescribingEventData to be a string, so we have to manually serialize SelfDescribingEventData
impl Serialize for ContextData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &json!({
                "schema": self.schema,
                "data": self.data,
            })
            .to_string(),
        )
    }
}
