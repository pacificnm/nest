//! Received MQTT messages and topic-filter matching.

/// One message received from a subscribed topic.
#[derive(Debug, Clone)]
pub struct MqttMessage {
    /// Topic the message was published on.
    pub topic: String,
    /// Raw message payload.
    pub payload: Vec<u8>,
    /// Whether the broker sent this as a retained message.
    pub retained: bool,
}

/// Returns true if `topic` matches an MQTT topic filter (`+` and `#` wildcards).
///
/// Delegates to `rumqttc`'s own `mqttbytes::matches` rather than
/// reimplementing MQTT wildcard matching by hand - the edge cases (`#` must
/// be last and alone in its level, `+` matches exactly one level) are easy to
/// get subtly wrong, and `rumqttc` already ships a tested implementation.
pub fn topic_matches_filter(topic: &str, filter: &str) -> bool {
    rumqttc::matches(topic, filter)
}
