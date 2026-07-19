//! MQTT client module for the Nest framework.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod client;
pub mod codes;
pub mod config;
pub mod error;
mod example;
pub mod message;
pub mod module;
mod runtime;
pub mod service;
#[cfg(test)]
mod test_support;

pub use client::MqttClient;
pub use config::{LastWillConfig, MqttConfig, MqttQos, TlsConfig};
pub use example::publish_and_receive;
pub use message::{topic_matches_filter, MqttMessage};
pub use module::{MqttModule, MQTT_MODULE_ID};
pub use service::MqttClientService;
