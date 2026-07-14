//! MQTT client service, registered via `MqttModule`.

use futures_util::Stream;
use nest_error::NestResult;

use crate::client::MqttClient;
use crate::config::MqttQos;
use crate::message::MqttMessage;

/// MQTT client registered via `MqttModule`.
///
/// Thin wrapper around [`MqttClient`] - the same relationship
/// `HttpClientService` has to `reqwest::Client`.
#[derive(Clone)]
pub struct MqttClientService {
    client: MqttClient,
}

impl MqttClientService {
    /// Wraps an already-connected [`MqttClient`].
    pub fn new(client: MqttClient) -> Self {
        Self { client }
    }

    /// Publishes a message to `topic`.
    pub async fn publish(
        &self,
        topic: &str,
        payload: impl Into<Vec<u8>>,
        qos: MqttQos,
        retain: bool,
    ) -> NestResult<()> {
        self.client.publish(topic, payload, qos, retain).await
    }

    /// Subscribes to `topic_filter`, returning a stream of matching messages.
    pub async fn subscribe(
        &self,
        topic_filter: &str,
        qos: MqttQos,
    ) -> NestResult<impl Stream<Item = MqttMessage>> {
        self.client.subscribe(topic_filter, qos).await
    }

    /// Disconnects cleanly from the broker.
    pub async fn disconnect(&self) -> NestResult<()> {
        self.client.disconnect().await
    }
}
