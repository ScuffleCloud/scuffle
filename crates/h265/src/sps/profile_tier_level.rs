use std::io;

use byteorder::ReadBytesExt;
use scuffle_bytes_util::BitReader;

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileTierLevel {
    pub sub_layer_profile_present_flags: Vec<bool>,
    pub sub_layer_level_present_flags: Vec<bool>,
    pub sub_layer_level_idcs: Vec<Option<u8>>,
}

impl ProfileTierLevel {
    pub fn parse<R: io::Read>(
        bit_reader: &mut BitReader<R>,
        profile_present_flag: bool,
        max_num_sub_layers_minus_1: u8,
    ) -> io::Result<Self> {
        if profile_present_flag {
            bit_reader.read_bits(
                2 // general_profile_space
                + 1 // general_tier_flag
                + 5, // general_profile_idc
            )?;
            bit_reader.read_bits(32)?; // general_profile_compatibility_flag[32]
            bit_reader.read_bits(
                1 // general_progressive_source_flag
                + 1 // general_interlaced_source_flag
                + 1 // general_non_packed_constraint_flag
                + 1, // general_frame_only_constraint_flag
            )?;
            bit_reader.read_bits(
                43 // 1. or 2. if branch or general_reserved_zero_43bits
                + 1, // general_inbld_flag or general_reserved_zero_bit
            )?;
        }

        bit_reader.read_bits(8)?; // general_level_idc

        let mut sub_layer_profile_present_flags = Vec::with_capacity(max_num_sub_layers_minus_1 as usize);
        let mut sub_layer_level_present_flags = Vec::with_capacity(max_num_sub_layers_minus_1 as usize);
        for _ in 0..max_num_sub_layers_minus_1 {
            sub_layer_profile_present_flags.push(bit_reader.read_bit()?); // sub_layer_profile_present_flag
            sub_layer_level_present_flags.push(bit_reader.read_bit()?); // sub_layer_level_present_flag
        }
        dbg!(&sub_layer_profile_present_flags, &sub_layer_level_present_flags);

        // reserved_zero_2bits
        if max_num_sub_layers_minus_1 > 0 && max_num_sub_layers_minus_1 < 8 {
            bit_reader.read_bits(2 * (8 - max_num_sub_layers_minus_1))?;
        }

        let mut sub_layer_level_idcs = vec![None; max_num_sub_layers_minus_1 as usize];
        for i in 0..max_num_sub_layers_minus_1 as usize {
            if sub_layer_profile_present_flags[i] {
                bit_reader.read_bits(
                    2 // sub_layer_profile_space
                    + 1 // sub_layer_tier_flag
                    + 5, // sub_layer_profile_idc
                )?;
                bit_reader.read_bits(32)?; // sub_layer_profile_compatibility_flag[32]
                bit_reader.read_bits(
                    1 // sub_layer_progressive_source_flag
                    + 1 // sub_layer_interlaced_source_flag
                    + 1 // sub_layer_non_packed_constraint_flag
                    + 1, // sub_layer_frame_only_constraint_flag
                )?;
                bit_reader.read_bits(
                    43 // sub_layer_reserved_zero_43bits
                    + 1, // sub_layer_reserved_zero_bit
                )?;
            }

            if sub_layer_level_present_flags[i] {
                sub_layer_level_idcs[i] = Some(bit_reader.read_u8()?); // sub_layer_level_idc
            }
        }

        Ok(ProfileTierLevel {
            sub_layer_profile_present_flags,
            sub_layer_level_present_flags,
            sub_layer_level_idcs,
        })
    }
}
