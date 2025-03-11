use binary_serde::recursive_array::RecursiveArraySingleItem;
use binary_serde::{binary_serde_bitfield, BinarySerde, BitfieldBitOrder};

use crate::CfuWriterError;

// Max is 7 components in CfuUpdateOfferResponse, 1 primary and 6 subcomponents
pub const MAX_CMPT_COUNT: usize = 7;
pub const MAX_SUBCMPT_COUNT: usize = 6;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of FwVersion
pub struct FwVersion {
    pub major: u8,
    pub minor: u16,
    pub variant: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of GetFwVersionResponse
pub struct GetFwVersionResponse {
    pub misc_and_protocol_version: u32,
    pub component_info: [FwVerComponentInfo; MAX_CMPT_COUNT],
    pub header: GetFwVersionResponseHeader,
}

const PROTOCOL_VER4: u8 = 0b0010;
#[derive(Copy, Clone, Debug, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation GetFwVersionResponseHeader
pub struct GetFwVersionResponseHeader {
    pub byte3: GetFwVerRespHeaderByte3,
    _reserved: u16,
    pub component_count: u8,
}

impl GetFwVersionResponseHeader {
    pub fn new(component_count: u8, byte3: GetFwVerRespHeaderByte3) -> Self {
        Self {
            component_count,
            _reserved: 0,
            byte3,
        }
    }
}

impl Default for GetFwVersionResponseHeader {
    fn default() -> Self {
        Self::new(1, GetFwVerRespHeaderByte3::NoSpecialFlags)
    }
}

#[derive(BinarySerde, Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum GetFwVerRespHeaderByte3 {
    #[default]
    NoSpecialFlags = PROTOCOL_VER4 << 4,
    ExtensionFlagSet = (PROTOCOL_VER4 << 4) | 1,
}

pub type ComponentId = u8;

#[derive(BinarySerde, Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum BankType {
    #[default]
    SingleBank = 1,
    DualBank = 2,
    TripleBank = 3,
    QuadBank = 4,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[binary_serde_bitfield(order = BitfieldBitOrder::MsbFirst)]
pub struct FwVerComponentInfo {
    #[bits(32)]
    pub fw_version: FwVersion, // u32
    #[bits(16)]
    pub vendor_specific: u16,
    #[bits(8)]
    pub component_id: ComponentId, // u8
    #[bits(4)]
    pub vendor_specific2: u8, // 4-bits
    #[bits(2)]
    reserved: u8,
    #[bits(2)]
    pub bank: BankType,
}

impl FwVerComponentInfo {
    pub fn new(fw_version: FwVersion, component_id: ComponentId, bank: BankType) -> Self {
        Self {
            fw_version,
            vendor_specific: 0,
            component_id,
            vendor_specific2: 0,
            reserved: 0,
            bank,
        }
    }
    pub fn new_with_vendor_specific_info(
        fw_version: FwVersion,
        component_id: ComponentId,
        bank: BankType,
        vendor_specific: u16,
        vendor_specific2: u8,
    ) -> Self {
        Self {
            fw_version,
            vendor_specific,
            component_id,
            vendor_specific2,
            reserved: 0,
            bank,
        }
    }
}

impl Default for FwVerComponentInfo {
    fn default() -> Self {
        Self::new(FwVersion::default(), 0, BankType::SingleBank)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of FwUpdateOfferCommand
pub struct FwUpdateOfferCommand {
    pub misc_and_protocol_version: u32, // u32
    pub vendor_specific: u32,
    pub firmware_version: FwVersion,              // u32
    pub component_info: UpdateOfferComponentInfo, // u32
}

impl FwUpdateOfferCommand {
    pub fn new(
        token: HostToken,
        component_id: ComponentId,
        firmware_version: FwVersion,
        vendor_specific: u32,
        misc: u32,
    ) -> Self {
        Self {
            component_info: UpdateOfferComponentInfo::new(token, component_id),
            firmware_version,
            vendor_specific,
            misc_and_protocol_version: misc,
        }
    }
    pub fn new_with_command(
        token: HostToken,
        component_id: ComponentId,
        firmware_version: FwVersion,
        vendor_specific: u32,
        command: InformationCodeValues,
        misc: u32,
    ) -> Self {
        Self {
            component_info: UpdateOfferComponentInfo::new_with_command(token, component_id, command),
            firmware_version,
            vendor_specific,
            misc_and_protocol_version: misc,
        }
    }
}

impl Default for FwUpdateOfferCommand {
    fn default() -> Self {
        Self::new(0, 0, FwVersion::default(), 0, 0)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of UpdateOfferComponentInfo
pub struct UpdateOfferComponentInfo {
    pub token: HostToken,
    pub component_id: ComponentId,
    pub byte1: UpdateOfferComponentInfoByte1,
    pub byte0: u8,
}

impl UpdateOfferComponentInfo {
    pub fn new(token: HostToken, component_id: ComponentId) -> Self {
        Self {
            token,
            component_id,
            byte1: UpdateOfferComponentInfoByte1::default(),
            byte0: UpdateOfferComponentInfoByte0::default().into(),
        }
    }

    pub fn new_with_command(token: HostToken, component_id: ComponentId, code: InformationCodeValues) -> Self {
        Self {
            token,
            component_id,
            byte1: UpdateOfferComponentInfoByte1::default(),
            byte0: UpdateOfferComponentInfoByte0::CommandCode(code).into(),
        }
    }
}

impl Default for UpdateOfferComponentInfo {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[binary_serde_bitfield(order = BitfieldBitOrder::MsbFirst)]
pub struct UpdateOfferComponentInfoByte1 {
    #[bits(1)]
    pub force_ignore_version: u8,
    #[bits(1)]
    pub force_reset: u8,
    #[bits(6)]
    pub reserved_byte1: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum UpdateOfferComponentInfoByte0 {
    /// default usage
    Segment(u8),
    /// Used for Extended Component Information packets
    CommandCode(InformationCodeValues),
}

impl Default for UpdateOfferComponentInfoByte0 {
    fn default() -> Self {
        Self::Segment(0)
    }
}

impl From<UpdateOfferComponentInfoByte0> for u8 {
    fn from(value: UpdateOfferComponentInfoByte0) -> Self {
        match value {
            UpdateOfferComponentInfoByte0::CommandCode(infcode) => u8::from(infcode),
            UpdateOfferComponentInfoByte0::Segment(num) => num,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum InformationCodeValues {
    StartEntireTransaction = 0x00,
    StartOfferList = 0x01,
    EndOfferList = 0x02,
    Extended(ExtendedCommandCode),
}

impl From<InformationCodeValues> for u8 {
    fn from(value: InformationCodeValues) -> Self {
        match value {
            InformationCodeValues::StartEntireTransaction => 0x00,
            InformationCodeValues::StartOfferList => 0x01,
            InformationCodeValues::EndOfferList => 0x02,
            InformationCodeValues::Extended(ExtendedCommandCode::OfferNotifyOnReady) => 0x01,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of FwUpdateOfferResponse
pub struct FwUpdateOfferResponse {
    _reserved: [u8; 3],                 // bytes 13-15
    pub status: CfuOfferStatus,         //byte 12
    _reserved2: [u8; 3],                // bytes 9-11
    pub rejectreasoncode: RejectReason, //byte8
    _reserved3: [u8; 4],                // bytes 4-7
    pub token: HostToken,               // byte3
    _reserved4: [u8; 3],                // bytes 0-2
}

impl FwUpdateOfferResponse {
    pub fn new_accept(token: HostToken) -> Self {
        Self {
            token,
            rejectreasoncode: RejectReason::OfferSwapPending, // not used for success cases
            status: CfuOfferStatus::Accept,
            _reserved: [0; 3],
            _reserved2: [0; 3],
            _reserved3: [0; 4],
            _reserved4: [0; 3],
        }
    }
    pub fn new_with_failure(token: HostToken, rejectreasoncode: RejectReason, status: CfuOfferStatus) -> Self {
        Self {
            token,
            rejectreasoncode,
            status,
            _reserved: [0; 3],
            _reserved2: [0; 3],
            _reserved3: [0; 4],
            _reserved4: [0; 3],
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum RejectReason {
    #[default]
    RejectOldFw = 0x00,
    RejectInvComponent = 0x01,
    OfferSwapPending = 0x02,
    VendorSpecific(u8),
}

impl BinarySerde for RejectReason {
    const SERIALIZED_SIZE: usize = u8::SERIALIZED_SIZE;
    type RecursiveArray = RecursiveArraySingleItem<u8>;
    fn binary_serialize(&self, buffer: &mut [u8], _endianness: binary_serde::Endianness) {
        buffer[0] = (*self).into();
    }

    fn binary_deserialize(
        buffer: &[u8],
        _endianness: binary_serde::Endianness,
    ) -> Result<Self, binary_serde::DeserializeError> {
        Ok(match buffer[0] {
            0x00 => RejectReason::RejectOldFw,
            0x01 => RejectReason::RejectInvComponent,
            0x02 => RejectReason::OfferSwapPending,
            val if val >= 0xEF => RejectReason::VendorSpecific(val),
            _ => {
                return Err(binary_serde::DeserializeError::InvalidEnumValue {
                    enum_name: "RejectReason",
                })
            }
        })
    }
}

impl From<RejectReason> for u8 {
    fn from(value: RejectReason) -> Self {
        match value {
            RejectReason::RejectOldFw => 0x00,
            RejectReason::RejectInvComponent => 0x01,
            RejectReason::OfferSwapPending => 0x02,
            // TODO limit this to 0xE0 to 0xFF only
            RejectReason::VendorSpecific(val) => val,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CfuCommands {
    OfferInfoStartEntireTransaction,
    OfferInfoStartStartOfferList,
    FwUpdateOffer,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ExtendedCommandCode {
    #[default]
    OfferNotifyOnReady = 0x01,
}

pub type HostToken = u8;
#[derive(Copy, Clone, Debug, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of ExtendedCommandPacket
pub struct ExtendedCommandPacketResponse {
    _reserved: [u8; 3],              // bytes 13-15
    pub status: CfuOfferStatus,      // byte 12
    _reserved2: [u8; 3],             // bytes 9-11
    pub reject_reason: RejectReason, // byte 8
    _reserved3: [u8; 4],             // bytes 4-7
    pub token: HostToken,            // byte 3
    _reserved4: [u8; 3],             // bytes 0-2
}

impl ExtendedCommandPacketResponse {
    pub fn new(status: CfuOfferStatus, reject_reason: RejectReason, token: HostToken) -> Self {
        Self {
            status,
            reject_reason,
            token,
            _reserved: [0; 3],
            _reserved2: [0; 3],
            _reserved3: [0; 4],
            _reserved4: [0; 3],
        }
    }
}

#[derive(BinarySerde, Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CfuOfferStatus {
    #[default]
    /// Component has decided to skip the offer, host must offer it again later
    Skip = 0x00,
    /// Component has accepted the offer
    Accept = 0x01,
    /// Component has rejected the offer
    Reject = 0x02,
    /// Component is busy, host must wait until the component is ready
    Busy = 0x03,
    /// Used when ComponentId is 0xFE
    Command = 0x04,
    /// Offer reqyest isn't recognized
    CmdNotSupported = 0xFF,
}

#[derive(BinarySerde, Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CfuOfferResponseStatus {
    #[default]
    /// Request completed successfully
    Success = 0x00,
    /// Component wasn't prepared to receive contents
    /// typically used in response to the first block
    ErrorPrepare = 0x01,
    /// Request couldn't write the block
    ErrorWrite = 0x02,
    /// Request couldn't setup the swap, in response to the last block flag
    ErrorComplete = 0x03,
    /// Verification of the DWORD failed, in repsonse to the verify flag
    ErrorVerify = 0x04,
    /// CRC of the image failed, in reponse to the last block flag
    ErrorCrc = 0x05,
    /// Signature of the image failed, in response to the last block flag
    ErrorSignature = 0x06,
    /// Version verification of the image failed, in response to the last block flag
    ErrorVersion = 0x07,
    /// Swap is pending, no further update commands can be accepted
    SwapPending = 0x08,
    /// Invalid destination address for the content
    ErrorInvalidAddr = 0x09,
    /// Content was received without accepting a valid offer
    ErrorNoOffer = 0x0A,
    /// General error for update offer command
    ErrorInvalid = 0x0B,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of FwUpdateContentCommand
pub struct FwUpdateContentCommand {
    pub data: [u8; DEFAULT_DATA_LENGTH],
    pub header: FwUpdateContentHeader,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of FwUpdateContentHeader
pub struct FwUpdateContentHeader {
    pub firmware_address: u32,
    pub sequence_num: u16,
    pub data_length: u8,
    pub flags: FwUpdateFlags,
}

#[derive(BinarySerde, Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum FwUpdateFlags {
    #[default]
    FirstBlock,
    LastBlock,
    FirstAndLastBlock,
    None,
}
impl From<FwUpdateFlags> for u8 {
    fn from(value: FwUpdateFlags) -> Self {
        match value {
            FwUpdateFlags::FirstBlock => 0x80,
            FwUpdateFlags::LastBlock => 0x40,
            FwUpdateFlags::FirstAndLastBlock => 0x80 | 0x40,
            FwUpdateFlags::None => 0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, BinarySerde)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// MSB first Representation of FwUpdateContentResponse
pub struct FwUpdateContentResponse {
    _reserved2: [u8; 11],
    pub status: CfuOfferResponseStatus,
    _reserved1: u16,
    pub sequence: u16,
}

impl FwUpdateContentResponse {
    pub fn new(sequence: u16, status: CfuOfferResponseStatus) -> Self {
        Self {
            sequence,
            status,
            _reserved1: 0,
            _reserved2: [0; 11],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum CfuProtocolError {
    /// Error with specific component
    UpdateError(u8),
    /// Error with specific component
    TimeoutError(u8),
    /// Invalid Block transition
    InvalidBlockTransition,
    /// Bad Response
    BadResponse,
    /// WriterError
    WriterError(CfuWriterError),
    /// ResponseError
    CfuResponseError(CfuOfferResponseStatus),
    /// StatusError
    CfuStatusError(CfuOfferStatus),
}

pub const DEFAULT_DATA_LENGTH: usize = 52; // bytes 8-59 are data bytes (52 total)
