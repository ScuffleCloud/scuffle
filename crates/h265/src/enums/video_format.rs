use nutype_enum::nutype_enum;

nutype_enum! {
    /// ISO/IEC 23008-2 - Table E.2
    pub enum VideoFormat(u8) {
        /// Component
        Component = 0,
        /// PAL
        PAL = 1,
        /// NTSC
        NTSC = 2,
        /// SECAM
        SECAM = 3,
        /// MAC
        MAC = 4,
        /// Unspecified video format
        Unspecified = 5,
    }
}
