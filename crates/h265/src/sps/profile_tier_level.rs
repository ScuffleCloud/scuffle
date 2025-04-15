use std::io;

use byteorder::{BigEndian, ReadBytesExt};
use scuffle_bytes_util::BitReader;

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileTierLevel {
    pub general_profile: Profile,
    pub general_level_idc: u8,
    pub sub_layer_profiles: Vec<Profile>,
    pub sub_layer_level_idcs: Vec<u8>,
}

impl ProfileTierLevel {
    pub fn parse<R: io::Read>(bit_reader: &mut BitReader<R>, max_num_sub_layers_minus_1: u8) -> io::Result<Self> {
        // profile_present_flag is always true when parsing SPSs only
        let mut general_profile = Profile::parse(bit_reader)?;
        // inbld_flag is inferred to be 0 when not present for the genral profile
        general_profile.inbld_flag = Some(general_profile.inbld_flag.unwrap_or(false));

        let general_level_idc = bit_reader.read_bits(8)? as u8;

        let mut sub_layer_profile_present_flags = Vec::with_capacity(max_num_sub_layers_minus_1 as usize);
        let mut sub_layer_level_present_flags = Vec::with_capacity(max_num_sub_layers_minus_1 as usize);
        for _ in 0..max_num_sub_layers_minus_1 {
            sub_layer_profile_present_flags.push(bit_reader.read_bit()?); // sub_layer_profile_present_flag
            sub_layer_level_present_flags.push(bit_reader.read_bit()?); // sub_layer_level_present_flag
        }

        // reserved_zero_2bits
        if max_num_sub_layers_minus_1 > 0 && max_num_sub_layers_minus_1 < 8 {
            bit_reader.read_bits(2 * (8 - max_num_sub_layers_minus_1))?;
        }

        let mut sub_layer_profiles = vec![None; max_num_sub_layers_minus_1 as usize];
        let mut sub_layer_level_idcs = vec![None; max_num_sub_layers_minus_1 as usize];

        for i in 0..max_num_sub_layers_minus_1 as usize {
            if sub_layer_profile_present_flags[i] {
                sub_layer_profiles[i] = Some(Profile::parse(bit_reader)?);
            }

            if sub_layer_level_present_flags[i] {
                sub_layer_level_idcs[i] = Some(bit_reader.read_u8()?);
            }
        }

        let mut last_profile = general_profile.clone();
        let mut sub_layer_profiles: Vec<_> = sub_layer_profiles
            .into_iter()
            .rev()
            .map(|profile| match profile {
                Some(profile) => {
                    let profile = profile.merge(&last_profile);
                    last_profile = profile.clone();
                    profile
                }
                None => last_profile.clone(),
            })
            .collect();
        sub_layer_profiles.reverse();

        let mut last_level_idc = general_level_idc;
        let mut sub_layer_level_idcs: Vec<_> = sub_layer_level_idcs
            .into_iter()
            .rev()
            .map(|idc| match idc {
                Some(idc) => {
                    last_level_idc = idc;
                    idc
                }
                None => last_level_idc,
            })
            .collect();
        sub_layer_level_idcs.reverse();

        Ok(ProfileTierLevel {
            general_profile,
            general_level_idc,
            sub_layer_profiles,
            sub_layer_level_idcs,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub profile_space: u8,
    pub tier_flag: bool,
    pub profile_idc: u8,
    pub profile_compatibility_flag: [bool; 32],
    pub progressive_source_flag: bool,
    pub interlaced_source_flag: bool,
    pub non_packed_constraint_flag: bool,
    pub frame_only_constraint_flag: bool,
    pub additional_flags: ProfileAdditionalFlags,
    pub inbld_flag: Option<bool>,
}

impl Profile {
    fn parse<R: io::Read>(bit_reader: &mut BitReader<R>) -> io::Result<Self> {
        let profile_space = bit_reader.read_bits(2)? as u8;
        let tier_flag = bit_reader.read_bit()?;
        let profile_idc = bit_reader.read_bits(5)? as u8;

        let mut profile_compatibility_flag = [false; 32];
        let flag_number = bit_reader.read_u32::<BigEndian>()?;
        for (i, profile_compatibility_flag) in profile_compatibility_flag.iter_mut().enumerate() {
            *profile_compatibility_flag = (flag_number >> (31 - i)) & 1 == 1;
        }

        let check_profile_idcs = |idcs: &[u8]| {
            idcs.iter()
                .any(|idc| profile_idc == *idc || profile_compatibility_flag[*idc as usize])
        };

        let progressive_source_flag = bit_reader.read_bit()?;
        let interlaced_source_flag = bit_reader.read_bit()?;
        let non_packed_constraint_flag = bit_reader.read_bit()?;
        let frame_only_constraint_flag = bit_reader.read_bit()?;

        let additional_flags = if check_profile_idcs(&[4, 5, 6, 7, 8, 9, 10, 11]) {
            let max_12bit_constraint_flag = bit_reader.read_bit()?;
            let max_10bit_constraint_flag = bit_reader.read_bit()?;
            let max_8bit_constraint_flag = bit_reader.read_bit()?;
            let max_422chroma_constraint_flag = bit_reader.read_bit()?;
            let max_420chroma_constraint_flag = bit_reader.read_bit()?;
            let max_monochrome_constraint_flag = bit_reader.read_bit()?;
            let intra_constraint_flag = bit_reader.read_bit()?;
            let one_picture_only_constraint_flag = bit_reader.read_bit()?;
            let lower_bit_rate_constraint_flag = bit_reader.read_bit()?;

            let max_14bit_constraint_flag = if check_profile_idcs(&[5, 9, 10, 11]) {
                let max_14bit_constraint_flag = bit_reader.read_bit()?;
                bit_reader.read_bits(33)?;
                Some(max_14bit_constraint_flag)
            } else {
                bit_reader.read_bits(34)?;
                None
            };

            ProfileAdditionalFlags::Full {
                max_12bit_constraint_flag,
                max_10bit_constraint_flag,
                max_8bit_constraint_flag,
                max_422chroma_constraint_flag,
                max_420chroma_constraint_flag,
                max_monochrome_constraint_flag,
                intra_constraint_flag,
                one_picture_only_constraint_flag,
                lower_bit_rate_constraint_flag,
                max_14bit_constraint_flag,
            }
        } else if check_profile_idcs(&[2]) {
            bit_reader.read_bits(7)?; // reserved_zero_7bits
            let one_picture_only_constraint_flag = bit_reader.read_bit()?;
            bit_reader.read_bits(35)?; // reserved_zero_35bits
            ProfileAdditionalFlags::Profile2 {
                one_picture_only_constraint_flag,
            }
        } else {
            bit_reader.read_bits(43)?; // reserved_zero_43bits
            ProfileAdditionalFlags::None
        };

        let inbld_flag = if check_profile_idcs(&[1, 2, 3, 4, 5, 9, 11]) {
            Some(bit_reader.read_bit()?)
        } else {
            bit_reader.read_bit()?; // reserved_zero_bit
            None
        };

        Ok(Profile {
            profile_space,
            tier_flag,
            profile_idc,
            profile_compatibility_flag,
            progressive_source_flag,
            interlaced_source_flag,
            non_packed_constraint_flag,
            frame_only_constraint_flag,
            additional_flags,
            inbld_flag,
        })
    }

    pub fn merge(self, defaults: &Self) -> Self {
        Self {
            additional_flags: self.additional_flags.merge(&defaults.additional_flags),
            inbld_flag: self.inbld_flag.or(defaults.inbld_flag),
            ..self
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProfileAdditionalFlags {
    Full {
        max_12bit_constraint_flag: bool,
        max_10bit_constraint_flag: bool,
        max_8bit_constraint_flag: bool,
        max_422chroma_constraint_flag: bool,
        max_420chroma_constraint_flag: bool,
        max_monochrome_constraint_flag: bool,
        intra_constraint_flag: bool,
        one_picture_only_constraint_flag: bool,
        lower_bit_rate_constraint_flag: bool,
        max_14bit_constraint_flag: Option<bool>,
    },
    Profile2 {
        one_picture_only_constraint_flag: bool,
    },
    None,
}

impl ProfileAdditionalFlags {
    pub fn merge(self, defaults: &Self) -> Self {
        match (&self, defaults) {
            (Self::Full { .. }, _) => self,
            (
                Self::Profile2 {
                    one_picture_only_constraint_flag,
                },
                Self::Full {
                    max_12bit_constraint_flag,
                    max_10bit_constraint_flag,
                    max_8bit_constraint_flag,
                    max_422chroma_constraint_flag,
                    max_420chroma_constraint_flag,
                    max_monochrome_constraint_flag,
                    intra_constraint_flag,
                    lower_bit_rate_constraint_flag,
                    max_14bit_constraint_flag,
                    ..
                },
            ) => Self::Full {
                max_12bit_constraint_flag: *max_12bit_constraint_flag,
                max_10bit_constraint_flag: *max_10bit_constraint_flag,
                max_8bit_constraint_flag: *max_8bit_constraint_flag,
                max_422chroma_constraint_flag: *max_422chroma_constraint_flag,
                max_420chroma_constraint_flag: *max_420chroma_constraint_flag,
                max_monochrome_constraint_flag: *max_monochrome_constraint_flag,
                intra_constraint_flag: *intra_constraint_flag,
                one_picture_only_constraint_flag: *one_picture_only_constraint_flag,
                lower_bit_rate_constraint_flag: *lower_bit_rate_constraint_flag,
                max_14bit_constraint_flag: *max_14bit_constraint_flag,
            },
            (Self::Profile2 { .. }, _) => self,
            (Self::None, _) => defaults.clone(),
        }
    }
}
