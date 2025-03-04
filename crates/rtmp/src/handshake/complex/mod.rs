use std::io::{self, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::{BufMut, Bytes, BytesMut};
use digest::DigestProcessor;
use rand::Rng;
use scuffle_bytes_util::BytesCursorExt;

use super::current_time;
use super::define::{self, RtmpVersion, ServerHandshakeState};
use crate::handshake::HandshakeError;
use crate::handshake::define::SchemaVersion;

pub mod digest;

/// Complex Handshake Server
/// Unfortunately there doesn't seem to be a good spec sheet for this.
/// https://blog.csdn.net/win_lin/article/details/13006803 is the best I could find.
pub struct ComplexHandshakeServer {
    version: RtmpVersion,
    requested_version: RtmpVersion,

    pub(super) state: ServerHandshakeState,
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
    pub fn handshake(&mut self, input: &mut io::Cursor<Bytes>, output: &mut Vec<u8>) -> Result<(), HandshakeError> {
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

    fn read_c0(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), HandshakeError> {
        // Version (8 bits): In C0, this field identifies the RTMP version
        //  requested by the client.
        self.requested_version = RtmpVersion(input.read_u8()?);

        // We only support version 3 for now.
        // Therefore we set the version to 3.
        self.version = RtmpVersion::Version3;

        Ok(())
    }

    fn read_c1(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), HandshakeError> {
        let c1_bytes = input.extract_bytes(define::RTMP_HANDSHAKE_SIZE)?;

        //  The first 4 bytes of C1 are the timestamp.
        self.c1_timestamp = (&c1_bytes[0..4]).read_u32::<BigEndian>()?;

        // The next 4 bytes are a version number.
        self.c1_version = (&c1_bytes[4..8]).read_u32::<BigEndian>()?;

        // The following 764 bytes are either the digest or the key.
        let data_digest = DigestProcessor::new(c1_bytes, define::RTMP_CLIENT_KEY_FIRST_HALF);

        let (c1_digest_data, schema_version) = data_digest.read_digest()?;

        self.c1_digest = c1_digest_data;
        self.schema_version = schema_version;

        Ok(())
    }

    fn read_c2(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), HandshakeError> {
        // We don't care too much about the data in C2, so we just read it
        //  and discard it.
        input.seek_relative(define::RTMP_HANDSHAKE_SIZE as i64)?;

        Ok(())
    }

    fn write_s0(&mut self, output: &mut Vec<u8>) -> Result<(), HandshakeError> {
        // The version of the protocol used in the handshake.
        // This server is using version 3 of the protocol.
        output.write_u8(self.version.0)?; // 8 bits version

        Ok(())
    }

    fn write_s1(&self, output: &mut Vec<u8>) -> Result<(), HandshakeError> {
        let mut writer = BytesMut::new().writer();

        // The first 4 bytes of S1 are the timestamp.
        writer.write_u32::<BigEndian>(current_time())?;

        // The next 4 bytes are a version number.
        writer.write_u32::<BigEndian>(define::RTMP_SERVER_VERSION)?;

        // We then write 1528 bytes of random data. (764 bytes for digest, 764 bytes for
        // key)
        let mut rng = rand::rng();
        for _ in 0..define::RTMP_HANDSHAKE_SIZE - define::TIME_VERSION_LENGTH {
            writer.write_u8(rng.random())?;
        }

        // The digest is loaded with the data that we just generated.
        let data_digest = DigestProcessor::new(writer.into_inner().freeze(), define::RTMP_SERVER_KEY_FIRST_HALF);

        // We use the same schema version as the client.
        let (first, second, third) = data_digest.generate_and_fill_digest(self.schema_version)?;

        // We then write the parts of the digest to the main writer.
        // Note: this is not a security issue since we do not flush the buffer until we
        // are done  with the handshake.
        output.write_all(&first)?;
        output.write_all(&second)?;
        output.write_all(&third)?;

        Ok(())
    }

    fn write_s2(&self, output: &mut Vec<u8>) -> Result<(), HandshakeError> {
        let start = output.len();

        // We write the current time to the first 4 bytes.
        output.write_u32::<BigEndian>(current_time())?;

        // We write the timestamp from C1 to the next 4 bytes.
        output.write_u32::<BigEndian>(self.c1_timestamp)?;

        // We then write 1528 bytes of random data. (764 bytes for digest, 764 bytes for
        // key)
        let mut rng = rand::rng();

        // define::RTMP_HANDSHAKE_SIZE - define::TIME_VERSION_LENGTH because we already
        // wrote 8 bytes. (timestamp and c1 timestamp)
        for _ in 0..define::RTMP_HANDSHAKE_SIZE - define::RTMP_DIGEST_LENGTH - define::TIME_VERSION_LENGTH {
            output.write_u8(rng.random())?;
        }

        // The digest is loaded with the data that we just generated.
        // This digest is used to generate the key. (digest of c1)
        let key_digest = DigestProcessor::new(Bytes::new(), define::RTMP_SERVER_KEY);

        // Create a digest of the random data using a key generated from the digest of
        // C1.
        let key = key_digest.make_digest(&self.c1_digest, &[])?;
        let data_digest = DigestProcessor::new(Bytes::new(), &key);

        // We then generate a digest using the key and the random data
        // We then extract the first 1504 bytes of the data.
        // define::RTMP_HANDSHAKE_SIZE - 32 = 1504
        // 32 is the size of the digest. for C2S2
        let digest = data_digest.make_digest(
            &output[start..start + define::RTMP_HANDSHAKE_SIZE - define::RTMP_DIGEST_LENGTH],
            &[],
        )?;

        // Write the random data  to the main writer.
        // Total Write = 1536 bytes (1504 + 32)
        output.write_all(&digest)?; // 32 bytes of digest

        Ok(())
    }
}
