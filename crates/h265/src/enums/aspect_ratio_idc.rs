use nutype_enum::nutype_enum;

nutype_enum! {
    /// Interpretation of sample aspect ratio indicator.
    ///
    /// ISO/IEC 23008-2 - Table E.1
    pub enum AspectRatioIdc(u8) {
        /// Unspecified
        Unspecified = 0,
        /// 1:1 (square)
        Square = 1,
        /// 12:11
        Aspect12_11 = 2,
        /// 10:11
        Aspect10_11 = 3,
        /// 16:11
        Aspect16_11 = 4,
        /// 40:33
        Aspect40_33 = 5,
        /// 24:11
        Aspect24_11 = 6,
        /// 20:11
        Aspect20_11 = 7,
        /// 32:11
        Aspect32_11 = 8,
        /// 80:33
        Aspect80_33 = 9,
        /// 18:11
        Aspect18_11 = 10,
        /// 15:11
        Aspect15_11 = 11,
        /// 64:33
        Aspect64_33 = 12,
        /// 160:99
        Aspect160_99 = 13,
        /// 4:3
        Aspect4_3 = 14,
        /// 3:2
        Aspect3_2 = 15,
        /// 2:1
        Aspect2_1 = 16,
        /// EXTENDED_SAR
        ExtendedSar = 255,
    }
}
