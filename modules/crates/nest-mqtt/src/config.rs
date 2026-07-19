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
    /// TLS transport configuration. `None` means a plaintext TCP connection.
    pub tls: Option<TlsConfig>,
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

/// TLS transport configuration for the MQTT connection (Phase 12, Issue
/// 12.1). Mirrors [`LastWillConfig`]'s shape: plain owned bytes, not
/// paths — callers read cert/key files themselves (e.g. via
/// [`TlsConfig::from_ca_file`]), keeping this crate free of any opinion
/// about where certs live on disk.
///
/// Maps directly onto `rumqttc`'s `TlsConfiguration::Simple` (verified
/// against the pinned `rumqttc = "0.25"` — actually `0.25.1` — source
/// directly, not guessed): `ca` is the PEM-encoded CA certificate used to
/// verify the broker's certificate, and `client_auth` is an optional
/// `(cert, key)` PEM pair for mutual TLS. rumqttc's rustls-backed TLS
/// support (`use-rustls-no-provider`) is part of its `default =
/// ["use-rustls"]` feature set, so no `Cargo.toml` feature change was
/// needed to use it here.
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// PEM-encoded CA certificate bytes, used to verify the broker's certificate.
    pub ca_cert: Vec<u8>,
    /// Optional `(client certificate, client key)` PEM bytes pair for mutual TLS.
    pub client_auth: Option<(Vec<u8>, Vec<u8>)>,
}

impl TlsConfig {
    /// Builds a `TlsConfig` from a PEM-encoded CA certificate's bytes, with
    /// no client certificate (server-authenticated TLS only — the common
    /// case for a self-hosted broker with a self-signed CA).
    pub fn new(ca_cert: impl Into<Vec<u8>>) -> Self {
        Self {
            ca_cert: ca_cert.into(),
            client_auth: None,
        }
    }

    /// Reads the CA certificate from `path` (PEM format).
    pub fn from_ca_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        Ok(Self::new(std::fs::read(path)?))
    }

    /// Sets a client certificate/key pair for mutual TLS.
    pub fn with_client_auth(mut self, cert: impl Into<Vec<u8>>, key: impl Into<Vec<u8>>) -> Self {
        self.client_auth = Some((cert.into(), key.into()));
        self
    }
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
            tls: None,
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

    /// Enables TLS for this connection. Callers must also point
    /// `broker_port` at the broker's TLS listener (e.g. Mosquitto's
    /// conventional `8883`, not the plaintext `1883`) — this method only
    /// configures the transport, not the port.
    pub fn with_tls(mut self, tls: TlsConfig) -> Self {
        self.tls = Some(tls);
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
        section.into_config().map(Some)
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
    /// Path to a PEM-encoded CA certificate. Presence enables TLS —
    /// pair with a `broker_port` pointed at the broker's TLS listener
    /// (e.g. Mosquitto's conventional `8883`).
    tls_ca_file: Option<String>,
    /// Path to a PEM-encoded client certificate, for mutual TLS.
    tls_client_cert_file: Option<String>,
    /// Path to a PEM-encoded client key, for mutual TLS.
    tls_client_key_file: Option<String>,
}

#[cfg(feature = "config")]
impl MqttSection {
    fn into_config(self) -> NestResult<MqttConfig> {
        let mut config = MqttConfig::new(self.broker_host, self.broker_port, self.client_id);
        config.keep_alive_secs = self.keep_alive_secs;
        config.capacity = self.capacity;
        if let (Some(username), Some(password)) = (self.username, self.password) {
            config = config.with_credentials(username, password);
        }
        if let Some(ca_file) = self.tls_ca_file {
            let mut tls = TlsConfig::from_ca_file(&ca_file).map_err(|error| {
                nest_error::NestError::unknown(format!(
                    "failed to read tls_ca_file {ca_file}: {error}"
                ))
            })?;
            if let (Some(cert_file), Some(key_file)) =
                (self.tls_client_cert_file, self.tls_client_key_file)
            {
                let cert = std::fs::read(&cert_file).map_err(|error| {
                    nest_error::NestError::unknown(format!(
                        "failed to read tls_client_cert_file {cert_file}: {error}"
                    ))
                })?;
                let key = std::fs::read(&key_file).map_err(|error| {
                    nest_error::NestError::unknown(format!(
                        "failed to read tls_client_key_file {key_file}: {error}"
                    ))
                })?;
                tls = tls.with_client_auth(cert, key);
            }
            config = config.with_tls(tls);
        }
        Ok(config)
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

#[cfg(all(test, feature = "config"))]
mod tests {
    use nest_config::{ConfigDocument, ConfigSource, LoadedConfig};

    use super::*;

    fn config_service(input: &str) -> ConfigService {
        let document = ConfigDocument::parse_toml(input).expect("valid toml");
        let loaded = LoadedConfig {
            document,
            source: ConfigSource::SearchDefaults,
            path: None,
        };
        ConfigService::new(loaded)
    }

    fn write_temp_file(name: &str, contents: &[u8]) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);

        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("nest-mqtt-test-{}-{unique}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(name);
        std::fs::write(&path, contents).expect("write temp file");
        path
    }

    #[test]
    fn from_config_service_enables_tls_when_tls_ca_file_is_set() {
        let ca_path = write_temp_file("ca.pem", b"fake ca cert");
        let toml = format!(
            "[mqtt]\nbroker_host = \"broker.example\"\nbroker_port = 8883\ntls_ca_file = \"{}\"\n",
            ca_path.display()
        );

        let config = MqttConfig::from_config_service(&config_service(&toml))
            .expect("parse should succeed")
            .expect("section should be present");

        let tls = config.tls.expect("tls should be Some");
        assert_eq!(tls.ca_cert, b"fake ca cert");
        assert_eq!(tls.client_auth, None);
    }

    #[test]
    fn from_config_service_enables_mutual_tls_when_client_cert_and_key_are_set() {
        let ca_path = write_temp_file("ca.pem", b"fake ca cert");
        let cert_path = write_temp_file("client.pem", b"fake client cert");
        let key_path = write_temp_file("client.key", b"fake client key");
        let toml = format!(
            "[mqtt]\ntls_ca_file = \"{}\"\ntls_client_cert_file = \"{}\"\ntls_client_key_file = \"{}\"\n",
            ca_path.display(),
            cert_path.display(),
            key_path.display()
        );

        let config = MqttConfig::from_config_service(&config_service(&toml))
            .expect("parse should succeed")
            .expect("section should be present");

        let tls = config.tls.expect("tls should be Some");
        assert_eq!(
            tls.client_auth,
            Some((b"fake client cert".to_vec(), b"fake client key".to_vec()))
        );
    }

    #[test]
    fn from_config_service_leaves_tls_none_when_tls_ca_file_is_absent() {
        let config = MqttConfig::from_config_service(&config_service(
            "[mqtt]\nbroker_host = \"broker.example\"\n",
        ))
        .expect("parse should succeed")
        .expect("section should be present");

        assert!(config.tls.is_none());
    }

    #[test]
    fn from_config_service_errors_when_tls_ca_file_does_not_exist() {
        let error = MqttConfig::from_config_service(&config_service(
            "[mqtt]\ntls_ca_file = \"/nonexistent/ca.pem\"\n",
        ))
        .expect_err("a missing ca file should be an error, not a silent None");

        assert!(error.to_string().contains("tls_ca_file"));
    }
}

#[cfg(feature = "config")]
fn default_keep_alive_secs() -> u16 {
    DEFAULT_KEEP_ALIVE_SECS
}

#[cfg(feature = "config")]
fn default_capacity() -> usize {
    DEFAULT_CAPACITY
}
