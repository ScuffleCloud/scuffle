bitflags::bitflags! {
    /// Represents the profile compatibility flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ProfileCompatibilityFlags: u32 {
        /// Profile flag 0
        const Profile0 = 1 << 31;
        /// Main Profile
        ///
        /// ISO/IEC 23008-2 - A.3.2
        const MainProfile = 1 << 30; // 1
        /// Main 10 Profile
        ///
        /// ISO/IEC 23008-2 - A.3.3
        const Main10Profile = 1 << 29; // 2
        /// Main Still Picture Profile
        ///
        /// ISO/IEC 23008-2 - A.3.4
        const MainStillPictureProfile = 1 << 28; // 3
        /// Format Range Extensions Profile
        ///
        /// ISO/IEC 23008-2 - A.3.5
        const FormatRangeExtensionsProfile = 1 << 27; // 4
        /// High Throughput Profile
        ///
        /// ISO/IEC 23008-2 - A.3.6
        const HighThroughputProfile = 1 << 26; // 5
        /// Profile flag 6
        const Profile6 = 1 << 25;
        /// Profile flag 7
        const Profile7 = 1 << 24;
        /// Profile flag 8
        const Profile8 = 1 << 23;
        /// Screen Content Coding Extensions Profile
        ///
        /// ISO/IEC 23008-2 - A.3.7
        const ScreenContentCodingExtensionsProfile = 1 << 22;
        /// Profile flag 10
        const Profile10 = 1 << 21;
        /// High Throughput Screen Content Coding Extensions Profile
        ///
        /// ISO/IEC 23008-2 - A.3.8
        const HighThroughputScreenContentCodingExtensionsProfile = 1 << 20;
        /// Profile flag 12
        const Profile12 = 1 << 19;
        /// Profile flag 13
        const Profile13 = 1 << 18;
        /// Profile flag 14
        const Profile14 = 1 << 17;
        /// Profile flag 15
        const Profile15 = 1 << 16;
        /// Profile flag 16
        const Profile16 = 1 << 15;
        /// Profile flag 17
        const Profile17 = 1 << 14;
        /// Profile flag 18
        const Profile18 = 1 << 13;
        /// Profile flag 19
        const Profile19 = 1 << 12;
        /// Profile flag 20
        const Profile20 = 1 << 11;
        /// Profile flag 21
        const Profile21 = 1 << 10;
        /// Profile flag 22
        const Profile22 = 1 << 9;
        /// Profile flag 23
        const Profile23 = 1 << 8;
        /// Profile flag 24
        const Profile24 = 1 << 7;
        /// Profile flag 25
        const Profile25 = 1 << 6;
        /// Profile flag 26
        const Profile26 = 1 << 5;
        /// Profile flag 27
        const Profile27 = 1 << 4;
        /// Profile flag 28
        const Profile28 = 1 << 3;
        /// Profile flag 29
        const Profile29 = 1 << 2;
        /// Profile flag 30
        const Profile30 = 1 << 1;
        /// Profile flag 31
        const Profile31 = 1 << 0;
    }
}
