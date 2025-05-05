use std::io;

use scuffle_bytes_util::{BitReader, range_check};
use scuffle_expgolomb::BitReaderExpGolombExt;

/// Sequence parameter set 3D extension.
///
/// `sps_3d_extension()`
///
/// - ISO/IEC 23008-2 - I.7.3.2.2.5
/// - ISO/IEC 23008-2 - I.7.4.3.2.5
#[derive(Debug, Clone, PartialEq)]
pub struct Sps3dExtension {
    /// All values for `d=0`
    pub d0: Sps3dExtensionD0,
    /// All values for `d=1`
    pub d1: Sps3dExtensionD1,
}

/// Directly part of [SPS 3D extension](Sps3dExtension).
#[derive(Debug, Clone, PartialEq)]
pub struct Sps3dExtensionD0 {
    /// Equal to `true` specifies that the derivation process for inter-view predicted
    /// merging candidates and the derivation process for disparity information merging candidates may be used
    /// in the decoding process of layers with `DepthFlag` equal to **0**.
    ///
    /// Equal to `false` specifies
    /// that derivation process for inter-view predicted merging candidates and the derivation process for
    /// disparity information merging candidates is not used in the decoding process of layers with `DepthFlag`
    /// equal to **0**.
    pub iv_di_mc_enabled_flag: bool,
    /// Equal to `true` specifies that motion vectors used for inter-view prediction may
    /// be scaled based on `view_id_val` values in the decoding process of layers with `DepthFlag` equal to **0**.
    ///
    /// Equal to `false` specifies that motion vectors used for inter-view prediction are
    /// not scaled based on `view_id_val` values in the decoding process of layers with `DepthFlag` equal to **0**.
    pub iv_mv_scal_enabled_flag: bool,
    /// When [`iv_di_mc_enabled_flag`](Sps3dExtensionD0::iv_di_mc_enabled_flag) is equal to `true`, is
    /// used to derive the minimum size of sub-block partitions used in the derivation process for sub-block
    /// partition motion vectors for an inter-layer predicted merging candidate in the decoding process of layers
    /// with `DepthFlag` equal to 0.
    ///
    /// The value is in range
    /// \[[`MinCbLog2SizeY`](crate::SpsRbsp::min_cb_log2_size_y) - 3, [`CtbLog2SizeY`](crate::SpsRbsp::ctb_log2_size_y) - 3\].
    pub log2_ivmc_sub_pb_size_minus3: u64,
    /// Equal to `true` specifies that the `iv_res_pred_weight_idx` syntax element may
    /// be present in coding units of layers with `DepthFlag` equal to 0.
    ///
    /// Equal to 0 specifies that the `iv_res_pred_weight_idx` syntax element is not present coding units of layers with
    /// `DepthFlag` equal to 0.
    pub iv_res_pred_enabled_flag: bool,
    /// Equal to `true` specifies that the derivation process for a depth or disparity
    /// sample array from a depth picture may be used in the derivation process for a disparity vector for texture
    /// layers in the decoding process of layers with `DepthFlag` equal to 0.
    ///
    /// Equal to `false` specifies that derivation process for a depth or disparity sample array from
    /// a depth picture is not used in the derivation process for a disparity vector for texture layers in
    /// the decoding process of layers with `DepthFlag` equal to 0.
    pub depth_ref_enabled_flag: bool,
    /// Equal to `true` specifies that the derivation process for a view synthesis prediction
    /// merging candidate may be used in the decoding process of layers with `DepthFlag` equal to 0.
    ///
    /// Equal to `false` specifies that the derivation process for a view synthesis prediction
    /// merging candidate is not used in the decoding process of layers with `DepthFlag` equal to 0.
    pub vsp_mc_enabled_flag: bool,
    /// Equal to `true` specifies that the `dbbp_flag` syntax element may be present in coding
    /// units of layers with `DepthFlag` equal to 0.
    ///
    /// Equal to `false` specifies that the `dbbp_flag`
    /// syntax element is not present in coding units of layers with `DepthFlag` equal to 0.
    pub dbbp_enabled_flag: bool,
}

/// Directly part of [SPS 3D extension](Sps3dExtension).
#[derive(Debug, Clone, PartialEq)]
pub struct Sps3dExtensionD1 {
    /// Equal to `true` specifies that the derivation process for inter-view predicted
    /// merging candidates and the derivation process for disparity information merging candidates may be used
    /// in the decoding process of layers with `DepthFlag` equal to **1**.
    ///
    /// Equal to `false` specifies
    /// that derivation process for inter-view predicted merging candidates and the derivation process for
    /// disparity information merging candidates is not used in the decoding process of layers with `DepthFlag`
    /// equal to **1**.
    pub iv_di_mc_enabled_flag: bool,
    /// Equal to `true` specifies that motion vectors used for inter-view prediction may
    /// be scaled based on `view_id_val` values in the decoding process of layers with `DepthFlag` equal to **1**.
    ///
    /// Equal to `false` specifies that motion vectors used for inter-view prediction are
    /// not scaled based on `view_id_val` values in the decoding process of layers with `DepthFlag` equal to **1**.
    pub iv_mv_scal_enabled_flag: bool,
    /// Equal to `true` specifies that the derivation process for motion vectors for the
    /// texture merge candidate may be used in the decoding process of layers with `DepthFlag` equal to 1.
    ///
    /// Equal to `false` specifies that the derivation process for motion vectors for the texture
    /// merge candidate is not used in the decoding process of layers with `DepthFlag` equal to 1.
    pub tex_mc_enabled_flag: bool,
    /// When this value is equal to `true`, is used to derive the
    /// minimum size of sub-block partitions used in the derivation process for sub-block partition motion
    /// vectors for an inter-layer predicted merging candidate in the decoding process of layers with `DepthFlag`
    /// equal to 1.
    ///
    /// The value is in range
    /// \[[`MinCbLog2SizeY`](crate::SpsRbsp::min_cb_log2_size_y) - 3, [`CtbLog2SizeY`](crate::SpsRbsp::ctb_log2_size_y) - 3\].
    pub log2_texmc_sub_pb_size_minus3: u64,
    /// Equal to `true` specifies that the intra prediction mode `INTRA_CONTOUR`
    /// using depth intra contour prediction may be used in the decoding process of layers with `DepthFlag` equal
    /// to 1.
    ///
    /// Equal to `false` specifies that the intra prediction mode `INTRA_CONTOUR`
    /// using depth intra contour prediction is not used in the decoding process of layers with `DepthFlag` equal
    /// to 1.
    pub intra_contour_enabled_flag: bool,
    /// Equal to `true` specifies that the `dc_only_flag` syntax element may be
    /// present in coding units coded in an intra prediction mode of layers with `DepthFlag` equal to 1, and that
    /// the intra prediction mode `INTRA_WEDGE` may be used in the decoding process of layers with `DepthFlag`
    /// equal to 1.
    ///
    /// Equal to `false` specifies that the `dc_only_flag` syntax element
    /// is not present in coding units coded in an intra prediction mode of layers with `DepthFlag` equal to 1 and
    /// that the intra prediction mode `INTRA_WEDGE` is not used in the decoding process of layers with
    /// `DepthFlag` equal to 1.
    pub intra_dc_only_wedge_enabled_flag: bool,
    /// Equal to `true` specifies that coding quadtree and coding unit
    /// partitioning information may be inter-component predicted in the decoding process of layers with
    /// `DepthFlag` equal to 1.
    ///
    /// Equal to `false` specifies that coding quadtree and
    /// coding unit partitioning information are not inter-component predicted in the decoding process of layers
    /// with `DepthFlag` equal to 1.
    pub cqt_cu_part_pred_enabled_flag: bool,
    /// Equal to `true` specifies that the dc_only_flag syntax element may be present
    /// in coding units coded an in inter prediction mode of layers with `DepthFlag` equal to 1.
    ///
    /// Equal to `false` specifies that the dc_only_flag syntax element is not present in
    /// coding units coded in an inter prediction mode of layers with `DepthFlag` equal to 1.
    pub inter_dc_only_enabled_flag: bool,
    /// Equal to `true` specifies that the `skip_intra_flag` syntax element may be present
    /// in coding units of layers with `DepthFlag` equal to 1.
    ///
    /// Equal to `false` specifies that
    /// the `skip_intra_flag` syntax element is not present in coding units of layers with `DepthFlag` equal to 1.
    pub skip_intra_enabled_flag: bool,
}

impl Sps3dExtension {
    pub(crate) fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        min_cb_log2_size_y: u64,
        ctb_log2_size_y: u64,
    ) -> io::Result<Self> {
        let iv_di_mc_enabled_flag = bit_reader.read_bit()?;
        let iv_mv_scal_enabled_flag = bit_reader.read_bit()?;
        let log2_ivmc_sub_pb_size_minus3 = bit_reader.read_exp_golomb()?;
        range_check!(
            log2_ivmc_sub_pb_size_minus3,
            min_cb_log2_size_y.saturating_sub(3),
            ctb_log2_size_y.saturating_sub(3)
        )?;

        let d0 = Sps3dExtensionD0 {
            iv_di_mc_enabled_flag,
            iv_mv_scal_enabled_flag,
            log2_ivmc_sub_pb_size_minus3,
            iv_res_pred_enabled_flag: bit_reader.read_bit()?,
            depth_ref_enabled_flag: bit_reader.read_bit()?,
            vsp_mc_enabled_flag: bit_reader.read_bit()?,
            dbbp_enabled_flag: bit_reader.read_bit()?,
        };

        let tex_mc_enabled_flag = bit_reader.read_bit()?;
        let log2_texmc_sub_pb_size_minus3 = bit_reader.read_exp_golomb()?;
        range_check!(
            log2_texmc_sub_pb_size_minus3,
            min_cb_log2_size_y.saturating_sub(3),
            ctb_log2_size_y.saturating_sub(3)
        )?;

        let d1 = Sps3dExtensionD1 {
            iv_di_mc_enabled_flag: d0.iv_di_mc_enabled_flag,
            iv_mv_scal_enabled_flag: d0.iv_mv_scal_enabled_flag,
            tex_mc_enabled_flag,
            log2_texmc_sub_pb_size_minus3,
            intra_contour_enabled_flag: bit_reader.read_bit()?,
            intra_dc_only_wedge_enabled_flag: bit_reader.read_bit()?,
            cqt_cu_part_pred_enabled_flag: bit_reader.read_bit()?,
            inter_dc_only_enabled_flag: bit_reader.read_bit()?,
            skip_intra_enabled_flag: bit_reader.read_bit()?,
        };

        Ok(Sps3dExtension { d0, d1 })
    }
}
