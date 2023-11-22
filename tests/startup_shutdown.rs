#![allow(non_snake_case)]

mod callbacks;

use callbacks::*;
use std::ffi::CString;
use MStarPlayer_mqtt_rust_plugin::*;

#[test]
fn startup_shutdown() {
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

    let empty_raw_string = CString::new("").unwrap();

    // Player can call all these functions. They must not cause a crash.
    mstarPlayingStateChanged(empty_raw_string.as_ptr(), true);
    mstarNextEntrySelected(empty_raw_string.as_ptr());
    mstarPreviousEntrySelected(empty_raw_string.as_ptr());
    mstarPlaylistEntrySelected(empty_raw_string.as_ptr(), 0, empty_raw_string.as_ptr(), 0.0);
    mstarPlaylistEntryDurationChanged(empty_raw_string.as_ptr(), 0, 0.0);
    mstarPlaylistEntryNameChanged(empty_raw_string.as_ptr(), 0, empty_raw_string.as_ptr());
    mstarTrackVolumeChanged(empty_raw_string.as_ptr(), empty_raw_string.as_ptr(), 0.0);
    mstarPositionChanged(empty_raw_string.as_ptr(), 0.0);

    mstarShutdown();
}
