# nest-mqtt

MQTT client module for the Nest framework.

**Crate path:** [`modules/crates/nest-mqtt`](../../modules/crates/nest-mqtt)

## Quick start

```rust
use nest_core::AppBuilder;
use nest_mqtt::{MqttConfig, MqttModule, MqttClientService, MqttQos};

#[tokio::main]
async fn main() -> nest_error::NestResult<()> {
    let built = AppBuilder::new()
        .module(MqttModule::new(MqttConfig::new(
            "broker.example.com",
            1883,
            "my-client",
        )))
        .build()?;

    let mqtt = built.context.service::<MqttClientService>()?;
    mqtt.publish("nest/example", b"hello".to_vec(), MqttQos::AtLeastOnce, false)
        .await?;
    Ok(())
}
```

## Runtime requirement

`MqttModule::configure` connects synchronously via a `block_on` helper (the
same pattern [`nest-data-postgres`](../nest-data-postgres/README.md) uses),
so `AppBuilder::build()` can be called from a plain synchronous context — no
`#[tokio::main]` is required just to register the module.
`MqttClientService`'s own methods (`publish`/`subscribe`/`disconnect`) are
async and need a Tokio runtime to `.await`.

## Configuration

```rust
use nest_mqtt::{LastWillConfig, MqttConfig, MqttQos};

MqttConfig::new("broker.example.com", 1883, "my-client")
    .with_credentials("user", "pass")
    .with_capacity(200)
    .with_last_will(LastWillConfig {
        topic: "nest/my-client/status".into(),
        payload: b"offline".to_vec(),
        qos: MqttQos::AtLeastOnce,
        retain: true,
    });
```

Or load from the `[mqtt]` config section:

```toml
[mqtt]
broker_host = "broker.example.com"
broker_port = 1883
client_id = "my-client"
```

```rust
use nest_mqtt::MqttConfig;

let config = MqttConfig::from_config_service(&config_service)?
    .unwrap_or_else(MqttConfig::default_local);
```

## Publishing and subscribing

```rust
use futures_util::StreamExt;
use nest_mqtt::MqttQos;

mqtt.publish("sensors/temp", b"21.5".to_vec(), MqttQos::AtLeastOnce, false)
    .await?;

let messages = mqtt.subscribe("sensors/#", MqttQos::AtLeastOnce).await?;
let mut messages = std::pin::pin!(messages);
while let Some(message) = messages.next().await {
    println!("{}: {:?}", message.topic, message.payload);
}
```

See [`publish_and_receive`](../../modules/crates/nest-mqtt/src/example.rs) in
the crate for a complete connect → subscribe → publish → receive example.

## Design notes

- The `EventLoop` is polled continuously in a dedicated background task,
  separate from any task calling `publish()`/`subscribe()` — required by
  `rumqttc` to avoid a self-deadlock once its internal channel fills.
- `rumqttc` types (`rumqttc::QoS`, etc.) are never exposed through the public
  API — `MqttQos` wraps them so consumers don't need a direct `rumqttc`
  dependency.
- Reconnection after a dropped connection is handled internally by `rumqttc`
  as long as the event loop keeps polling — no manual reconnect logic needed.

## Tests

All tests run automatically against a disposable `eclipse-mosquitto:2`
broker via `testcontainers` — no manual setup required:

```bash
cargo test -p nest-mqtt
```

## Module integration

```rust
pub const MQTT_MODULE_ID: ModuleId = ModuleId("nest-mqtt");
```

Registered via `MqttModule`; lookup with `ctx.service::<MqttClientService>()?`.

## Related

- [nest-data-postgres](../nest-data-postgres/README.md) — the
  `block_on`-in-`configure()` pattern this crate mirrors
- [phase-2-nest-mqtt-build](../plan/phase-2-nest-mqtt-build.md) —
  implementation plan
