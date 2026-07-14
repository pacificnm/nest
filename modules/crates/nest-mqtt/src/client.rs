//! MQTT client: owns the `rumqttc` handle and drives its event loop.

use futures_util::StreamExt;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions};
use tokio::sync::broadcast;

use crate::codes::{NEST_MQTT_PUBLISH_FAILED, NEST_MQTT_SUBSCRIBE_FAILED};
use crate::config::{MqttConfig, MqttQos};
use crate::error::{client_error_to_nest, connection_error_to_nest};
use crate::message::{topic_matches_filter, MqttMessage};

/// Owns the `rumqttc` client handle and drives its event loop in a background task.
#[derive(Clone)]
pub struct MqttClient {
    client: AsyncClient,
    /// Every incoming `Publish` is broadcast here; `subscribe()` callers filter by topic.
    incoming: broadcast::Sender<MqttMessage>,
}

impl MqttClient {
    /// Connects and spawns the event-loop-polling task.
    ///
    /// **Must not** be called from the same task that will call
    /// [`Self::publish`]/[`Self::subscribe`] without `.await`ing this first -
    /// the returned client and the spawned polling task must run
    /// independently, or the bounded channel between them can fill and
    /// deadlock (see the crate-level docs).
    pub async fn connect(config: &MqttConfig) -> nest_error::NestResult<Self> {
        let mut opts = MqttOptions::new(&config.client_id, &config.broker_host, config.broker_port);
        opts.set_keep_alive(std::time::Duration::from_secs(u64::from(
            config.keep_alive_secs,
        )));
        if let (Some(user), Some(pass)) = (&config.username, &config.password) {
            opts.set_credentials(user, pass);
        }
        if let Some(lwt) = &config.last_will {
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

        Ok(Self {
            client,
            incoming: tx,
        })
    }

    /// Publishes a message to `topic`.
    pub async fn publish(
        &self,
        topic: &str,
        payload: impl Into<Vec<u8>>,
        qos: MqttQos,
        retain: bool,
    ) -> nest_error::NestResult<()> {
        self.client
            .publish(topic, to_rumqttc_qos(qos), retain, payload.into())
            .await
            .map_err(|error| client_error_to_nest(NEST_MQTT_PUBLISH_FAILED, error))
    }

    /// Subscribes to `topic_filter`, returning a stream of matching messages.
    pub async fn subscribe(
        &self,
        topic_filter: &str,
        qos: MqttQos,
    ) -> nest_error::NestResult<impl futures_util::Stream<Item = MqttMessage>> {
        self.client
            .subscribe(topic_filter, to_rumqttc_qos(qos))
            .await
            .map_err(|error| client_error_to_nest(NEST_MQTT_SUBSCRIBE_FAILED, error))?;

        let filter = topic_filter.to_string();
        let rx = self.incoming.subscribe();
        Ok(
            tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(move |msg| {
                let filter = filter.clone();
                async move {
                    match msg {
                        Ok(m) if topic_matches_filter(&m.topic, &filter) => Some(m),
                        _ => None,
                    }
                }
            }),
        )
    }

    /// Disconnects cleanly from the broker.
    pub async fn disconnect(&self) -> nest_error::NestResult<()> {
        self.client
            .disconnect()
            .await
            .map_err(|error| client_error_to_nest(NEST_MQTT_PUBLISH_FAILED, error))
    }
}

async fn run_event_loop(mut eventloop: EventLoop, tx: broadcast::Sender<MqttMessage>) {
    loop {
        match eventloop.poll().await {
            Ok(Event::Incoming(Incoming::Publish(publish))) => {
                let _ = tx.send(MqttMessage {
                    topic: publish.topic,
                    payload: publish.payload.to_vec(),
                    retained: publish.retain,
                });
            }
            Ok(_) => continue, // acks, pings, etc. - not surfaced to subscribers
            Err(err) => {
                // rumqttc's own EventLoop::poll docs confirm it reconnects
                // internally as long as polling continues, so this is
                // intentionally not a `break` - just log and keep polling.
                let nest_error = connection_error_to_nest(err);
                tracing::warn!(error = %nest_error, "mqtt event loop error, reconnecting");
            }
        }
    }
}

fn to_rumqttc_qos(qos: MqttQos) -> rumqttc::QoS {
    match qos {
        MqttQos::AtMostOnce => rumqttc::QoS::AtMostOnce,
        MqttQos::AtLeastOnce => rumqttc::QoS::AtLeastOnce,
        MqttQos::ExactlyOnce => rumqttc::QoS::ExactlyOnce,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn publish_and_subscribe_round_trip() {
        let broker = crate::test_support::start_broker().await;

        let subscriber_config = MqttConfig::new(&broker.host, broker.port, "test-subscriber");
        let subscriber = MqttClient::connect(&subscriber_config).await.unwrap();
        let messages = subscriber
            .subscribe("nest/mqtt/test", MqttQos::AtLeastOnce)
            .await
            .unwrap();
        // `impl Stream` return types aren't `Unpin` by default, and
        // `StreamExt::next()` requires it - pin the stream on the stack
        // rather than changing subscribe()'s public return type.
        let mut messages = std::pin::pin!(messages);

        // subscribe() only enqueues the SUBSCRIBE packet on the event loop's
        // channel and returns - it doesn't wait for the broker's SUBACK - so
        // give the event loop a moment to actually process it before
        // publishing, or the message can be sent before the broker considers
        // us subscribed.
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        let publisher_config = MqttConfig::new(&broker.host, broker.port, "test-publisher");
        let publisher = MqttClient::connect(&publisher_config).await.unwrap();
        publisher
            .publish(
                "nest/mqtt/test",
                b"hello".to_vec(),
                MqttQos::AtLeastOnce,
                false,
            )
            .await
            .unwrap();

        let received = tokio::time::timeout(std::time::Duration::from_secs(5), messages.next())
            .await
            .expect("timed out waiting for the published message")
            .expect("message stream ended unexpectedly");

        assert_eq!(received.topic, "nest/mqtt/test");
        assert_eq!(received.payload, b"hello");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn last_will_delivered_on_unclean_disconnect() {
        let broker = crate::test_support::start_broker().await;

        let observer_config = MqttConfig::new(&broker.host, broker.port, "test-lwt-observer");
        let observer = MqttClient::connect(&observer_config).await.unwrap();
        let messages = observer
            .subscribe("nest/mqtt/lwt", MqttQos::AtLeastOnce)
            .await
            .unwrap();
        let mut messages = std::pin::pin!(messages);
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

        // MqttClient::connect() spawns the event-loop-polling task with
        // `tokio::spawn` and discards the JoinHandle - the task's lifetime is
        // NOT tied to the returned MqttClient, so simply dropping the client
        // handle does not close the underlying connection. To simulate an
        // unclean disconnect (no MQTT DISCONNECT packet, just the network
        // connection dying) without adding a test-only backdoor to
        // MqttClient's public API, connect the "victim" client on its own
        // dedicated runtime, then kill that runtime outright - this aborts
        // the background task (and the TCP socket it owns) mid-flight.
        let victim_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        let victim_config = MqttConfig::new(&broker.host, broker.port, "test-lwt-victim")
            .with_last_will(crate::config::LastWillConfig {
                topic: "nest/mqtt/lwt".to_string(),
                payload: b"gone".to_vec(),
                qos: MqttQos::AtLeastOnce,
                retain: false,
            });
        let _victim_client = victim_runtime
            .spawn(async move { MqttClient::connect(&victim_config).await })
            .await
            .expect("victim connect task panicked")
            .expect("victim client failed to connect");

        // Give the victim's CONNECT (with its Will) time to be acknowledged
        // by the broker before we pull the rug out from under it.
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        victim_runtime.shutdown_background();

        let received = tokio::time::timeout(std::time::Duration::from_secs(10), messages.next())
            .await
            .expect("timed out waiting for the LWT message")
            .expect("message stream ended unexpectedly");

        assert_eq!(received.topic, "nest/mqtt/lwt");
        assert_eq!(received.payload, b"gone");
    }
}
