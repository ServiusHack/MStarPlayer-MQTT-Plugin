#![allow(non_snake_case)]

mod mqtt;
pub mod plugin_interface_v2;

use core::ffi::{c_char, c_double, c_int};
use log::{debug, error, info, warn};
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
    let config = config.as_ref();
    let config = match config {
        Some(config) => config,
        None => {
            info!("Not publishing message since plugin wasn't configured yet.");
            return;
        }
    };
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
            if let Err(e) = client.try_publish(topic, QoS::AtLeastOnce, false, payload) {
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
    _player_name: *const c_char,
    _playlist_index: c_int,
    _playlist_entry_name: *const c_char,
    _duration: c_double,
) {
    debug!("mstarPlaylistEntrySelected");
}

#[no_mangle]
pub extern "C" fn mstarPlaylistEntryDurationChanged(
    _player_name: *const c_char,
    _playlist_index: c_int,
    _duration: c_double,
) {
    debug!("mstarPlaylistEntryDurationChanged");
}

#[no_mangle]
pub extern "C" fn mstarPlaylistEntryNameChanged(
    _player_name: *const c_char,
    _playlist_index: c_int,
    _playlist_entry_name: *const c_char,
) {
    debug!("mstarPlaylistEntryNameChanged");
}

#[no_mangle]
pub extern "C" fn mstarTrackVolumeChanged(
    _player_name: *const c_char,
    _track_name: *const c_char,
    _volume: c_double,
) {
    debug!("mstarTrackVolumeChanged");
}

#[no_mangle]
pub extern "C" fn mstarPositionChanged(player_name: *const c_char, position: c_double) {
    debug!("mstarPositionChanged");

    let payload = position.to_string().into_bytes();
    publish(player_name, "position", payload);
}

slint::slint! {
    import { LineEdit, SpinBox, StandardButton, VerticalBox, GroupBox } from "std-widgets.slint";
    export component MainWindow inherits Window {
        in property<string> default-server;
        in property<string> server <=> server-edit.text;

        in property<int> port <=> portEdit.value;

        in property<string> default-client-name;
        in property<string> client-name <=> client-name-edit.text;

        in property<string> default-topic-prefix;
        in property<string> topic-prefix <=> topic-prefix-edit.text;

        callback save();
        callback abort();

        title: "M*Player MQTT Plugin";

        VerticalBox {
            alignment: start;

            Text {
                text: "Configure MQTT connection";
                font-size: 24px;
                horizontal-alignment: center;
            }

            HorizontalLayout {
                GroupBox {
                    title: "Server";
                    horizontal-stretch: 1;
                    server-edit := LineEdit {
                        placeholder-text: root.default-server;
                    }
                }

                GroupBox {
                    title: "Port";
                    portEdit := SpinBox {
                        enabled: true;
                        minimum: 1;
                        maximum: 65535;
                    }
                }
            }

            GroupBox {
                title: "Client Name";
                client-name-edit := LineEdit {
                    placeholder-text: root.default-client-name;
                }
            }

            GroupBox {
                title: "Topic Prefix";
                topic-prefix-edit := LineEdit {
                    placeholder-text: root.default-topic-prefix;
                }
            }

            HorizontalLayout {
                alignment: center;
                StandardButton {
                    kind: apply;
                    enabled: server-edit.text != "" &&
                        client-name-edit.text != "" &&
                        topic-prefix-edit.text != "";
                    clicked => {
                        root.save();
                    }
                }
                StandardButton {
                    kind: cancel;
                    clicked => {
                        root.abort();
                    }
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn mstarConfigure() {
    debug!("mstarConfigure");
    let window = MainWindow::new().unwrap();

    {
        let default_config = Configuration::default();
        let config = CONFIG.read().unwrap();
        let config = config.as_ref().unwrap_or(&default_config);

        window.set_server(config.server.clone().into());
        window.set_port(config.port as i32);
        window.set_client_name(config.client_name.clone().into());
        window.set_topic_prefix(config.topic_prefix.clone().into());

        window.set_default_server(default_config.server.into());
        window.set_default_client_name(default_config.client_name.into());
        window.set_default_topic_prefix(default_config.topic_prefix.into());
    }

    let weak = window.as_weak();
    window.on_save(move || {
        let window = weak.unwrap();

        let config = Configuration {
            server: window.get_server().into(),
            port: window.get_port() as u16,
            client_name: window.get_client_name().into(),
            topic_prefix: window.get_topic_prefix().into(),
        };
        *CONFIG.write().unwrap() = Some(config);
        mqtt::setup();

        window.hide().unwrap();
    });

    let weak = window.as_weak();
    window.on_abort(move || weak.unwrap().hide().unwrap());

    window.run().unwrap();
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

    if configuration_text.is_empty() {
        warn!("Configuration was empty.");
        mqtt::teardown();
        *CONFIG.write().unwrap() = None;
        return;
    }

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
    let configuration = match config.as_ref() {
        None => String::new(),
        Some(config) => {
            format!(
                "{}\n{}\n{}\n{}",
                config.server, config.port, config.client_name, config.topic_prefix
            )
        }
    };

    CString::new(configuration).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn mstarFreeConfigurationText(configuration_text: *const c_char) {
    debug!("mstarFreeConfigurationText");
    unsafe {
        let _configuration = CString::from_raw(configuration_text as *mut i8);
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
        let input_configuration = "127.0.0.1\n1\nclient\ntopic";
        let input_configuration_raw = CString::new(input_configuration).unwrap();
        unsafe {
            mstarLoadConfiguration(input_configuration_raw.as_ptr());
        }

        let configuration_raw = mstarGetConfiguration();
        let configuration = unsafe { CStr::from_ptr(configuration_raw) };
        assert_eq!(configuration.to_str().unwrap(), input_configuration);
        mstarFreeConfigurationText(configuration_raw);
    }
}
