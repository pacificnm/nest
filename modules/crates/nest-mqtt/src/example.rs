//! Standalone example: connect, publish, and receive over MQTT.

use futures_util::StreamExt;
use nest_error::{NestError, NestResult};

use crate::client::MqttClient;
use crate::config::{MqttConfig, MqttQos};
use crate::message::MqttMessage;

/// Connects to `broker_host`/`broker_port`, publishes `payload` to `topic`,
/// and returns the first message received back on that same topic.
///
/// Demonstrates the minimal end-to-end usage of [`MqttClient`]: connect,
/// subscribe, publish, receive.
pub async fn publish_and_receive(
    broker_host: &str,
    broker_port: u16,
    topic: &str,
    payload: impl Into<Vec<u8>>,
) -> NestResult<MqttMessage> {
    let config = MqttConfig::new(broker_host, broker_port, "nest-mqtt-example");
    let client = MqttClient::connect(&config).await?;

    let messages = client.subscribe(topic, MqttQos::AtLeastOnce).await?;
    let mut messages = std::pin::pin!(messages);

    // subscribe() only enqueues the SUBSCRIBE request and returns before the
    // broker's SUBACK arrives - give the event loop a moment to actually
    // subscribe before publishing (see MqttClient::subscribe's docs).
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    client
        .publish(topic, payload, MqttQos::AtLeastOnce, false)
        .await?;

    messages
        .next()
        .await
        .ok_or_else(|| NestError::network("message stream ended unexpectedly"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn publish_and_receive_round_trip() {
        let broker = crate::test_support::start_broker().await;
        let message = publish_and_receive(
            &broker.host,
            broker.port,
            "nest/mqtt/example",
            b"hi".to_vec(),
        )
        .await
        .unwrap();
        assert_eq!(message.topic, "nest/mqtt/example");
        assert_eq!(message.payload, b"hi");
    }
}
