//! A set of helper functions to encode and decode exponential-golomb values.
//!
//! This crate extends upon the [`BitReader`] and [`BitWriter`] from the
//! [`scuffle-bytes-util`][scuffle_bytes_util] crate to provide functionality
//! for reading and writing Exp-Golomb encoded numbers.
#![cfg_attr(feature = "docs", doc = "\n\nSee the [changelog][changelog] for a full release history.")]
#![cfg_attr(feature = "docs", doc = "## Feature flags")]
#![cfg_attr(feature = "docs", doc = document_features::document_features!())]
//! ## Usage
//!
//! ```rust
//! # fn test() -> std::io::Result<()> {
//! use scuffle_expgolomb::{BitReaderExpGolombExt, BitWriterExpGolombExt};
//! use scuffle_bytes_util::{BitReader, BitWriter};
//!
//! let mut bit_writer = BitWriter::default();
//! bit_writer.write_exp_golomb(0)?;
//! bit_writer.write_exp_golomb(1)?;
//! bit_writer.write_exp_golomb(2)?;
//!
//! let data: Vec<u8> = bit_writer.finish()?;
//!
//! let mut bit_reader = BitReader::new(std::io::Cursor::new(data));
//!
//! let result = bit_reader.read_exp_golomb()?;
//! assert_eq!(result, 0);
//!
//! let result = bit_reader.read_exp_golomb()?;
//! assert_eq!(result, 1);
//!
//! let result = bit_reader.read_exp_golomb()?;
//! assert_eq!(result, 2);
//! # Ok(())
//! # }
//! # test().expect("failed to run test");
//! ```
//!
//! ## License
//!
//! This project is licensed under the MIT or Apache-2.0 license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use std::io;

use scuffle_bytes_util::{BitReader, BitWriter};

/// Extension trait for reading Exp-Golomb encoded numbers from a bit reader
///
/// See: <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
///
/// - [`BitReader`]
pub trait BitReaderExpGolombExt {
    /// Reads an Exp-Golomb encoded number
    fn read_exp_golomb(&mut self) -> io::Result<u64>;

    /// Reads a signed Exp-Golomb encoded number
    fn read_signed_exp_golomb(&mut self) -> io::Result<i64> {
        let exp_glob = self.read_exp_golomb()?;

        if exp_glob.is_multiple_of(2) {
            Ok(-((exp_glob / 2) as i64))
        } else {
            Ok((exp_glob / 2) as i64 + 1)
        }
    }
}

impl<R: io::Read> BitReaderExpGolombExt for BitReader<R> {
    fn read_exp_golomb(&mut self) -> io::Result<u64> {
        let mut leading_zeros = 0;
        while !self.read_bit()? {
            leading_zeros += 1;
        }

        let mut result = 1;
        for _ in 0..leading_zeros {
            result <<= 1;
            result |= self.read_bit()? as u64;
        }

        Ok(result - 1)
    }
}

/// Extension trait for writing Exp-Golomb encoded numbers to a bit writer
///
/// See: <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
///
/// - [`BitWriter`]
pub trait BitWriterExpGolombExt {
    /// Writes an Exp-Golomb encoded number
    fn write_exp_golomb(&mut self, input: u64) -> io::Result<()>;

    /// Writes a signed Exp-Golomb encoded number
    fn write_signed_exp_golomb(&mut self, number: i64) -> io::Result<()> {
        let number = if number <= 0 {
            -number as u64 * 2
        } else {
            number as u64 * 2 - 1
        };

        self.write_exp_golomb(number)
    }
}

impl<W: io::Write> BitWriterExpGolombExt for BitWriter<W> {
    fn write_exp_golomb(&mut self, input: u64) -> io::Result<()> {
        let mut number = input + 1;
        let mut leading_zeros = 0;
        while number > 1 {
            number >>= 1;
            leading_zeros += 1;
        }

        for _ in 0..leading_zeros {
            self.write_bit(false)?;
        }

        self.write_bits(input + 1, leading_zeros + 1)?;

        Ok(())
    }
}

/// Returns the number of bits that a signed Exp-Golomb encoded number would take up.
///
/// See: <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
pub fn size_of_signed_exp_golomb(number: i64) -> u64 {
    let number = if number <= 0 {
        -number as u64 * 2
    } else {
        number as u64 * 2 - 1
    };

    size_of_exp_golomb(number)
}

/// Returns the number of bits that an Exp-Golomb encoded number would take up.
///
/// See: <https://en.wikipedia.org/wiki/Exponential-Golomb_coding>
pub fn size_of_exp_golomb(number: u64) -> u64 {
    let mut number = number + 1;
    let mut leading_zeros = 0;
    while number > 1 {
        number >>= 1;
        leading_zeros += 1;
    }

    leading_zeros * 2 + 1
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use bytes::Buf;
    use scuffle_bytes_util::{BitReader, BitWriter};

    use crate::{BitReaderExpGolombExt, BitWriterExpGolombExt, size_of_exp_golomb, size_of_signed_exp_golomb};

    pub(crate) fn get_remaining_bits(reader: &BitReader<std::io::Cursor<Vec<u8>>>) -> usize {
        let remaining = reader.get_ref().remaining();

        if reader.is_aligned() {
            remaining * 8
        } else {
            remaining * 8 + (8 - reader.bit_pos() as usize)
        }
    }

    #[test]
    fn test_exp_glob_decode() {
        let mut bit_writer = BitWriter::<Vec<u8>>::default();

        bit_writer.write_bits(0b1, 1).unwrap(); // 0
        bit_writer.write_bits(0b010, 3).unwrap(); // 1
        bit_writer.write_bits(0b011, 3).unwrap(); // 2
        bit_writer.write_bits(0b00100, 5).unwrap(); // 3
        bit_writer.write_bits(0b00101, 5).unwrap(); // 4
        bit_writer.write_bits(0b00110, 5).unwrap(); // 5
        bit_writer.write_bits(0b00111, 5).unwrap(); // 6

        let data = bit_writer.finish().unwrap();

        let mut bit_reader = BitReader::new(std::io::Cursor::new(data));

        let remaining_bits = get_remaining_bits(&bit_reader);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 0);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 1);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 1);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 4);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 2);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 7);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 3);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 12);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 4);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 17);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 5);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 22);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 6);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 27);
    }

    #[test]
    fn test_signed_exp_glob_decode() {
        let mut bit_writer = BitWriter::<Vec<u8>>::default();

        bit_writer.write_bits(0b1, 1).unwrap(); // 0
        bit_writer.write_bits(0b010, 3).unwrap(); // 1
        bit_writer.write_bits(0b011, 3).unwrap(); // -1
        bit_writer.write_bits(0b00100, 5).unwrap(); // 2
        bit_writer.write_bits(0b00101, 5).unwrap(); // -2
        bit_writer.write_bits(0b00110, 5).unwrap(); // 3
        bit_writer.write_bits(0b00111, 5).unwrap(); // -3

        let data = bit_writer.finish().unwrap();

        let mut bit_reader = BitReader::new(std::io::Cursor::new(data));

        let remaining_bits = get_remaining_bits(&bit_reader);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, 0);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 1);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, 1);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 4);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, -1);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 7);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, 2);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 12);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, -2);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 17);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, 3);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 22);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, -3);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 27);
    }

    #[test]
    fn test_exp_glob_encode() {
        let mut bit_writer = BitWriter::<Vec<u8>>::default();

        bit_writer.write_exp_golomb(0).unwrap();
        bit_writer.write_exp_golomb(1).unwrap();
        bit_writer.write_exp_golomb(2).unwrap();
        bit_writer.write_exp_golomb(3).unwrap();
        bit_writer.write_exp_golomb(4).unwrap();
        bit_writer.write_exp_golomb(5).unwrap();
        bit_writer.write_exp_golomb(6).unwrap();
        bit_writer.write_exp_golomb(u64::MAX - 1).unwrap();

        let data = bit_writer.finish().unwrap();

        let mut bit_reader = BitReader::new(std::io::Cursor::new(data));

        let remaining_bits = get_remaining_bits(&bit_reader);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 0);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 1);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 1);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 4);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 2);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 7);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 3);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 12);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 4);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 17);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 5);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 22);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, 6);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 27);

        let result = bit_reader.read_exp_golomb().unwrap();
        assert_eq!(result, u64::MAX - 1);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 154);
    }

    #[test]
    fn test_signed_exp_glob_encode() {
        let mut bit_writer = BitWriter::<Vec<u8>>::default();

        bit_writer.write_signed_exp_golomb(0).unwrap();
        bit_writer.write_signed_exp_golomb(1).unwrap();
        bit_writer.write_signed_exp_golomb(-1).unwrap();
        bit_writer.write_signed_exp_golomb(2).unwrap();
        bit_writer.write_signed_exp_golomb(-2).unwrap();
        bit_writer.write_signed_exp_golomb(3).unwrap();
        bit_writer.write_signed_exp_golomb(-3).unwrap();
        bit_writer.write_signed_exp_golomb(i64::MAX).unwrap();

        let data = bit_writer.finish().unwrap();

        let mut bit_reader = BitReader::new(std::io::Cursor::new(data));

        let remaining_bits = get_remaining_bits(&bit_reader);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, 0);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 1);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, 1);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 4);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, -1);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 7);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, 2);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 12);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, -2);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 17);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, 3);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 22);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, -3);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 27);

        let result = bit_reader.read_signed_exp_golomb().unwrap();
        assert_eq!(result, i64::MAX);
        assert_eq!(get_remaining_bits(&bit_reader), remaining_bits - 154);
    }

    #[test]
    fn test_expg_sizes() {
        assert_eq!(1, size_of_exp_golomb(0)); // 0b1
        assert_eq!(3, size_of_exp_golomb(1)); // 0b010
        assert_eq!(3, size_of_exp_golomb(2)); // 0b011
        assert_eq!(5, size_of_exp_golomb(3)); // 0b00100
        assert_eq!(5, size_of_exp_golomb(4)); // 0b00101
        assert_eq!(5, size_of_exp_golomb(5)); // 0b00110
        assert_eq!(5, size_of_exp_golomb(6)); // 0b00111

        assert_eq!(1, size_of_signed_exp_golomb(0)); // 0b1
        assert_eq!(3, size_of_signed_exp_golomb(1)); // 0b010
        assert_eq!(3, size_of_signed_exp_golomb(-1)); // 0b011
        assert_eq!(5, size_of_signed_exp_golomb(2)); // 0b00100
        assert_eq!(5, size_of_signed_exp_golomb(-2)); // 0b00101
        assert_eq!(5, size_of_signed_exp_golomb(3)); // 0b00110
        assert_eq!(5, size_of_signed_exp_golomb(-3)); // 0b00111
    }
}

/// Changelogs generated by [scuffle_changelog]
#[cfg(feature = "docs")]
#[scuffle_changelog::changelog]
pub mod changelog {}
