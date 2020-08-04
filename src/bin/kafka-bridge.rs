#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use kafka_bridge::kafka;
#[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
use kafka_bridge::kafka::SASLConfig;
use std::sync::mpsc;
use std::{env, process, thread, time};

// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Configuration via Environmental Variables
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
struct Configuration {
    pub kafka_brokers: Vec<String>,
    pub kafka_topic: String,
    pub kafka_group: String,
    pub kafka_partition: i32,
    pub pubnub_host: String,
    pub pubnub_channel: String,
    pub pubnub_channel_root: String,
    pub publish_key: String,
    pub subscribe_key: String,
    pub secret_key: String,
    #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
    pub sasl_cfg: SASLConfig,
}

fn environment_variables() -> Configuration {
    Configuration {
        kafka_brokers: fetch_env_var("KAFKA_BROKERS")
            .split(',')
            .map(std::string::ToString::to_string)
            .collect(),
        kafka_topic: fetch_env_var("KAFKA_TOPIC"),
        kafka_group: fetch_env_var("KAFKA_GROUP"),
        kafka_partition: fetch_env_var("KAFKA_PARTITION")
            .parse::<i32>()
            .unwrap(),
        pubnub_host: "psdsn.pubnub.com:80".into(),
        pubnub_channel: fetch_env_var("PUBNUB_CHANNEL"),
        pubnub_channel_root: fetch_env_var("PUBNUB_CHANNEL_ROOT"),
        publish_key: fetch_env_var("PUBNUB_PUBLISH_KEY"),
        subscribe_key: fetch_env_var("PUBNUB_SUBSCRIBE_KEY"),
        secret_key: fetch_env_var("PUBNUB_SECRET_KEY"),
        #[cfg(feature = "sasl-plain")]
        sasl_cfg: SASLConfig {
            username: fetch_env_var("SASL_USERNAME"),
            password: fetch_env_var("SASL_PASSWORD"),
        },
        #[cfg(feature = "sasl-ssl")]
        sasl_cfg: SASLConfig {
            username: fetch_env_var("SASL_USERNAME"),
            password: fetch_env_var("SASL_PASSWORD"),
            ca_location: fetch_env_var("SSL_CA_LOCATION"),
            certificate_location: fetch_env_var("SSL_CERTIFICATE_LOCATION"),
            key_location: fetch_env_var("SSL_KEY_LOCATION"),
            key_password: fetch_env_var("SSL_KEY_PASSWORD"),
        },
    }
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let channel = if self.pubnub_channel_root.is_empty() {
            self.pubnub_channel.to_string()
        } else {
            format!(
                "{root}.{channel}",
                channel = self.pubnub_channel,
                root = self.pubnub_channel_root
            )
        };

        let protocol = "https";
        let domain = "www.pubnub.com";
        let url = "/docs/console";
        write!(f,
            "{proto}://{domain}/{url}?channel={channel}&sub={sub}&pub={pub}",
            proto=protocol,
            domain=domain,
            url=url,
            channel=channel,
            sub=self.subscribe_key,
            pub=self.publish_key,
        )
    }
}

fn fetch_env_var(name: &str) -> String {
    if let Ok(value) = env::var(name) {
        value
    } else {
        eprintln!("Missing '{}' Environmental Variable", name);
        process::exit(1);
    }
}

#[allow(clippy::too_many_lines)]
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
// Main Loop
// =-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
fn main() {
    // Async Channels
    let (kafka_message_tx, pubnub_publish_rx) = mpsc::channel();
    let (pubnub_message_tx, kafka_publish_rx) = mpsc::channel();

    // Receive messages from PubNub
    // Saves messages into MPSC for Kafka Producer Thread
    let pubnub_subscriber_thread = thread::Builder::new()
        .name("PubNub Subscriber Thread".into())
        .spawn(move || loop {
            use kafka_bridge::pubnub;

            let config = environment_variables();
            let host = &config.pubnub_host;
            let root = &config.pubnub_channel_root;
            let channel = &config.pubnub_channel;
            let subscribe_key = &config.subscribe_key;
            let secret_key = &config.secret_key;
            let agent = "kafka-bridge";

            let mut pubnub = match pubnub::SubscribeClient::new(
                host,
                root,
                channel,
                subscribe_key,
                secret_key,
                agent,
            ) {
                Ok(pubnub) => pubnub,
                Err(_error) => {
                    thread::sleep(time::Duration::new(1, 0));
                    continue;
                }
            };

            loop {
                let message = match pubnub.next_message() {
                    Ok(message) => message,
                    Err(_error) => continue,
                };
                pubnub_message_tx
                    .send(message)
                    .expect("KAFKA mpsc::channel channel write");
            }
        });

    // Send messages to PubNub
    // Receives messages from MPSC from Kafka and Publishes to PubNub
    let pubnub_publisher_thread = thread::Builder::new()
        .name("PubNub Publisher Thread".into())
        .spawn(move || loop {
            use kafka_bridge::pubnub;

            let config = environment_variables();
            let host = &config.pubnub_host;
            let root = &config.pubnub_channel_root;
            let publish_key = &config.publish_key;
            let subscribe_key = &config.subscribe_key;
            let secret_key = &config.secret_key;
            let agent = "kafka-bridge";

            let mut pubnub = match pubnub::PublishClient::new(
                host,
                root,
                publish_key,
                subscribe_key,
                secret_key,
                agent,
            ) {
                Ok(pubnub) => pubnub,
                Err(_error) => {
                    thread::sleep(time::Duration::new(1, 0));
                    continue;
                }
            };

            // Message Receiver Loop
            loop {
                let message: kafka::Message =
                    pubnub_publish_rx.recv().expect("MPSC Channel Receiver");
                let channel = &message.topic;
                let data = &message.data;

                // Retry Loop on Failure
                loop {
                    match pubnub.publish(channel, data) {
                        Ok(_timetoken) => break,
                        Err(_error) => {
                            thread::sleep(time::Duration::new(1, 0))
                        }
                    };
                }
            }
        });

    // Send messages to Kafka
    // Reads MPSC from PubNub Subscriptions and Sends to Kafka
    let kafka_publisher_thread = thread::Builder::new()
        .name("KAFKA Publisher Thread".into())
        .spawn(move || loop {
            let config = environment_variables();

            #[cfg(not(any(feature = "sasl-plain", feature = "sasl-ssl")))]
            let kafka = kafka::PublishClient::new(
                &config.kafka_brokers,
                &config.kafka_topic,
            );

            #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
            let kafka = kafka::PublishClient::new_with_sasl(
                &config.kafka_brokers,
                &config.kafka_topic,
                &config.sasl_cfg,
            );

            let mut kafka = match kafka {
                Ok(kafka) => kafka,
                Err(_error) => {
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }
            };

            loop {
                let message: kafka_bridge::pubnub::Message =
                    kafka_publish_rx.recv().expect("MPSC Channel Receiver");
                match kafka.produce(&message.data) {
                    Ok(()) => {}
                    Err(_error) => {
                        thread::sleep(time::Duration::from_millis(1000));
                    }
                };
            }
        });

    // Receive messages from Kafka
    // Consumes messages on Kafka topic and sends to MPSC PubNub Publisher
    let kafka_subscriber_thread = thread::Builder::new()
        .name("KAFKA Subscriber Thread".into())
        .spawn(move || loop {
            let config = environment_variables();
            #[cfg(not(any(feature = "sasl-plain", feature = "sasl-ssl")))]
            let kafka = kafka::SubscribeClient::new(
                &config.kafka_brokers,
                kafka_message_tx.clone(),
                &config.kafka_topic,
                &config.kafka_group,
                config.kafka_partition,
            );

            #[cfg(any(feature = "sasl-plain", feature = "sasl-ssl"))]
            let kafka = kafka::SubscribeClient::new_with_sasl(
                &config.kafka_brokers,
                kafka_message_tx.clone(),
                &config.kafka_topic,
                &config.kafka_group,
                config.kafka_partition,
                &config.sasl_cfg,
            );

            let mut kafka = match kafka {
                Ok(kafka) => kafka,
                Err(error) => {
                    println!("Retrying Consumer Connection {:?}", error);
                    thread::sleep(time::Duration::from_millis(1000));
                    continue;
                }
            };

            // Send KAFKA Messages to pubnub_publish_rx via kafka_message_tx
            kafka.consume().expect("Error consuming Kafka messages");
        });

    // Print Follow-on Instructions
    let config = environment_variables();
    println!("{{\"info\":\"Dashboard: {}\"}}", config);

    // The Threads Gather
    pubnub_subscriber_thread
        .expect("PubNub Subscriber thread builder join handle")
        .join()
        .expect("Joining PubNub Subscriber Thread");
    pubnub_publisher_thread
        .expect("PubNub Publisher thread builder join handle")
        .join()
        .expect("Joining PubNub Publisher Thread");
    kafka_publisher_thread
        .expect("KAFKA Publisher thread builder join handle")
        .join()
        .expect("Joining KAFKA Publisher Thread");
    kafka_subscriber_thread
        .expect("KAFKA Subscriber thread builder join handle")
        .join()
        .expect("Joining KAFKA Subscriber Thread");
}
