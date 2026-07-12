# Phase 2 Task Spec — Build `nest-mqtt`

**Repo:** `pacificnm/nest` (framework repo)
**New crate:** `modules/crates/nest-mqtt`
**Branch:** `feature/nest-mqtt-v1` (confirm branch/PR workflow before opening)
**Pinned dependency:** `rumqttc = "0.25"` (confirmed current on crates.io as of this
writing — check crates.io yourself before pinning if time has passed; do not
guess a version that doesn't exist)

## Ground truth (verified against the real repo — read before starting)

- **Pattern to mirror:** `core/crates/nest-http-client` (`config.rs`, `module.rs`, `service.rs`) — same file layout, same module-registration idiom.
- `Module` trait (`nest-core/src/module.rs`) is **synchronous**: `fn configure(&self, app: &mut AppBuilder) -> NestResult<()>`. There is no async `configure`. Any async setup (connecting) must run inside a `block_on` call, exactly like `nest-data-postgres`'s `module.rs` does (`block_on(PostgresConnection::connect_named(...))`). Copy that crate's `runtime.rs` `block_on` helper into `nest-mqtt` — do not reinvent it differently.
- `Lifecycle` trait (`nest-core/src/lifecycle.rs`): `fn on_startup(&mut self, ctx: Arc<AppContext>) -> NestResult<()>` and `fn on_shutdown(...)`, both sync, both with default no-op bodies. Register via `AppBuilder::register_lifecycle`.
- `AppBuilder` methods you'll use: `register_service<T: Service>(&mut self, service: T) -> NestResult<()>`, `register_lifecycle<L: Lifecycle + 'static>(&mut self, handler: L) -> &mut Self`.
- `NestError` constructors available (`nest-error/src/error.rs`): `NestError::network(msg)`, `NestError::config(msg)`, `.with_code(...)`, `.with_source(...)`. Use `network` for connect/publish/subscribe failures, `config` for bad `MqttConfig`.
- `rumqttc` 0.25 API (confirmed from crates.io/docs.rs): `MqttOptions::new(client_id, host, port)`, `AsyncClient::new(mqtt_options, capacity) -> (AsyncClient, EventLoop)`. **The `EventLoop` must be polled continuously in a dedicated task** — the crate's own docs warn that if the task calling `publish()`/`subscribe()` is the same task driving `eventloop.poll()`, it can self-deadlock once the bounded channel fills. This is the single most important correctness constraint for this crate — design around it explicitly, don't discover it during debugging.

---

## Design

```
modules/crates/nest-mqtt/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── codes.rs        # NEST_MQTT_* error code constants
    ├── error.rs         # rumqttc error -> NestError mapping
    ├── config.rs         # MqttConfig
    ├── client.rs         # MqttClient: owns AsyncClient + spawns the EventLoop-polling task
    ├── message.rs         # MqttMessage type, topic-filter matching helper
    ├── service.rs         # MqttClientService (the registered Service, wraps MqttClient)
    ├── module.rs         # MqttModule (Module + Lifecycle impls)
    └── runtime.rs         # block_on helper, copied from nest-data-postgres's version
```

### `codes.rs`

```rust
//! Stable error codes for nest-mqtt.
pub const NEST_MQTT_CONFIG: &str = "NEST_MQTT_CONFIG";
pub const NEST_MQTT_CONNECT_FAILED: &str = "NEST_MQTT_CONNECT_FAILED";
pub const NEST_MQTT_PUBLISH_FAILED: &str = "NEST_MQTT_PUBLISH_FAILED";
pub const NEST_MQTT_SUBSCRIBE_FAILED: &str = "NEST_MQTT_SUBSCRIBE_FAILED";
```

### `config.rs`

```rust
/// Connection options for [`crate::MqttClient`].
#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub broker_host: String,
    pub broker_port: u16,
    pub client_id: String,
    pub keep_alive_secs: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    /// Channel capacity between the client handle and the polling event loop.
    pub capacity: usize,
    /// Last-Will-and-Testament, published by the broker if this client disconnects uncleanly.
    pub last_will: Option<LastWillConfig>,
}

#[derive(Debug, Clone)]
pub struct LastWillConfig {
    pub topic: String,
    pub payload: Vec<u8>,
    pub qos: MqttQos,
    pub retain: bool,
}

/// Provider-agnostic QoS wrapper so callers don't need a direct `rumqttc` dependency
/// just to specify QoS — mirrors how other Nest modules avoid leaking the underlying
/// crate's types into the public API where practical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MqttQos {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}
```

Add `MqttConfig::new(broker_host, broker_port, client_id)` (defaults: `keep_alive_secs: 30`, `capacity: 100`, no auth, no LWT), builder methods `with_credentials`, `with_capacity`, `with_last_will`, and `from_env`/`from_config_service` following the exact pattern in `nest-ai-ollama`'s `OllamaConfig::from_config_service` (reads a `[mqtt]` TOML section via `ConfigService`, falls back to a sensible local default if absent — check that function's actual body before copying its shape, since this spec is describing the pattern from memory, not quoting it verbatim).

### `error.rs`

```rust
pub fn mqtt_error_to_nest(context: &str, error: impl std::fmt::Display) -> nest_error::NestError {
    nest_error::NestError::network(format!("{context}: {error}"))
        .with_code(crate::codes::NEST_MQTT_CONNECT_FAILED) // caller overrides code per call site
}
```

Write real conversions for `rumqttc::ClientError` (publish/subscribe failures →
`NEST_MQTT_PUBLISH_FAILED` / `NEST_MQTT_SUBSCRIBE_FAILED`) and
`rumqttc::ConnectionError` (event-loop/connect failures →
`NEST_MQTT_CONNECT_FAILED`) as **two separate functions**, not one generic
one — check `rumqttc::ClientError` and `rumqttc::ConnectionError`'s actual
variants via `cargo doc` before writing the match arms; do not guess variant
names.

### `message.rs`

```rust
/// One message received from a subscribed topic.
#[derive(Debug, Clone)]
pub struct MqttMessage {
    pub topic: String,
    pub payload: Vec<u8>,
    pub retained: bool,
}

/// Returns true if `topic` matches an MQTT topic filter (`+` and `#` wildcards).
///
/// VERIFY BEFORE WRITING: check whether `rumqttc` or its `mqttbytes` dependency
/// already exposes a topic-matching function (e.g. under `rumqttc::mqttbytes` or
/// a `matches` module) before implementing this by hand. Only write a manual
/// implementation if nothing usable is exported — MQTT wildcard matching has
/// edge cases (`#` must be last and alone in its level, `+` matches exactly one
/// level) that are easy to get subtly wrong; reusing a tested implementation
/// beats writing a new one.
pub fn topic_matches_filter(topic: &str, filter: &str) -> bool {
    todo!("implement per the note above, after checking for an existing helper")
}
```

### `client.rs` — the core design

This is the piece that isn't a mechanical copy of `nest-ai-ollama`'s pattern —
work through it carefully:

```rust
use std::sync::Arc;
use rumqttc::{AsyncClient, EventLoop, Event, Incoming, MqttOptions};
use tokio::sync::broadcast;

/// Owns the rumqttc client handle and drives its event loop in a background task.
#[derive(Clone)]
pub struct MqttClient {
    client: AsyncClient,
    // Every incoming Publish is broadcast here; `subscribe()` callers filter by topic.
    incoming: broadcast::Sender<crate::message::MqttMessage>,
}

impl MqttClient {
    /// Connects and spawns the event-loop-polling task. Must be called from
    /// within a Tokio runtime context (the caller — `MqttModule::configure`,
    /// via `block_on` — provides this).
    pub async fn connect(config: &crate::config::MqttConfig) -> nest_error::NestResult<Self> {
        let mut opts = MqttOptions::new(&config.client_id, &config.broker_host, config.broker_port);
        opts.set_keep_alive(std::time::Duration::from_secs(config.keep_alive_secs as u64));
        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            opts.set_credentials(user, pass);
        }
        if let Some(lwt) = &config.last_will {
            // CHECK: confirm rumqttc 0.25's LastWill constructor signature via docs.rs
            // before writing this line — do not guess field order.
            opts.set_last_will(rumqttc::LastWill::new(
                &lwt.topic,
                lwt.payload.clone(),
                to_rumqttc_qos(lwt.qos),
                lwt.retain,
            ));
        }

        let (client, eventloop) = AsyncClient::new(opts, config.capacity);
        let (tx, _rx) = broadcast::channel(config.capacity.max(16));

        // The critical piece: poll the EventLoop in its own dedicated task, forever,
        // separate from any task that calls publish()/subscribe() on `client`.
        let tx_clone = tx.clone();
        tokio::spawn(run_event_loop(eventloop, tx_clone));

        Ok(Self { client, incoming: tx })
    }

    pub async fn publish(
        &self,
        topic: &str,
        payload: impl Into<Vec<u8>>,
        qos: crate::config::MqttQos,
        retain: bool,
    ) -> nest_error::NestResult<()> {
        self.client
            .publish(topic, to_rumqttc_qos(qos), retain, payload.into())
            .await
            .map_err(|e| crate::error::publish_error_to_nest(e))
    }

    pub async fn subscribe(
        &self,
        topic_filter: &str,
        qos: crate::config::MqttQos,
    ) -> nest_error::NestResult<impl futures_util::Stream<Item = crate::message::MqttMessage>> {
        self.client
            .subscribe(topic_filter, to_rumqttc_qos(qos))
            .await
            .map_err(|e| crate::error::subscribe_error_to_nest(e))?;

        let filter = topic_filter.to_string();
        let rx = self.incoming.subscribe();
        Ok(tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |msg| {
                let filter = filter.clone();
                async move {
                    match msg {
                        Ok(m) if crate::message::topic_matches_filter(&m.topic, &filter) => Some(m),
                        _ => None,
                    }
                }
            }))
    }

    pub async fn disconnect(&self) -> nest_error::NestResult<()> {
        self.client
            .disconnect()
            .await
            .map_err(|e| crate::error::publish_error_to_nest(e))
    }
}

async fn run_event_loop(
    mut eventloop: EventLoop,
    tx: broadcast::Sender<crate::message::MqttMessage>,
) {
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                let _ = tx.send(crate::message::MqttMessage {
                    topic: publish.topic,
                    payload: publish.payload.to_vec(),
                    retained: publish.retain,
                });
            }
            Ok(_) => continue, // acks, pings, etc. — not surfaced to subscribers
            Err(err) => {
                tracing::warn!(error = %err, "mqtt event loop error, continuing");
                // Do not break here — rumqttc reconnects internally on most transport
                // errors. VERIFY this assumption against rumqttc 0.25's actual
                // reconnect behavior before shipping; if it does NOT auto-reconnect,
                // this loop needs explicit backoff-and-retry logic instead of a bare
                // `continue`.
            }
        }
    }
}

fn to_rumqttc_qos(qos: crate::config::MqttQos) -> rumqttc::QoS {
    match qos {
        crate::config::MqttQos::AtMostOnce => rumqttc::QoS::AtMostOnce,
        crate::config::MqttQos::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
        crate::config::MqttQos::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
    }
}
```

Add `tokio-stream` (for `BroadcastStream`) and `futures-util` to `Cargo.toml`
if not already available transitively — check first.

**Explicit unresolved item, flagged rather than guessed:** whether `rumqttc`
0.25 auto-reconnects after a transport-level error inside `eventloop.poll()`,
or whether the caller must detect the error and reconnect manually. This
changes `run_event_loop`'s error-handling branch materially. Check the
`rumqttc` changelog/docs for this before finalizing — do not ship a silent
`continue` if the library actually requires manual reconnect, since that
would produce a event loop that spins doing nothing after a dropped
connection instead of recovering.

### `service.rs` — `MqttClientService`

Thin wrapper exposing `MqttClient`'s methods as the registered `Service`,
same relationship `HttpClientService` has to `reqwest::Client` — mostly
delegation, add doc comments per item (crate should use `#![deny(missing_docs)]`
matching `nest-data-postgres`'s convention).

### `module.rs` — `MqttModule`

Mirror `HttpClientModule`/`OllamaModule`'s shape exactly:

```rust
pub const MQTT_MODULE_ID: ModuleId = ModuleId("nest-mqtt");

pub struct MqttModule {
    config: MqttConfig,
}

impl MqttModule {
    pub fn new(config: MqttConfig) -> Self { Self { config } }
}

impl Module for MqttModule {
    fn id(&self) -> ModuleId { MQTT_MODULE_ID }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let client = crate::runtime::block_on(MqttClient::connect(&self.config))?;
        app.register_service(MqttClientService::new(client.clone()))?;
        app.register_lifecycle(MqttLifecycle { client });
        Ok(())
    }
}

struct MqttLifecycle {
    client: MqttClient,
}

impl Lifecycle for MqttLifecycle {
    fn on_shutdown(&mut self, _ctx: Arc<AppContext>) -> NestResult<()> {
        crate::runtime::block_on(self.client.disconnect())
    }
}
```

---

## Tests (`testcontainers-rs`, per the Phase 1 convention — no `#[ignore]`, no manual setup)

- `client.rs`: connect to a real (containerized) Mosquitto, publish on one client, subscribe on another, assert the message arrives. Use `testcontainers::GenericImage::new("eclipse-mosquitto", "2")` with a minimal anonymous-access config (`WaitFor::message_on_stdout` matching Mosquitto's startup log line — check the actual log line the image emits before writing the `WaitFor` condition, don't guess the exact string).
- `module.rs`: `MqttModule::new(config)` registers `MqttClientService`, same shape as `HttpClientModule`'s own test (`assert!(built.context.has_service::<MqttClientService>())`).
- LWT test: connect a client with a configured last-will, subscribe to the LWT topic from a second client, force-kill the first connection (drop it without a clean disconnect — check how to simulate this against a real broker, e.g. killing the underlying TCP connection rather than calling `disconnect()`), assert the LWT message arrives on the second client.

**Acceptance for Phase 2:** `cargo test -p nest-mqtt` passes with Docker
running and zero manual steps; `cargo doc -p nest-mqtt` builds cleanly under
`#![deny(missing_docs)]`; a standalone example (mirroring
`nest-http-client`'s `example.rs`) connects, publishes, and receives a
message against a real broker.

## Explicit "do not" list

- Do not let the task calling `publish()`/`subscribe()` also drive `eventloop.poll()` — that's the specific deadlock risk called out above.
- Do not guess `rumqttc` 0.25's exact type/method signatures where flagged above (`LastWill::new`, reconnect behavior, `ClientError`/`ConnectionError` variants) — check `cargo doc` or docs.rs for the pinned version and fix the sketch code accordingly.
- Do not leak `rumqttc` types (`rumqttc::QoS`, etc.) through `nest-mqtt`'s public API — wrap them (`MqttQos`, etc.) so downstream consumers (Sparrow) don't need a direct `rumqttc` dependency, consistent with how other Nest modules avoid leaking their underlying HTTP/DB crate's types.
