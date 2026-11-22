//! Complete-clip MIDI CRUD and a small Bitwig instrument adapter.
pub mod adapter;
pub mod api;
#[doc(hidden)]
pub mod automation;
pub mod control;
pub mod controller;
pub mod cues;
pub mod decode;
pub mod editor;
pub mod entity;
pub mod export_audio;
pub mod export_midi;
pub mod file_evidence;
pub mod files;
pub mod frame;
pub mod instrument;
pub mod kontakt_process;
pub mod launcher;
pub mod midi;
pub mod mix;
pub mod operations;
pub mod point;
pub mod project_address;
pub mod protocol;
pub mod randomize;
pub mod session;

pub mod selection;
