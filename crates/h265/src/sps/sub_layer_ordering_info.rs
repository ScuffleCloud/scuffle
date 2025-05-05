use std::io;

use scuffle_bytes_util::{BitReader, range_check};
use scuffle_expgolomb::BitReaderExpGolombExt;

/// Info for each sub-layer in the SPS.
///
/// Directly part of [SPS RBSP](crate::SpsRbsp).
#[derive(Debug, Clone, PartialEq)]
pub struct SubLayerOrderingInfo {
    /// `sps_max_dec_pic_buffering_minus1[i]` plus 1 specifies the maximum required size of the decoded
    /// picture buffer for the CVS in units of picture storage buffers when `HighestTid` is equal to `i`.
    pub sps_max_dec_pic_buffering_minus1: Vec<u64>,
    /// `sps_max_num_reorder_pics[i]` indicates the maximum allowed number of pictures with `PicOutputFlag`
    /// equal to 1 that can precede any picture with `PicOutputFlag` equal to 1 in the CVS in decoding order and
    /// follow that picture with `PicOutputFlag` equal to 1 in output order when `HighestTid` is equal to i.
    pub sps_max_num_reorder_pics: Vec<u64>,
    /// `sps_max_latency_increase_plus1[i]` not equal to 0 is used to compute the value of
    /// [`SpsMaxLatencyPictures[i]`](SubLayerOrderingInfo::sps_max_latency_pictures_at),
    /// which specifies the maximum number of pictures with `PicOutputFlag` equal
    /// to 1 that can precede any picture with `PicOutputFlag` equal to 1 in the CVS in output order and follow that
    /// picture with `PicOutputFlag` equal to 1 in decoding order when `HighestTid` is equal to i.
    pub sps_max_latency_increase_plus1: Vec<u32>,
}

impl SubLayerOrderingInfo {
    pub(crate) fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        sps_sub_layer_ordering_info_present_flag: bool,
        sps_max_sub_layers_minus1: u8,
    ) -> io::Result<Self> {
        let mut sps_max_dec_pic_buffering_minus1 = vec![0; sps_max_sub_layers_minus1 as usize + 1];
        let mut sps_max_num_reorder_pics = vec![0; sps_max_sub_layers_minus1 as usize + 1];
        let mut sps_max_latency_increase_plus1 = vec![0; sps_max_sub_layers_minus1 as usize + 1];

        if sps_sub_layer_ordering_info_present_flag {
            for i in 0..=sps_max_sub_layers_minus1 as usize {
                sps_max_dec_pic_buffering_minus1[i] = bit_reader.read_exp_golomb()?;
                // (A-2) defines MaxDpbSize which is always at most 16
                range_check!(sps_max_dec_pic_buffering_minus1[i], 0, 16)?;
                if i > 0 && sps_max_dec_pic_buffering_minus1[i] < sps_max_dec_pic_buffering_minus1[i - 1] {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "sps_max_dec_pic_buffering_minus1[i] must be greater than or equal to sps_max_dec_pic_buffering_minus1[i-1]",
                    ));
                }

                sps_max_num_reorder_pics[i] = bit_reader.read_exp_golomb()?;
                range_check!(sps_max_num_reorder_pics[i], 0, sps_max_dec_pic_buffering_minus1[i])?;
                if i > 0 && sps_max_num_reorder_pics[i] < sps_max_num_reorder_pics[i - 1] {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "sps_max_num_reorder_pics[i] must be greater than or equal to sps_max_num_reorder_pics[i-1]",
                    ));
                }

                let sps_max_latency_increase_plus1_i = bit_reader.read_exp_golomb()?;
                range_check!(sps_max_latency_increase_plus1_i, 0, 2u64.pow(32) - 2)?;
                sps_max_latency_increase_plus1[i] = sps_max_latency_increase_plus1_i as u32;
            }
        } else {
            // From the spec, page 108 and 109:
            // When sps_max_dec_pic_buffering_minus1[i] is not present (...) due to
            // sps_sub_layer_ordering_info_present_flag being equal to 0, it is inferred to be equal to
            // sps_max_dec_pic_buffering_minus1[sps_max_sub_layers_minus1].

            let sps_max_dec_pic_buffering_minus1_i = bit_reader.read_exp_golomb()?;
            // (A-2) defines MaxDpbSize which is always at most 16
            range_check!(sps_max_dec_pic_buffering_minus1_i, 0, 16)?;
            sps_max_dec_pic_buffering_minus1.fill(sps_max_dec_pic_buffering_minus1_i);

            let sps_max_num_reorder_pics_i = bit_reader.read_exp_golomb()?;
            range_check!(sps_max_num_reorder_pics_i, 0, sps_max_dec_pic_buffering_minus1_i)?;
            sps_max_num_reorder_pics.fill(sps_max_num_reorder_pics_i);

            let sps_max_latency_increase_plus1_i = bit_reader.read_exp_golomb()?;
            range_check!(sps_max_latency_increase_plus1_i, 0, 2u64.pow(32) - 2)?;
            sps_max_latency_increase_plus1.fill(sps_max_latency_increase_plus1_i as u32);
        }

        Ok(SubLayerOrderingInfo {
            sps_max_dec_pic_buffering_minus1,
            sps_max_num_reorder_pics,
            sps_max_latency_increase_plus1,
        })
    }

    /// Specifies the maximum number of pictures with `PicOutputFlag` equal
    /// to 1 that can precede any picture with `PicOutputFlag` equal to 1 in the CVS in output order and follow that
    /// picture with `PicOutputFlag` equal to 1 in decoding order when `HighestTid` is equal to i.
    ///
    /// Calculates the full `SpsMaxLatencyPictures` array.
    ///
    /// Use [`SubLayerOrderingInfo::sps_max_latency_pictures_at`] to only calculate one specific value `SpsMaxLatencyPictures[i]`.
    ///
    /// `SpsMaxLatencyPictures[i] = sps_max_num_reorder_pics[i] + sps_max_latency_increase_plus1[i] − 1` (7-9)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2
    pub fn sps_max_latency_pictures(&self) -> Vec<Option<u64>> {
        self.sps_max_num_reorder_pics
            .iter()
            .zip(self.sps_max_latency_increase_plus1.iter())
            .map(|(reorder, latency)| Some(reorder + latency.checked_sub(1)? as u64))
            .collect()
    }

    /// Calculates `SpsMaxLatencyPictures[i]`.
    ///
    /// See [`sps_max_latency_pictures`](SubLayerOrderingInfo::sps_max_latency_pictures) for details.
    ///
    /// `SpsMaxLatencyPictures[i] = sps_max_num_reorder_pics[i] + sps_max_latency_increase_plus1[i] − 1` (7-9)
    ///
    /// ISO/IEC 23008-2 - 7.4.3.2
    pub fn sps_max_latency_pictures_at(&self, i: usize) -> Option<u64> {
        Some(self.sps_max_num_reorder_pics.get(i)? + self.sps_max_latency_increase_plus1.get(i)?.checked_sub(1)? as u64)
    }
}
