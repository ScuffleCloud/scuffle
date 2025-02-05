use ffmpeg_sys_next::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DecoderCodec(*const AVCodec);

impl std::fmt::Debug for DecoderCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_null() {
            return f
                .debug_struct("DecoderCodec")
                .field("name", &c"null")
                .field("id", &AVCodecID::AV_CODEC_ID_NONE)
                .finish();
        }

        // Safety: `self.0` is a non-null pointer.
        let name = unsafe { std::ffi::CStr::from_ptr((*self.0).name) };
        f.debug_struct("DecoderCodec")
            .field("name", &name)
            // Safety: `self.0` is a non-null pointer.
            .field("id", unsafe { &(*self.0).id })
            .finish()
    }
}

impl DecoderCodec {
    pub fn empty() -> Self {
        Self(std::ptr::null())
    }

    pub fn new(codec_id: AVCodecID) -> Option<Self> {
        // Safety: `avcodec_find_decoder` is safe to call.
        let codec = unsafe { avcodec_find_decoder(codec_id) };
        if codec.is_null() {
            None
        } else {
            Some(Self(codec))
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        let c_name = std::ffi::CString::new(name).ok()?;

        // Safety: `avcodec_find_decoder_by_name` is safe to call with a valid c-string.
        let codec = unsafe { avcodec_find_decoder_by_name(c_name.as_ptr()) };
        if codec.is_null() {
            None
        } else {
            Some(Self(codec))
        }
    }

    pub fn as_ptr(&self) -> *const AVCodec {
        self.0
    }

    pub fn from_ptr(ptr: *const AVCodec) -> Self {
        Self(ptr)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EncoderCodec(*const AVCodec);

impl std::fmt::Debug for EncoderCodec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_null() {
            return f
                .debug_struct("EncoderCodec")
                .field("name", &c"null")
                .field("id", &AVCodecID::AV_CODEC_ID_NONE)
                .finish();
        }

        // Safety: `self.0` is a valid pointer.
        let name = unsafe { std::ffi::CStr::from_ptr((*self.0).name) };
        f.debug_struct("EncoderCodec")
            .field("name", &name)
            // Safety: `self.0` is a valid pointer.
            .field("id", unsafe { &(*self.0).id })
            .finish()
    }
}

impl EncoderCodec {
    pub fn empty() -> Self {
        Self(std::ptr::null())
    }

    pub fn new(codec_id: AVCodecID) -> Option<Self> {
        // Safety: `avcodec_find_encoder` is safe to call.
        let codec = unsafe { avcodec_find_encoder(codec_id) };
        if codec.is_null() {
            None
        } else {
            Some(Self(codec))
        }
    }

    pub fn by_name(name: &str) -> Option<Self> {
        let c_name = std::ffi::CString::new(name).ok()?;
        // Safety: `avcodec_find_encoder_by_name` is safe to call with a valid c-string.
        let codec = unsafe { avcodec_find_encoder_by_name(c_name.as_ptr()) };
        if codec.is_null() {
            None
        } else {
            Some(Self(codec))
        }
    }

    pub fn as_ptr(&self) -> *const AVCodec {
        self.0
    }

    pub fn from_ptr(ptr: *const AVCodec) -> Self {
        Self(ptr)
    }
}

impl From<EncoderCodec> for *const AVCodec {
    fn from(codec: EncoderCodec) -> Self {
        codec.0
    }
}

impl From<DecoderCodec> for *const AVCodec {
    fn from(codec: DecoderCodec) -> Self {
        codec.0
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use ffmpeg_sys_next::AVCodecID::{self, AV_CODEC_ID_AAC, AV_CODEC_ID_H264};
    use ffmpeg_sys_next::{avcodec_find_decoder, avcodec_find_encoder, AVCodec};

    use crate::codec::{DecoderCodec, EncoderCodec};

    #[test]
    fn test_decoder_codec_debug_null() {
        let decoder_codec = DecoderCodec::empty();
        let debug_output = format!("{:?}", decoder_codec);

        insta::assert_snapshot!(debug_output, @r#"DecoderCodec { name: "null", id: AV_CODEC_ID_NONE }"#);
    }

    #[test]
    fn test_decoder_codec_debug_non_null() {
        let decoder_codec = DecoderCodec::new(AV_CODEC_ID_H264).expect("H264 codec should be available");
        let debug_output = format!("{:?}", decoder_codec);

        insta::assert_snapshot!(debug_output, @r#"DecoderCodec { name: "h264", id: AV_CODEC_ID_H264 }"#);
    }

    #[test]
    fn test_decoder_codec_new_invalid_codec_id() {
        let invalid_codec_id = AVCodecID::AV_CODEC_ID_NONE;
        let result = DecoderCodec::new(invalid_codec_id);

        assert!(
            result.is_none(),
            "Expected `DecoderCodec::new` to return None for an invalid codec ID"
        );
    }

    #[test]
    fn test_decoder_codec_by_name_valid() {
        let result = DecoderCodec::by_name("h264");

        assert!(
            result.is_some(),
            "Expected `DecoderCodec::by_name` to return Some for a valid codec name"
        );

        let codec = result.unwrap();
        assert!(!codec.as_ptr().is_null(), "Expected a non-null codec pointer");
    }

    #[test]
    fn test_decoder_codec_by_name_invalid() {
        let invalid_codec_name = "nonexistent_codec";
        let result = DecoderCodec::by_name(invalid_codec_name);

        assert!(
            result.is_none(),
            "Expected `DecoderCodec::by_name` to return None for an invalid codec name"
        );
    }

    #[test]
    fn test_decoder_codec_from_ptr_valid() {
        let codec_ptr = unsafe { avcodec_find_decoder(AVCodecID::AV_CODEC_ID_H264) };
        assert!(!codec_ptr.is_null(), "Expected a valid codec pointer for H264");

        let decoder_codec = DecoderCodec::from_ptr(codec_ptr);
        assert_eq!(
            decoder_codec.as_ptr(),
            codec_ptr,
            "Expected the codec pointer in DecoderCodec to match the original pointer"
        );
    }

    #[test]
    fn test_encoder_codec_debug_valid() {
        let codec_ptr = unsafe { avcodec_find_encoder(AVCodecID::AV_CODEC_ID_MPEG4) };
        assert!(!codec_ptr.is_null(), "Expected a valid codec pointer for MPEG4");

        let encoder_codec = EncoderCodec(codec_ptr);
        insta::assert_debug_snapshot!(encoder_codec, @r#"
        EncoderCodec {
            name: "mpeg4",
            id: AV_CODEC_ID_MPEG4,
        }
        "#);
    }

    #[test]
    fn test_encoder_codec_debug_null() {
        let encoder_codec = EncoderCodec(std::ptr::null());
        insta::assert_debug_snapshot!(encoder_codec, @r#"
        EncoderCodec {
            name: "null",
            id: AV_CODEC_ID_NONE,
        }
        "#);
    }

    #[test]
    fn test_encoder_codec_empty() {
        let encoder_codec = EncoderCodec::empty();
        assert!(
            encoder_codec.as_ptr().is_null(),
            "Expected the encoder codec pointer to be null"
        );

        insta::assert_debug_snapshot!(encoder_codec, @r#"
        EncoderCodec {
            name: "null",
            id: AV_CODEC_ID_NONE,
        }
        "#);
    }

    #[test]
    fn test_encoder_codec_new_invalid_codec() {
        let invalid_codec_id = AVCodecID::AV_CODEC_ID_NONE;
        let result = EncoderCodec::new(invalid_codec_id);

        assert!(result.is_none(), "Expected None for an invalid codec ID");
    }

    #[test]
    fn test_encoder_codec_by_name_valid() {
        let result = EncoderCodec::by_name("mpeg4");
        assert!(result.is_some(), "Expected a valid encoder codec for the name {}", "mpeg4");

        let encoder_codec = result.unwrap();
        assert!(!encoder_codec.as_ptr().is_null(), "Expected a non-null encoder codec pointer");
    }

    #[test]
    fn test_encoder_codec_by_name_invalid() {
        let invalid_encoder_name = "invalid_encoder_name";
        let result = EncoderCodec::by_name(invalid_encoder_name);

        assert!(
            result.is_none(),
            "Expected None for an invalid encoder name {}",
            invalid_encoder_name
        );
    }

    #[test]
    fn test_encoder_codec_into_raw_ptr() {
        let valid_codec_id = AV_CODEC_ID_AAC;
        let encoder_codec = EncoderCodec::new(valid_codec_id).expect("Expected a valid encoder codec for AAC");
        let raw_ptr: *const AVCodec = encoder_codec.into();

        assert_eq!(
            raw_ptr,
            encoder_codec.as_ptr(),
            "The raw pointer should match the encoder codec's internal pointer"
        );
    }

    #[test]
    fn test_decoder_codec_into_raw_ptr() {
        let valid_codec_id = AV_CODEC_ID_AAC;
        let decoder_codec = DecoderCodec::new(valid_codec_id).expect("Expected a valid decoder codec for AAC");
        let raw_ptr: *const AVCodec = decoder_codec.into();

        assert_eq!(
            raw_ptr,
            decoder_codec.as_ptr(),
            "The raw pointer should match the decoder codec's internal pointer"
        );
    }

    #[test]
    fn test_codec_into_raw_ptr_empty() {
        let empty_encoder_codec = EncoderCodec::empty();
        let raw_ptr: *const AVCodec = empty_encoder_codec.into();
        assert!(raw_ptr.is_null(), "The raw pointer should be null for an empty EncoderCodec");

        let empty_decoder_codec = DecoderCodec::empty();
        let raw_ptr: *const AVCodec = empty_decoder_codec.into();
        assert!(raw_ptr.is_null(), "The raw pointer should be null for an empty DecoderCodec");
    }
}
