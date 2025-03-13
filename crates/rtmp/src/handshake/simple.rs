use std::io::{self, Seek, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use bytes::Bytes;
use rand::Rng;
use scuffle_bytes_util::BytesCursorExt;

use super::current_time;
use super::define::{self, RtmpVersion, ServerHandshakeState};

/// Simple Handshake Server
/// RTMP Spec 1.0 - 5.2
pub struct SimpleHandshakeServer {
    version: RtmpVersion,
    requested_version: RtmpVersion,
    state: ServerHandshakeState,
    c1_bytes: Bytes,
    c1_timestamp: u32,
}

impl Default for SimpleHandshakeServer {
    fn default() -> Self {
        Self {
            state: ServerHandshakeState::ReadC0C1,
            c1_bytes: Bytes::new(),
            c1_timestamp: 0,
            version: RtmpVersion::Version3,
            requested_version: RtmpVersion(0),
        }
    }
}

impl SimpleHandshakeServer {
    /// Returns true if the handshake is finished.
    pub fn is_finished(&self) -> bool {
        self.state == ServerHandshakeState::Finish
    }

    /// Perform the handshake, writing to the output and reading from the input.
    pub fn handshake(&mut self, input: &mut io::Cursor<Bytes>, output: &mut Vec<u8>) -> Result<(), crate::error::Error> {
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

    fn read_c0(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), crate::error::Error> {
        // Version (8 bits): In C0, this field identifies the RTMP version
        //  requested by the client.
        self.requested_version = RtmpVersion::from(input.read_u8()?);

        // We only support version 3 for now.
        // Therefore we set the version to 3.
        self.version = RtmpVersion::Version3;

        Ok(())
    }

    fn read_c1(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), crate::error::Error> {
        // Time (4 bytes): This field contains a timestamp, which SHOULD be
        //  used as the epoch for all future chunks sent from this endpoint.
        //  This may be 0, or some arbitrary value. To synchronize multiple
        //  chunkstreams, the endpoint may wish to send the current value of
        //  the other chunkstream’s timestamp.
        self.c1_timestamp = input.read_u32::<BigEndian>()?;

        // Zero (4 bytes): This field MUST be all 0s.
        input.read_u32::<BigEndian>()?;

        // Random data (1528 bytes): This field can contain any arbitrary
        //  values. Since each endpoint has to distinguish between the
        //  response to the handshake it has initiated and the handshake
        //  initiated by its peer,this data SHOULD send something sufficiently
        //  random. But there is no need for cryptographically-secure
        //  randomness, or even dynamic values.
        self.c1_bytes = input.extract_bytes(define::RTMP_HANDSHAKE_SIZE - define::TIME_VERSION_LENGTH)?;

        Ok(())
    }

    fn read_c2(&mut self, input: &mut io::Cursor<Bytes>) -> Result<(), crate::error::Error> {
        // We don't care too much about the data in C2, so we just read it
        //  and discard it.
        // We should technically check that the timestamp is the same as
        //  the one we sent in S1, but we don't care. And that the random
        //  data is the same as the one we sent in S2, but we don't care.
        //  Some clients are not strict to spec and send different data.
        // We can just ignore it and not be super strict.
        input.seek_relative(define::RTMP_HANDSHAKE_SIZE as i64)?;

        Ok(())
    }

    /// Defined in RTMP Specification 1.0 - 5.2.2
    fn write_s0(&mut self, output: &mut Vec<u8>) -> Result<(), crate::error::Error> {
        // Version (8 bits): In S0, this field identifies the RTMP
        //  version selected by the server. The version defined by this
        //  specification is 3. A server that does not recognize the
        //  client’s requested version SHOULD respond with 3. The client MAY
        //  choose to degrade to version 3, or to abandon the handshake.
        output.write_u8(self.version.0)?;

        Ok(())
    }

    /// Defined in RTMP Specification 1.0 - 5.2.3
    fn write_s1(&mut self, output: &mut Vec<u8>) -> Result<(), crate::error::Error> {
        // Time (4 bytes): This field contains a timestamp, which SHOULD be
        //  used as the epoch for all future chunks sent from this endpoint.
        //  This may be 0, or some arbitrary value. To synchronize multiple
        //  chunkstreams, the endpoint may wish to send the current value of
        //  the other chunkstream’s timestamp.
        output.write_u32::<BigEndian>(current_time())?;

        // Zero(4 bytes): This field MUST be all 0s.
        output.write_u32::<BigEndian>(0)?;

        // Random data (1528 bytes): This field can contain any arbitrary
        //  values. Since each endpoint has to distinguish between the
        //  response to the handshake it has initiated and the handshake
        //  initiated by its peer,this data SHOULD send something sufficiently
        //  random. But there is no need for cryptographically-secure
        //  randomness, or even dynamic values.
        let mut rng = rand::rng();
        for _ in 0..1528 {
            output.write_u8(rng.random())?;
        }

        Ok(())
    }

    fn write_s2(&mut self, output: &mut Vec<u8>) -> Result<(), crate::error::Error> {
        // Time (4 bytes): This field MUST contain the timestamp sent by the C1 (for
        // S2).
        output.write_u32::<BigEndian>(self.c1_timestamp)?;

        // Time2 (4 bytes): This field MUST contain the timestamp at which the
        //  previous packet(s1 or c1) sent by the peer was read.
        output.write_u32::<BigEndian>(current_time())?;

        // Random echo (1528 bytes): This field MUST contain the random data
        //  field sent by the peer in S1 (for C2) or S2 (for C1). Either peer
        //  can use the time and time2 fields together with the current
        //  timestamp as a quick estimate of the bandwidth and/or latency of
        //  the connection, but this is unlikely to be useful.
        output.write_all(&self.c1_bytes[..])?;

        Ok(())
    }
}
