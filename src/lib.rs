#![deny(unsafe_code)]
#![deny(warnings)]
#![no_std]

pub mod diagnostic;
mod name;
mod pgn;
pub mod protocol;
mod sa;
mod slots;
pub mod spn;
pub mod transport;

pub use name::*;
pub use pgn::*;
pub use sa::*;

/// Maximum number of bytes in a PGN.
pub const PGN_MAX_LENGTH: usize = 3;
/// Maximum number of bytes in a PDU.
pub const PDU_MAX_LENGTH: usize = 8;

/// PDU error byte.
pub const PDU_ERROR: u8 = 0xfe;
/// PDU not available byte.
pub const PDU_NOT_AVAILABLE: u8 = 0xff;

/// ASCII delimiter for variable length fields.
pub const FIELD_DELIMITER: u8 = b'*';

/// 29-bit identifier mask.
pub const ID_BIT_MASK: u32 = 0x1fff_ffff;

/// Protocol Data Unit Format.
///
/// There are two different PDU formats. PDU1 format is used for sending messages with a specific
/// destination address. PDU2 format can only sent broadcasts. The PDU format byte in the identifier
/// determines the message format. If the PDU format byte is less than 240 (0xF0) then the format is
/// PDU1 and if it is greater than 239 it is PDU2.
#[derive(Debug, PartialEq, Eq)]
pub enum PDUFormat {
    PDU1(u8),
    PDU2(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id(u32);

/// Frame ID
impl Id {
    /// Construct new Frame ID from raw integer.
    ///
    /// The ID is masked to 29 bits to ensure that the ID is within the valid range.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id & ID_BIT_MASK)
    }

    /// Return ID as raw integer.
    #[inline]
    #[must_use]
    pub const fn as_raw(&self) -> u32 {
        self.0
    }

    /// Frame priority
    ///
    /// The priority ranges from 0 to 7, where 0 is the highest priority and 7 the lowest priority.
    ///
    /// Default priority for informational, proprietary, request and acknowledgement frames is 6.
    /// Default priority for control frames (e.g., speeding up or slowing down the vehicle) is 3.
    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn priority(&self) -> u8 {
        (self.0 >> 26) as u8
    }

    /// Data page (DP)
    ///
    /// Returns the data page bit of the frame ID.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn data_page(&self) -> u8 {
        ((self.0 >> 24) & 0x1) as u8
    }

    /// Parameter Group Number (PGN)
    ///
    /// Returns the parameter group number of the frame ID.
    #[must_use]
    pub fn pgn(&self) -> PGN {
        self.pgn_raw().into()
    }

    /// Parameter Group Number
    ///
    /// Returns the raw parameter group number of the frame ID.
    #[must_use]
    pub fn pgn_raw(&self) -> u32 {
        match self.pdu_format() {
            PDUFormat::PDU1(_) => (self.0 >> 8) & 0xff00,
            PDUFormat::PDU2(_) => (self.0 >> 8) & 0xffff,
        }
    }

    /// PDU Format (PF)
    ///
    /// Returns the PDU format of the frame ID.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn pdu_format(&self) -> PDUFormat {
        let format: u8 = ((self.0 >> 16) & 0xff) as u8;
        if format < 0xf0 {
            PDUFormat::PDU1(format)
        } else {
            PDUFormat::PDU2(format)
        }
    }

    /// Test if the frame is a broadcast frame
    ///
    /// Returns true if the frame is a broadcast frame, false otherwise.
    #[must_use]
    pub fn is_broadcast(&self) -> bool {
        match self.pdu_format() {
            PDUFormat::PDU1(_) => self.destination_address() == Some(0xff),
            PDUFormat::PDU2(_) => true,
        }
    }

    /// Frame Destination Address (DA)
    ///
    /// Returns the destination address of the frame ID.
    ///
    /// The destination address is only available on PDU1 frames.
    #[must_use]
    pub fn destination_address(&self) -> Option<u8> {
        match self.pdu_format() {
            PDUFormat::PDU1(_) => Some(self.pdu_specific()),
            PDUFormat::PDU2(_) => None,
        }
    }

    /// Frame Group Extension (GE)
    ///
    /// Returns the group extension of the frame ID.
    ///
    /// The group extension is only available on PDU2 frames.
    #[must_use]
    pub fn group_extension(&self) -> Option<u8> {
        match self.pdu_format() {
            PDUFormat::PDU2(_) => Some(self.pdu_specific()),
            PDUFormat::PDU1(_) => None,
        }
    }

    /// PDU Specific (PS)
    ///
    /// Returns the PDU specific value of the frame ID.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn pdu_specific(&self) -> u8 {
        ((self.0 >> 8) & 0xff) as u8
    }

    /// Device Source Address (SA)
    ///
    /// Returns the source address of the frame ID.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn source_address(&self) -> u8 {
        (self.0 & 0xff) as u8
    }
}

impl core::fmt::Display for Id {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(da) = self.destination_address() {
            write!(
                f,
                "[{:08X?}] Prio: {} PGN: {} DA: 0x{:X?}",
                self.as_raw(),
                self.priority(),
                self.pgn_raw(),
                da
            )
        } else {
            write!(
                f,
                "[{:08X?}] Prio: {} PGN: {}",
                self.as_raw(),
                self.priority(),
                self.pgn_raw()
            )
        }
    }
}

pub struct IdBuilder {
    /// Message priority.
    priority: u8,
    /// Parameter group number.
    pgn: u32,
    /// Source address.
    source_address: u8,
    /// Destination address.
    destination_address: u8,
}

impl IdBuilder {
    /// Construct ID builder from PGN.
    #[must_use]
    pub fn from_pgn(pgn: PGN) -> Self {
        Self {
            priority: 6,
            pgn: pgn.into(),
            source_address: 0,
            destination_address: 0,
        }
    }

    /// Set the priority.
    #[inline]
    #[must_use]
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(7);
        self
    }

    // TODO: Rename to 'source_address'
    /// Set the sender address.
    #[inline]
    #[must_use]
    pub fn sa(mut self, address: u8) -> Self {
        self.source_address = address;
        self
    }

    // TODO: Rename to 'destination_address'
    /// Set the destination address.
    #[inline]
    #[must_use]
    pub fn da(mut self, address: u8) -> Self {
        self.destination_address = address;
        self
    }

    /// Build frame ID.
    #[must_use]
    pub fn build(self) -> Id {
        let mut id =
            u32::from(self.priority) << 26 | self.pgn << 8 | u32::from(self.source_address);

        if let PDUFormat::PDU1(_) = Id::new(id).pdu_format() {
            id |= u32::from(self.destination_address) << 8;
        }

        Id::new(id)
    }
}

/// Data frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Frame ID.
    id: Id,
    /// PDU.
    pdu: [u8; PDU_MAX_LENGTH],
    /// PDU length.
    pdu_length: usize,
}

impl Frame {
    /// Construct a new frame.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the frame.
    /// * `pdu` - The Protocol Data Unit (PDU) of the frame.
    ///
    /// # Returns
    ///
    /// A new `Frame` instance.
    #[must_use]
    pub fn new(id: Id, pdu: [u8; PDU_MAX_LENGTH]) -> Self {
        Self {
            id,
            pdu,
            pdu_length: PDU_MAX_LENGTH,
        }
    }

    /// Construct a new frame from raw ID and PDU.
    ///
    /// # Arguments
    ///
    /// * `id` - The raw ID of the frame.
    /// * `pdu` - The Protocol Data Unit (PDU) of the frame.
    ///
    /// # Returns
    ///
    /// A new `Frame` instance.
    #[must_use]
    pub fn from_raw(id: u32, pdu: [u8; PDU_MAX_LENGTH]) -> Self {
        Self {
            id: Id::new(id),
            pdu,
            pdu_length: PDU_MAX_LENGTH,
        }
    }

    /// Get the ID of the frame.
    ///
    /// # Returns
    ///
    /// A reference to the `Id` of the frame.
    #[inline]
    #[must_use]
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns a slice of the PDU data.
    ///
    /// # Returns
    ///
    /// A slice of the PDU data.
    #[inline]
    #[must_use]
    pub fn pdu(&self) -> &[u8] {
        &self.pdu[..self.pdu_length]
    }

    /// Returns the length of the PDU data.
    ///
    /// # Returns
    ///
    /// The length of the PDU data.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.pdu_length
    }

    /// Returns `true` if the PDU data is empty.
    ///
    /// # Returns
    ///
    /// `true` if the PDU data is empty, `false` otherwise.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pdu_length == 0
    }
}

impl core::fmt::Display for Frame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}    {:02X?}", self.id(), self.pdu())
    }
}

impl AsRef<[u8]> for Frame {
    fn as_ref(&self) -> &[u8] {
        &self.pdu[..self.pdu_length]
    }
}

pub struct FrameBuilder {
    /// Frame ID.
    id: Id,
    /// Protocol Data Unit.
    pdu: [u8; PDU_MAX_LENGTH],
    /// PDU length.
    pdu_length: usize,
}

impl Default for FrameBuilder {
    fn default() -> Self {
        Self {
            id: Id::new(0),
            pdu: [PDU_NOT_AVAILABLE; PDU_MAX_LENGTH],
            pdu_length: 0,
        }
    }
}

impl FrameBuilder {
    /// Construct new frame builder.
    #[must_use]
    pub fn new(id: Id) -> Self {
        Self::default().id(id)
    }

    /// Set the frame ID.
    #[inline]
    #[must_use]
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    /// Copy PDU data from slice.
    ///
    /// If the source slice is larger than `PDU_MAX_LENGTH` (8 bytes),
    /// only the first 8 bytes will be copied.
    #[must_use]
    pub fn copy_from_slice(mut self, src: &[u8]) -> Self {
        let pdu_length = src.len().min(PDU_MAX_LENGTH);
        self.pdu[..pdu_length].copy_from_slice(&src[..pdu_length]);
        self.pdu_length = pdu_length;
        self
    }

    /// Set PDU length.
    #[inline]
    #[must_use]
    pub fn set_len(mut self, len: usize) -> Self {
        self.pdu_length = len.min(PDU_MAX_LENGTH);
        self
    }

    /// Construct frame.
    #[must_use]
    pub fn build(self) -> Frame {
        Frame {
            id: self.id,
            pdu: self.pdu,
            pdu_length: self.pdu_length,
        }
    }
}

impl AsMut<[u8]> for FrameBuilder {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.pdu
    }
}

#[cfg(test)]
mod tests {
    use crate::{FrameBuilder, Id, IdBuilder, PDUFormat, PDU_MAX_LENGTH, PDU_NOT_AVAILABLE, PGN};

    #[test]
    fn id_decode_1() {
        let id = Id::new(0x18EA_FF00);

        assert_eq!(id.as_raw(), 0x18EA_FF00);
        assert_eq!(id.priority(), 6);
        assert_eq!(id.data_page(), 0);
        assert_eq!(id.pgn_raw(), 59904);
        assert_eq!(id.pgn(), PGN::Request);
        assert_eq!(id.pdu_format(), PDUFormat::PDU1(234));
        assert!(id.is_broadcast());
        assert_eq!(id.pdu_specific(), 255);
        assert_eq!(id.destination_address(), Some(255));
        assert_eq!(id.group_extension(), None);
        assert_eq!(id.source_address(), 0);
    }

    #[test]
    fn id_decode_2() {
        let id = Id::new(0x18EA_687A);

        assert_eq!(id.as_raw(), 0x18EA_687A);
        assert_eq!(id.priority(), 6);
        assert_eq!(id.data_page(), 0);
        assert_eq!(id.pgn_raw(), 59904);
        assert_eq!(id.pgn(), PGN::Request);
        assert_eq!(id.pdu_format(), PDUFormat::PDU1(234));
        assert!(!id.is_broadcast());
        assert_eq!(id.pdu_specific(), 104);
        assert_eq!(id.destination_address(), Some(0x68));
        assert_eq!(id.group_extension(), None);
        assert_eq!(id.source_address(), 0x7A);
    }

    #[test]
    fn id_decode_3() {
        let id = Id::new(0x0CFE_6CEE);

        assert_eq!(id.as_raw(), 0x0CFE_6CEE);
        assert_eq!(id.priority(), 3);
        assert_eq!(id.data_page(), 0);
        assert_eq!(id.pgn_raw(), 65132);
        assert_eq!(id.pdu_format(), PDUFormat::PDU2(254));
        assert!(id.is_broadcast());
        assert_eq!(id.pdu_specific(), 108);
        assert_eq!(id.destination_address(), None);
        assert_eq!(id.group_extension(), Some(108));
        assert_eq!(id.source_address(), 238);
    }

    #[test]
    fn id_decode_4() {
        let id = Id::new(0x0DFE_6CEE);

        assert_eq!(id.as_raw(), 0x0DFE_6CEE);
        assert_eq!(id.priority(), 3);
        assert_eq!(id.data_page(), 1);
        assert_eq!(id.pgn_raw(), 65132);
        assert_eq!(id.pdu_format(), PDUFormat::PDU2(254));
        assert!(id.is_broadcast());
        assert_eq!(id.pdu_specific(), 108);
        assert_eq!(id.destination_address(), None);
        assert_eq!(id.group_extension(), Some(108));
        assert_eq!(id.source_address(), 238);
    }

    #[test]
    fn id_build_1() {
        let id = IdBuilder::from_pgn(PGN::Transfer)
            .priority(3)
            .sa(139)
            .build();

        assert_eq!(id, Id::new(0x0CCA_008B));
    }

    #[test]
    fn id_build_2() {
        let id = IdBuilder::from_pgn(PGN::Transfer)
            .priority(3)
            .da(0x34)
            .sa(139)
            .build();

        assert_eq!(id, Id::new(0x0CCA_348B));
    }

    #[test]
    fn id_build_3() {
        let id = IdBuilder::from_pgn(PGN::ElectronicEngineController1)
            .priority(3)
            .da(0)
            .sa(12)
            .build();

        assert_eq!(id, Id::new(0x0CF0_040C));
        assert_eq!(id.pgn_raw(), 61444);
    }

    #[test]
    fn id_build_4() {
        let id = IdBuilder::from_pgn(PGN::VehicleElectricalPower1)
            .sa(234)
            .build();

        assert_eq!(id, Id::new(0x18FE_F7EA));
    }

    #[test]
    fn id_build_5() {
        let id = IdBuilder::from_pgn(PGN::Other(126_720)).sa(234).build();

        assert_eq!(id, Id::new(0x19EF_00EA));
    }

    #[test]
    fn frame_build_1() {
        let frame = FrameBuilder::new(IdBuilder::from_pgn(PGN::Request).da(0x20).sa(0x10).build())
            .copy_from_slice(&[0x1, 0x2, 0x3])
            .build();

        assert_eq!(frame.id(), &Id::new(0x18EA_2010));
        assert_eq!(frame.pdu(), &[0x1, 0x2, 0x3]);
        assert_eq!(frame.len(), 3);
        assert!(!frame.is_empty());
    }

    #[test]
    fn frame_build_2() {
        let frame = FrameBuilder::new(
            IdBuilder::from_pgn(PGN::AddressClaimed)
                .priority(3)
                .sa(0x10)
                .build(),
        )
        .copy_from_slice(&[PDU_NOT_AVAILABLE; PDU_MAX_LENGTH])
        .build();

        assert_eq!(frame.id(), &Id::new(0x0CEE_0010));
        assert_eq!(frame.pdu(), &[PDU_NOT_AVAILABLE; PDU_MAX_LENGTH]);
        assert_eq!(frame.len(), PDU_MAX_LENGTH);
        assert!(!frame.is_empty());
    }

    #[test]
    fn frame_build_3() {
        let frame = FrameBuilder::new(IdBuilder::from_pgn(PGN::Transfer).build()).build();

        assert_eq!(frame.id(), &Id::new(0x18CA_0000));
        assert_eq!(frame.pdu(), &[]);
        assert_eq!(frame.len(), 0);
        assert!(frame.is_empty());
    }

    #[test]
    fn frame_build_4() {
        let mut frame_builder =
            FrameBuilder::default().id(IdBuilder::from_pgn(PGN::Transfer).build());

        frame_builder
            .as_mut()
            .copy_from_slice(&[0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]);

        frame_builder = frame_builder.set_len(8);

        let frame = frame_builder.build();

        assert_eq!(frame.id(), &Id::new(0x18CA_0000));
        assert_eq!(frame.pdu(), &[0x8, 0x7, 0x6, 0x5, 0x4, 0x3, 0x2, 0x1]);
        assert_eq!(frame.len(), PDU_MAX_LENGTH);
        assert!(!frame.is_empty());
    }

    #[test]
    fn frame_build_5() {
        let frame = FrameBuilder::default()
            .id(IdBuilder::from_pgn(PGN::ElectronicEngineController2).build())
            .set_len(8)
            .build();

        assert_eq!(frame.pdu(), &[PDU_NOT_AVAILABLE; PDU_MAX_LENGTH]);
        assert_eq!(frame.len(), PDU_MAX_LENGTH);
        assert!(!frame.is_empty());
    }

    #[test]
    fn pdu_format_boundary() {
        // Test PDU1/PDU2 boundary at PF=240 (0xF0)
        let pdu1_max = Id::new(0x0CEF_FF00); // PF=239 (0xEF) - should be PDU1
        let pdu2_min = Id::new(0x0CF0_0000); // PF=240 (0xF0) - should be PDU2

        assert!(matches!(pdu1_max.pdu_format(), PDUFormat::PDU1(_)));
        assert!(matches!(pdu2_min.pdu_format(), PDUFormat::PDU2(_)));

        // Verify destination address only on PDU1
        assert!(pdu1_max.destination_address().is_some());
        assert!(pdu2_min.destination_address().is_none());

        // Verify group extension only on PDU2
        assert!(pdu1_max.group_extension().is_none());
        assert!(pdu2_min.group_extension().is_some());
    }

    #[test]
    fn id_29bit_mask() {
        // Test that IDs are properly masked to 29 bits
        let over_29bits = 0xFFFF_FFFF;
        let id = Id::new(over_29bits);

        assert_eq!(id.as_raw(), 0x1FFF_FFFF); // Should be masked to 29 bits
        assert!(id.as_raw() <= 0x1FFF_FFFF);
    }

    #[test]
    fn priority_range() {
        // Test priority is clamped to 0-7
        let id = IdBuilder::from_pgn(PGN::Request)
            .priority(255) // Invalid priority
            .sa(0x10)
            .build();

        assert_eq!(id.priority(), 7); // Should be clamped to max value 7
    }

    #[test]
    fn frame_empty() {
        // Test empty frame handling
        let frame = FrameBuilder::new(Id::new(0)).build();

        assert!(frame.is_empty());
        assert_eq!(frame.len(), 0);
        assert_eq!(frame.pdu(), &[]);
    }

    #[test]
    fn frame_zero_length_slice() {
        // Test copying empty slice
        let frame = FrameBuilder::new(Id::new(0x18EA_0000))
            .copy_from_slice(&[])
            .build();

        assert!(frame.is_empty());
        assert_eq!(frame.len(), 0);
    }

    #[test]
    fn pgn_extraction_pdu1_zeros_ps() {
        // Test that PGN extraction zeros out PS byte for PDU1
        let id = Id::new(0x0CEA_5500); // PF=EA (234, PDU1), PS=55, SA=00

        // For PDU1, PS is destination address, not part of PGN
        assert_eq!(id.pgn_raw(), 0xEA00); // Should have PS=00
        assert!(id.pgn_raw().trailing_zeros() >= 8); // Low byte should be zero
    }

    #[test]
    fn pgn_extraction_pdu2_includes_ps() {
        // Test that PGN extraction includes PS byte for PDU2
        let id = Id::new(0x0CFE_6C00); // PF=FE (254, PDU2), PS=6C, SA=00

        // For PDU2, PS is group extension, part of PGN
        assert_eq!(id.pgn_raw(), 0xFE6C);
        assert!(id.pgn_raw() & 0xFF != 0); // Low byte should be non-zero
    }
}
