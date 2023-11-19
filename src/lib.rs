#![allow(non_snake_case)]

use core::ffi::{c_char, c_double, c_float, c_int, c_void};
use log::debug;
use std::ffi::CString;

#[no_mangle]
pub extern "C" fn mstarPluginVersion() -> c_int {
    2
}

type ListPlayersCallbackFunction = extern "C" fn(*const c_char, *const c_void);
type ListPlayersFunction = extern "C" fn(*const c_char, ListPlayersCallbackFunction, *const c_void);
type PlayFunction = extern "C" fn(*const c_char);
type StopFunction = extern "C" fn(*const c_char);
type NextFunction = extern "C" fn(*const c_char);
type PreviousFunction = extern "C" fn(*const c_char);
type ListTracksCallbackFunction = extern "C" fn(*const c_char, *const c_char, *const c_void);
type ListTracksFunction = extern "C" fn(*const c_char, ListTracksCallbackFunction, *const c_void);
type SetTrackVolumeFunction = extern "C" fn(*const c_char, *const c_char, c_float);

#[repr(C)]
pub struct Init {
    listPlayers: ListPlayersFunction,
    play: PlayFunction,
    stop: StopFunction,
    next: NextFunction,
    previous: PreviousFunction,
    listTracks: ListTracksFunction,
    setTrackVolume: SetTrackVolumeFunction,
}

#[no_mangle]
pub extern "C" fn mstarInit(init: &Init) {
    env_logger::init();
    debug!("mstarInit");
}

#[no_mangle]
pub extern "C" fn mstarPlayingStateChanged(player_name: *const c_char, is_playing: bool) {
    debug!("mstarPlayingStateChanged");
}

#[no_mangle]
pub extern "C" fn mstarNextEntrySelected(player_name: *const c_char) {
    debug!("mstarNextEntrySelected");
}

#[no_mangle]
pub extern "C" fn mstarPreviousEntrySelected(player_name: *const c_char) {
    debug!("mstarPreviousEntrySelected");
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
}

#[no_mangle]
pub extern "C" fn mstarConfigure() {
    debug!("mstarConfigure");
}

#[no_mangle]
pub extern "C" fn mstarShutdown() {
    debug!("mstarShutdown");
}

#[no_mangle]
pub extern "C" fn mstarLoadConfiguration(configuration_text: *const c_char) {
    debug!("mstarLoadConfiguration");
}

#[no_mangle]
pub extern "C" fn mstarGetConfiguration() -> *const c_char {
    debug!("mstarGetConfiguration");
    let configuration = "";

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
