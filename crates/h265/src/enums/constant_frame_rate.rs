use nutype_enum::nutype_enum;

nutype_enum! {
    /// Represents all possible values of the `constant_frame_rate` field in the
    /// [`HEVCDecoderConfigurationRecord`](crate::config::HEVCDecoderConfigurationRecord).
    ///
    /// ISO/IEC 14496-15 - 8.3.2.1.3
    pub enum ConstantFrameRate(u8) {
        /// Indicates that the stream may or may not be of constant frame rate.
        Unknown = 0,
        /// Indicates that the stream to which this configuration record
        /// applies is of constant frame rate.
        Constant = 1,
        /// Indicates that the representation of each temporal
        /// layer in the stream is of constant frame rate.
        TemporalLayerConstant = 2,
    }
}
