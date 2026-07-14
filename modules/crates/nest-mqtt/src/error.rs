//! Maps `rumqttc` errors to [`nest_error::NestError`].

use nest_error::NestError;

/// Converts a client-side send failure into a [`NestError`].
///
/// `rumqttc::ClientError` has exactly two variants (`Request`/`TryRequest`),
/// both meaning "the channel to the event loop is closed or full" - the error
/// itself carries no information about which operation (publish vs.
/// subscribe) triggered it, so the caller supplies the appropriate code
/// (e.g. [`crate::codes::NEST_MQTT_PUBLISH_FAILED`] or
/// [`crate::codes::NEST_MQTT_SUBSCRIBE_FAILED`]).
pub fn client_error_to_nest(code: &str, error: rumqttc::ClientError) -> NestError {
    NestError::network(error.to_string())
        .with_code(code)
        .with_source(error)
}

/// Converts an MQTT event-loop/connection failure into a [`NestError`].
///
/// All `rumqttc::ConnectionError` variants originate from `EventLoop::poll`
/// (transport, protocol, or connect-time failures), so these always map to
/// [`crate::codes::NEST_MQTT_CONNECT_FAILED`].
pub fn connection_error_to_nest(error: rumqttc::ConnectionError) -> NestError {
    NestError::network(error.to_string())
        .with_code(crate::codes::NEST_MQTT_CONNECT_FAILED)
        .with_source(error)
}
