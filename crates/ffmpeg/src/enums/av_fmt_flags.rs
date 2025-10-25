use nutype_enum::{bitwise_enum, nutype_enum};

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVFormatFlags>() == std::mem::size_of_val(&AVFMT_NOFILE));
};

nutype_enum! {
    /// Format flags used in FFmpeg's `AVFormatContext` configuration.
    ///
    /// These flags are **format-specific capabilities** that describe the inherent
    /// characteristics and limitations of a format (container). They are read-only
    /// properties that indicate what features a format supports or doesn't support.
    ///
    /// For example, `NoFile` indicates the format doesn't need a regular file (like
    /// network protocols), while `GlobalHeader` indicates the format uses global codec
    /// headers.
    ///
    /// See the official FFmpeg documentation:
    /// <https://ffmpeg.org/doxygen/trunk/avformat_8h.html>
    pub enum AVFormatFlags(i32) {
        /// The format does not require a file to be opened explicitly.
        /// - **Used for**: Protocol-based formats like `rtmp://`, `http://`
        /// - **Equivalent to**: `AVFMT_NOFILE`
        NoFile = AVFMT_NOFILE as _,

        /// Requires a numbered sequence of files (e.g., `%03d` in filenames).
        /// - **Used for**: Image sequences, segment-based formats.
        /// - **Equivalent to**: `AVFMT_NEEDNUMBER`
        NeedNumber = AVFMT_NEEDNUMBER as _,

        /// The format is experimental and may be subject to changes.
        /// - **Used for**: Newer formats that are not yet stable.
        /// - **Equivalent to**: `AVFMT_EXPERIMENTAL`
        Experimental = AVFMT_EXPERIMENTAL as _,

        /// Displays stream identifiers when logging or printing metadata.
        /// - **Equivalent to**: `AVFMT_SHOW_IDS`
        ShowIds = AVFMT_SHOW_IDS as _,

        /// Uses a global header instead of individual packet headers.
        /// - **Used for**: Codecs that require an extradata header (e.g., H.264, AAC in MP4).
        /// - **Equivalent to**: `AVFMT_GLOBALHEADER`
        GlobalHeader = AVFMT_GLOBALHEADER as _,

        /// The format does not store timestamps.
        /// - **Used for**: Raw formats (e.g., raw audio, raw video).
        /// - **Equivalent to**: `AVFMT_NOTIMESTAMPS`
        NoTimestamps = AVFMT_NOTIMESTAMPS as _,

        /// The format has a generic index.
        /// - **Used for**: Formats that require seeking but don't use timestamp-based indexing.
        /// - **Equivalent to**: `AVFMT_GENERIC_INDEX`
        GenericIndex = AVFMT_GENERIC_INDEX as _,

        /// The format supports discontinuous timestamps.
        /// - **Used for**: Live streams where timestamps may reset (e.g., HLS, RTSP).
        /// - **Equivalent to**: `AVFMT_TS_DISCONT`
        TsDiscontinuous = AVFMT_TS_DISCONT as _,

        /// The format supports variable frame rates.
        /// - **Used for**: Video formats where frame duration varies (e.g., MKV, MP4).
        /// - **Equivalent to**: `AVFMT_VARIABLE_FPS`
        VariableFps = AVFMT_VARIABLE_FPS as _,

        /// The format does not store dimensions (width & height).
        /// - **Used for**: Audio-only formats, raw formats.
        /// - **Equivalent to**: `AVFMT_NODIMENSIONS`
        NoDimensions = AVFMT_NODIMENSIONS as _,

        /// The format does not contain any stream information.
        /// - **Used for**: Metadata-only containers.
        /// - **Equivalent to**: `AVFMT_NOSTREAMS`
        NoStreams = AVFMT_NOSTREAMS as _,

        /// The format does not support binary search for seeking.
        /// - **Used for**: Formats where linear scanning is required (e.g., live streams).
        /// - **Equivalent to**: `AVFMT_NOBINSEARCH`
        NoBinarySearch = AVFMT_NOBINSEARCH as _,

        /// The format does not support generic stream search.
        /// - **Used for**: Specialized formats that require specific handling.
        /// - **Equivalent to**: `AVFMT_NOGENSEARCH`
        NoGenericSearch = AVFMT_NOGENSEARCH as _,

        /// The format does not support byte-based seeking.
        /// - **Used for**: Formats that only support timestamp-based seeking.
        /// - **Equivalent to**: `AVFMT_NO_BYTE_SEEK`
        NoByteSeek = AVFMT_NO_BYTE_SEEK as _,

        /// Allows flushing of buffered data.
        /// - **Used for**: Streaming formats that support mid-stream flushing.
        /// - **Equivalent to**: `AVFMT_ALLOW_FLUSH`
        /// Deprecated in FFMPEG 8.0, see https://ffmpeg.org/pipermail/ffmpeg-devel/2023-September/314836.html
        /// AllowFlush = AVFMT_ALLOW_FLUSH as _,

        /// The format does not require strict timestamp ordering.
        /// - **Used for**: Formats where out-of-order timestamps are common.
        /// - **Equivalent to**: `AVFMT_TS_NONSTRICT`
        TsNonStrict = AVFMT_TS_NONSTRICT as _,

        /// The format allows negative timestamps.
        /// - **Used for**: Certain formats that support negative PTS/DTS.
        /// - **Equivalent to**: `AVFMT_TS_NEGATIVE`
        TsNegative = AVFMT_TS_NEGATIVE as _,

        /// Seeks are performed relative to presentation timestamps (PTS).
        /// - **Used for**: Formats that use PTS instead of DTS for seeking.
        /// - **Equivalent to**: `AVFMT_SEEK_TO_PTS`
        SeekToPts = AVFMT_SEEK_TO_PTS as _,
    }
}

bitwise_enum!(AVFormatFlags);

impl PartialEq<i32> for AVFormatFlags {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVFormatFlags {
    fn from(value: u32) -> Self {
        AVFormatFlags(value as _)
    }
}

impl From<AVFormatFlags> for u32 {
    fn from(value: AVFormatFlags) -> Self {
        value.0 as u32
    }
}

const _: () = {
    assert!(std::mem::size_of::<AVFmtFlags>() == std::mem::size_of_val(&AVFMT_FLAG_GENPTS));
};

nutype_enum! {
    /// Format flags used in FFmpeg's `AVFormatContext`.
    ///
    /// These flags are **user-configurable options** that control how FFmpeg should
    /// behave when reading or writing media. Unlike `AVFormatFlags` which describe
    /// format capabilities, these flags modify the runtime behavior of demuxers and
    /// muxers.
    ///
    /// For example, `GenPts` tells FFmpeg to generate missing timestamps, while
    /// `FastSeek` enables optimized seeking behavior.
    ///
    /// See the official FFmpeg documentation:
    /// <https://ffmpeg.org/doxygen/trunk/avformat_8h.html>
    pub enum AVFmtFlags(i32) {
        /// Generate **Presentation Timestamps (PTS)** if they are missing.
        /// - **Used for**: Formats that may not provide timestamps.
        /// - **Binary representation**: `0b0000000000000001`
        /// - **Equivalent to**: `AVFMT_FLAG_GENPTS`
        GenPts = AVFMT_FLAG_GENPTS as _,

        /// Ignore the index when seeking.
        /// - **Used for**: Faster seeking in formats that rely on indexes.
        /// - **Binary representation**: `0b0000000000000010`
        /// - **Equivalent to**: `AVFMT_FLAG_IGNIDX`
        IgnoreIndex = AVFMT_FLAG_IGNIDX as _,

        /// Open input in **non-blocking mode**.
        /// - **Used for**: Asynchronous reading.
        /// - **Binary representation**: `0b0000000000000100`
        /// - **Equivalent to**: `AVFMT_FLAG_NONBLOCK`
        NonBlock = AVFMT_FLAG_NONBLOCK as _,

        /// Ignore **Decoding Timestamps (DTS)**.
        /// - **Used for**: Cases where only PTS is needed.
        /// - **Binary representation**: `0b0000000000001000`
        /// - **Equivalent to**: `AVFMT_FLAG_IGNDTS`
        IgnoreDts = AVFMT_FLAG_IGNDTS as _,

        /// Do not fill in missing information in streams.
        /// - **Used for**: Avoiding unwanted automatic corrections.
        /// - **Binary representation**: `0b0000000000010000`
        /// - **Equivalent to**: `AVFMT_FLAG_NOFILLIN`
        NoFillIn = AVFMT_FLAG_NOFILLIN as _,

        /// Do not parse frames.
        /// - **Used for**: Formats where parsing is unnecessary.
        /// - **Binary representation**: `0b0000000000100000`
        /// - **Equivalent to**: `AVFMT_FLAG_NOPARSE`
        NoParse = AVFMT_FLAG_NOPARSE as _,

        /// Disable internal buffering.
        /// - **Used for**: Real-time applications requiring low latency.
        /// - **Binary representation**: `0b0000000001000000`
        /// - **Equivalent to**: `AVFMT_FLAG_NOBUFFER`
        NoBuffer = AVFMT_FLAG_NOBUFFER as _,

        /// Use **custom I/O** instead of standard file I/O.
        /// - **Used for**: Implementing custom read/write operations.
        /// - **Binary representation**: `0b0000000010000000`
        /// - **Equivalent to**: `AVFMT_FLAG_CUSTOM_IO`
        CustomIO = AVFMT_FLAG_CUSTOM_IO as _,

        /// Discard **corrupt** frames.
        /// - **Used for**: Ensuring only valid frames are processed.
        /// - **Binary representation**: `0b0000000100000000`
        /// - **Equivalent to**: `AVFMT_FLAG_DISCARD_CORRUPT`
        DiscardCorrupt = AVFMT_FLAG_DISCARD_CORRUPT as _,

        /// **Flush packets** after writing.
        /// - **Used for**: Streaming to avoid buffering delays.
        /// - **Binary representation**: `0b0000001000000000`
        /// - **Equivalent to**: `AVFMT_FLAG_FLUSH_PACKETS`
        FlushPackets = AVFMT_FLAG_FLUSH_PACKETS as _,

        /// Ensure **bit-exact** output.
        /// - **Used for**: Regression testing, avoiding encoding variations.
        /// - **Binary representation**: `0b0000010000000000`
        /// - **Equivalent to**: `AVFMT_FLAG_BITEXACT`
        BitExact = AVFMT_FLAG_BITEXACT as _,

        /// Sort packets by **Decoding Timestamp (DTS)**.
        /// - **Used for**: Ensuring ordered input.
        /// - **Binary representation**: `0b0001000000000000`
        /// - **Equivalent to**: `AVFMT_FLAG_SORT_DTS`
        SortDts = AVFMT_FLAG_SORT_DTS as _,

        /// Enable **fast seeking**.
        /// - **Used for**: Improving seek performance in large files.
        /// - **Binary representation**: `0b0010000000000000`
        /// - **Equivalent to**: `AVFMT_FLAG_FAST_SEEK`
        FastSeek = AVFMT_FLAG_FAST_SEEK as _,

        /// Stop **decoding at the shortest stream**.
        /// - **Used for**: Ensuring synchronization in multi-stream files.
        /// - **Binary representation**: `0b0100000000000000`
        /// - **Equivalent to**: `AVFMT_FLAG_SHORTEST`
        /// Deprecated in FFMPEG 8.0, see https://ffmpeg.org/pipermail/ffmpeg-devel/2023-August/313668.html
        /// Shortest = AVFMT_FLAG_SHORTEST as _,

        /// **Automatically apply bitstream filters**.
        /// - **Used for**: Simplifying format conversions.
        /// - **Binary representation**: `0b1000000000000000`
        /// - **Equivalent to**: `AVFMT_FLAG_AUTO_BSF`
        AutoBsf = AVFMT_FLAG_AUTO_BSF as _,
    }
}

bitwise_enum!(AVFmtFlags);

impl PartialEq<i32> for AVFmtFlags {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVFmtFlags {
    fn from(value: u32) -> Self {
        AVFmtFlags(value as _)
    }
}

impl From<AVFmtFlags> for u32 {
    fn from(value: AVFmtFlags) -> Self {
        value.0 as u32
    }
}
