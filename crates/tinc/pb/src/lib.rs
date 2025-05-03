#![allow(clippy::all)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

//! Protobuf Compiled Definitions for Tinc

include!(concat!(env!("OUT_DIR"), "/tinc.rs"));

/// The raw protobuf file
pub const TINC_ANNOTATIONS: &str = include_str!("tinc/annotations.proto");
