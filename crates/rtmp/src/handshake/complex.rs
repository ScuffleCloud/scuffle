//! This module contains the complex handshake for the RTMP protocol.
//!
//! Unfortunately there doesn't seem to be a good spec sheet for this.
//! This implementation is based on this Chinese forum post because it's the best we could find:
//! <https://blog.csdn.net/win_lin/article/details/13006803>

use std::io::{self, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use digest::DigestProcessor;
use rand::Rng;
use scuffle_bytes_util::BytesCursorExt;

use super::{RTMP_HANDSHAKE_SIZE, RtmpVersion, ServerHandshakeState, TIME_VERSION_LENGTH, current_time};

pub mod digest;
pub mod error;

/// This is some magic number, I do not know why its 0x04050001, however, the
/// reference implementation uses this value.
pub const RTMP_SERVER_VERSION: u32 = 0x04050001;

/// This is the length of the digest.
/// There is a lot of random data before and after the digest, however, the
/// digest is always 32 bytes.
pub const RTMP_DIGEST_LENGTH: usize = 32;

/// This is the first half of the server key.
pub const RTMP_SERVER_KEY_FIRST_HALF: &[u8] = b"Genuine Adobe Flash Media Server 001";

/// This is the first half of the client key.
pub const RTMP_CLIENT_KEY_FIRST_HALF: &[u8] = b"Genuine Adobe Flash Player 001";

/// This is the second half of the server/client key.
/// Used for the complex handshake.
pub const RTMP_SERVER_KEY: &[u8] = &[
    0x47, 0x65, 0x6e, 0x75, 0x69, 0x6e, 0x65, 0x20, 0x41, 0x64, 0x6f, 0x62, 0x65, 0x20, 0x46, 0x6c, 0x61, 0x73, 0x68, 0x20,
    0x4d, 0x65, 0x64, 0x69, 0x61, 0x20, 0x53, 0x65, 0x72, 0x76, 0x65, 0x72, 0x20, 0x30, 0x30, 0x31, 0xf0, 0xee, 0xc2, 0x4a,
    0x80, 0x68, 0xbe, 0xe8, 0x2e, 0x00, 0xd0, 0xd1, 0x02, 0x9e, 0x7e, 0x57, 0x6e, 0xec, 0x5d, 0x2d, 0x29, 0x80, 0x6f, 0xab,
    0x93, 0xb8, 0xe6, 0x36, 0xcf, 0xeb, 0x31, 0xae,
];

/// The schema version.
///
/// For the complex handshake the schema is either 0 or 1.
/// A chunk is 764 bytes. (1536 - 8) / 2 = 764
/// A schema of 0 means the digest is after the key, thus the digest is at
/// offset 776 bytes (768 + 8). A schema of 1 means the digest is before the key
/// thus the offset is at offset 8 bytes (0 + 8). Where 8 bytes is the time and
/// version. (4 bytes each) The schema is determined by the client.
/// The server will always use the schema the client uses.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SchemaVersion {
    /// Schema 0.
    Schema0,
    /// Schema 1.
    Schema1,
}

/// Complex Handshake Server.
pub struct ComplexHandshakeServer {
    version: RtmpVersion,
    requested_version: RtmpVersion,
    state: ServerHandshakeState,
    schema_version: SchemaVersion,
    c1_digest: Bytes,
    c1_timestamp: u32,
    c1_version: u32,
}

impl Default for ComplexHandshakeServer {
    fn default() -> Self {
        Self {
            state: ServerHandshakeState::ReadC0C1,
            c1_digest: Bytes::default(),
            c1_timestamp: 0,
            version: RtmpVersion::Version3,
            requested_version: RtmpVersion(0),
            c1_version: 0,
            schema_version: SchemaVersion::Schema0,
        }
    }
}

impl ComplexHandshakeServer {
    /// Returns true if the handshake is finished.
    pub fn is_finished(&self) -> bool {
        self.state == ServerHandshakeState::Finish
    }

    /// Perform the complex handshake.
    pub fn handshake(&mut self, input: &mut io::Cursor<Bytes>, output: &mut Vec<u8>) -> Result<(), crate::error::RtmpError> {
        match self.state {
            ServerHandshakeState::ReadC0C1 => {
                self.read_c0(input)?;
                self.read_c1(input)?;
                self.write_s0(output)?;
                self.write_s1(output)?;
                self.write_s2(output)?;
                self.state = ServerHandshakeState::ReadC2;
            }
            ServerHandshakeState::ReadC2 => {
                self.read_c2(input)?;
                self.state = ServerHandshakeState::Finish;
            }
            ServerHandshakeState::Finish => {}
        }

        Ok(())
    }

    fn read_c0(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), crate::error::RtmpError> {
        // Version (8 bits): In C0, this field identifies the RTMP version
        // requested by the client.
        self.requested_version = RtmpVersion(input.read_u8()?);

        // We only support version 3 for now.
        // Therefore we set the version to 3.
        self.version = RtmpVersion::Version3;

        Ok(())
    }

    fn read_c1(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), crate::error::RtmpError> {
        let c1_bytes = input.extract_bytes(RTMP_HANDSHAKE_SIZE)?;

        // The first 4 bytes of C1 are the timestamp.
        self.c1_timestamp = (&c1_bytes[0..4]).read_u32::<BigEndian>()?;

        // The next 4 bytes are a version number.
        self.c1_version = (&c1_bytes[4..8]).read_u32::<BigEndian>()?;

        // The following 764 bytes are either the digest or the key.
        let data_digest = DigestProcessor::new(c1_bytes, RTMP_CLIENT_KEY_FIRST_HALF);

        let (c1_digest_data, schema_version) = data_digest.read_digest()?;

        self.c1_digest = c1_digest_data;
        self.schema_version = schema_version;

        Ok(())
    }

    fn read_c2(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), crate::error::RtmpError> {
        // We don't care too much about the data in C2, so we just read it
        // and discard it.
        input.seek_relative(RTMP_HANDSHAKE_SIZE as i64)?;

        Ok(())
    }

    fn write_s0(&mut self, output: &mut Vec<u8>) -> Result<(), crate::error::RtmpError> {
        // The version of the protocol used in the handshake.
        // This server is using version 3 of the protocol.
        output.write_u8(self.version.0)?; // 8 bits version

        Ok(())
    }

    fn write_s1(&self, output: &mut Vec<u8>) -> Result<(), crate::error::RtmpError> {
        let mut writer = BytesMut::new().writer();

        // The first 4 bytes of S1 are the timestamp.
        writer.write_u32::<BigEndian>(current_time())?;

        // The next 4 bytes are a version number.
        writer.write_u32::<BigEndian>(RTMP_SERVER_VERSION)?;

        // We then write 1528 bytes of random data.
        // 764 bytes for the digest, 764 bytes for the key.
        let mut rng = rand::rng();
        for _ in 0..RTMP_HANDSHAKE_SIZE - TIME_VERSION_LENGTH {
            writer.write_u8(rng.random())?;
        }

        // The digest is loaded with the data that we just generated.
        let data_digest = DigestProcessor::new(writer.into_inner().freeze(), RTMP_SERVER_KEY_FIRST_HALF);

        // We use the same schema version as the client and then write the result of the digest to the main writer.
        data_digest.generate_and_fill_digest(self.schema_version)?.write_to(output)?;

        Ok(())
    }

    fn write_s2(&self, output: &mut Vec<u8>) -> Result<(), crate::error::RtmpError> {
        let start = output.len();

        // We write the current time to the first 4 bytes.
        output.write_u32::<BigEndian>(current_time())?;

        // We write the timestamp from C1 to the next 4 bytes.
        output.write_u32::<BigEndian>(self.c1_timestamp)?;

        // We then write 1528 bytes of random data.
        // 764 bytes for the digest, 764 bytes for the key.
        let mut rng = rand::rng();

        // RTMP_HANDSHAKE_SIZE - TIME_VERSION_LENGTH because we already
        // wrote 8 bytes. (timestamp and c1 timestamp)
        for _ in 0..RTMP_HANDSHAKE_SIZE - RTMP_DIGEST_LENGTH - TIME_VERSION_LENGTH {
            output.write_u8(rng.random())?;
        }

        // The digest is loaded with the data that we just generated.
        // This digest is used to generate the key. (digest of c1)
        let key_digest = DigestProcessor::new(Bytes::new(), RTMP_SERVER_KEY);

        // Create a digest of the random data using a key generated from the digest of
        // C1.
        let key = key_digest.make_digest(&self.c1_digest, &[])?;
        let data_digest = DigestProcessor::new(Bytes::new(), &key);

        // We then generate a digest using the key and the random data
        // We then extract the first 1504 bytes of the data.
        // RTMP_HANDSHAKE_SIZE - 32 = 1504
        // 32 is the size of the digest. for C2S2
        let digest = data_digest.make_digest(&output[start..start + RTMP_HANDSHAKE_SIZE - RTMP_DIGEST_LENGTH], &[])?;

        // Write the random data  to the main writer.
        // Total Write = 1536 bytes (1504 + 32)
        output.write_all(&digest)?; // 32 bytes of digest

        Ok(())
    }
}
