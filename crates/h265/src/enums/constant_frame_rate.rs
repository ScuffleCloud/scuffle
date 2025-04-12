use nutype_enum::nutype_enum;

nutype_enum! {
    pub enum ConstantFrameRate(u8) {
        /// The stream may or may not be of constant frame rate.
        Unknown = 0,
        /// The stream is of constant frame rate.
        Constant = 1,
        /// The representation of each temporal layer in the stream is of constant frame rate.
        TemporalLayerConstant = 2,
    }
}
