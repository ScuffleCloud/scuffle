use nutype_enum::nutype_enum;

nutype_enum! {
    /// Indicates the type of parallelism that is used to meet the restrictions imposed
    /// by [`min_spatial_segmentation_idc`](crate::HEVCDecoderConfigurationRecord::min_spatial_segmentation_idc) when the value of
    /// [`min_spatial_segmentation_idc`](crate::HEVCDecoderConfigurationRecord::min_spatial_segmentation_idc) is greater than 0.
    ///
    /// ISO/IEC 14496-15 - 8.3.2.1.3
    pub enum ParallelismType(u8) {
        /// The stream supports mixed types of parallel decoding or the parallelism type is unknown.
        MixedOrUnknown = 0,
        /// The stream supports slice based parallel decoding.
        Slice = 1,
        /// The stream supports tile based parallel decoding.
        Tile = 2,
        /// The stream supports entropy coding sync based parallel decoding.
        EntropyCodingSync = 3,
    }
}
