//! Adaptation of PluginInterfaceV2.h of M*Player for Rust
//!
//! The types in this module reflect the types specified in the PluginInterfaceV2.h header.
//! They are required to be ABI compatible with M*Player.
//! **Exception:** Signatures of functions that must be implemented by the plugin aren't repeated here.

use core::ffi::{c_char, c_float, c_void};

pub type ListPlayersCallbackFunction = extern "C" fn(*const c_char, *const c_void);
pub type ListPlayersFunction =
    extern "C" fn(*const c_char, ListPlayersCallbackFunction, *const c_void);
pub type PlayFunction = extern "C" fn(*const c_char);
pub type StopFunction = extern "C" fn(*const c_char);
pub type NextFunction = extern "C" fn(*const c_char);
pub type PreviousFunction = extern "C" fn(*const c_char);
pub type ListTracksCallbackFunction = extern "C" fn(*const c_char, *const c_char, *const c_void);
pub type ListTracksFunction =
    extern "C" fn(*const c_char, ListTracksCallbackFunction, *const c_void);
pub type SetTrackVolumeFunction = extern "C" fn(*const c_char, *const c_char, c_float);

#[repr(C)]
#[derive(Clone)]
pub struct Init {
    pub listPlayers: ListPlayersFunction,
    pub play: PlayFunction,
    pub stop: StopFunction,
    pub next: NextFunction,
    pub previous: PreviousFunction,
    pub listTracks: ListTracksFunction,
    pub setTrackVolume: SetTrackVolumeFunction,
}
