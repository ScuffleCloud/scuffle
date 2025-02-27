use nutype_enum::nutype_enum;

nutype_enum! {
    /// The `AspectRatioIdc` is a nutype enum for `aspect_ratio_idc` as defined in
    /// ISO/IEC-14496-10-2022 - E.2.1 Table E-1.
    ///
    /// Values 17..=254** are reserved (should be ignored if encountered)
    /// **Value 255 (`ExtendedSar`)** indicates that the aspect ratio is specified by
    ///   additional fields (`sar_width` and `sar_height`) in the bitstream.
    ///
    /// ## Examples of aspect_ratio_idc values:
    /// - `1` => 1:1 ("square")
    /// - `4` => 16:11
    /// - `14` => 4:3
    /// - `15` => 3:2
    /// - `16` => 2:1
    pub enum AspectRatioIdc(u8) {
        /// 0: Unspecified (not used in decoding)
        Unspecified = 0,

        /// 1: 1:1 (square)
        /// ## Examples
        /// - 7680 x 4320 16:9 w/o horizontal overscan
        /// - 3840 x 2160 16:9 w/o horizontal overscan
        /// - 1280 x 720 16:9 w/o horizontal overscan
        /// - 1920 x 1080 16:9 w/o horizontal overscan (cropped from 1920x1088)
        /// - 640 x 480 4:3 w/o horizontal overscan
        Square = 1,

        /// 2: 12:11
        /// ## Examples
        /// - 720 x 576 4:3 with horizontal overscan
        /// - 352 x 288 4:3 w/o horizontal overscan
        Aspect12_11 = 2,

        /// 3: 10:11
        /// ## Examples
        /// - 720 x 480 4:3 with horizontal overscan
        /// - 352 x 240 4:3 w/o horizontal overscan
        Aspect10_11 = 3,

        /// 4: 16:11
        /// ## Examples
        /// - 720 x 576 16:9 with horizontal overscan
        /// - 528 x 576 4:3 w/o horizontal overscan
        Aspect16_11 = 4,

        /// 5: 40:33
        /// ## Examples
        /// - 720 x 480 16:9 with horizontal overscan
        /// - 528 x 480 4:3 w/o horizontal overscan
        Aspect40_33 = 5,

        /// 6: 24:11
        /// ## Examples
        /// - 352 x 576 4:3 w/o horizontal overscan
        /// - 480 x 576 16:9 with horizontal overscan
        Aspect24_11 = 6,

        /// 7: 20:11
        /// ## Examples
        /// - 352 x 480 4:3 w/o horizontal overscan
        /// - 480 x 480 16:9 with horizontal overscan
        Aspect20_11 = 7,

        /// 8: 32:11
        /// ## Example
        /// - 352 x 576 16:9 w/o horizontal overscan
        Aspect32_11 = 8,

        /// 9: 80:33
        /// ## Example
        /// - 352 x 480 16:9 w/o horizontal overscan
        Aspect80_33 = 9,

        /// 10: 18:11
        /// ## Example
        /// - 480 x 576 16:9 with horizontal overscan
        Aspect18_11 = 10,

        /// 11: 15:11
        /// ## Example
        /// - 480 x 480 4:3 with horizontal overscan
        Aspect15_11 = 11,

        /// 12: 64:33
        /// ## Example
        /// - 528 x 576 16:9 w/o horizontal overscan
        Aspect64_33 = 12,

        /// 13: 160:99
        /// ## Example
        /// - 528 x 480 16:9 w/o horizontal overscan
        Aspect160_99 = 13,

        /// 14: 4:3
        /// ## Example
        /// - 1440 x 1080 16:9 w/o horizontal overscan
        Aspect4_3 = 14,

        /// 15: 3:2
        /// ## Example
        /// - 1280 x 1080 16:9 w/o horizontal overscan
        Aspect3_2 = 15,

        /// 16: 2:1
        /// ## Example
        /// - 960 x 1080 16:9 w/o horizontal overscan
        Aspect2_1 = 16,

        /// 17..=254: Reserved (should be ignored)
        Reserved = 17,

        /// 255: Extended SAR (use `sar_width` & `sar_height` from bitstream)
        ExtendedSar = 255
    }
}

impl From<AspectRatioIdc> for u64 {
    fn from(value: AspectRatioIdc) -> Self {
        value.0 as u64
    }
}
