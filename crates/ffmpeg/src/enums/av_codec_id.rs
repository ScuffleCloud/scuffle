use nutype_enum::nutype_enum;

use crate::ffi::*;

const _: () = {
    assert!(std::mem::size_of::<AVCodecID>() == std::mem::size_of_val(&AV_CODEC_ID_NONE));
};

nutype_enum! {
    /// Enum representing various FFmpeg codec IDs.
    ///
    /// Each codec corresponds to an FFmpeg-supported format, including video, audio, and subtitle codecs.
    /// The full list of FFmpeg codecs can be found in the official documentation:
    /// - [FFmpeg Doxygen - avcodec.h](https://ffmpeg.org/doxygen/trunk/avcodec_8h_source.html)
    /// - [FFmpeg Codecs List](https://ffmpeg.org/ffmpeg-codecs.html)
    ///
    /// These IDs are directly mapped from `AV_CODEC_ID_*` constants in FFmpeg.
    pub enum AVCodecID(i32) {
        /// No codec specified.
        None = AV_CODEC_ID_NONE as _,

        /// MPEG-1 Video codec.
        /// Commonly used in Video CDs and early digital broadcasting.
        Mpeg1Video = AV_CODEC_ID_MPEG1VIDEO as _,

        /// MPEG-2 Video codec.
        /// Used in DVDs, digital TV broadcasting, and early HD video.
        Mpeg2Video = AV_CODEC_ID_MPEG2VIDEO as _,

        /// H.261 video codec.
        /// An early video compression standard used for video conferencing.
        H261 = AV_CODEC_ID_H261 as _,

        /// H.263 video codec.
        /// A predecessor to H.264, used in video conferencing and mobile video.
        H263 = AV_CODEC_ID_H263 as _,

        /// RealVideo 1.0 codec.
        /// An early proprietary video format from RealNetworks.
        Rv10 = AV_CODEC_ID_RV10 as _,

        /// RealVideo 2.0 codec.
        /// Improved version of RealVideo for streaming applications.
        Rv20 = AV_CODEC_ID_RV20 as _,

        /// Motion JPEG codec.
        /// Stores video frames as individual JPEG images.
        Mjpeg = AV_CODEC_ID_MJPEG as _,

        /// Motion JPEG-B codec.
        /// A variant of Motion JPEG with a slightly different encoding method.
        MjpegB = AV_CODEC_ID_MJPEGB as _,

        /// Lossless JPEG codec.
        /// Used for medical imaging and other applications needing lossless compression.
        Ljpeg = AV_CODEC_ID_LJPEG as _,

        /// SP5X codec.
        /// Used in certain digital cameras.
        Sp5X = AV_CODEC_ID_SP5X as _,

        /// JPEG-LS codec.
        /// A lossless JPEG-based compression format.
        JpegLs = AV_CODEC_ID_JPEGLS as _,

        /// MPEG-4 Part 2 video codec.
        /// Used in DivX, Xvid, and some early video formats before H.264.
        Mpeg4 = AV_CODEC_ID_MPEG4 as _,

        /// Raw video codec.
        /// Uncompressed video frames.
        RawVideo = AV_CODEC_ID_RAWVIDEO as _,

        /// Microsoft MPEG-4 Version 1 codec.
        /// An early proprietary MPEG-4-based codec.
        MsMpeg4V1 = AV_CODEC_ID_MSMPEG4V1 as _,

        /// Microsoft MPEG-4 Version 2 codec.
        /// Improved version of the earlier Microsoft MPEG-4 codec.
        MsMpeg4V2 = AV_CODEC_ID_MSMPEG4V2 as _,

        /// Microsoft MPEG-4 Version 3 codec.
        /// Used in older Windows Media Video (WMV) files.
        MsMpeg4V3 = AV_CODEC_ID_MSMPEG4V3 as _,

        /// Windows Media Video 7 codec.
        /// Early WMV format used for streaming.
        Wmv1 = AV_CODEC_ID_WMV1 as _,

        /// Windows Media Video 8 codec.
        /// Improved version of WMV1.
        Wmv2 = AV_CODEC_ID_WMV2 as _,

        /// H.263+ video codec.
        /// An improved version of H.263 with better compression efficiency.
        H263P = AV_CODEC_ID_H263P as _,

        /// H.263i video codec.
        /// An interlaced variant of H.263.
        H263I = AV_CODEC_ID_H263I as _,

        /// FLV1 codec.
        /// Used in Adobe Flash Video (.flv) files.
        Flv1 = AV_CODEC_ID_FLV1 as _,

        /// Sorenson Video 1 codec.
        /// Used in early QuickTime videos.
        Svq1 = AV_CODEC_ID_SVQ1 as _,

        /// Sorenson Video 3 codec.
        /// A more advanced version used in some QuickTime movies.
        Svq3 = AV_CODEC_ID_SVQ3 as _,

        /// DV Video codec.
        /// Used in Digital Video (DV) camcorders and professional video production.
        DvVideo = AV_CODEC_ID_DVVIDEO as _,

        /// HuffYUV codec.
        /// A lossless video compression codec commonly used for archiving.
        Huffyuv = AV_CODEC_ID_HUFFYUV as _,

        /// Creative Labs YUV codec.
        /// Used in some old hardware-accelerated video capture cards.
        Cyuv = AV_CODEC_ID_CYUV as _,

        /// H.264 / AVC codec.
        /// One of the most widely used video codecs, offering efficient compression.
        H264 = AV_CODEC_ID_H264 as _,

        /// Indeo Video 3 codec.
        /// A proprietary video format developed by Intel.
        Indeo3 = AV_CODEC_ID_INDEO3 as _,

        /// VP3 codec.
        /// A predecessor to Theora, developed by On2 Technologies.
        Vp3 = AV_CODEC_ID_VP3 as _,

        /// Theora codec.
        /// An open-source video codec based on VP3.
        Theora = AV_CODEC_ID_THEORA as _,

        /// ASUS Video 1 codec.
        /// Used in ASUS hardware-based video capture solutions.
        Asv1 = AV_CODEC_ID_ASV1 as _,

        /// ASUS Video 2 codec.
        /// An improved version of ASUS Video 1.
        Asv2 = AV_CODEC_ID_ASV2 as _,

        /// FFV1 codec.
        /// A lossless video codec developed for archival purposes.
        Ffv1 = AV_CODEC_ID_FFV1 as _,

        /// 4X Movie codec.
        /// Used in some old video games.
        FourXm = AV_CODEC_ID_4XM as _,

        /// VCR1 codec.
        /// An early proprietary format for video recording.
        Vcr1 = AV_CODEC_ID_VCR1 as _,

        /// Cirrus Logic JPEG codec.
        /// Used in certain video capture hardware.
        Cljr = AV_CODEC_ID_CLJR as _,

        /// MDEC codec.
        /// Used in PlayStation video files.
        Mdec = AV_CODEC_ID_MDEC as _,

        /// RoQ codec.
        /// Used in some video game cutscenes, notably Quake III.
        Roq = AV_CODEC_ID_ROQ as _,

        /// Interplay Video codec.
        /// Used in some video game cutscenes from Interplay.
        InterplayVideo = AV_CODEC_ID_INTERPLAY_VIDEO as _,

        /// Xan WC3 codec.
        /// Used in certain games developed by Westwood Studios.
        XanWc3 = AV_CODEC_ID_XAN_WC3 as _,

        /// Xan WC4 codec.
        /// An improved version of Xan WC3.
        XanWc4 = AV_CODEC_ID_XAN_WC4 as _,

        /// RPZA codec.
        /// Used in early Apple QuickTime videos.
        Rpza = AV_CODEC_ID_RPZA as _,

        /// Cinepak codec.
        /// A widely used video codec in the 1990s for CD-ROM games and early digital videos.
        Cinepak = AV_CODEC_ID_CINEPAK as _,

        /// Westwood Studios VQA codec.
        /// Used in games developed by Westwood Studios.
        WsVqa = AV_CODEC_ID_WS_VQA as _,

        /// Microsoft RLE codec.
        /// Used for simple Run-Length Encoding (RLE) video compression.
        MsRle = AV_CODEC_ID_MSRLE as _,

        /// Microsoft Video 1 codec.
        /// A basic, low-quality video codec used in early Windows applications.
        MsVideo1 = AV_CODEC_ID_MSVIDEO1 as _,

        /// id CIN codec.
        /// Used in some id Software game cutscenes.
        Idcin = AV_CODEC_ID_IDCIN as _,

        /// QuickTime 8BPS codec.
        /// A simple video compression format used in QuickTime.
        EightBps = AV_CODEC_ID_8BPS as _,

        /// Apple Graphics SMC codec.
        /// A very simple codec used in QuickTime.
        Smc = AV_CODEC_ID_SMC as _,

        /// Autodesk FLIC codec.
        /// Used in animations from Autodesk software.
        Flic = AV_CODEC_ID_FLIC as _,

        /// TrueMotion 1 codec.
        /// A codec developed by Duck Corporation for video compression.
        Truemotion1 = AV_CODEC_ID_TRUEMOTION1 as _,

        /// VMD Video codec.
        /// Used in Sierra game cutscenes.
        VmdVideo = AV_CODEC_ID_VMDVIDEO as _,

        /// Microsoft MSZH codec.
        /// A simple lossless video codec.
        Mszh = AV_CODEC_ID_MSZH as _,

        /// Zlib codec.
        /// Uses zlib compression for simple lossless video encoding.
        Zlib = AV_CODEC_ID_ZLIB as _,

        /// QuickTime RLE codec.
        /// A run-length encoding format used in QuickTime movies.
        Qtrle = AV_CODEC_ID_QTRLE as _,

        /// TechSmith Screen Capture Codec.
        /// Used in Camtasia screen recordings.
        Tscc = AV_CODEC_ID_TSCC as _,

        /// Ultimotion codec.
        /// Developed by IBM for early digital video.
        Ulti = AV_CODEC_ID_ULTI as _,

        /// QuickDraw codec.
        /// A legacy codec used in Apple QuickTime.
        Qdraw = AV_CODEC_ID_QDRAW as _,

        /// VIXL codec.
        /// A lesser-known video codec.
        Vixl = AV_CODEC_ID_VIXL as _,

        /// QPEG codec.
        /// Used in old video playback software.
        Qpeg = AV_CODEC_ID_QPEG as _,

        /// PNG codec.
        /// A lossless image format that can also store video sequences.
        Png = AV_CODEC_ID_PNG as _,

        /// Portable Pixmap (PPM) codec.
        /// A simple, uncompressed image format.
        Ppm = AV_CODEC_ID_PPM as _,

        /// Portable Bitmap (PBM) codec.
        /// A monochrome image format.
        Pbm = AV_CODEC_ID_PBM as _,

        /// Portable Graymap (PGM) codec.
        /// A grayscale image format.
        Pgm = AV_CODEC_ID_PGM as _,

        /// Portable Graymap with YUV format (PGMYUV).
        /// A grayscale format with additional chroma information.
        PgmYuv = AV_CODEC_ID_PGMYUV as _,

        /// Portable Arbitrary Map (PAM) codec.
        /// A more flexible version of PNM image formats.
        Pam = AV_CODEC_ID_PAM as _,

        /// FFmpeg Huffman codec.
        /// A lossless video compression format.
        FfvHuff = AV_CODEC_ID_FFVHUFF as _,

        /// RealVideo 3.0 codec.
        /// Used in RealMedia streaming.
        Rv30 = AV_CODEC_ID_RV30 as _,

        /// RealVideo 4.0 codec.
        /// An improved version of RealVideo 3.0.
        Rv40 = AV_CODEC_ID_RV40 as _,

        /// VC-1 codec.
        /// A video codec developed by Microsoft, used in Blu-ray and streaming.
        Vc1 = AV_CODEC_ID_VC1 as _,

        /// Windows Media Video 9 codec.
        /// Also known as VC-1 Simple/Main profile.
        Wmv3 = AV_CODEC_ID_WMV3 as _,

        /// LOCO codec.
        /// A low-complexity lossless video codec.
        Loco = AV_CODEC_ID_LOCO as _,

        /// Winnov WNV1 codec.
        /// Used in some early video capture cards.
        Wnv1 = AV_CODEC_ID_WNV1 as _,

        /// Autodesk AASC codec.
        /// Used for animation compression in early Autodesk software.
        Aasc = AV_CODEC_ID_AASC as _,

        /// Indeo Video 2 codec.
        /// A proprietary format from Intel, predating Indeo 3.
        Indeo2 = AV_CODEC_ID_INDEO2 as _,

        /// Fraps codec.
        /// A lossless codec used in game recording software.
        Fraps = AV_CODEC_ID_FRAPS as _,

        /// TrueMotion 2 codec.
        /// An improved version of TrueMotion 1, used in older games.
        Truemotion2 = AV_CODEC_ID_TRUEMOTION2 as _,

        /// BMP codec.
        /// A lossless image format commonly used for raw bitmaps.
        Bmp = AV_CODEC_ID_BMP as _,

        /// CamStudio codec.
        /// Used in screen recording software.
        Cscd = AV_CODEC_ID_CSCD as _,

        /// American Laser Games codec.
        /// Used in arcade laserdisc-based games.
        MmVideo = AV_CODEC_ID_MMVIDEO as _,

        /// DosBox ZMBV codec.
        /// A lossless video codec optimized for DOSBox.
        Zmbv = AV_CODEC_ID_ZMBV as _,

        /// AVS Video codec.
        /// Used in Chinese digital television broadcasting.
        Avs = AV_CODEC_ID_AVS as _,

        /// Smacker Video codec.
        /// Used in video game cutscenes.
        SmackVideo = AV_CODEC_ID_SMACKVIDEO as _,

        /// NuppelVideo codec.
        /// Used in MythTV for recording TV broadcasts.
        Nuv = AV_CODEC_ID_NUV as _,

        /// Karl Morton's Video Codec.
        /// Used in certain retro multimedia applications.
        Kmvc = AV_CODEC_ID_KMVC as _,

        /// Flash Screen Video codec.
        /// Used in early versions of Adobe Flash video.
        FlashSv = AV_CODEC_ID_FLASHSV as _,

        /// Chinese AVS video codec.
        /// Similar to H.264, used in Chinese video applications.
        Cavs = AV_CODEC_ID_CAVS as _,

        /// JPEG 2000 codec.
        /// A successor to JPEG, offering better compression and quality.
        Jpeg2000 = AV_CODEC_ID_JPEG2000 as _,

        /// VMware Video codec.
        /// Used in VMware Workstation recordings.
        Vmnc = AV_CODEC_ID_VMNC as _,

        /// VP5 codec.
        /// A proprietary On2 video codec, predecessor to VP6.
        Vp5 = AV_CODEC_ID_VP5 as _,

        /// VP6 codec.
        /// A widely used On2 video codec, often found in Flash video.
        Vp6 = AV_CODEC_ID_VP6 as _,

        /// VP6 Flash codec.
        /// A variant of VP6 optimized for Adobe Flash.
        Vp6F = AV_CODEC_ID_VP6F as _,

        /// Targa video codec.
        /// Used for storing uncompressed TGA images in video sequences.
        Targa = AV_CODEC_ID_TARGA as _,

        /// DSICIN Video codec.
        /// Used in games by Westwood Studios.
        DsicinVideo = AV_CODEC_ID_DSICINVIDEO as _,

        /// Tiertex SEQ Video codec.
        /// Used in old DOS and Amiga video games.
        TiertexSeqVideo = AV_CODEC_ID_TIERTEXSEQVIDEO as _,

        /// TIFF codec.
        /// A flexible image format supporting both lossless and compressed storage.
        Tiff = AV_CODEC_ID_TIFF as _,

        /// GIF codec.
        /// Used for simple animations and images with transparency.
        Gif = AV_CODEC_ID_GIF as _,

        /// DXA codec.
        /// Used in Feeble Files and Broken Sword game cutscenes.
        Dxa = AV_CODEC_ID_DXA as _,

        /// DNxHD codec.
        /// A professional intermediate codec developed by Avid.
        DnxHd = AV_CODEC_ID_DNXHD as _,

        /// THP Video codec.
        /// Used in cutscenes on the Nintendo GameCube and Wii.
        Thp = AV_CODEC_ID_THP as _,

        /// SGI Video codec.
        /// A legacy format used on SGI workstations.
        Sgi = AV_CODEC_ID_SGI as _,

        /// C93 Video codec.
        /// Used in some Sierra game cutscenes.
        C93 = AV_CODEC_ID_C93 as _,

        /// Bethesda Softworks Video codec.
        /// Used in older Bethesda games.
        BethSoftVid = AV_CODEC_ID_BETHSOFTVID as _,

        /// PowerTV PTX codec.
        /// A proprietary video format.
        Ptx = AV_CODEC_ID_PTX as _,

        /// RenderWare TXD codec.
        /// Used in Grand Theft Auto III and other RenderWare-based games.
        Txd = AV_CODEC_ID_TXD as _,

        /// VP6A codec.
        /// A variant of VP6 with alpha channel support.
        Vp6A = AV_CODEC_ID_VP6A as _,

        /// Anime Music Video codec.
        /// A simple codec used for encoding anime clips.
        Amv = AV_CODEC_ID_AMV as _,

        /// Beam Software VB codec.
        /// Used in older game cutscenes.
        Vb = AV_CODEC_ID_VB as _,

        /// PCX codec.
        /// A legacy image format from the DOS era.
        Pcx = AV_CODEC_ID_PCX as _,

        /// Sun Raster Image codec.
        /// A legacy image format from Sun Microsystems.
        Sunrast = AV_CODEC_ID_SUNRAST as _,

        /// Indeo Video 4 codec.
        /// An improved version of Indeo 3 with better compression.
        Indeo4 = AV_CODEC_ID_INDEO4 as _,

        /// Indeo Video 5 codec.
        /// A later version of Indeo with better efficiency.
        Indeo5 = AV_CODEC_ID_INDEO5 as _,

        /// Mimic codec.
        /// Used in certain screen recording applications.
        Mimic = AV_CODEC_ID_MIMIC as _,

        /// Escape 124 codec.
        /// A proprietary video compression format.
        Escape124 = AV_CODEC_ID_ESCAPE124 as _,

        /// Dirac codec.
        /// An open-source video codec developed by the BBC.
        Dirac = AV_CODEC_ID_DIRAC as _,

        /// Bink Video codec.
        /// Used in many game cutscenes.
        BinkVideo = AV_CODEC_ID_BINKVIDEO as _,

        /// IFF Interleaved Bitmap codec.
        /// Used in Amiga image files.
        IffIlbm = AV_CODEC_ID_IFF_ILBM as _,

        /// KGV1 codec.
        /// A proprietary video format.
        Kgv1 = AV_CODEC_ID_KGV1 as _,

        /// YOP Video codec.
        /// Used in some video game cutscenes.
        Yop = AV_CODEC_ID_YOP as _,

        /// VP8 codec.
        /// A widely used open-source video codec, a predecessor to VP9.
        Vp8 = AV_CODEC_ID_VP8 as _,

        /// Pictor codec.
        /// Used in early graphic applications.
        Pictor = AV_CODEC_ID_PICTOR as _,

        /// ANSI Art codec.
        /// Used for text-based animations.
        Ansi = AV_CODEC_ID_ANSI as _,

        /// A64 Multi codec.
        /// Used for encoding video in the Commodore 64 format.
        A64Multi = AV_CODEC_ID_A64_MULTI as _,

        /// A64 Multi5 codec.
        /// A variant of A64 Multi with additional encoding options.
        A64Multi5 = AV_CODEC_ID_A64_MULTI5 as _,

        /// R10K codec.
        /// A high-bit-depth raw video format.
        R10K = AV_CODEC_ID_R10K as _,

        /// MXPEG codec.
        /// A proprietary codec used in security cameras.
        MxPeg = AV_CODEC_ID_MXPEG as _,

        /// Lagarith codec.
        /// A lossless video codec used for archival purposes.
        Lagarith = AV_CODEC_ID_LAGARITH as _,

        /// Apple ProRes codec.
        /// A professional intermediate codec commonly used in video editing.
        ProRes = AV_CODEC_ID_PRORES as _,

        /// Bitmap Brothers JV codec.
        /// Used in old games for video sequences.
        Jv = AV_CODEC_ID_JV as _,

        /// DFA codec.
        /// A proprietary format used in some multimedia applications.
        Dfa = AV_CODEC_ID_DFA as _,

        /// WMV3 Image codec.
        /// A still image format based on Windows Media Video 9.
        Wmv3Image = AV_CODEC_ID_WMV3IMAGE as _,

        /// VC-1 Image codec.
        /// A still image format based on the VC-1 video codec.
        Vc1Image = AV_CODEC_ID_VC1IMAGE as _,

        /// Ut Video codec.
        /// A lossless video codec optimized for fast encoding and decoding.
        UtVideo = AV_CODEC_ID_UTVIDEO as _,

        /// BMV Video codec.
        /// Used in some old video games.
        BmvVideo = AV_CODEC_ID_BMV_VIDEO as _,

        /// VBLE codec.
        /// A proprietary video compression format.
        Vble = AV_CODEC_ID_VBLE as _,

        /// Dxtory codec.
        /// Used in game recording software for high-performance capture.
        Dxtory = AV_CODEC_ID_DXTORY as _,

        /// V410 codec.
        /// A 10-bit YUV 4:4:4 format.
        V410 = AV_CODEC_ID_V410 as _,

        /// XWD codec.
        /// Used for storing window dumps from the X Window System.
        Xwd = AV_CODEC_ID_XWD as _,

        /// CDXL codec.
        /// An animation format used on the Commodore Amiga.
        Cdxl = AV_CODEC_ID_CDXL as _,

        /// XBM codec.
        /// A simple monochrome bitmap format used in X11.
        Xbm = AV_CODEC_ID_XBM as _,

        /// ZeroCodec.
        /// A lossless video codec used in screen recording.
        ZeroCodec = AV_CODEC_ID_ZEROCODEC as _,

        /// MSS1 codec.
        /// Microsoft Screen Codec 1, used for remote desktop applications.
        Mss1 = AV_CODEC_ID_MSS1 as _,

        /// MSA1 codec.
        /// Microsoft Screen Codec 2, an improved version of MSS1.
        Msa1 = AV_CODEC_ID_MSA1 as _,

        /// TSCC2 codec.
        /// A version of TechSmith Screen Capture Codec.
        Tscc2 = AV_CODEC_ID_TSCC2 as _,

        /// MTS2 codec.
        /// A proprietary video format.
        Mts2 = AV_CODEC_ID_MTS2 as _,

        /// CLLC codec.
        /// A proprietary video codec.
        Cllc = AV_CODEC_ID_CLLC as _,

        /// MSS2 codec.
        /// Microsoft Screen Codec 2, used in Windows Media video recordings.
        Mss2 = AV_CODEC_ID_MSS2 as _,

        /// VP9 codec.
        /// A popular open-source video codec, successor to VP8.
        Vp9 = AV_CODEC_ID_VP9 as _,

        /// AIC codec.
        /// Apple Intermediate Codec, used for professional video editing.
        Aic = AV_CODEC_ID_AIC as _,

        /// Escape 130 codec.
        /// A proprietary video compression format.
        Escape130 = AV_CODEC_ID_ESCAPE130 as _,

        /// G2M codec.
        /// GoToMeeting screen recording codec.
        G2M = AV_CODEC_ID_G2M as _,

        /// WebP codec.
        /// A modern image format optimized for the web.
        WebP = AV_CODEC_ID_WEBP as _,

        /// HNM4 Video codec.
        /// Used in some video game cutscenes.
        Hnm4Video = AV_CODEC_ID_HNM4_VIDEO as _,

        /// HEVC (H.265) codec.
        /// A high-efficiency video codec, successor to H.264.
        Hevc = AV_CODEC_ID_HEVC as _,

        /// FIC codec.
        /// A proprietary video compression format.
        Fic = AV_CODEC_ID_FIC as _,

        /// Alias PIX codec.
        /// Used in old Alias/Wavefront animations.
        AliasPix = AV_CODEC_ID_ALIAS_PIX as _,

        /// BRender PIX codec.
        /// A proprietary video compression format.
        BRenderPix = AV_CODEC_ID_BRENDER_PIX as _,

        /// PAF Video codec.
        /// Used in some multimedia applications.
        PafVideo = AV_CODEC_ID_PAF_VIDEO as _,

        /// OpenEXR codec.
        /// A high-dynamic-range image format used in film production.
        Exr = AV_CODEC_ID_EXR as _,

        /// VP7 codec.
        /// An older proprietary video codec from On2 Technologies.
        Vp7 = AV_CODEC_ID_VP7 as _,

        /// SANM codec.
        /// A proprietary video format.
        Sanm = AV_CODEC_ID_SANM as _,

        /// SGI RLE codec.
        /// A run-length encoding format used on SGI workstations.
        SgiRle = AV_CODEC_ID_SGIRLE as _,

        /// MVC1 codec.
        /// Multiview Video Coding (MVC) for stereoscopic 3D video.
        Mvc1 = AV_CODEC_ID_MVC1 as _,

        /// MVC2 codec.
        /// Another variant of Multiview Video Coding.
        Mvc2 = AV_CODEC_ID_MVC2 as _,

        /// HQX codec.
        /// A high-quality video codec.
        Hqx = AV_CODEC_ID_HQX as _,

        /// TDSC codec.
        /// A proprietary video compression format.
        Tdsc = AV_CODEC_ID_TDSC as _,

        /// HQ/HQA codec.
        /// A professional-grade video codec.
        HqHqa = AV_CODEC_ID_HQ_HQA as _,

        /// HAP codec.
        /// A high-performance video codec for real-time applications.
        Hap = AV_CODEC_ID_HAP as _,

        /// DDS codec.
        /// A format used for texture compression in graphics applications.
        Dds = AV_CODEC_ID_DDS as _,

        /// DXV codec.
        /// A proprietary video codec used in Resolume VJ software.
        Dxv = AV_CODEC_ID_DXV as _,

        /// Screenpresso codec.
        /// A proprietary screen recording codec.
        Screenpresso = AV_CODEC_ID_SCREENPRESSO as _,

        /// RSCC codec.
        /// A proprietary screen capture codec.
        Rscc = AV_CODEC_ID_RSCC as _,

        /// AVS2 codec.
        /// A Chinese video codec similar to H.264.
        Avs2 = AV_CODEC_ID_AVS2 as _,

        /// PGX codec.
        /// A simple image format.
        Pgx = AV_CODEC_ID_PGX as _,

        /// AVS3 codec.
        /// A next-generation video codec developed in China.
        Avs3 = AV_CODEC_ID_AVS3 as _,

        /// MSP2 codec.
        /// A proprietary video format.
        Msp2 = AV_CODEC_ID_MSP2 as _,

        /// VVC codec (H.266).
        /// A next-generation video compression standard.
        Vvc = AV_CODEC_ID_VVC as _,

        /// Y41P codec.
        /// A planar YUV format.
        Y41P = AV_CODEC_ID_Y41P as _,

        /// AVRP codec.
        /// A simple video format.
        Avrp = AV_CODEC_ID_AVRP as _,

        /// 012V codec.
        /// A proprietary video compression format.
        Zero12V = AV_CODEC_ID_012V as _,

        /// AVUI codec.
        /// A proprietary video format.
        Avui = AV_CODEC_ID_AVUI as _,

        /// Targa Y216 codec.
        /// A format for storing uncompressed YUV video.
        TargaY216 = AV_CODEC_ID_TARGA_Y216 as _,

        /// V308 codec.
        /// A planar YUV 4:4:4 format.
        V308 = AV_CODEC_ID_V308 as _,

        /// V408 codec.
        /// A planar YUV 4:4:4 format with alpha.
        V408 = AV_CODEC_ID_V408 as _,

        /// YUV4 codec.
        /// A raw YUV video format.
        Yuv4 = AV_CODEC_ID_YUV4 as _,

        /// AVRN codec.
        /// A proprietary video compression format.
        Avrn = AV_CODEC_ID_AVRN as _,

        /// CPIA codec.
        /// Used in early webcams.
        Cpia = AV_CODEC_ID_CPIA as _,

        /// XFace codec.
        /// A low-bandwidth animated face codec.
        XFace = AV_CODEC_ID_XFACE as _,

        /// Snow codec.
        /// A wavelet-based video codec developed by FFmpeg.
        Snow = AV_CODEC_ID_SNOW as _,

        /// SMVJPEG codec.
        /// A variant of Motion JPEG.
        SmvJpeg = AV_CODEC_ID_SMVJPEG as _,

        /// APNG codec.
        /// Animated PNG format.
        Apng = AV_CODEC_ID_APNG as _,

        /// Daala codec.
        /// An experimental open-source video codec.
        Daala = AV_CODEC_ID_DAALA as _,

        /// CineForm HD codec.
        /// A professional-grade intermediate codec.
        Cfhd = AV_CODEC_ID_CFHD as _,

        /// TrueMotion 2RT codec.
        /// A real-time variant of TrueMotion 2.
        Truemotion2Rt = AV_CODEC_ID_TRUEMOTION2RT as _,

        /// M101 codec.
        /// A proprietary video format.
        M101 = AV_CODEC_ID_M101 as _,

        /// MagicYUV codec.
        /// A high-performance lossless video codec.
        MagicYuv = AV_CODEC_ID_MAGICYUV as _,

        /// SheerVideo codec.
        /// A professional-grade lossless video codec.
        SheerVideo = AV_CODEC_ID_SHEERVIDEO as _,

        /// YLC codec.
        /// A proprietary video compression format.
        Ylc = AV_CODEC_ID_YLC as _,

        /// PSD codec.
        /// Adobe Photoshop image format.
        Psd = AV_CODEC_ID_PSD as _,

        /// Pixlet codec.
        /// A video codec developed by Apple for high-performance playback.
        Pixlet = AV_CODEC_ID_PIXLET as _,

        /// SpeedHQ codec.
        /// A proprietary intermediate codec developed by NewTek.
        SpeedHq = AV_CODEC_ID_SPEEDHQ as _,

        /// FMVC codec.
        /// A proprietary video format.
        Fmvc = AV_CODEC_ID_FMVC as _,

        /// SCPR codec.
        /// A screen recording codec.
        Scpr = AV_CODEC_ID_SCPR as _,

        /// ClearVideo codec.
        /// A wavelet-based video compression format.
        ClearVideo = AV_CODEC_ID_CLEARVIDEO as _,

        /// XPM codec.
        /// X Pixmap format, used in X Window System.
        Xpm = AV_CODEC_ID_XPM as _,

        /// AV1 codec.
        /// A modern open-source video codec designed for high compression efficiency.
        Av1 = AV_CODEC_ID_AV1 as _,

        /// BitPacked codec.
        /// A proprietary bit-packing format.
        BitPacked = AV_CODEC_ID_BITPACKED as _,

        /// MSCC codec.
        /// A proprietary video format.
        Mscc = AV_CODEC_ID_MSCC as _,

        /// SRGC codec.
        /// A proprietary video format.
        Srgc = AV_CODEC_ID_SRGC as _,

        /// SVG codec.
        /// Scalable Vector Graphics format.
        Svg = AV_CODEC_ID_SVG as _,

        /// GDV codec.
        /// A proprietary video format.
        Gdv = AV_CODEC_ID_GDV as _,

        /// FITS codec.
        /// Flexible Image Transport System, used in astronomy.
        Fits = AV_CODEC_ID_FITS as _,

        /// IMM4 codec.
        /// A proprietary video format.
        Imm4 = AV_CODEC_ID_IMM4 as _,

        /// Prosumer codec.
        /// A proprietary video format.
        Prosumer = AV_CODEC_ID_PROSUMER as _,

        /// MWSC codec.
        /// A proprietary video format.
        Mwsc = AV_CODEC_ID_MWSC as _,

        /// WCMV codec.
        /// A proprietary video format.
        Wcmv = AV_CODEC_ID_WCMV as _,

        /// RASC codec.
        /// A proprietary video format.
        Rasc = AV_CODEC_ID_RASC as _,

        /// HYMT codec.
        /// A proprietary video compression format.
        Hymt = AV_CODEC_ID_HYMT as _,

        /// ARBC codec.
        /// A proprietary video format.
        Arbc = AV_CODEC_ID_ARBC as _,

        /// AGM codec.
        /// A proprietary video format.
        Agm = AV_CODEC_ID_AGM as _,

        /// LSCR codec.
        /// A proprietary video format.
        Lscr = AV_CODEC_ID_LSCR as _,

        /// VP4 codec.
        /// An early proprietary video codec from On2 Technologies.
        Vp4 = AV_CODEC_ID_VP4 as _,

        /// IMM5 codec.
        /// A proprietary video format.
        Imm5 = AV_CODEC_ID_IMM5 as _,

        /// MVDV codec.
        /// A proprietary video format.
        Mvdv = AV_CODEC_ID_MVDV as _,

        /// MVHA codec.
        /// A proprietary video format.
        Mvha = AV_CODEC_ID_MVHA as _,

        /// CDToons codec.
        /// A proprietary video format.
        CdToons = AV_CODEC_ID_CDTOONS as _,

        /// MV30 codec.
        /// A proprietary video format.
        Mv30 = AV_CODEC_ID_MV30 as _,

        /// NotchLC codec.
        /// A GPU-accelerated intermediate codec for Notch software.
        NotchLc = AV_CODEC_ID_NOTCHLC as _,

        /// PFM codec.
        /// Portable FloatMap image format.
        Pfm = AV_CODEC_ID_PFM as _,

        /// MobiClip codec.
        /// A proprietary video format used in Nintendo DS games.
        MobiClip = AV_CODEC_ID_MOBICLIP as _,

        /// PhotoCD codec.
        /// A high-quality image format used for storing photographs.
        PhotoCd = AV_CODEC_ID_PHOTOCD as _,

        /// IPU codec.
        /// Used in PlayStation 2 video playback.
        Ipu = AV_CODEC_ID_IPU as _,

        /// Argo codec.
        /// A proprietary video format.
        Argo = AV_CODEC_ID_ARGO as _,

        /// CRI codec.
        /// A proprietary video format used in Japanese games.
        Cri = AV_CODEC_ID_CRI as _,

        /// Simbiosis IMX codec.
        /// A proprietary video format.
        SimbiosisImx = AV_CODEC_ID_SIMBIOSIS_IMX as _,

        /// SGA Video codec.
        /// A proprietary video format.
        SgaVideo = AV_CODEC_ID_SGA_VIDEO as _,

        /// GEM codec.
        /// A proprietary video format.
        Gem = AV_CODEC_ID_GEM as _,

        /// VBN codec.
        /// A proprietary video format.
        Vbn = AV_CODEC_ID_VBN as _,

        /// JPEG XL codec.
        /// A modern successor to JPEG with better compression and quality.
        JpegXl = AV_CODEC_ID_JPEGXL as _,

        /// QOI codec.
        /// Quite OK Image format, a simple lossless image format.
        Qoi = AV_CODEC_ID_QOI as _,

        /// PHM codec.
        /// A proprietary image format.
        Phm = AV_CODEC_ID_PHM as _,

        /// Radiance HDR codec.
        /// A high-dynamic-range image format.
        RadianceHdr = AV_CODEC_ID_RADIANCE_HDR as _,

        /// WBMP codec.
        /// Wireless Bitmap format, used in early mobile applications.
        Wbmp = AV_CODEC_ID_WBMP as _,

        /// Media100 codec.
        /// A professional video format.
        Media100 = AV_CODEC_ID_MEDIA100 as _,

        /// VQC codec.
        /// A proprietary video format.
        Vqc = AV_CODEC_ID_VQC as _,

        /// PDV codec.
        /// A proprietary video format.
        Pdv = AV_CODEC_ID_PDV as _,

        /// EVC codec.
        /// Essential Video Coding, a next-generation video format.
        Evc = AV_CODEC_ID_EVC as _,

        /// RTV1 codec.
        /// A proprietary video format.
        Rtv1 = AV_CODEC_ID_RTV1 as _,

        /// VMIX codec.
        /// A proprietary video format.
        Vmix = AV_CODEC_ID_VMIX as _,

        /// LEAD codec.
        /// A proprietary video format.
        Lead = AV_CODEC_ID_LEAD as _,

        /// PCM Signed 16-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmS16Le = AV_CODEC_ID_PCM_S16LE as _,

        /// PCM Signed 16-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmS16Be = AV_CODEC_ID_PCM_S16BE as _,

        /// PCM Unsigned 16-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmU16Le = AV_CODEC_ID_PCM_U16LE as _,

        /// PCM Unsigned 16-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmU16Be = AV_CODEC_ID_PCM_U16BE as _,

        /// PCM Signed 8-bit codec.
        /// Uncompressed raw audio format.
        PcmS8 = AV_CODEC_ID_PCM_S8 as _,

        /// PCM Unsigned 8-bit codec.
        /// Uncompressed raw audio format.
        PcmU8 = AV_CODEC_ID_PCM_U8 as _,

        /// PCM Mu-Law codec.
        /// A logarithmic audio compression format used in telephony.
        PcmMuLaw = AV_CODEC_ID_PCM_MULAW as _,

        /// PCM A-Law codec.
        /// A logarithmic audio compression format used in telephony.
        PcmALaw = AV_CODEC_ID_PCM_ALAW as _,

        /// PCM Signed 32-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmS32Le = AV_CODEC_ID_PCM_S32LE as _,

        /// PCM Signed 32-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmS32Be = AV_CODEC_ID_PCM_S32BE as _,

        /// PCM Unsigned 32-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmU32Le = AV_CODEC_ID_PCM_U32LE as _,

        /// PCM Unsigned 32-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmU32Be = AV_CODEC_ID_PCM_U32BE as _,

        /// PCM Signed 24-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmS24Le = AV_CODEC_ID_PCM_S24LE as _,

        /// PCM Signed 24-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmS24Be = AV_CODEC_ID_PCM_S24BE as _,

        /// PCM Unsigned 24-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmU24Le = AV_CODEC_ID_PCM_U24LE as _,

        /// PCM Unsigned 24-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmU24Be = AV_CODEC_ID_PCM_U24BE as _,

        /// PCM Signed 24-bit DAUD codec.
        /// Used in digital audio applications.
        PcmS24Daud = AV_CODEC_ID_PCM_S24DAUD as _,

        /// PCM Zork codec.
        /// A proprietary raw audio format.
        PcmZork = AV_CODEC_ID_PCM_ZORK as _,

        /// PCM Signed 16-bit Little Endian Planar codec.
        /// Uncompressed raw audio format stored in planar format.
        PcmS16LePlanar = AV_CODEC_ID_PCM_S16LE_PLANAR as _,

        /// PCM DVD codec.
        /// Used for storing PCM audio in DVD media.
        PcmDvd = AV_CODEC_ID_PCM_DVD as _,

        /// PCM Floating-Point 32-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmF32Be = AV_CODEC_ID_PCM_F32BE as _,

        /// PCM Floating-Point 32-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmF32Le = AV_CODEC_ID_PCM_F32LE as _,

        /// PCM Floating-Point 64-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmF64Be = AV_CODEC_ID_PCM_F64BE as _,

        /// PCM Floating-Point 64-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmF64Le = AV_CODEC_ID_PCM_F64LE as _,

        /// PCM Blu-ray codec.
        /// Used in Blu-ray Disc audio.
        PcmBluray = AV_CODEC_ID_PCM_BLURAY as _,

        /// PCM LXF codec.
        /// Used in Leitch/Harris LXF broadcast video format.
        PcmLxf = AV_CODEC_ID_PCM_LXF as _,

        /// S302M codec.
        /// Used in professional audio applications.
        S302M = AV_CODEC_ID_S302M as _,

        /// PCM Signed 8-bit Planar codec.
        /// Uncompressed raw audio stored in planar format.
        PcmS8Planar = AV_CODEC_ID_PCM_S8_PLANAR as _,

        /// PCM Signed 24-bit Little Endian Planar codec.
        /// Uncompressed raw audio stored in planar format.
        PcmS24LePlanar = AV_CODEC_ID_PCM_S24LE_PLANAR as _,

        /// PCM Signed 32-bit Little Endian Planar codec.
        /// Uncompressed raw audio stored in planar format.
        PcmS32LePlanar = AV_CODEC_ID_PCM_S32LE_PLANAR as _,

        /// PCM Signed 16-bit Big Endian Planar codec.
        /// Uncompressed raw audio stored in planar format.
        PcmS16BePlanar = AV_CODEC_ID_PCM_S16BE_PLANAR as _,

        /// PCM Signed 64-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmS64Le = AV_CODEC_ID_PCM_S64LE as _,

        /// PCM Signed 64-bit Big Endian codec.
        /// Uncompressed raw audio format.
        PcmS64Be = AV_CODEC_ID_PCM_S64BE as _,

        /// PCM Floating-Point 16-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmF16Le = AV_CODEC_ID_PCM_F16LE as _,

        /// PCM Floating-Point 24-bit Little Endian codec.
        /// Uncompressed raw audio format.
        PcmF24Le = AV_CODEC_ID_PCM_F24LE as _,

        /// PCM VIDC codec.
        /// A proprietary raw audio format.
        PcmVidc = AV_CODEC_ID_PCM_VIDC as _,

        /// PCM SGA codec.
        /// A proprietary raw audio format.
        PcmSga = AV_CODEC_ID_PCM_SGA as _,

        /// ADPCM IMA QuickTime codec.
        /// Adaptive Differential Pulse-Code Modulation used in QuickTime.
        AdpcmImaQt = AV_CODEC_ID_ADPCM_IMA_QT as _,

        /// ADPCM IMA WAV codec.
        /// Adaptive Differential Pulse-Code Modulation used in WAV files.
        AdpcmImaWav = AV_CODEC_ID_ADPCM_IMA_WAV as _,

        /// ADPCM IMA DK3 codec.
        /// Adaptive Differential Pulse-Code Modulation, variant DK3.
        AdpcmImaDk3 = AV_CODEC_ID_ADPCM_IMA_DK3 as _,

        /// ADPCM IMA DK4 codec.
        /// Adaptive Differential Pulse-Code Modulation, variant DK4.
        AdpcmImaDk4 = AV_CODEC_ID_ADPCM_IMA_DK4 as _,

        /// ADPCM IMA Westwood codec.
        /// Used in Westwood Studios video games.
        AdpcmImaWs = AV_CODEC_ID_ADPCM_IMA_WS as _,

        /// ADPCM IMA Smacker codec.
        /// Used in Smacker video format.
        AdpcmImaSmjpeg = AV_CODEC_ID_ADPCM_IMA_SMJPEG as _,

        /// ADPCM Microsoft codec.
        /// Microsoft variant of Adaptive Differential Pulse-Code Modulation.
        AdpcmMs = AV_CODEC_ID_ADPCM_MS as _,

        /// ADPCM 4X Movie codec.
        /// Used in 4X Movie video format.
        Adpcm4Xm = AV_CODEC_ID_ADPCM_4XM as _,

        /// ADPCM XA codec.
        /// Used in PlayStation XA audio format.
        AdpcmXa = AV_CODEC_ID_ADPCM_XA as _,

        /// ADPCM ADX codec.
        /// Used in ADX audio format, common in Sega games.
        AdpcmAdx = AV_CODEC_ID_ADPCM_ADX as _,

        /// ADPCM Electronic Arts codec.
        /// Used in Electronic Arts games.
        AdpcmEa = AV_CODEC_ID_ADPCM_EA as _,

        /// ADPCM G.726 codec.
        /// ITU-T standard for speech compression.
        AdpcmG726 = AV_CODEC_ID_ADPCM_G726 as _,

        /// ADPCM Creative codec.
        /// Used in Creative Labs sound hardware.
        AdpcmCt = AV_CODEC_ID_ADPCM_CT as _,

        /// ADPCM SWF codec.
        /// Used in Adobe Flash audio.
        AdpcmSwf = AV_CODEC_ID_ADPCM_SWF as _,

        /// ADPCM Yamaha codec.
        /// A variant of ADPCM used in Yamaha audio applications.
        AdpcmYamaha = AV_CODEC_ID_ADPCM_YAMAHA as _,

        /// ADPCM Sound Blaster Pro 4-bit codec.
        /// Used in Sound Blaster Pro hardware.
        AdpcmSbpro4 = AV_CODEC_ID_ADPCM_SBPRO_4 as _,

        /// ADPCM Sound Blaster Pro 3-bit codec.
        /// Used in Sound Blaster Pro hardware.
        AdpcmSbpro3 = AV_CODEC_ID_ADPCM_SBPRO_3 as _,

        /// ADPCM Sound Blaster Pro 2-bit codec.
        /// Used in Sound Blaster Pro hardware.
        AdpcmSbpro2 = AV_CODEC_ID_ADPCM_SBPRO_2 as _,

        /// ADPCM THP codec.
        /// Used in Nintendo THP video files.
        AdpcmThp = AV_CODEC_ID_ADPCM_THP as _,

        /// ADPCM IMA AMV codec.
        /// Used in AMV video format.
        AdpcmImaAmv = AV_CODEC_ID_ADPCM_IMA_AMV as _,

        /// ADPCM Electronic Arts R1 codec.
        /// Used in EA games.
        AdpcmEaR1 = AV_CODEC_ID_ADPCM_EA_R1 as _,

        /// ADPCM Electronic Arts R3 codec.
        /// Used in EA games.
        AdpcmEaR3 = AV_CODEC_ID_ADPCM_EA_R3 as _,

        /// ADPCM Electronic Arts R2 codec.
        /// Used in EA games.
        AdpcmEaR2 = AV_CODEC_ID_ADPCM_EA_R2 as _,

        /// ADPCM IMA Electronic Arts SEAD codec.
        /// Used in Electronic Arts games.
        AdpcmImaEaSead = AV_CODEC_ID_ADPCM_IMA_EA_SEAD as _,

        /// ADPCM IMA Electronic Arts EACS codec.
        /// Used in Electronic Arts games.
        AdpcmImaEaEacs = AV_CODEC_ID_ADPCM_IMA_EA_EACS as _,

        /// ADPCM Electronic Arts XAS codec.
        /// Used in Electronic Arts games.
        AdpcmEaXas = AV_CODEC_ID_ADPCM_EA_XAS as _,

        /// ADPCM Electronic Arts Maxis XA codec.
        /// Used in Maxis-developed games.
        AdpcmEaMaxisXa = AV_CODEC_ID_ADPCM_EA_MAXIS_XA as _,

        /// ADPCM IMA ISS codec.
        /// Used in ISS audio format.
        AdpcmImaIss = AV_CODEC_ID_ADPCM_IMA_ISS as _,

        /// ADPCM G.722 codec.
        /// Used in telephony applications.
        AdpcmG722 = AV_CODEC_ID_ADPCM_G722 as _,

        /// ADPCM IMA APC codec.
        /// A proprietary ADPCM format.
        AdpcmImaApc = AV_CODEC_ID_ADPCM_IMA_APC as _,

        /// ADPCM VIMA codec.
        /// A proprietary ADPCM format.
        AdpcmVima = AV_CODEC_ID_ADPCM_VIMA as _,

        /// ADPCM AFC codec.
        /// A proprietary ADPCM format.
        AdpcmAfc = AV_CODEC_ID_ADPCM_AFC as _,

        /// ADPCM IMA OKI codec.
        /// A proprietary ADPCM format.
        AdpcmImaOki = AV_CODEC_ID_ADPCM_IMA_OKI as _,

        /// ADPCM DTK codec.
        /// Used in some proprietary applications.
        AdpcmDtk = AV_CODEC_ID_ADPCM_DTK as _,

        /// ADPCM IMA RAD codec.
        /// A proprietary ADPCM format.
        AdpcmImaRad = AV_CODEC_ID_ADPCM_IMA_RAD as _,

        /// ADPCM G.726LE codec.
        /// A variant of G.726 with little-endian encoding.
        AdpcmG726Le = AV_CODEC_ID_ADPCM_G726LE as _,

        /// ADPCM THP LE codec.
        /// Used in Nintendo THP files with little-endian storage.
        AdpcmThpLe = AV_CODEC_ID_ADPCM_THP_LE as _,

        /// ADPCM PlayStation codec.
        /// Used in PlayStation audio formats.
        AdpcmPsx = AV_CODEC_ID_ADPCM_PSX as _,

        /// ADPCM AICA codec.
        /// Used in Sega Dreamcast AICA sound chip.
        AdpcmAica = AV_CODEC_ID_ADPCM_AICA as _,

        /// ADPCM IMA DAT4 codec.
        /// A proprietary ADPCM format.
        AdpcmImaDat4 = AV_CODEC_ID_ADPCM_IMA_DAT4 as _,

        /// ADPCM MTAF codec.
        /// A proprietary ADPCM format.
        AdpcmMtaf = AV_CODEC_ID_ADPCM_MTAF as _,

        /// ADPCM AGM codec.
        /// A proprietary ADPCM format.
        AdpcmAgm = AV_CODEC_ID_ADPCM_AGM as _,

        /// ADPCM Argo codec.
        /// A proprietary ADPCM format.
        AdpcmArgo = AV_CODEC_ID_ADPCM_ARGO as _,

        /// ADPCM IMA SSI codec.
        /// A proprietary ADPCM format.
        AdpcmImaSsi = AV_CODEC_ID_ADPCM_IMA_SSI as _,

        /// ADPCM Zork codec.
        /// A proprietary ADPCM format used in Zork games.
        AdpcmZork = AV_CODEC_ID_ADPCM_ZORK as _,

        /// ADPCM IMA APM codec.
        /// A proprietary ADPCM format.
        AdpcmImaApm = AV_CODEC_ID_ADPCM_IMA_APM as _,

        /// ADPCM IMA ALP codec.
        /// A proprietary ADPCM format.
        AdpcmImaAlp = AV_CODEC_ID_ADPCM_IMA_ALP as _,

        /// ADPCM IMA MTF codec.
        /// A proprietary ADPCM format.
        AdpcmImaMtf = AV_CODEC_ID_ADPCM_IMA_MTF as _,

        /// ADPCM IMA Cunning codec.
        /// A proprietary ADPCM format.
        AdpcmImaCunning = AV_CODEC_ID_ADPCM_IMA_CUNNING as _,

        /// ADPCM IMA Moflex codec.
        /// Used in Moflex multimedia format.
        AdpcmImaMoflex = AV_CODEC_ID_ADPCM_IMA_MOFLEX as _,

        /// ADPCM IMA Acorn codec.
        /// A proprietary ADPCM format.
        AdpcmImaAcorn = AV_CODEC_ID_ADPCM_IMA_ACORN as _,

        /// ADPCM XMD codec.
        /// A proprietary ADPCM format.
        AdpcmXmd = AV_CODEC_ID_ADPCM_XMD as _,

        /// AMR Narrowband codec.
        /// Adaptive Multi-Rate codec, used in mobile telephony.
        AmrNb = AV_CODEC_ID_AMR_NB as _,

        /// AMR Wideband codec.
        /// A higher-quality variant of AMR.
        AmrWb = AV_CODEC_ID_AMR_WB as _,

        /// RealAudio 1.44 kbps codec.
        /// Used in RealMedia audio streams.
        Ra144 = AV_CODEC_ID_RA_144 as _,

        /// RealAudio 2.88 kbps codec.
        /// Used in RealMedia audio streams.
        Ra288 = AV_CODEC_ID_RA_288 as _,

        /// RoQ DPCM codec.
        /// Used in video game audio, notably Quake III.
        RoqDpcm = AV_CODEC_ID_ROQ_DPCM as _,

        /// Interplay DPCM codec.
        /// Used in Interplay Entertainment video game audio.
        InterplayDpcm = AV_CODEC_ID_INTERPLAY_DPCM as _,

        /// Xan DPCM codec.
        /// Used in certain Xan-based multimedia formats.
        XanDpcm = AV_CODEC_ID_XAN_DPCM as _,

        /// Sol DPCM codec.
        /// Used in some multimedia applications.
        SolDpcm = AV_CODEC_ID_SOL_DPCM as _,

        /// SDX2 DPCM codec.
        /// A proprietary DPCM format.
        Sdx2Dpcm = AV_CODEC_ID_SDX2_DPCM as _,

        /// Gremlin DPCM codec.
        /// Used in Gremlin Interactive games.
        GremlinDpcm = AV_CODEC_ID_GREMLIN_DPCM as _,

        /// DERF DPCM codec.
        /// A proprietary DPCM format.
        DerfDpcm = AV_CODEC_ID_DERF_DPCM as _,

        /// WADY DPCM codec.
        /// A proprietary DPCM format.
        WadyDpcm = AV_CODEC_ID_WADY_DPCM as _,

        /// CBD2 DPCM codec.
        /// A proprietary DPCM format.
        Cbd2Dpcm = AV_CODEC_ID_CBD2_DPCM as _,

        /// MP2 codec.
        /// MPEG Audio Layer II, commonly used in digital radio and TV.
        Mp2 = AV_CODEC_ID_MP2 as _,

        /// MP3 codec.
        /// MPEG Audio Layer III, one of the most popular audio formats.
        Mp3 = AV_CODEC_ID_MP3 as _,

        /// AAC codec.
        /// Advanced Audio Coding, widely used in streaming and mobile applications.
        Aac = AV_CODEC_ID_AAC as _,

        /// AC3 codec.
        /// Dolby Digital audio codec, used in DVDs and broadcasting.
        Ac3 = AV_CODEC_ID_AC3 as _,

        /// DTS codec.
        /// Digital Theater Systems audio, commonly used in Blu-ray and cinema.
        Dts = AV_CODEC_ID_DTS as _,

        /// Vorbis codec.
        /// A free, open-source audio codec.
        Vorbis = AV_CODEC_ID_VORBIS as _,

        /// DV Audio codec.
        /// Used in Digital Video (DV) camcorders.
        DvAudio = AV_CODEC_ID_DVAUDIO as _,

        /// Windows Media Audio v1 codec.
        /// Early version of WMA format.
        WmaV1 = AV_CODEC_ID_WMAV1 as _,

        /// Windows Media Audio v2 codec.
        /// An improved version of WMA.
        WmaV2 = AV_CODEC_ID_WMAV2 as _,

        /// MACE 3 codec.
        /// Used in old Macintosh applications.
        Mace3 = AV_CODEC_ID_MACE3 as _,

        /// MACE 6 codec.
        /// A higher compression variant of MACE 3.
        Mace6 = AV_CODEC_ID_MACE6 as _,

        /// VMD Audio codec.
        /// Used in Sierra VMD multimedia format.
        VmdAudio = AV_CODEC_ID_VMDAUDIO as _,

        /// FLAC codec.
        /// Free Lossless Audio Codec, widely used for high-quality audio storage.
        Flac = AV_CODEC_ID_FLAC as _,

        /// MP3 ADU codec.
        /// A variant of MP3 optimized for streaming.
        Mp3Adu = AV_CODEC_ID_MP3ADU as _,

        /// MP3-on-MP4 codec.
        /// MP3 audio stored in an MP4 container.
        Mp3On4 = AV_CODEC_ID_MP3ON4 as _,

        /// Shorten codec.
        /// A lossless audio compression format.
        Shorten = AV_CODEC_ID_SHORTEN as _,

        /// ALAC codec.
        /// Apple Lossless Audio Codec, used in iTunes and Apple devices.
        Alac = AV_CODEC_ID_ALAC as _,

        /// Westwood SND1 codec.
        /// Used in Westwood Studios games.
        WestwoodSnd1 = AV_CODEC_ID_WESTWOOD_SND1 as _,

        /// GSM codec.
        /// A low-bitrate speech codec used in mobile networks.
        Gsm = AV_CODEC_ID_GSM as _,

        /// QDM2 codec.
        /// Used in older QuickTime audio formats.
        Qdm2 = AV_CODEC_ID_QDM2 as _,

        /// Cook codec.
        /// A proprietary RealAudio format.
        Cook = AV_CODEC_ID_COOK as _,

        /// TrueSpeech codec.
        /// A low-bitrate speech codec developed by DSP Group.
        TrueSpeech = AV_CODEC_ID_TRUESPEECH as _,

        /// TTA codec.
        /// The True Audio codec, a lossless compression format.
        Tta = AV_CODEC_ID_TTA as _,

        /// Smacker Audio codec.
        /// Used in Smacker video files.
        SmackAudio = AV_CODEC_ID_SMACKAUDIO as _,

        /// QCELP codec.
        /// Qualcomm's PureVoice codec, used in early mobile phones.
        Qcelp = AV_CODEC_ID_QCELP as _,

        /// WavPack codec.
        /// A lossless and hybrid audio compression format.
        WavPack = AV_CODEC_ID_WAVPACK as _,

        /// Discworld II Audio codec.
        /// Used in certain FMV-based video games.
        DsicinAudio = AV_CODEC_ID_DSICINAUDIO as _,

        /// IMC codec.
        /// Intel Music Coder, a proprietary speech codec.
        Imc = AV_CODEC_ID_IMC as _,

        /// Musepack v7 codec.
        /// A lossy audio format optimized for high-quality compression.
        Musepack7 = AV_CODEC_ID_MUSEPACK7 as _,

        /// MLP codec.
        /// Meridian Lossless Packing, used in high-definition audio.
        Mlp = AV_CODEC_ID_MLP as _,

        /// GSM Microsoft codec.
        /// A variant of GSM used in Microsoft applications.
        GsmMs = AV_CODEC_ID_GSM_MS as _,

        /// ATRAC3 codec.
        /// Sony's Adaptive Transform Acoustic Coding, used in MiniDisc and PSP.
        Atrac3 = AV_CODEC_ID_ATRAC3 as _,

        /// APE codec.
        /// Monkey's Audio, a lossless audio format.
        Ape = AV_CODEC_ID_APE as _,

        /// Nellymoser codec.
        /// Used in Flash-based streaming audio.
        Nellymoser = AV_CODEC_ID_NELLYMOSER as _,

        /// Musepack v8 codec.
        /// A newer version of the Musepack audio format.
        Musepack8 = AV_CODEC_ID_MUSEPACK8 as _,

        /// Speex codec.
        /// A speech codec optimized for low bitrate applications.
        Speex = AV_CODEC_ID_SPEEX as _,

        /// Windows Media Audio Voice codec.
        /// Used for low-bitrate speech in Windows Media applications.
        WmaVoice = AV_CODEC_ID_WMAVOICE as _,

        /// Windows Media Audio Professional codec.
        /// A high-fidelity version of Windows Media Audio.
        WmaPro = AV_CODEC_ID_WMAPRO as _,

        /// Windows Media Audio Lossless codec.
        /// A lossless compression format from Microsoft.
        WmaLossless = AV_CODEC_ID_WMALOSSLESS as _,

        /// ATRAC3+ codec.
        /// An improved version of Sony's ATRAC3 format.
        Atrac3P = AV_CODEC_ID_ATRAC3P as _,

        /// Enhanced AC-3 codec.
        /// Also known as E-AC-3, used in digital broadcasting and Blu-ray.
        Eac3 = AV_CODEC_ID_EAC3 as _,

        /// SIPR codec.
        /// A proprietary RealAudio codec.
        Sipr = AV_CODEC_ID_SIPR as _,

        /// MP1 codec.
        /// MPEG Audio Layer I, an early form of MP2/MP3.
        Mp1 = AV_CODEC_ID_MP1 as _,

        /// TwinVQ codec.
        /// A low-bitrate audio codec developed by NTT.
        TwinVq = AV_CODEC_ID_TWINVQ as _,

        /// TrueHD codec.
        /// A lossless audio format used in Blu-ray.
        TrueHd = AV_CODEC_ID_TRUEHD as _,

        /// MPEG-4 ALS codec.
        /// A lossless audio codec in the MPEG-4 standard.
        Mp4Als = AV_CODEC_ID_MP4ALS as _,

        /// ATRAC1 codec.
        /// The original Adaptive Transform Acoustic Coding format from Sony.
        Atrac1 = AV_CODEC_ID_ATRAC1 as _,

        /// Bink Audio RDFT codec.
        /// Used in Bink video files.
        BinkAudioRdft = AV_CODEC_ID_BINKAUDIO_RDFT as _,

        /// Bink Audio DCT codec.
        /// Another audio format used in Bink multimedia.
        BinkAudioDct = AV_CODEC_ID_BINKAUDIO_DCT as _,

        /// AAC LATM codec.
        /// A variant of AAC used in transport streams.
        AacLatm = AV_CODEC_ID_AAC_LATM as _,

        /// QDMC codec.
        /// A proprietary QuickTime audio format.
        Qdmc = AV_CODEC_ID_QDMC as _,

        /// CELT codec.
        /// A low-latency audio codec, later integrated into Opus.
        Celt = AV_CODEC_ID_CELT as _,

        /// G.723.1 codec.
        /// A speech codec used in VoIP applications.
        G723_1 = AV_CODEC_ID_G723_1 as _,

        /// G.729 codec.
        /// A low-bitrate speech codec commonly used in telephony.
        G729 = AV_CODEC_ID_G729 as _,

        /// 8SVX Exponential codec.
        /// An audio format used on Amiga computers.
        EightSvxExp = AV_CODEC_ID_8SVX_EXP as _,

        /// 8SVX Fibonacci codec.
        /// Another variant of the 8SVX Amiga audio format.
        EightSvxFib = AV_CODEC_ID_8SVX_FIB as _,

        /// BMV Audio codec.
        /// Used in multimedia applications.
        BmvAudio = AV_CODEC_ID_BMV_AUDIO as _,

        /// RALF codec.
        /// A proprietary RealAudio format.
        Ralf = AV_CODEC_ID_RALF as _,

        /// IAC codec.
        /// An obscure proprietary format.
        Iac = AV_CODEC_ID_IAC as _,

        /// iLBC codec.
        /// Internet Low Bitrate Codec, used in VoIP.
        Ilbc = AV_CODEC_ID_ILBC as _,

        /// Opus codec.
        /// A highly efficient and low-latency audio codec for streaming and VoIP.
        Opus = AV_CODEC_ID_OPUS as _,

        /// Comfort Noise codec.
        /// Used in VoIP applications to generate artificial background noise.
        ComfortNoise = AV_CODEC_ID_COMFORT_NOISE as _,

        /// TAK codec.
        /// A lossless audio compression format.
        Tak = AV_CODEC_ID_TAK as _,

        /// MetaSound codec.
        /// A proprietary audio format.
        MetaSound = AV_CODEC_ID_METASOUND as _,

        /// PAF Audio codec.
        /// Used in some multimedia applications.
        PafAudio = AV_CODEC_ID_PAF_AUDIO as _,

        /// On2 AVC codec.
        /// A proprietary format from On2 Technologies.
        On2Avc = AV_CODEC_ID_ON2AVC as _,

        /// DSS SP codec.
        /// Used in digital dictation software.
        DssSp = AV_CODEC_ID_DSS_SP as _,

        /// Codec2 codec.
        /// A very low-bitrate speech codec for radio communications.
        Codec2 = AV_CODEC_ID_CODEC2 as _,

        /// FFmpeg WaveSynth codec.
        /// A synthetic waveform generator.
        FfwaveSynth = AV_CODEC_ID_FFWAVESYNTH as _,

        /// Sonic codec.
        /// An experimental lossy audio format.
        Sonic = AV_CODEC_ID_SONIC as _,

        /// Sonic LS codec.
        /// A lossless version of Sonic.
        SonicLs = AV_CODEC_ID_SONIC_LS as _,

        /// EVRC codec.
        /// A speech codec used in CDMA networks.
        Evrc = AV_CODEC_ID_EVRC as _,

        /// SMV codec.
        /// A speech codec for mobile networks.
        Smv = AV_CODEC_ID_SMV as _,

        /// DSD LSBF codec.
        /// Direct Stream Digital format with least-significant-bit first ordering.
        DsdLsbf = AV_CODEC_ID_DSD_LSBF as _,

        /// DSD MSBF codec.
        /// Direct Stream Digital format with most-significant-bit first ordering.
        DsdMsbf = AV_CODEC_ID_DSD_MSBF as _,

        /// DSD LSBF Planar codec.
        /// Planar version of DSD LSBF.
        DsdLsbfPlanar = AV_CODEC_ID_DSD_LSBF_PLANAR as _,

        /// DSD MSBF Planar codec.
        /// Planar version of DSD MSBF.
        DsdMsbfPlanar = AV_CODEC_ID_DSD_MSBF_PLANAR as _,

        /// 4GV codec.
        /// A speech codec used in cellular networks.
        FourGv = AV_CODEC_ID_4GV as _,

        /// Interplay ACM codec.
        /// Used in Interplay Entertainment video games.
        InterplayAcm = AV_CODEC_ID_INTERPLAY_ACM as _,

        /// XMA1 codec.
        /// Xbox Media Audio version 1.
        Xma1 = AV_CODEC_ID_XMA1 as _,

        /// XMA2 codec.
        /// Xbox Media Audio version 2.
        Xma2 = AV_CODEC_ID_XMA2 as _,

        /// DST codec.
        /// Direct Stream Transfer, used in Super Audio CDs.
        Dst = AV_CODEC_ID_DST as _,

        /// ATRAC3AL codec.
        /// A variant of ATRAC3 used in some Sony devices.
        Atrac3Al = AV_CODEC_ID_ATRAC3AL as _,

        /// ATRAC3PAL codec.
        /// A variant of ATRAC3 used in some Sony devices.
        Atrac3Pal = AV_CODEC_ID_ATRAC3PAL as _,

        /// Dolby E codec.
        /// Used in professional broadcast audio.
        DolbyE = AV_CODEC_ID_DOLBY_E as _,

        /// aptX codec.
        /// A Bluetooth audio codec optimized for high quality.
        Aptx = AV_CODEC_ID_APTX as _,

        /// aptX HD codec.
        /// A higher-quality version of aptX.
        AptxHd = AV_CODEC_ID_APTX_HD as _,

        /// SBC codec.
        /// A standard Bluetooth audio codec.
        Sbc = AV_CODEC_ID_SBC as _,

        /// ATRAC9 codec.
        /// A high-efficiency Sony audio codec used in PlayStation consoles.
        Atrac9 = AV_CODEC_ID_ATRAC9 as _,

        /// HCOM codec.
        /// A proprietary audio compression format.
        Hcom = AV_CODEC_ID_HCOM as _,

        /// ACELP Kelvin codec.
        /// A speech codec.
        AcelpKelvin = AV_CODEC_ID_ACELP_KELVIN as _,

        /// MPEG-H 3D Audio codec.
        /// A next-generation audio standard with 3D sound.
        Mpegh3DAudio = AV_CODEC_ID_MPEGH_3D_AUDIO as _,

        /// Siren codec.
        /// A speech codec used in VoIP.
        Siren = AV_CODEC_ID_SIREN as _,

        /// HCA codec.
        /// A proprietary format used in Japanese games.
        Hca = AV_CODEC_ID_HCA as _,

        /// FastAudio codec.
        /// A proprietary format.
        FastAudio = AV_CODEC_ID_FASTAUDIO as _,

        /// MSN Siren codec.
        /// Used in older MSN Messenger voice communication.
        MsnSiren = AV_CODEC_ID_MSNSIREN as _,

        /// DFPWM codec.
        /// A low-bitrate waveform compression format.
        Dfpwm = AV_CODEC_ID_DFPWM as _,

        /// Bonk codec.
        /// A lossy audio compression format.
        Bonk = AV_CODEC_ID_BONK as _,

        /// Misc4 codec.
        /// A proprietary audio format.
        Misc4 = AV_CODEC_ID_MISC4 as _,

        /// APAC codec.
        /// A proprietary audio format.
        Apac = AV_CODEC_ID_APAC as _,

        /// FTR codec.
        /// A proprietary audio format.
        Ftr = AV_CODEC_ID_FTR as _,

        /// WAVARC codec.
        /// A proprietary audio format.
        WavArc = AV_CODEC_ID_WAVARC as _,

        /// RKA codec.
        /// A proprietary audio format.
        Rka = AV_CODEC_ID_RKA as _,

        /// AC4 codec.
        /// A next-generation Dolby audio codec for broadcasting and streaming.
        Ac4 = AV_CODEC_ID_AC4 as _,

        /// OSQ codec.
        /// A proprietary audio format.
        Osq = AV_CODEC_ID_OSQ as _,

        /// QOA codec.
        /// Quite OK Audio, a simple and efficient lossy audio codec.
        Qoa = AV_CODEC_ID_QOA as _,

        /// LC3 codec.
        /// Low Complexity Communication Codec, used in Bluetooth LE Audio.
        #[cfg(not(docsrs))]
        Lc3 = AV_CODEC_ID_LC3 as _,

        /// DVD Subtitle codec.
        /// Subtitle format used in DVDs.
        DvdSubtitle = AV_CODEC_ID_DVD_SUBTITLE as _,

        /// DVB Subtitle codec.
        /// Subtitle format used in DVB broadcasts.
        DvbSubtitle = AV_CODEC_ID_DVB_SUBTITLE as _,

        /// Text codec.
        /// A simple text-based subtitle format.
        Text = AV_CODEC_ID_TEXT as _,

        /// XSUB codec.
        /// Subtitle format used in DivX video files.
        Xsub = AV_CODEC_ID_XSUB as _,

        /// SSA codec.
        /// SubStation Alpha subtitle format, used in anime fansubs.
        Ssa = AV_CODEC_ID_SSA as _,

        /// MOV Text codec.
        /// Text-based subtitles stored in QuickTime/MOV containers.
        MovText = AV_CODEC_ID_MOV_TEXT as _,

        /// HDMV PGS Subtitle codec.
        /// Blu-ray subtitle format using graphical images.
        HdmvPgsSubtitle = AV_CODEC_ID_HDMV_PGS_SUBTITLE as _,

        /// DVB Teletext codec.
        /// Teletext format used in DVB broadcasts.
        DvbTeletext = AV_CODEC_ID_DVB_TELETEXT as _,

        /// SRT codec.
        /// SubRip Subtitle format, one of the most common subtitle formats.
        Srt = AV_CODEC_ID_SRT as _,

        /// MicroDVD codec.
        /// A simple subtitle format using timestamps.
        MicroDvd = AV_CODEC_ID_MICRODVD as _,

        /// EIA-608 codec.
        /// Closed captioning format used in analog TV broadcasts.
        Eia608 = AV_CODEC_ID_EIA_608 as _,

        /// JacoSub codec.
        /// A subtitle format used in older multimedia applications.
        JacoSub = AV_CODEC_ID_JACOSUB as _,

        /// SAMI codec.
        /// Synchronized Accessible Media Interchange, a subtitle format from Microsoft.
        Sami = AV_CODEC_ID_SAMI as _,

        /// RealText codec.
        /// Subtitle format used in RealMedia files.
        RealText = AV_CODEC_ID_REALTEXT as _,

        /// STL codec.
        /// EBU STL subtitle format, used in broadcasting.
        Stl = AV_CODEC_ID_STL as _,

        /// SubViewer 1 codec.
        /// A simple subtitle format similar to SRT.
        SubViewer1 = AV_CODEC_ID_SUBVIEWER1 as _,

        /// SubViewer codec.
        /// A newer version of the SubViewer subtitle format.
        SubViewer = AV_CODEC_ID_SUBVIEWER as _,

        /// SubRip codec.
        /// Another name for the SRT subtitle format.
        SubRip = AV_CODEC_ID_SUBRIP as _,

        /// WebVTT codec.
        /// A subtitle format used for web video.
        WebVtt = AV_CODEC_ID_WEBVTT as _,

        /// MPL2 codec.
        /// A simple subtitle format used in multimedia players.
        Mpl2 = AV_CODEC_ID_MPL2 as _,

        /// VPlayer codec.
        /// A subtitle format used in older multimedia applications.
        VPlayer = AV_CODEC_ID_VPLAYER as _,

        /// PJS codec.
        /// A simple subtitle format.
        Pjs = AV_CODEC_ID_PJS as _,

        /// Advanced SSA codec.
        /// An improved version of SSA subtitles.
        Ass = AV_CODEC_ID_ASS as _,

        /// HDMV Text Subtitle codec.
        /// A subtitle format used in Blu-ray movies.
        HdmvTextSubtitle = AV_CODEC_ID_HDMV_TEXT_SUBTITLE as _,

        /// TTML codec.
        /// Timed Text Markup Language, used for subtitles and captions.
        Ttml = AV_CODEC_ID_TTML as _,

        /// ARIB Caption codec.
        /// A subtitle format used in Japanese digital broadcasting.
        AribCaption = AV_CODEC_ID_ARIB_CAPTION as _,

        /// TrueType Font codec.
        /// Used to embed font data in multimedia files.
        Ttf = AV_CODEC_ID_TTF as _,

        /// SCTE-35 codec.
        /// Standard for inserting cue points in digital broadcasting.
        Scte35 = AV_CODEC_ID_SCTE_35 as _,

        /// EPG codec.
        /// Electronic Program Guide data for digital TV.
        Epg = AV_CODEC_ID_EPG as _,

        /// Binary Text codec.
        /// A proprietary subtitle format.
        BinText = AV_CODEC_ID_BINTEXT as _,

        /// XBIN codec.
        /// A text mode animation format used in DOS.
        Xbin = AV_CODEC_ID_XBIN as _,

        /// IDF codec.
        /// A proprietary subtitle format.
        Idf = AV_CODEC_ID_IDF as _,

        /// OpenType Font codec.
        /// Used to embed OpenType fonts in multimedia files.
        Otf = AV_CODEC_ID_OTF as _,

        /// SMPTE KLV codec.
        /// Metadata encoding format used in broadcasting.
        SmpteKlv = AV_CODEC_ID_SMPTE_KLV as _,

        /// DVD Navigation codec.
        /// Data format used for interactive DVD menus.
        DvdNav = AV_CODEC_ID_DVD_NAV as _,

        /// Timed ID3 codec.
        /// Stores metadata in streaming audio formats.
        TimedId3 = AV_CODEC_ID_TIMED_ID3 as _,

        /// Binary Data codec.
        /// Used for arbitrary binary data storage in multimedia files.
        BinData = AV_CODEC_ID_BIN_DATA as _,

        /// SMPTE 2038 codec.
        /// A metadata format used in digital broadcasting.
        Smpte2038 = AV_CODEC_ID_SMPTE_2038 as _,

        /// LCEVC codec.
        /// Low Complexity Enhancement Video Coding, a scalable video enhancement format.
        #[cfg(not(docsrs))]
        Lcevc = AV_CODEC_ID_LCEVC as _,

        /// Probe codec.
        /// Used internally by FFmpeg to detect the correct codec.
        Probe = AV_CODEC_ID_PROBE as _,

        /// MPEG-2 Transport Stream codec.
        /// A container format for digital broadcasting.
        Mpeg2Ts = AV_CODEC_ID_MPEG2TS as _,

        /// MPEG-4 Systems codec.
        /// A container format for MPEG-4 multimedia.
        Mpeg4Systems = AV_CODEC_ID_MPEG4SYSTEMS as _,

        /// FFmpeg Metadata codec.
        /// Stores metadata in multimedia files.
        FfMetadata = AV_CODEC_ID_FFMETADATA as _,

        /// Wrapped AVFrame codec.
        /// Used internally by FFmpeg to wrap raw frame data.
        WrappedAvFrame = AV_CODEC_ID_WRAPPED_AVFRAME as _,

        /// Null Video codec.
        /// A placeholder for discarded video streams.
        VNull = AV_CODEC_ID_VNULL as _,

        /// Null Audio codec.
        /// A placeholder for discarded audio streams.
        ANull = AV_CODEC_ID_ANULL as _,
    }
}

impl PartialEq<i32> for AVCodecID {
    fn eq(&self, other: &i32) -> bool {
        self.0 == *other
    }
}

impl From<u32> for AVCodecID {
    fn from(value: u32) -> Self {
        AVCodecID(value as _)
    }
}

impl From<AVCodecID> for u32 {
    fn from(value: AVCodecID) -> Self {
        value.0 as u32
    }
}
