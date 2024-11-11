use log::{debug, error, warn};
use rumqttc::{Client, MqttOptions, Publish, QoS};
use std::ffi::CString;
use std::sync::Mutex;
use std::thread;

use crate::{CONFIG, INIT};

pub static CLIENT: Mutex<Option<Client>> = Mutex::new(None);
static JOIN_HANDLE: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);

fn handle_message(p: Publish) {
    let topic_parts: Vec<&str> = p.topic.split('/').collect();

    if topic_parts.len() != 4 {
        warn!("Received malformed topic (require 4 levels): {}", p.topic);
        return;
    }

    let received_prefix = topic_parts[0];
    let received_scope = topic_parts[1];
    let received_player_name = topic_parts[2];
    let received_command = topic_parts[3];

    {
        let config = CONFIG.read().unwrap();
        let expected_prefix = &config
            .as_ref()
            .expect("CONFIG should be set by mstarInit")
            .topic_prefix;
        if received_prefix != expected_prefix {
            warn!(
                "Received malformed topic (expected '{}' prefix): {}",
                expected_prefix, p.topic
            );
            return;
        }
    }

    {
        let expected_scope = "control";
        if received_scope != expected_scope {
            warn!(
                "Received malformed topic (expected '{}' scope): {}",
                expected_scope, p.topic
            );
            return;
        }
    }

    let received_player_name = match CString::new(received_player_name) {
        Ok(s) => s,
        Err(e) => {
            error!(
                "Received topic with invalid ({}) player name: {}",
                e, p.topic
            );
            return;
        }
    };

    let init = INIT.read().unwrap();
    let init = init.as_ref().expect("INIT should be set by mstarInit");

    match received_command {
        "play" => {
            (init.play)(received_player_name.as_ptr());
        }
        "stop" => {
            (init.stop)(received_player_name.as_ptr());
        }
        "next" => {
            (init.next)(received_player_name.as_ptr());
        }
        "previous" => {
            (init.previous)(received_player_name.as_ptr());
        }
        _ => {
            warn!("Received topic with unknown command: {}", p.topic);
        }
    }
}

/// (Re-)establish MQTT connection.
pub fn setup() {
    // First end previous MQTT connection.
    teardown();

    debug!("Setting up MQTT connection");

    let config = CONFIG.read().unwrap();
    let config = &config.as_ref().expect("CONFIG should be set by mstarInit");
    let options = MqttOptions::new(
        config.client_name.clone(),
        config.server.clone(),
        config.port,
    );
    let topic_prefix = config.topic_prefix.clone();

    let (client, mut connection) = Client::new(options, 10);

    *CLIENT.lock().unwrap() = Some(client.clone());
    *JOIN_HANDLE.lock().unwrap() = Some(thread::spawn(move || {
        loop {
            client
                .subscribe(format!("{}/control/#", topic_prefix), QoS::AtMostOnce)
                .unwrap();

            for (i, notification) in connection.iter().enumerate() {
                match notification {
                    Ok(rumqttc::Event::Incoming(rumqttc::Incoming::Publish(p))) => {
                        handle_message(p);
                    }
                    Ok(rumqttc::Event::Outgoing(rumqttc::Outgoing::Disconnect)) => {
                        return;
                    }
                    Ok(notify) => {
                        debug!("{i}. Notification = {notify:?}");
                    }
                    Err(error) => {
                        error!("{:#?}", error);
                        break;
                    }
                }
            }
            // slow down error rate
            std::thread::sleep(std::time::Duration::from_secs(1));

            connection.eventloop.clean();
        }
    }));
}

/// Gracefully end MQTT connection, if one exists.
pub fn teardown() {
    debug!("Tearing down MQTT connection");

    let mut client = CLIENT.lock().unwrap();

    match client.as_mut() {
        Some(client) => {
            if let Err(e) = client.disconnect() {
                error!("{}", e);
            }
        }
        None => {
            debug!("Not disconnecting from MQTT since we're not connected.");
            return;
        }
    }

    // Destroy MQTT client.
    *client = None;

    // Wait for connection thread to exit.
    let handle = JOIN_HANDLE
        .lock()
        .unwrap()
        .take()
        .expect("thread join handle should exist when client was created");
    if handle.join().is_err() {
        error!("error while joining MQTT connection thread");
    }
}
