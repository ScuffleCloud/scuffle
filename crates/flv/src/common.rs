//! Common types used in the FLV format.

use nutype_enum::nutype_enum;

nutype_enum! {
    /// Type of multitrack.
    ///
    /// Used by both audio and video pipeline.
    pub enum AvMultitrackType(u8) {
        /// One track.
        OneTrack = 0,
        /// Many tracks with one codec.
        ManyTracks = 1,
        /// Many tracks with many codecs.
        ManyTracksManyCodecs = 2,
    }
}
