use crate::{Frame, FrameBuilder, IdBuilder, PDU_NOT_AVAILABLE, PGN};

/// Maximum number of data bytes
pub const DATA_MAX_LENGTH: usize = 1785;
/// Maximum number of data bytes per frame
pub const DATA_FRAME_SIZE: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionManagement {
    RequestToSend = 0x10,
    ClearToSend = 0x11,
    EndOfMessageAcknowledgment = 0x13,
    BroadcastAnnounceMessage = 0x20,
    Abort = 0xff,
}

impl TryFrom<u8> for ConnectionManagement {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x10 => Ok(ConnectionManagement::RequestToSend),
            0x11 => Ok(ConnectionManagement::ClearToSend),
            0x13 => Ok(ConnectionManagement::EndOfMessageAcknowledgment),
            0x20 => Ok(ConnectionManagement::BroadcastAnnounceMessage),
            0xff => Ok(ConnectionManagement::Abort),
            _ => Err(value),
        }
    }
}

pub enum BroadcastTransportState {
    ConnectionManagement,
    DataTransfer(u8),
}

pub struct BroadcastTransport {
    sa: u8,
    pgn: PGN,
    data: [u8; DATA_MAX_LENGTH],
    data_length: usize,
    tail: usize,
    state: BroadcastTransportState,
}

impl BroadcastTransport {
    #[must_use]
    pub fn new(sa: u8, pgn: PGN) -> Self {
        Self {
            sa,
            pgn,
            data: [PDU_NOT_AVAILABLE; DATA_MAX_LENGTH],
            data_length: 0,
            tail: 0,
            state: BroadcastTransportState::ConnectionManagement,
        }
    }

    #[must_use]
    pub fn with_data(mut self, data: &[u8]) -> Self {
        let len = data.len().min(DATA_MAX_LENGTH);
        self.data[..len].copy_from_slice(&data[..len]);
        self.data_length = len;
        self.tail = len;
        self
    }

    /// Returns a slice of the transport data.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data[..self.tail]
    }

    /// Returns the length of the transport data.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.tail
    }

    /// Returns `true` if the transport data is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tail == 0
    }

    #[must_use]
    pub fn packet_count(&self) -> usize {
        let quotient = self.data_length / DATA_FRAME_SIZE;
        let remainder = self.data_length % DATA_FRAME_SIZE;

        if remainder > 0 {
            quotient + 1
        } else {
            quotient
        }
    }

    pub fn next_frame(&mut self) -> Frame {
        match self.state {
            BroadcastTransportState::ConnectionManagement => {
                #[allow(clippy::cast_possible_truncation)]
                let data_length = (self.data_length as u16).to_le_bytes();
                #[allow(clippy::cast_possible_truncation)]
                let packets = self.packet_count() as u8;
                let byte_array = self.pgn.to_le_bytes();

                let frame = FrameBuilder::new(
                    IdBuilder::from_pgn(PGN::TransportProtocolConnectionManagement)
                        .priority(7)
                        .sa(self.sa)
                        .da(0xff)
                        .build(),
                )
                .copy_from_slice(&[
                    ConnectionManagement::BroadcastAnnounceMessage as u8,
                    data_length[0],
                    data_length[1],
                    packets,
                    PDU_NOT_AVAILABLE,
                    byte_array[0],
                    byte_array[1],
                    byte_array[2],
                ])
                .build();

                self.state = BroadcastTransportState::DataTransfer(0);

                frame
            }
            BroadcastTransportState::DataTransfer(packet) => {
                let start = packet as usize * DATA_FRAME_SIZE;

                // Return a padding frame if called beyond the data boundary
                if start >= self.data_length {
                    return FrameBuilder::new(
                        IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer)
                            .priority(7)
                            .sa(self.sa)
                            .da(0xff)
                            .build(),
                    )
                    .set_len(8)
                    .build();
                }

                let sequence = packet + 1;
                let end = (start + DATA_FRAME_SIZE).min(self.data_length);

                let mut frame_builder = FrameBuilder::new(
                    IdBuilder::from_pgn(PGN::TransportProtocolDataTransfer)
                        .priority(7)
                        .sa(self.sa)
                        .da(0xff)
                        .build(),
                );

                let payload = frame_builder.as_mut();
                payload[0] = sequence;

                let data_chunk = &self.data[start..end];
                payload[1..=data_chunk.len()].copy_from_slice(data_chunk);

                let frame = frame_builder.set_len(8).build();

                self.state = BroadcastTransportState::DataTransfer(sequence);

                frame
            }
        }
    }

    pub fn from_frame(&mut self, frame: &Frame) {
        let pgn = frame.id().pgn();
        if pgn == PGN::TransportProtocolConnectionManagement {
            let data = frame.as_ref();
            if data.len() < 8 {
                return; // Malformed CM frame
            }
            let data_length = u16::from_le_bytes([data[1], data[2]]) as usize;

            if data[0] == ConnectionManagement::BroadcastAnnounceMessage as u8 {
                self.pgn = PGN::from_le_bytes([data[5], data[6], data[7]]);
                self.data_length = data_length.min(DATA_MAX_LENGTH);
                self.state = BroadcastTransportState::DataTransfer(0);
            }
        } else if pgn == PGN::TransportProtocolDataTransfer {
            let data = frame.as_ref();
            if data.len() < 2 {
                return; // Malformed DT frame
            }
            let sequence = data[0];

            // Validate sequence number: must be 1-255
            if sequence == 0 {
                return; // Invalid sequence
            }

            let data_chunk = &data[1..];
            let start = (sequence as usize - 1) * DATA_FRAME_SIZE;
            let end = start + data_chunk.len();

            // Bounds check: ensure we don't write beyond DATA_MAX_LENGTH
            if end > DATA_MAX_LENGTH {
                return; // Out of bounds
            }

            self.tail = self.data_length.min(end);
            self.data[start..end].copy_from_slice(data_chunk);
        }
    }
}

impl AsRef<[u8]> for BroadcastTransport {
    fn as_ref(&self) -> &[u8] {
        &self.data[..self.tail]
    }
}

#[cfg(test)]
mod tests {
    use crate::Id;

    use super::*;

    #[test]
    fn test_broadcast_transport() {
        let data = [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09];

        let mut transport = BroadcastTransport::new(0x01, PGN::AddressClaimed).with_data(&data);

        let frame = transport.next_frame();
        assert_eq!(frame.id().as_raw(), 0x1CEC_FF01);
        assert_eq!(frame.len(), 8);
        assert_eq!(
            frame.as_ref(),
            &[0x20, 0x09, 0x00, 0x02, 0xFF, 0x00, 0xEE, 0x00]
        );

        let frame = transport.next_frame();
        assert_eq!(frame.id().as_raw(), 0x1CEB_FF01);
        assert_eq!(frame.len(), 8);
        assert_eq!(
            frame.as_ref(),
            &[0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
        );

        let frame = transport.next_frame();
        assert_eq!(frame.id().as_raw(), 0x1CEB_FF01);
        assert_eq!(frame.len(), 8);
        assert_eq!(
            frame.as_ref(),
            &[0x02, 0x08, 0x09, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]
        );
    }

    #[test]
    fn test_broadcast_transport2() {
        let frame1 = [0x20, 0x09, 0x00, 0x02, 0xFF, 0x00, 0xEE, 0x00];
        let frame2 = [0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        let frame3 = [0x02, 0x08, 0x09, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];

        let mut transport = BroadcastTransport::new(0x01, PGN::AddressClaimed);

        transport.from_frame(
            &FrameBuilder::new(Id::new(0x1CEC_FF01))
                .copy_from_slice(&frame1)
                .build(),
        );
        assert_eq!(transport.len(), 0);
        assert_eq!(transport.packet_count(), 2);

        transport.from_frame(
            &FrameBuilder::new(Id::new(0x1CEB_FF01))
                .copy_from_slice(&frame2)
                .build(),
        );
        transport.from_frame(
            &FrameBuilder::new(Id::new(0x1CEB_FF01))
                .copy_from_slice(&frame3)
                .build(),
        );
        assert_eq!(transport.len(), 9);
        assert_eq!(
            transport.data(),
            &[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09]
        );
    }

    #[test]
    fn test_oversized_data() {
        // Test that oversized data is clamped to DATA_MAX_LENGTH (1785 bytes)
        let large_data = [0xAB; DATA_MAX_LENGTH + 100];
        let transport = BroadcastTransport::new(0x01, PGN::AddressClaimed)
            .with_data(&large_data);

        // Should clamp to DATA_MAX_LENGTH
        assert_eq!(transport.len(), DATA_MAX_LENGTH);
        assert_eq!(transport.data().len(), DATA_MAX_LENGTH);
        assert!(transport.data().iter().all(|&b| b == 0xAB));
    }

    #[test]
    fn test_invalid_sequence_zero() {
        // Test that sequence number 0 is rejected (should be 1-255)
        let mut transport = BroadcastTransport::new(0x01, PGN::AddressClaimed);

        // First, receive the connection management frame announcing 9 bytes
        let cm_frame = [0x20, 0x09, 0x00, 0x02, 0xFF, 0x00, 0xEE, 0x00];
        transport.from_frame(
            &FrameBuilder::new(Id::new(0x1CEC_FF01))
                .copy_from_slice(&cm_frame)
                .build(),
        );

        // Verify CM was processed
        assert_eq!(transport.data_length, 9);

        // Now try to receive a data frame with sequence 0 (invalid)
        let invalid_frame = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07];
        transport.from_frame(
            &FrameBuilder::new(Id::new(0x1CEB_FF01))
                .copy_from_slice(&invalid_frame)
                .build(),
        );

        // tail should still be 0 (frame rejected due to invalid sequence)
        assert_eq!(transport.tail, 0);
    }

    #[test]
    fn test_max_packets() {
        // Test with maximum packet count (255 packets * 7 bytes = 1785 bytes)
        let max_data = [0xCC; DATA_MAX_LENGTH];
        let transport = BroadcastTransport::new(0x01, PGN::ProprietaryA)
            .with_data(&max_data);

        let expected_packets = DATA_MAX_LENGTH.div_ceil(DATA_FRAME_SIZE);
        assert_eq!(transport.packet_count(), expected_packets);
        assert_eq!(transport.len(), DATA_MAX_LENGTH);

        // Verify all data is present
        assert_eq!(transport.data(), &max_data[..]);
    }

    #[test]
    fn test_empty_transport() {
        // Test transport with no data
        let transport = BroadcastTransport::new(0x01, PGN::Request);

        assert_eq!(transport.len(), 0);
        assert!(transport.is_empty());
        assert_eq!(transport.packet_count(), 0);
    }
}
