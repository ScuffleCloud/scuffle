use nutype_enum::nutype_enum;

nutype_enum! {
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
