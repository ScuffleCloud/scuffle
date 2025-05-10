#![allow(clippy::all)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

//! Protobuf Compiled Definitions for Tinc

include!(concat!(env!("OUT_DIR"), "/tinc.rs"));

/// The raw protobuf file
pub const TINC_ANNOTATIONS: &str = include_str!("tinc/annotations.proto");
pub const TINC_ANNOTATIONS_PB_PATH: &str = concat!(env!("OUT_DIR"), "/tinc.annotations.pb");
pub const TINC_ANNOTATIONS_PB: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/tinc.annotations.pb"));
