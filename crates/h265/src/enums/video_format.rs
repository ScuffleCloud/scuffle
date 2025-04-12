use nutype_enum::nutype_enum;

nutype_enum! {
    pub enum VideoFormat(u8) {
        Component = 0,
        PAL = 1,
        NTSC = 2,
        SECAM = 3,
        MAC = 4,
        Unspecified = 5,
    }
}
