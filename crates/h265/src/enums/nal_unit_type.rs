use nutype_enum::nutype_enum;

nutype_enum! {
    /// NAL (Network Abstraction Layer) unit types as defined by ISO/IEC 23008-2 Table 7-1.
    pub enum NALUnitType(u8) {
        /// Coded slice segment of a non-TSA, non-STSA trailing picture
        ///
        /// NAL unit type class: VCL
        TrailN = 0,
        /// Coded slice segment of a non-TSA, non-STSA trailing picture
        ///
        /// NAL unit type class: VCL
        TrailR = 1,
        /// Coded slice segment of a TSA picture
        ///
        /// NAL unit type class: VCL
        TsaN = 2,
        /// Coded slice segment of a TSA picture
        ///
        /// NAL unit type class: VCL
        TsaR = 3,
        /// Coded slice segment of an STSA picture
        ///
        /// NAL unit type class: VCL
        StsaN = 4,
        /// Coded slice segment of an STSA picture
        ///
        /// NAL unit type class: VCL
        StsaR = 5,
        /// Coded slice segment of a RADL picture
        ///
        /// NAL unit type class: VCL
        RadlN = 6,
        /// Coded slice segment of a RADL picture
        ///
        /// NAL unit type class: VCL
        RadlR = 7,
        /// Coded slice segment of a RASL picture
        ///
        /// NAL unit type class: VCL
        RaslN = 8,
        /// Coded slice segment of a RASL picture
        ///
        /// NAL unit type class: VCL
        RaslR = 9,
        /// Reserved non-IRAP SLNR VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVclN10 = 10,
        /// Reserved non-IRAP sub-layer reference VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVclR11 = 11,
        /// Reserved non-IRAP SLNR VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVclN12 = 12,
        /// Reserved non-IRAP sub-layer reference VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVclR13 = 13,
        /// Reserved non-IRAP SLNR VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVclN14 = 14,
        /// Reserved non-IRAP sub-layer reference VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVclR15 = 15,
        /// Coded slice segment of a BLA picture
        ///
        /// NAL unit type class: VCL
        BlaWLp = 16,
        /// Coded slice segment of a BLA picture
        ///
        /// NAL unit type class: VCL
        BlaWRadl = 17,
        /// Coded slice segment of a BLA picture
        ///
        /// NAL unit type class: VCL
        BlaNLp = 18,
        /// Coded slice segment of an IDR picture
        ///
        /// NAL unit type class: VCL
        IdrWRadl = 19,
        /// Coded slice segment of an IDR picture
        ///
        /// NAL unit type class: VCL
        IdrNLp = 20,
        /// Coded slice segment of a CRA picture
        ///
        /// NAL unit type class: VCL
        CraNut = 21,
        /// Reserved IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvIrapVcl22 = 22,
        /// Reserved IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvIrapVcl23 = 23,
        /// Reserved non-IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVcl24 = 24,
        /// Reserved non-IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVcl25 = 25,
        /// Reserved non-IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVcl26 = 26,
        /// Reserved non-IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVcl27 = 27,
        /// Reserved non-IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVcl28 = 28,
        /// Reserved non-IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVcl29 = 29,
        /// Reserved non-IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVcl30 = 30,
        /// Reserved non-IRAP VCL NAL unit types
        ///
        /// NAL unit type class: VCL
        RsvVcl31 = 31,
        /// Video parameter set
        ///
        /// NAL unit type class: non-VCL
        VpsNut = 32,
        /// Sequence parameter set
        ///
        /// NAL unit type class: non-VCL
        SpsNut = 33,
        /// Picture parameter set
        ///
        /// NAL unit type class: non-VCL
        PpsNut = 34,
        /// Access unit delimiter
        ///
        /// NAL unit type class: non-VCL
        AudNut = 35,
        /// End of sequence
        ///
        /// NAL unit type class: non-VCL
        EosNut = 36,
        /// End of bitstream
        ///
        /// NAL unit type class: non-VCL
        EobNut = 37,
        /// Filler data
        ///
        /// NAL unit type class: non-VCL
        FdNut = 38,
        /// Supplemental enhancement information
        ///
        /// NAL unit type class: non-VCL
        PrefixSeiNut = 39,
        /// Supplemental enhancement information
        ///
        /// NAL unit type class: non-VCL
        SuffixSeiNut = 40,
        /// Reserved
        ///
        /// NAL unit type class: non-VCL
        RsvNvcl41 = 41,
        /// Reserved
        ///
        /// NAL unit type class: non-VCL
        RsvNvcl42 = 42,
        /// Reserved
        ///
        /// NAL unit type class: non-VCL
        RsvNvcl43 = 43,
        /// Reserved
        ///
        /// NAL unit type class: non-VCL
        RsvNvcl44 = 44,
        /// Reserved
        ///
        /// NAL unit type class: non-VCL
        RsvNvcl45 = 45,
        /// Reserved
        ///
        /// NAL unit type class: non-VCL
        RsvNvcl46 = 46,
        /// Reserved
        ///
        /// NAL unit type class: non-VCL
        RsvNvcl47 = 47,
    }
}

impl NALUnitType {
    /// Returns `true` if the NAL unit type class of this NAL unit type is VCL (Video Coding Layer).
    ///
    /// See ISO/IEC 23008-2 - Table 7-1, NAL unit type class column.
    pub fn is_vcl(&self) -> bool {
        (0..=31).contains(&self.0)
    }
}
