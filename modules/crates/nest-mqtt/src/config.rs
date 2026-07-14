//! MQTT connection configuration.

#[cfg(feature = "config")]
use nest_config::ConfigService;
#[cfg(feature = "config")]
use nest_error::NestResult;
#[cfg(feature = "config")]
use serde::Deserialize;

/// Default MQTT broker host for local development.
pub const DEFAULT_BROKER_HOST: &str = "127.0.0.1";
/// Default MQTT broker port.
pub const DEFAULT_BROKER_PORT: u16 = 1883;
/// Default MQTT client id when none is configured.
pub const DEFAULT_CLIENT_ID: &str = "nest-mqtt";
/// Default keep-alive interval, in seconds.
pub const DEFAULT_KEEP_ALIVE_SECS: u16 = 30;
/// Default channel capacity between the client handle and the polling event loop.
pub const DEFAULT_CAPACITY: usize = 100;

/// Connection options for the MQTT client.
#[derive(Debug, Clone)]
pub struct MqttConfig {
    /// Broker hostname or IP address.
    pub broker_host: String,
    /// Broker TCP port.
    pub broker_port: u16,
    /// Client identifier presented to the broker.
    pub client_id: String,
    /// Keep-alive interval, in seconds.
    pub keep_alive_secs: u16,
    /// Username for broker authentication, if required.
    pub username: Option<String>,
    /// Password for broker authentication, if required.
    pub password: Option<String>,
    /// Channel capacity between the client handle and the polling event loop.
    pub capacity: usize,
    /// Last-Will-and-Testament, published by the broker if this client disconnects uncleanly.
    pub last_will: Option<LastWillConfig>,
}

/// Last-Will-and-Testament configuration.
#[derive(Debug, Clone)]
pub struct LastWillConfig {
    /// Topic the will message is published to.
    pub topic: String,
    /// Will message payload.
    pub payload: Vec<u8>,
    /// Delivery guarantee for the will message.
    pub qos: MqttQos,
    /// Whether the broker retains the will message.
    pub retain: bool,
}

/// Provider-agnostic QoS wrapper so callers don't need a direct `rumqttc`
/// dependency just to specify QoS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqttQos {
    /// Fire-and-forget; the message may be lost.
    AtMostOnce,
    /// Delivered at least once; duplicates are possible.
    AtLeastOnce,
    /// Delivered exactly once.
    ExactlyOnce,
}

impl MqttConfig {
    /// Creates config with sensible defaults (no auth, no LWT, 30s keep-alive,
    /// capacity 100).
    pub fn new(
        broker_host: impl Into<String>,
        broker_port: u16,
        client_id: impl Into<String>,
    ) -> Self {
        Self {
            broker_host: broker_host.into(),
            broker_port,
            client_id: client_id.into(),
            keep_alive_secs: DEFAULT_KEEP_ALIVE_SECS,
            username: None,
            password: None,
            capacity: DEFAULT_CAPACITY,
            last_will: None,
        }
    }

    /// Creates config pointed at a local broker on the default MQTT port.
    pub fn default_local() -> Self {
        Self::new(DEFAULT_BROKER_HOST, DEFAULT_BROKER_PORT, DEFAULT_CLIENT_ID)
    }

    /// Sets broker username/password.
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Sets the channel capacity between the client handle and the polling event loop.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Sets the Last-Will-and-Testament.
    pub fn with_last_will(mut self, last_will: LastWillConfig) -> Self {
        self.last_will = Some(last_will);
        self
    }

    /// Builds config from `MQTT_BROKER_HOST`, `MQTT_BROKER_PORT`, and
    /// `MQTT_CLIENT_ID`, falling back to local defaults for any that are unset
    /// or unparsable.
    pub fn from_env() -> Self {
        let host =
            std::env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| DEFAULT_BROKER_HOST.to_string());
        let port = std::env::var("MQTT_BROKER_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(DEFAULT_BROKER_PORT);
        let client_id =
            std::env::var("MQTT_CLIENT_ID").unwrap_or_else(|_| DEFAULT_CLIENT_ID.to_string());
        Self::new(host, port, client_id)
    }

    /// Loads the `[mqtt]` section from a config service.
    ///
    /// Returns `Ok(None)` if no `[mqtt]` section is present or it's explicitly
    /// disabled (`enabled = false`), matching
    /// [`nest_ai_ollama`](https://docs.rs/nest-ai-ollama)'s
    /// `OllamaConfig::from_config_service` convention. Returns `Err` only if a
    /// present section fails to deserialize.
    #[cfg(feature = "config")]
    pub fn from_config_service(service: &ConfigService) -> NestResult<Option<Self>> {
        let Ok(section) = service.section::<MqttSection>("mqtt") else {
            return Ok(None);
        };
        if !section.enabled {
            return Ok(None);
        }
        Ok(Some(section.into_config()))
    }
}

/// `[mqtt]` config section.
#[cfg(feature = "config")]
#[derive(Debug, Clone, Deserialize)]
struct MqttSection {
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default = "default_broker_host")]
    broker_host: String,
    #[serde(default = "default_broker_port")]
    broker_port: u16,
    #[serde(default = "default_client_id")]
    client_id: String,
    #[serde(default = "default_keep_alive_secs")]
    keep_alive_secs: u16,
    username: Option<String>,
    password: Option<String>,
    #[serde(default = "default_capacity")]
    capacity: usize,
}

#[cfg(feature = "config")]
impl MqttSection {
    fn into_config(self) -> MqttConfig {
        let mut config = MqttConfig::new(self.broker_host, self.broker_port, self.client_id);
        config.keep_alive_secs = self.keep_alive_secs;
        config.capacity = self.capacity;
        if let (Some(username), Some(password)) = (self.username, self.password) {
            config = config.with_credentials(username, password);
        }
        config
    }
}

#[cfg(feature = "config")]
fn default_enabled() -> bool {
    true
}

#[cfg(feature = "config")]
fn default_broker_host() -> String {
    DEFAULT_BROKER_HOST.to_string()
}

#[cfg(feature = "config")]
fn default_broker_port() -> u16 {
    DEFAULT_BROKER_PORT
}

#[cfg(feature = "config")]
fn default_client_id() -> String {
    DEFAULT_CLIENT_ID.to_string()
}

#[cfg(feature = "config")]
fn default_keep_alive_secs() -> u16 {
    DEFAULT_KEEP_ALIVE_SECS
}

#[cfg(feature = "config")]
fn default_capacity() -> usize {
    DEFAULT_CAPACITY
}
