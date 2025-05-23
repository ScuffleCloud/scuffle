//! Digest processing for complex handshakes.

use std::io;

use bytes::Bytes;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use super::error::ComplexHandshakeError;
use super::{RTMP_DIGEST_LENGTH, SchemaVersion};
use crate::handshake::{CHUNK_LENGTH, TIME_VERSION_LENGTH};

/// A digest processor.
///
/// This is used to process the digest of a message.
pub struct DigestProcessor<'a> {
    data: Bytes,
    key: &'a [u8],
}

/// The result of a digest.
///
/// Use [`DigestProcessor::generate_and_fill_digest`] to create a `DigestResult`
/// and [`DigestResult::write_to`] to write the result to a buffer.
pub struct DigestResult {
    /// The left part.
    pub left: Bytes,
    /// The digest.
    pub digest: [u8; 32],
    /// The right part.
    pub right: Bytes,
}

impl DigestResult {
    /// Write the digest result to a given buffer.
    pub fn write_to(&self, writer: &mut impl io::Write) -> io::Result<()> {
        writer.write_all(&self.left)?;
        writer.write_all(&self.digest)?;
        writer.write_all(&self.right)?;

        Ok(())
    }
}

impl<'a> DigestProcessor<'a> {
    /// Create a new digest processor.
    pub const fn new(data: Bytes, key: &'a [u8]) -> Self {
        Self { data, key }
    }

    /// Read digest from message
    ///
    /// According the the spec the schema can either be in the order of
    /// - time, version, key, digest (schema 0) or
    /// - time, version, digest, key (schema 1)
    pub fn read_digest(&self) -> Result<(Bytes, SchemaVersion), ComplexHandshakeError> {
        if let Ok(digest) = self.generate_and_validate(SchemaVersion::Schema0) {
            Ok((digest, SchemaVersion::Schema0))
        } else {
            let digest = self.generate_and_validate(SchemaVersion::Schema1)?;
            Ok((digest, SchemaVersion::Schema1))
        }
    }

    /// Generate and fill digest based on the schema version.
    pub fn generate_and_fill_digest(&self, version: SchemaVersion) -> Result<DigestResult, ComplexHandshakeError> {
        let (left_part, _, right_part) = self.split_message(version)?;
        let computed_digest = self.make_digest(&left_part, &right_part)?;

        // The reason we return 3 parts vs 1 is because if we return 1 part we need to
        // copy the memory But this is unnecessary because we are just going to write it
        // into a buffer.
        Ok(DigestResult {
            left: left_part,
            digest: computed_digest,
            right: right_part,
        })
    }

    fn find_digest_offset(&self, version: SchemaVersion) -> Result<usize, ComplexHandshakeError> {
        const OFFSET_LENGTH: usize = 4;

        // in schema 0 the digest is after the key (which is after the time and version)
        // in schema 1 the digest is after the time and version
        let schema_offset = match version {
            SchemaVersion::Schema0 => CHUNK_LENGTH + TIME_VERSION_LENGTH,
            SchemaVersion::Schema1 => TIME_VERSION_LENGTH,
        };

        // No idea why this isn't a be u32.
        // It seems to be 4 x 8bit values we add together.
        // We then mod it by the chunk length - digest length - offset length
        // Then add the schema offset and offset length to get the digest offset
        Ok((*self.data.get(schema_offset).unwrap() as usize
            + *self.data.get(schema_offset + 1).unwrap() as usize
            + *self.data.get(schema_offset + 2).unwrap() as usize
            + *self.data.get(schema_offset + 3).unwrap() as usize)
            % (CHUNK_LENGTH - RTMP_DIGEST_LENGTH - OFFSET_LENGTH)
            + schema_offset
            + OFFSET_LENGTH)
    }

    fn split_message(&self, version: SchemaVersion) -> Result<(Bytes, Bytes, Bytes), ComplexHandshakeError> {
        let digest_offset = self.find_digest_offset(version)?;

        // We split the message into 3 parts:
        // 1. The part before the digest
        // 2. The digest
        // 3. The part after the digest
        // This is so we can calculate the digest.
        // We then compare it to the digest we read from the message.
        // If they are the same we have a valid message.

        // Slice is a O(1) operation and does not copy the memory.
        let left_part = self.data.slice(0..digest_offset);
        let digest_data = self.data.slice(digest_offset..digest_offset + RTMP_DIGEST_LENGTH);
        let right_part = self.data.slice(digest_offset + RTMP_DIGEST_LENGTH..);

        Ok((left_part, digest_data, right_part))
    }

    /// Make a digest from the left and right parts using the key.
    pub fn make_digest(&self, left: &[u8], right: &[u8]) -> Result<[u8; 32], ComplexHandshakeError> {
        // New hmac from the key
        let mut mac = Hmac::<Sha256>::new_from_slice(self.key).unwrap();
        // Update the hmac with the left and right parts
        mac.update(left);
        mac.update(right);

        // Finalize the hmac and get the digest
        let result = mac.finalize().into_bytes();
        if result.len() != RTMP_DIGEST_LENGTH {
            return Err(ComplexHandshakeError::DigestLengthNotCorrect);
        }

        // This does a copy of the memory but its only 32 bytes so its not a big deal.
        Ok(result.into())
    }

    fn generate_and_validate(&self, version: SchemaVersion) -> Result<Bytes, ComplexHandshakeError> {
        // We need the 3 parts so we can calculate the digest and compare it to the
        // digest we read from the message.
        let (left_part, digest_data, right_part) = self.split_message(version)?;

        // If the digest we calculated is the same as the digest we read from the
        // message we have a valid message.
        if digest_data == self.make_digest(&left_part, &right_part)?.as_ref() {
            Ok(digest_data)
        } else {
            // This does not mean the message is invalid, it just means we need to try the
            // other schema. If both schemas fail then the message is invalid and its likely
            // a simple handshake.
            Err(ComplexHandshakeError::CannotGenerate)
        }
    }
}
