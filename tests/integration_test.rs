#![allow(non_snake_case)]

use core::ffi::{c_char, c_float, c_void};
use core::time::Duration;
use log::error;
use mockall::predicate::*;
use mockall::*;
use rumqttc::{Client, Connection, MqttOptions, Publish, QoS};
use std::ffi::{CStr, CString};
use MStarPlayer_mqtt_rust_plugin::plugin_interface_v2::*;
use MStarPlayer_mqtt_rust_plugin::*;

#[automock]
pub trait Callbacks {
    fn list_players(
        player_name: *const c_char,
        callback: ListPlayersCallbackFunction,
        user_data: *const c_void,
    );
    fn play(player_name: *const c_char);
    fn stop(player_name: *const c_char);
    fn next(player_name: *const c_char);
    fn previous(player_name: *const c_char);
    fn list_tracks(
        player_name: *const c_char,
        callback: ListTracksCallbackFunction,
        user_data: *const c_void,
    );
    fn set_track_volume(player_name: *const c_char, track_name: *const c_char, volume: c_float);
}

extern "C" fn listPlayers(
    player_name: *const c_char,
    callback: ListPlayersCallbackFunction,
    user_data: *const c_void,
) {
    MockCallbacks::list_players(player_name, callback, user_data);
}

extern "C" fn play(player_name: *const c_char) {
    MockCallbacks::play(player_name);
}

extern "C" fn stop(player_name: *const c_char) {
    MockCallbacks::stop(player_name);
}

extern "C" fn next(player_name: *const c_char) {
    MockCallbacks::next(player_name);
}

extern "C" fn previous(player_name: *const c_char) {
    MockCallbacks::previous(player_name);
}

extern "C" fn listTracks(
    player_name: *const c_char,
    callback: ListTracksCallbackFunction,
    user_data: *const c_void,
) {
    MockCallbacks::list_tracks(player_name, callback, user_data);
}

extern "C" fn setTrackVolume(
    player_name: *const c_char,
    track_name: *const c_char,
    volume: c_float,
) {
    MockCallbacks::set_track_volume(player_name, track_name, volume);
}

/// Time to wait for MQTT messages.
static TIMEOUT: Duration = Duration::new(1, 0);

/// MQTT server to connect to for testing.
static SERVER: &str = "127.0.0.1";
static PORT: u16 = 1883;
static TOPIC_PREFIX: &str = "integration-test";

fn wait_for_publish(connection: &mut Connection) -> Option<Publish> {
    loop {
        match connection.recv_timeout(TIMEOUT) {
            Ok(Ok(rumqttc::Event::Incoming(rumqttc::Incoming::Publish(p)))) => {
                return Some(p);
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                error!("{}", e);
                return None;
            }
            Err(_) => {
                error!("Timeout waiting for MQTT message");
                return None;
            }
        }
    }
}

fn wait_for_no_publish(connection: &mut Connection) {
    loop {
        match connection.recv_timeout(TIMEOUT) {
            Ok(Ok(rumqttc::Event::Incoming(rumqttc::Incoming::Publish(_)))) => {
                panic!("Should not have received a message.");
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                error!("{}", e);
            }
            Err(_) => {
                break;
            }
        }
    }
}

fn publish_and_wait(client: &mut Client, topic: String, connection: &mut Connection) {
    client
        .publish(topic, QoS::AtLeastOnce, false, Vec::new())
        .unwrap();
    loop {
        match connection.recv_timeout(TIMEOUT) {
            Ok(Ok(rumqttc::Event::Incoming(rumqttc::Incoming::PubAck(_)))) => {
                break;
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                error!("{}", e);
            }
            Err(_) => {
                error!("Timeout waiting for publishing of MQTT message");
            }
        }
    }

    // Wait some extra time for the received message to be handled.
    std::thread::sleep(Duration::from_millis(10));
}

fn new_player_name_predicate(player_name: &CString) -> impl Fn(&*const c_char) -> bool {
    let player_name = player_name.clone();
    move |p: &*const c_char| {
        let p = unsafe { CStr::from_ptr(p.clone()) };

        p.to_bytes() == player_name.as_bytes()
    }
}

#[test]
fn mqtt_interaction() {
    let init = plugin_interface_v2::Init {
        listPlayers,
        play,
        stop,
        next,
        previous,
        listTracks,
        setTrackVolume,
    };
    mstarInit(&init);

    let input_configuration = format!("{SERVER}\n{PORT}\nMStarPlayer-MQTT-sut\n{TOPIC_PREFIX}");
    let input_configuration_raw = CString::new(input_configuration).unwrap();
    unsafe {
        mstarLoadConfiguration(input_configuration_raw.as_ptr());
    }

    let options = MqttOptions::new("MStarPlayer-MQTT-test", SERVER, PORT);

    let (mut client, mut connection) = Client::new(options, 10);

    client
        .subscribe(format!("{}/monitor/#", TOPIC_PREFIX), QoS::AtMostOnce)
        .unwrap();

    let player_name = CString::new("Test Player").unwrap();
    mstarPlayingStateChanged(player_name.as_ptr(), true);

    if let Some(p) = wait_for_publish(&mut connection) {
        assert_eq!(
            p.topic,
            format!("{TOPIC_PREFIX}/monitor/Test Player/playing")
        );
        assert!(p.payload.is_empty());
    }

    mstarNextEntrySelected(player_name.as_ptr());

    if let Some(p) = wait_for_publish(&mut connection) {
        assert_eq!(p.topic, format!("{TOPIC_PREFIX}/monitor/Test Player/next"));
        assert!(p.payload.is_empty());
    }

    mstarPreviousEntrySelected(player_name.as_ptr());

    if let Some(p) = wait_for_publish(&mut connection) {
        assert_eq!(
            p.topic,
            format!("{TOPIC_PREFIX}/monitor/Test Player/previous")
        );
        assert!(p.payload.is_empty());
    }

    mstarPlaylistEntrySelected(player_name.as_ptr(), 0, player_name.as_ptr(), 0.0);

    wait_for_no_publish(&mut connection);

    mstarPlaylistEntryDurationChanged(player_name.as_ptr(), 0, 0.0);

    wait_for_no_publish(&mut connection);

    mstarPlaylistEntryNameChanged(player_name.as_ptr(), 0, player_name.as_ptr());

    wait_for_no_publish(&mut connection);

    mstarTrackVolumeChanged(player_name.as_ptr(), player_name.as_ptr(), 0.0);

    wait_for_no_publish(&mut connection);

    mstarPositionChanged(player_name.as_ptr(), 0.0);

    if let Some(p) = wait_for_publish(&mut connection) {
        assert_eq!(
            p.topic,
            format!("{TOPIC_PREFIX}/monitor/Test Player/position")
        );
        assert_eq!(String::from_utf8(p.payload.to_vec()).unwrap(), "0");
    }

    {
        let player_name = player_name.clone();
        let ctx = MockCallbacks::play_context();
        ctx.expect()
            .once()
            .return_const(())
            .withf(new_player_name_predicate(&player_name));

        publish_and_wait(
            &mut client,
            format!("{TOPIC_PREFIX}/control/Test Player/play"),
            &mut connection,
        );
    }

    {
        let player_name = player_name.clone();
        let ctx = MockCallbacks::stop_context();
        ctx.expect()
            .once()
            .return_const(())
            .withf(new_player_name_predicate(&player_name));

        publish_and_wait(
            &mut client,
            format!("{TOPIC_PREFIX}/control/Test Player/stop"),
            &mut connection,
        );
    }

    {
        let player_name = player_name.clone();
        let ctx = MockCallbacks::next_context();
        ctx.expect()
            .once()
            .return_const(())
            .withf(new_player_name_predicate(&player_name));

        publish_and_wait(
            &mut client,
            format!("{TOPIC_PREFIX}/control/Test Player/next"),
            &mut connection,
        );
    }

    {
        let player_name = player_name.clone();
        let ctx = MockCallbacks::previous_context();
        ctx.expect()
            .once()
            .return_const(())
            .withf(new_player_name_predicate(&player_name));

        publish_and_wait(
            &mut client,
            format!("{TOPIC_PREFIX}/control/Test Player/previous"),
            &mut connection,
        );
    }

    mstarShutdown();
}
