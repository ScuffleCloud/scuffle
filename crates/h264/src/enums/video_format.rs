use nutype_enum::nutype_enum;

nutype_enum! {
    /// The `VideoFormat` is a nutype enum for `video_format` as defined in
    /// ISO/IEC-14496-10-2022 - E.2.1 Table E-2.
    ///
    /// Defaults to 5 (unspecified).
    pub enum VideoFormat(u8) {
        /// The video type is component.
        Component = 0,

        /// The video type is PAL.
        PAL = 1,

        /// The video type is NTSC.
        NTSC = 2,

        /// The video type is SECAM.
        SECAM = 3,

        /// The video type is MAC.
        MAC = 4,

        /// The video type is Unspecified.
        Unspecified = 5,

        /// The video type is Reserved.
        Reserved1 = 6,

        /// The video type is Reserved.
        Reserved2 = 7,
    }
}

impl From<VideoFormat> for u64 {
    fn from(value: VideoFormat) -> Self {
        value.0 as u64
    }
}
