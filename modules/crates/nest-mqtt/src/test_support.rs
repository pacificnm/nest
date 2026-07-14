//! Test-only helpers for spinning up a disposable Mosquitto broker.
#![cfg(test)]

use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage};

/// Holds a running Mosquitto container alive for the test's duration;
/// dropping it stops the container.
pub struct TestBroker {
    _container: ContainerAsync<GenericImage>,
    pub host: String,
    pub port: u16,
}

/// Starts a disposable Mosquitto broker and returns its host/port.
///
/// The stock `eclipse-mosquitto:2` image's default
/// `/mosquitto/config/mosquitto.conf` already sets `listener 1883` +
/// `allow_anonymous true` - confirmed by inspecting a running container
/// directly, no custom config file/mount is needed.
pub async fn start_broker() -> TestBroker {
    let container = GenericImage::new("eclipse-mosquitto", "2")
        .with_exposed_port(1883.tcp())
        // Mosquitto logs everything to stderr, not stdout (confirmed by
        // running the image directly and checking each stream separately -
        // stdout is empty). The version number in "mosquitto version X.Y.Z
        // running" changes across image updates, so match on the stable
        // "running" suffix rather than a pinned version string.
        .with_wait_for(WaitFor::message_on_stderr("running"))
        .start()
        .await
        .expect("failed to start mosquitto testcontainer");
    let host = container
        .get_host()
        .await
        .expect("container host")
        .to_string();
    let port = container
        .get_host_port_ipv4(1883)
        .await
        .expect("container port");
    TestBroker {
        _container: container,
        host,
        port,
    }
}
