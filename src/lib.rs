#![allow(non_snake_case)]

mod mqtt;
mod plugin_interface_v2;

use core::ffi::{c_char, c_double, c_int};
use log::{debug, error, warn};
use plugin_interface_v2::*;
use rumqttc::QoS;
use std::ffi::{CStr, CString};
use std::sync::RwLock;

struct Configuration {
    server: String,
    port: u16,
    client_name: String,
    topic_prefix: String,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            server: "127.0.0.1".into(),
            port: 1883,
            client_name: "MStarPlayer".into(),
            topic_prefix: "MStarPlayer".into(),
        }
    }
}

static CONFIG: RwLock<Option<Configuration>> = RwLock::new(None);

static INIT: RwLock<Option<Init>> = RwLock::new(None);

#[no_mangle]
pub extern "C" fn mstarPluginVersion() -> c_int {
    2
}

#[no_mangle]
pub extern "C" fn mstarInit(init: &Init) {
    env_logger::init();
    debug!("mstarInit");

    *INIT.write().unwrap() = Some(init.clone());
}

fn publish(player_name: *const c_char, event: &str, payload: Vec<u8>) {
    let config = CONFIG.read().unwrap();
    let config = &config.as_ref().expect("CONFIG should be set by mstarInit");
    let prefix = &config.topic_prefix;

    let player_name = unsafe { CStr::from_ptr(player_name) };
    let player_name = match player_name.to_str() {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let topic = format!("{prefix}/monitor/{player_name}/{event}");

    let mut client = mqtt::CLIENT.lock().unwrap();
    match client.as_mut() {
        Some(client) => {
            if let Err(e) = client.publish(topic, QoS::AtLeastOnce, false, payload) {
                error!("{}", e);
            }
        }
        None => {
            warn!("No MQTT connection yet.");
        }
    }
}

#[no_mangle]
pub extern "C" fn mstarPlayingStateChanged(player_name: *const c_char, is_playing: bool) {
    debug!("mstarPlayingStateChanged");

    let state = if is_playing { "playing" } else { "stopped" };
    publish(player_name, state, Vec::new());
}

#[no_mangle]
pub extern "C" fn mstarNextEntrySelected(player_name: *const c_char) {
    debug!("mstarNextEntrySelected");

    publish(player_name, "next", Vec::new());
}

#[no_mangle]
pub extern "C" fn mstarPreviousEntrySelected(player_name: *const c_char) {
    debug!("mstarPreviousEntrySelected");

    publish(player_name, "previous", Vec::new());
}

#[no_mangle]
pub extern "C" fn mstarPlaylistEntrySelected(
    player_name: *const c_char,
    playlist_index: c_int,
    playlist_entry_name: *const c_char,
    duration: c_double,
) {
    debug!("mstarPlaylistEntrySelected");
}

#[no_mangle]
pub extern "C" fn mstarPlaylistEntryDurationChanged(
    player_name: *const c_char,
    playlist_index: c_int,
    duration: c_double,
) {
    debug!("mstarPlaylistEntryDurationChanged");
}

#[no_mangle]
pub extern "C" fn mstarPlaylistEntryNameChanged(
    player_name: *const c_char,
    playlist_index: c_int,
    playlist_entry_name: *const c_char,
) {
    debug!("mstarPlaylistEntryNameChanged");
}

#[no_mangle]
pub extern "C" fn mstarTrackVolumeChanged(
    player_name: *const c_char,
    track_name: *const c_char,
    volume: c_double,
) {
    debug!("mstarTrackVolumeChanged");
}

#[no_mangle]
pub extern "C" fn mstarPositionChanged(player_name: *const c_char, position: c_double) {
    debug!("mstarPositionChanged");

    let payload = position.to_string().into_bytes();
    publish(player_name, "position", payload);
}

#[no_mangle]
pub extern "C" fn mstarConfigure() {
    debug!("mstarConfigure");

    *CONFIG.write().unwrap() = Some(Default::default());
    mqtt::setup();
}

#[no_mangle]
pub extern "C" fn mstarShutdown() {
    debug!("mstarShutdown");

    mqtt::teardown();
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mstarLoadConfiguration(configuration_text: *const c_char) {
    debug!("mstarLoadConfiguration");

    let configuration_text = unsafe { CStr::from_ptr(configuration_text) };
    let configuration_text = match configuration_text.to_str() {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            return;
        }
    };

    let parts: Vec<&str> = configuration_text.split('\n').collect();

    if parts.len() != 4 {
        error!("Malformed configuration.");
        return;
    }

    let port: u16 = match parts[1].parse() {
        Ok(port) => port,
        Err(e) => {
            error!("Malformed port in configuration: {e}");
            return;
        }
    };

    *CONFIG.write().unwrap() = Some(Configuration {
        server: parts[0].into(),
        port,
        client_name: parts[2].into(),
        topic_prefix: parts[3].into(),
    });

    mqtt::setup();
}

#[no_mangle]
pub extern "C" fn mstarGetConfiguration() -> *const c_char {
    debug!("mstarGetConfiguration");

    let config = CONFIG.read().unwrap();
    let config = &config.as_ref().expect("CONFIG should be set by mstarInit");
    let configuration = format!(
        "{}\n{}\n{}\n{}",
        config.server, config.port, config.client_name, config.topic_prefix
    );

    CString::new(configuration).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn mstarFreeConfigurationText(configuration_text: *const c_char) {
    debug!("mstarFreeConfigurationText");
    unsafe {
        let configuration = CString::from_raw(configuration_text as *mut i8);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version() {
        let result = mstarPluginVersion();
        assert_eq!(result, 2);
    }

    #[test]
    fn configuration() {
        let configuration = mstarGetConfiguration();
        mstarFreeConfigurationText(configuration);
    }
}
