use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

use crate::range_check::range_check;

#[derive(Debug, Clone, PartialEq)]
pub struct Sps3dExtension {
    pub d0: Sps3dExtensionD0,
    pub d1: Sps3dExtensionD1,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sps3dExtensionD0 {
    pub iv_di_mc_enabled_flag: bool,
    pub iv_mv_scal_enabled_flag: bool,
    pub log2_ivmc_sub_pb_size_minus3: u64,
    pub iv_res_pred_enabled_flag: bool,
    pub depth_ref_enabled_flag: bool,
    pub vsp_mc_enabled_flag: bool,
    pub dbbp_enabled_flag: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sps3dExtensionD1 {
    pub iv_di_mc_enabled_flag: bool,
    pub iv_mv_scal_enabled_flag: bool,
    pub tex_mc_enabled_flag: bool,
    pub log2_texmc_sub_pb_size_minus3: u64,
    pub intra_contour_enabled_flag: bool,
    pub intra_dc_only_wedge_enabled_flag: bool,
    pub cqt_cu_part_pred_enabled_flag: bool,
    pub inter_dc_only_enabled_flag: bool,
    pub skip_intra_enabled_flag: bool,
}

impl Sps3dExtension {
    pub fn parse<R: io::Read>(
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
