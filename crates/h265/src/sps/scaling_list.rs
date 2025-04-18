use std::io;

use scuffle_bytes_util::BitReader;
use scuffle_expgolomb::BitReaderExpGolombExt;

/// `ScalingList[0][0..5][i]`
///
/// ISO/IEC 23008-2 - Table 7-5
const TABLE_7_5: [i64; 16] = [16; 16];

/// `ScalingList[1..3][0..2][i]`
///
/// /// ISO/IEC 23008-2 - Table 7-6
#[rustfmt::skip]
const TABLE_7_6_02: [i64; 64] = [
    //0  1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
    16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 17, 16, 17, 16, 17, 18,
    17, 18, 18, 17, 18, 21, 19, 20, 21, 20, 19, 21, 24, 22, 22, 24,
    24, 22, 22, 24, 25, 25, 27, 30, 27, 25, 25, 29, 31, 35, 35, 31,
    29, 36, 41, 44, 41, 36, 47, 54, 54, 47, 65, 70, 65, 88, 88, 115,
];

/// ScalingList[1..3][3..5][i]
///
/// ISO/IEC 23008-2 - Table 7-6
#[rustfmt::skip]
const TABLE_7_6_35: [i64; 64] = [
    //0  1   2   3   4   5   6   7   8   9  10  11  12  13  14  15
    16, 16, 16, 16, 16, 16, 16, 16, 16, 16, 17, 17, 17, 17, 17, 18,
    18, 18, 18, 18, 18, 20, 20, 20, 20, 20, 20, 20, 24, 24, 24, 24,
    24, 24, 24, 24, 25, 25, 25, 25, 25, 25, 25, 28, 28, 28, 28, 28,
    28, 33, 33, 33, 33, 33, 41, 41, 41, 41, 54, 54, 54, 71, 71, 91,
];

/// ScalingList[1..3][i][j]
///
/// ISO/IEC 23008-2 - 7.4.5
const TABLE_7_6: [[i64; 64]; 6] = [
    TABLE_7_6_02,
    TABLE_7_6_02,
    TABLE_7_6_02,
    TABLE_7_6_35,
    TABLE_7_6_35,
    TABLE_7_6_35,
];

/// Scaling list data.
///
/// `scaling_list_data()`
///
/// - ISO/IEC 23008-2 - 7.3.4
/// - ISO/IEC 23008-2 - 7.4.5
#[derive(Debug, Clone, PartialEq)]
pub struct ScalingListData {
    /// The resulting scaling list.
    ///
    /// `ScalingList[0..3][0..5][0..63]`
    pub scaling_list: [[[i64; 64]; 6]; 4],
}

impl ScalingListData {
    pub(crate) fn parse<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<Self> {
        let mut scaling_list = [[[0; 64]; 6]; 4];

        for (size_id, scaling_column) in scaling_list.iter_mut().enumerate() {
            let mut matrix_id = 0;

            while matrix_id < 6 {
                let scaling_list_pred_mode_flag = bit_reader.read_bit()?;

                if !scaling_list_pred_mode_flag {
                    // the values of the scaling list are the same as the values of a reference scaling list.

                    let scaling_list_pred_matrix_id_delta = bit_reader.read_exp_golomb()? as usize;

                    if scaling_list_pred_matrix_id_delta == 0 {
                        // the scaling list is inferred from the default scaling list
                        if size_id == 0 {
                            scaling_column[matrix_id][0..16].copy_from_slice(&TABLE_7_5);
                        } else {
                            let end = usize::min(63, (1 << (4 + (size_id << 1))) - 1);
                            scaling_column[matrix_id][0..end].copy_from_slice(&TABLE_7_6[matrix_id][0..end]);
                        }
                    } else {
                        // the scaling list is inferred from the reference scaling list
                        if size_id == 0 {
                            scaling_column[matrix_id][0..16].copy_from_slice(&TABLE_7_5);
                        } else {
                            let ref_matrix_id =
                                matrix_id - scaling_list_pred_matrix_id_delta * (if size_id == 3 { 3 } else { 1 });
                            let end = usize::min(63, (1 << (4 + (size_id << 1))) - 1);
                            scaling_column[matrix_id][0..end].copy_from_slice(&TABLE_7_6[ref_matrix_id][0..end]);
                        }
                    }
                } else {
                    // the values of the scaling list are explicitly signalled.

                    let mut next_coef = 8;
                    let coef_num = usize::min(64, 1 << (4 + (size_id << 1)));

                    if size_id > 1 {
                        let scaling_list_dc_coef_minus8 = bit_reader.read_signed_exp_golomb()?;
                        next_coef = scaling_list_dc_coef_minus8 + 8;
                    }

                    for i in 0..coef_num {
                        let scaling_list_delta_coef = bit_reader.read_signed_exp_golomb()?;
                        next_coef = (next_coef + scaling_list_delta_coef + 256) % 256;
                        scaling_column[matrix_id][i] = next_coef;
                    }
                }

                matrix_id += if size_id == 3 { 3 } else { 1 };
            }
        }

        Ok(Self { scaling_list })
    }
}
