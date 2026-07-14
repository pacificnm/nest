//! Stable error codes for nest-mqtt.

/// MQTT configuration error.
pub const NEST_MQTT_CONFIG: &str = "NEST_MQTT_CONFIG";

/// MQTT broker connection failure.
pub const NEST_MQTT_CONNECT_FAILED: &str = "NEST_MQTT_CONNECT_FAILED";

/// MQTT publish failure.
pub const NEST_MQTT_PUBLISH_FAILED: &str = "NEST_MQTT_PUBLISH_FAILED";

/// MQTT subscribe failure.
pub const NEST_MQTT_SUBSCRIBE_FAILED: &str = "NEST_MQTT_SUBSCRIBE_FAILED";
