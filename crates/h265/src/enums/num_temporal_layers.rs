use nutype_enum::nutype_enum;

nutype_enum! {
    /// The number of temporal layers in the stream.
    ///
    /// `0` and `1` are special values.
    ///
    /// Any other value represents the actual number of temporal layers.
    pub enum NumTemporalLayers(u8) {
        /// The stream might be temporally scalable.
        Unknown = 0,
        /// The stream is not temporally scalable.
        NotScalable = 1,
    }
}
