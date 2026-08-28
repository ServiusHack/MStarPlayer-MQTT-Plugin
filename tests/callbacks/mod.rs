use core::ffi::{c_char, c_float, c_int, c_void};
use mockall::*;
use MStarPlayer_mqtt_plugin::plugin_interface_v4::*;

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
    fn select(player_name: *const c_char, playlist_index: c_int);
    fn list_tracks(
        player_name: *const c_char,
        callback: ListTracksCallbackFunction,
        user_data: *const c_void,
    );
    fn set_track_volume(player_name: *const c_char, track_name: *const c_char, volume: c_float);
}

pub extern "C" fn listPlayers(
    player_name: *const c_char,
    callback: ListPlayersCallbackFunction,
    user_data: *const c_void,
) {
    MockCallbacks::list_players(player_name, callback, user_data);
}

pub extern "C" fn play(player_name: *const c_char) {
    MockCallbacks::play(player_name);
}

pub extern "C" fn stop(player_name: *const c_char) {
    MockCallbacks::stop(player_name);
}

pub extern "C" fn next(player_name: *const c_char) {
    MockCallbacks::next(player_name);
}

pub extern "C" fn previous(player_name: *const c_char) {
    MockCallbacks::previous(player_name);
}

pub extern "C" fn select(player_name: *const c_char, playlist_index: c_int) {
    MockCallbacks::select(player_name, playlist_index);
}

pub extern "C" fn listTracks(
    player_name: *const c_char,
    callback: ListTracksCallbackFunction,
    user_data: *const c_void,
) {
    MockCallbacks::list_tracks(player_name, callback, user_data);
}

pub extern "C" fn setTrackVolume(
    player_name: *const c_char,
    track_name: *const c_char,
    volume: c_float,
) {
    MockCallbacks::set_track_volume(player_name, track_name, volume);
}
