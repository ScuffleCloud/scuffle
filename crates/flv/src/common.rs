use nutype_enum::nutype_enum;

nutype_enum! {
    pub enum AvMultitrackType(u8) {
        OneTrack = 0,
        ManyTracks = 1,
        ManyTracksManyCodecs = 2,
    }
}
