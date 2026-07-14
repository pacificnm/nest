//! `MqttModule`: registers [`crate::MqttClientService`] and a shutdown [`Lifecycle`].

use std::sync::Arc;

use nest_core::{AppBuilder, AppContext, Lifecycle, Module, ModuleId, NestResult};

use crate::client::MqttClient;
use crate::config::MqttConfig;
use crate::service::MqttClientService;

/// Module id for [`MqttModule`].
pub const MQTT_MODULE_ID: ModuleId = ModuleId("nest-mqtt");

/// Registers an MQTT connection with [`MqttClientService`].
pub struct MqttModule {
    config: MqttConfig,
}

impl MqttModule {
    /// Creates a module that connects using the given config.
    pub fn new(config: MqttConfig) -> Self {
        Self { config }
    }
}

impl Module for MqttModule {
    fn id(&self) -> ModuleId {
        MQTT_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let client = crate::runtime::block_on(MqttClient::connect(&self.config))?;
        app.register_service(MqttClientService::new(client.clone()))?;
        app.register_lifecycle(MqttLifecycle { client });
        Ok(())
    }
}

/// Disconnects the MQTT client cleanly on application shutdown.
struct MqttLifecycle {
    client: MqttClient,
}

impl Lifecycle for MqttLifecycle {
    fn on_shutdown(&mut self, _ctx: Arc<AppContext>) -> NestResult<()> {
        crate::runtime::block_on(self.client.disconnect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_registers_mqtt_client_service() {
        // MqttClient::connect() only sets up the client handle and spawns the
        // event-loop-polling task - the actual TCP connect happens lazily
        // inside that background task's first poll(), not here - so this
        // doesn't need a reachable broker, matching HttpClientModule's own
        // registration test (no live server needed there either).
        let config = MqttConfig::new("127.0.0.1", 1883, "test-module-registration");
        let built = AppBuilder::new()
            .module(MqttModule::new(config))
            .build()
            .unwrap();
        assert!(built.context.has_service::<MqttClientService>());
    }
}
