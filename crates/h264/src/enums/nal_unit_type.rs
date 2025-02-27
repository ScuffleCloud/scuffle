use nutype_enum::nutype_enum;

nutype_enum! {
    /// NAL (Network Abstraction Layer) unit types as defined by ISO/IEC 14496-10:2022 (Table 7-1).
    ///
    /// ## Decoder Behavior:
    /// - **Some NAL units may be ignored** depending on the decoder.
    /// - Decoders using **Annex A** must ignore unit types **14, 15, and 20**.
    /// - **Types 0 and 24-31** are application-specific and do not affect decoding.
    /// - **Reserved values** should be ignored.
    ///
    /// ## IDR (Instantaneous Decoder Refresh) Pictures:
    /// - If `nal_unit_type` is **5**, the picture **must not contain** types **1-4**.
    /// - `IdrPicFlag` is **1** if `nal_unit_type == 5`, otherwise **0**.
    pub enum NALUnitType(u8) {
        /// Unspecified (not used in decoding)
        Unspecified1 = 0,

        /// Regular video slice (non-IDR picture)
        NonIDRSliceLayerWithoutPartitioning = 1,

        /// Coded slice data partition A
        SliceDataPartitionALayer = 2,

        /// Coded slice data partition B
        SliceDataPartitionBLayer = 3,

        /// Coded slice data partition C
        SliceDataPartitionCLayer = 4,

        /// IDR picture (used to refresh the video stream)
        IDRSliceLayerWithoutPartitioning = 5,

        /// Extra metadata (Supplemental Enhancement Information)
        SEI = 6,

        /// Sequence Parameter Set (SPS) – contains video configuration details
        SPS = 7,

        /// Picture Parameter Set (PPS) – contains picture-specific settings
        PPS = 8,

        /// Marks the start of a new access unit (frame boundary)
        AccessUnitDelimiter = 9,

        /// End of video sequence
        EndOfSeq = 10,

        /// End of video stream
        EndOfStream = 11,

        /// Extra filler data (can be ignored)
        FillerData = 12,

        /// Extension to SPS (used for advanced encoding features)
        SPSExtension = 13,

        /// Prefix NAL unit (ignored by Annex A decoders)
        PrefixNalUnit = 14,

        /// Subset of SPS (ignored by Annex A decoders)
        SubsetSPS = 15,

        /// Depth parameter set (used for 3D video)
        DepthParameterSet = 16,

        /// Reserved (should be ignored)
        Reserved1 = 17,

        /// Reserved (should be ignored)
        Reserved2 = 18,

        /// Auxiliary coded slice (may be ignored by some decoders)
        AuxCodedPictureSliceLayerWithoutPartitioning = 19,

        /// Additional slice data for extended coding (ignored by Annex A decoders)
        SliceLayerExtension = 20,

        /// Slice extension for depth/3D-AVC video (ignored by some decoders)
        SliceLayerExtension2 = 21,

        /// Reserved (should be ignored)
        Reserved3 = 22,

        /// Reserved (should be ignored)
        Reserved4 = 23,

        /// Unspecified (application-defined use)
        Unspecified2 = 24
    }
}

impl From<NALUnitType> for u64 {
    fn from(value: NALUnitType) -> Self {
        value.0 as u64
    }
}
