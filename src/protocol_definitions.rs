use core::convert::TryFrom;

use crate::writer::CfuWriterError;

// Max is 7 components in CfuUpdateOfferResponse, 1 primary and 6 subcomponents
pub const MAX_CMPT_COUNT: usize = 7;
pub const MAX_SUBCMPT_COUNT: usize = 6;

// Error types related to marshalling
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ConversionError {
    ByteConversionError,
    ValueOutOfRange,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwVersion
pub struct FwVersion {
    pub variant: u8,
    pub minor: u16,
    pub major: u8,
}

// Explicit conversions for FwVersion and u32
impl FwVersion {
    pub fn new(fw_version: u32) -> Self {
        Self {
            variant: (fw_version & 0xFF) as u8,
            minor: ((fw_version >> 8) & 0xFFFF) as u16,
            major: ((fw_version >> 24) & 0xFF) as u8,
        }
    }
}

impl From<FwVersion> for u32 {
    fn from(ver: FwVersion) -> Self {
        ((ver.major as u32) << 24) | ((ver.minor as u32) << 8) | ver.variant as u32
    }
}

impl From<u32> for FwVersion {
    fn from(ver: u32) -> Self {
        Self {
            variant: (ver & 0xFF) as u8,
            minor: ((ver >> 8) & 0xFFFF) as u16,
            major: ((ver >> 24) & 0xFF) as u8,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of GetFwVersionResponse
pub struct GetFwVersionResponse {
    pub header: GetFwVersionResponseHeader,
    pub component_info: [FwVerComponentInfo; MAX_CMPT_COUNT],
}

// CFU protocol spec at ver 2.0
const PROTOCOL_VER: u8 = 0b0010;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of GetFwVersionResponseHeader
pub struct GetFwVersionResponseHeader {
    pub component_count: u8,
    _reserved: u16,
    pub byte3: GetFwVerRespHeaderByte3,
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum GetFwVerRespHeaderByte3 {
    #[default]
    NoSpecialFlags = PROTOCOL_VER << 4,
    ExtensionFlagSet = (PROTOCOL_VER << 4) | 1,
}

pub type ComponentId = u8;
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SpecialComponentIds {
    /// Special Component ID in the Component Information bytes for Offer Command Extended.
    Command = 0xFE,
    /// Special Component ID in the Component Information bytes for Offer Information.
    Info = 0xFF,
}

// Conversion from u8 to SpecialComponentIds
impl TryFrom<u8> for SpecialComponentIds {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xFE => Ok(SpecialComponentIds::Command),
            0xFF => Ok(SpecialComponentIds::Info),
            _ => Err(ConversionError::ValueOutOfRange),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum BankType {
    /// Optional Bank Type for Vendor Specific use.
    VendorSpecific(u8),
}

impl From<BankType> for u8 {
    fn from(bank_type: BankType) -> Self {
        match bank_type {
            BankType::VendorSpecific(val) => val,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwVerComponentInfo
pub struct FwVerComponentInfo {
    pub fw_version: FwVersion,     // u32
    pub packed_byte: u8,           // u8, bits 0-1 for bank, bits 2-3 for reserved, bits 4-7 for vendor_specific0
    pub component_id: ComponentId, // u8
    pub vendor_specific1: u16,     // u16
}

impl FwVerComponentInfo {
    pub fn new(fw_version: FwVersion, component_id: ComponentId) -> Self {
        Self {
            fw_version,
            packed_byte: 0,
            component_id,
            vendor_specific1: 0,
        }
    }
    pub fn new_with_vendor_specific_info(
        fw_version: FwVersion,
        component_id: ComponentId,
        bank: BankType,
        vendor_specific0: u8,
        vendor_specific1: u16,
    ) -> Self {
        let bank: u8 = bank.into();
        let packed_byte = (bank & 0x3) | ((vendor_specific0 & 0xF) << 4); // Bits 0-1 and 4-7
        Self {
            fw_version,
            packed_byte,
            component_id,
            vendor_specific1,
        }
    }
}

impl Default for FwVerComponentInfo {
    fn default() -> Self {
        Self::new(FwVersion::default(), 0)
    }
}

// Convert to bytes
impl TryFrom<&GetFwVersionResponse> for [u8; 60] {
    type Error = ConversionError;

    fn try_from(response: &GetFwVersionResponse) -> Result<Self, Self::Error> {
        let mut bytes = [0u8; 60];

        // Serialize header
        bytes[0] = response.header.component_count;
        bytes[1..3].copy_from_slice(&response.header._reserved.to_le_bytes());
        bytes[3] = response.header.byte3 as u8;

        // Serialize component_info
        let mut offset = 4;
        for i in 0..response.header.component_count as usize {
            let component = &response
                .component_info
                .get(i)
                .ok_or(ConversionError::ByteConversionError)?;
            *bytes.get_mut(offset).ok_or(ConversionError::ByteConversionError)? = component.packed_byte;
            *bytes.get_mut(offset + 1).ok_or(ConversionError::ByteConversionError)? = component.component_id;
            bytes
                .get_mut(offset + 2..offset + 4)
                .ok_or(ConversionError::ByteConversionError)?
                .copy_from_slice(&component.vendor_specific1.to_le_bytes());
            *bytes.get_mut(offset + 4).ok_or(ConversionError::ByteConversionError)? = component.fw_version.major;
            bytes
                .get_mut(offset + 5..offset + 7)
                .ok_or(ConversionError::ByteConversionError)?
                .copy_from_slice(&component.fw_version.minor.to_le_bytes());
            *bytes.get_mut(offset + 7).ok_or(ConversionError::ByteConversionError)? = component.fw_version.variant;
            offset += 8;
        }

        Ok(bytes)
    }
}

// Convert from bytes
impl TryFrom<&[u8; 60]> for GetFwVersionResponse {
    type Error = ConversionError;

    #[allow(clippy::indexing_slicing)] // static_check and fixed size array guarantees indexing is safe
    fn try_from(bytes: &[u8; 60]) -> Result<Self, Self::Error> {
        const _: () = assert!(MAX_CMPT_COUNT * 8 + 4 <= 60, "Component count exceeds maximum allowed");

        let component_count = bytes[0];

        if component_count as usize > MAX_CMPT_COUNT {
            return Err(ConversionError::ValueOutOfRange);
        }

        let _reserved = u16::from_le_bytes(
            bytes[1..3]
                .try_into()
                .map_err(|_| ConversionError::ByteConversionError)?,
        );
        let byte3 = match bytes[3] {
            0x20 => GetFwVerRespHeaderByte3::NoSpecialFlags,
            0x21 => GetFwVerRespHeaderByte3::ExtensionFlagSet,
            _ => return Err(ConversionError::ValueOutOfRange),
        };

        let mut component_info = [FwVerComponentInfo::default(); MAX_CMPT_COUNT];
        let mut offset = 4;
        for component in component_info.iter_mut().take(component_count as usize) {
            component.packed_byte = bytes[offset];
            component.component_id = bytes[offset + 1];
            component.vendor_specific1 = u16::from_le_bytes(
                bytes[offset + 2..offset + 4]
                    .try_into()
                    .map_err(|_| ConversionError::ByteConversionError)?,
            );
            component.fw_version.major = bytes[offset + 4];
            component.fw_version.minor = u16::from_le_bytes(
                bytes[offset + 5..offset + 7]
                    .try_into()
                    .map_err(|_| ConversionError::ByteConversionError)?,
            );
            component.fw_version.variant = bytes[offset + 7];
            offset += 8;
        }

        Ok(GetFwVersionResponse {
            header: GetFwVersionResponseHeader {
                component_count,
                _reserved,
                byte3,
            },
            component_info,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwUpdateOffer
pub struct FwUpdateOffer {
    pub component_info: UpdateOfferComponentInfo, // u32
    pub firmware_version: FwVersion,              // u32
    pub vendor_specific: u32,                     // u32
    pub misc_and_protocol_version: u32,           // u32
}

impl FwUpdateOffer {
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
}

impl Default for FwUpdateOffer {
    fn default() -> Self {
        Self::new(HostToken::Driver, 0, FwVersion::default(), 0, 0)
    }
}

// Convert to bytes
impl From<&FwUpdateOffer> for [u8; 32] {
    fn from(command: &FwUpdateOffer) -> Self {
        let mut bytes = [0u8; 32];

        // Serialize component_info
        bytes[0] = command.component_info.segment_number;
        bytes[1] = command.component_info.byte1.packed_byte;
        bytes[2] = command.component_info.component_id;
        bytes[3] = command.component_info.token.into();

        // Serialize firmware_version
        bytes[7] = command.firmware_version.major;
        bytes[5..7].copy_from_slice(&command.firmware_version.minor.to_le_bytes());
        bytes[4] = command.firmware_version.variant;

        // Serialize vendor_specific
        bytes[8..12].copy_from_slice(&command.vendor_specific.to_le_bytes());

        // Serialize misc_and_protocol_version
        bytes[12..16].copy_from_slice(&command.misc_and_protocol_version.to_le_bytes());

        bytes
    }
}

// Convert from bytes
impl TryFrom<&[u8; 32]> for FwUpdateOffer {
    type Error = ConversionError;

    fn try_from(bytes: &[u8; 32]) -> Result<Self, Self::Error> {
        let component_info = UpdateOfferComponentInfo {
            segment_number: bytes[0],
            byte1: UpdateOfferComponentInfoByte1 { packed_byte: bytes[1] },
            component_id: bytes[2],
            token: HostToken::try_from(bytes[3]).map_err(|_| ConversionError::ByteConversionError)?,
        };

        let firmware_version = FwVersion {
            major: bytes[7],
            minor: u16::from_le_bytes(
                bytes[5..7]
                    .try_into()
                    .map_err(|_| ConversionError::ByteConversionError)?,
            ),
            variant: bytes[4],
        };

        let vendor_specific = u32::from_le_bytes(
            bytes[8..12]
                .try_into()
                .map_err(|_| ConversionError::ByteConversionError)?,
        );
        let misc_and_protocol_version = u32::from_le_bytes(
            bytes[12..16]
                .try_into()
                .map_err(|_| ConversionError::ByteConversionError)?,
        );

        Ok(FwUpdateOffer {
            component_info,
            firmware_version,
            vendor_specific,
            misc_and_protocol_version,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of UpdateOfferComponentInfo
pub struct UpdateOfferComponentInfo {
    pub segment_number: u8,
    pub byte1: UpdateOfferComponentInfoByte1,
    pub component_id: ComponentId,
    pub token: HostToken,
}

impl UpdateOfferComponentInfo {
    pub fn new(token: HostToken, component_id: ComponentId) -> Self {
        Self {
            segment_number: u8::default(),
            byte1: UpdateOfferComponentInfoByte1::default(),
            component_id,
            token,
        }
    }
}

impl Default for UpdateOfferComponentInfo {
    fn default() -> Self {
        Self::new(HostToken::Driver, 0)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UpdateOfferComponentInfoByte1 {
    pub packed_byte: u8, // 8-bits: 6 bits for reserved, 1 bit for force_reset, 1 bit for force_ignore_version
}
impl UpdateOfferComponentInfoByte1 {
    pub fn new(force_ignore_version: bool, force_reset: bool) -> Self {
        let mut packed_byte = 0;
        packed_byte |= ((force_ignore_version as u8) << 7) & 0b10000000; // Bit 7
        packed_byte |= ((force_reset as u8) << 6) & 0b01000000; // Bit 6
        Self { packed_byte }
    }

    pub fn force_ignore_version(&self) -> bool {
        (self.packed_byte & 0b10000000) != 0
    }

    pub fn force_reset(&self) -> bool {
        (self.packed_byte & 0b01000000) != 0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of OfferInformationComponentInfo
pub struct OfferInformationComponentInfo {
    pub code: OfferInformationCodeValues,
    _reserved: u8,
    pub component_id: SpecialComponentIds,
    pub token: HostToken,
}

impl OfferInformationComponentInfo {
    pub fn new(token: HostToken, component_id: SpecialComponentIds, code: OfferInformationCodeValues) -> Self {
        Self {
            code,
            _reserved: 0,
            component_id,
            token,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwUpdateOfferInformation
pub struct FwUpdateOfferInformation {
    pub component_info: OfferInformationComponentInfo,
    _reserved0: u32,
    _reserved1: u32,
    _reserved2: u32,
}

impl FwUpdateOfferInformation {
    pub fn new(component_info: OfferInformationComponentInfo) -> Self {
        Self {
            component_info,
            _reserved0: 0,
            _reserved1: 0,
            _reserved2: 0,
        }
    }
}

// Convert to bytes
impl From<&FwUpdateOfferInformation> for [u8; 16] {
    fn from(info: &FwUpdateOfferInformation) -> Self {
        let mut bytes = [0u8; 16];

        // Serialize the component_info
        bytes[0..4].copy_from_slice(&[
            info.component_info.code.into(),
            0, // component_info._reserved is reserved
            info.component_info.component_id as u8,
            info.component_info.token.into(),
        ]);

        // Initialize [4..16] with 0
        bytes[4..8].copy_from_slice(&[0; 4]); // info._reserved0 is reserved
        bytes[8..12].copy_from_slice(&[0; 4]); // info._reserved1 is reserved
        bytes[12..16].copy_from_slice(&[0; 4]); // info._reserved2 is reserved

        bytes
    }
}

// Convert from bytes
impl TryFrom<&[u8; 16]> for FwUpdateOfferInformation {
    type Error = ConversionError;

    fn try_from(bytes: &[u8; 16]) -> Result<Self, Self::Error> {
        let code = match bytes[0] {
            0x00 => OfferInformationCodeValues::StartEntireTransaction,
            0x01 => OfferInformationCodeValues::StartOfferList,
            0x02 => OfferInformationCodeValues::EndOfferList,
            _ => return Err(ConversionError::ValueOutOfRange),
        };

        let reserved = 0; // bytes[1] is reserved
        let component_id = SpecialComponentIds::try_from(bytes[2]).map_err(|_| ConversionError::ValueOutOfRange)?;
        if component_id != SpecialComponentIds::Info {
            return Err(ConversionError::ValueOutOfRange);
        }
        let token = HostToken::try_from(bytes[3])?;

        Ok(FwUpdateOfferInformation {
            component_info: OfferInformationComponentInfo {
                code,
                _reserved: reserved,
                component_id,
                token,
            },
            _reserved0: 0,
            _reserved1: 0,
            _reserved2: 0,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OfferInformationCodeValues {
    StartEntireTransaction = 0x00,
    StartOfferList = 0x01,
    EndOfferList = 0x02,
    // Vendor specific extensions
    VendorSpecific(u8),
}

impl From<OfferInformationCodeValues> for u8 {
    fn from(value: OfferInformationCodeValues) -> Self {
        match value {
            OfferInformationCodeValues::StartEntireTransaction => 0x00,
            OfferInformationCodeValues::StartOfferList => 0x01,
            OfferInformationCodeValues::EndOfferList => 0x02,
            OfferInformationCodeValues::VendorSpecific(val) => val,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwUpdateOfferExtended
pub struct OfferExtendedComponentInfo {
    pub code: OfferCommandExtendedCodeValues,
    _reserved: u8,
    pub component_id: SpecialComponentIds,
    pub token: HostToken,
}

impl OfferExtendedComponentInfo {
    pub fn new(token: HostToken, component_id: SpecialComponentIds, code: OfferCommandExtendedCodeValues) -> Self {
        Self {
            code,
            _reserved: 0,
            component_id,
            token,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwUpdateOfferExtended
pub struct FwUpdateOfferExtended {
    pub component_info: OfferExtendedComponentInfo,
    _reserved0: u32,
    _reserved1: u32,
    _reserved2: u32,
}

impl FwUpdateOfferExtended {
    pub fn new(component_info: OfferExtendedComponentInfo) -> Self {
        Self {
            component_info,
            _reserved0: 0,
            _reserved1: 0,
            _reserved2: 0,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OfferCommandExtendedCodeValues {
    #[default]
    OfferNotifyOnReady = 0x01,
    // Vendor specific extensions
    VendorSpecific(u8),
}

// Convert to bytes
impl From<OfferCommandExtendedCodeValues> for u8 {
    fn from(value: OfferCommandExtendedCodeValues) -> Self {
        match value {
            OfferCommandExtendedCodeValues::OfferNotifyOnReady => 0x01,
            OfferCommandExtendedCodeValues::VendorSpecific(val) => val,
        }
    }
}

// Convert from bytes
impl From<u8> for OfferCommandExtendedCodeValues {
    fn from(value: u8) -> Self {
        match value {
            0x01 => OfferCommandExtendedCodeValues::OfferNotifyOnReady,
            val => OfferCommandExtendedCodeValues::VendorSpecific(val),
        }
    }
}

// Convert to bytes
impl From<&FwUpdateOfferExtended> for [u8; 16] {
    fn from(command: &FwUpdateOfferExtended) -> Self {
        let mut bytes = [0u8; 16];
        // Serialize the component_info
        bytes[0..4].copy_from_slice(&[
            command.component_info.code.into(),
            0, // component_info._reserved is reserved
            command.component_info.component_id as u8,
            command.component_info.token.into(),
        ]);

        // Initialize [4..16] with 0
        bytes[4..8].copy_from_slice(&[0; 4]); // command._reserved0 is reserved
        bytes[8..12].copy_from_slice(&[0; 4]); // command._reserved1 is reserved
        bytes[12..16].copy_from_slice(&[0; 4]); // command._reserved2 is reserved

        bytes
    }
}

// Convert from bytes
impl TryFrom<&[u8; 16]> for FwUpdateOfferExtended {
    type Error = ConversionError;

    fn try_from(bytes: &[u8; 16]) -> Result<Self, Self::Error> {
        let code = OfferCommandExtendedCodeValues::from(bytes[0]);
        let reserved = 0; // bytes[1] is reserved
        let component_id = SpecialComponentIds::try_from(bytes[2]).map_err(|_| ConversionError::ValueOutOfRange)?;
        if component_id != SpecialComponentIds::Command {
            return Err(ConversionError::ValueOutOfRange);
        }
        let token = HostToken::try_from(bytes[3])?;

        Ok(FwUpdateOfferExtended {
            component_info: OfferExtendedComponentInfo {
                code,
                _reserved: reserved,
                component_id,
                token,
            },

            _reserved0: 0, // bytes[4..8] is reserved
            _reserved1: 0, // bytes[8..12] is reserved
            _reserved2: 0, // bytes[12..16] is reserved
        })
    }
}

pub const DEFAULT_DATA_LENGTH: usize = 52; // bytes 8-59 are data bytes (52 total)

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwUpdateContentCommand
pub struct FwUpdateContentCommand {
    pub header: FwUpdateContentHeader,
    pub data: [u8; DEFAULT_DATA_LENGTH],
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwUpdateContentHeader
pub struct FwUpdateContentHeader {
    pub flags: u8,
    pub data_length: u8,
    pub sequence_num: u16,
    pub firmware_address: u32,
}

// Convert to bytes
impl From<&FwUpdateContentCommand> for [u8; 60] {
    fn from(command: &FwUpdateContentCommand) -> Self {
        let mut bytes = [0u8; 60];

        // Serialize header
        bytes[0] = command.header.flags;
        bytes[1] = command.header.data_length;
        bytes[2..4].copy_from_slice(&command.header.sequence_num.to_le_bytes());
        bytes[4..8].copy_from_slice(&command.header.firmware_address.to_le_bytes());

        // Serialize data
        bytes[8..].copy_from_slice(&command.data);

        bytes
    }
}

// Convert from bytes
impl TryFrom<&[u8; 60]> for FwUpdateContentCommand {
    type Error = ConversionError;

    fn try_from(bytes: &[u8; 60]) -> Result<Self, Self::Error> {
        let flags = bytes[0];
        let data_length = bytes[1];
        let sequence_num = u16::from_le_bytes(
            bytes[2..4]
                .try_into()
                .map_err(|_| ConversionError::ByteConversionError)?,
        );
        let firmware_address = u32::from_le_bytes(
            bytes[4..8]
                .try_into()
                .map_err(|_| ConversionError::ByteConversionError)?,
        );

        let mut data = [0u8; DEFAULT_DATA_LENGTH];
        data.copy_from_slice(&bytes[8..]);

        Ok(FwUpdateContentCommand {
            header: FwUpdateContentHeader {
                flags,
                data_length,
                sequence_num,
                firmware_address,
            },
            data,
        })
    }
}

pub const FW_UPDATE_FLAG_FIRST_BLOCK: u8 = 0x80;
pub const FW_UPDATE_FLAG_LAST_BLOCK: u8 = 0x40;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HostToken {
    #[default]
    Driver = 0xA0,
    Tool = 0xB0,
    // Allow Vendor Specific values
    VendorSpecific(u8),
}

// Convert to byte
impl From<HostToken> for u8 {
    fn from(token: HostToken) -> Self {
        match token {
            HostToken::Driver => 0xA0,
            HostToken::Tool => 0xB0,
            HostToken::VendorSpecific(val) => val,
        }
    }
}

// Convert from byte
impl TryFrom<u8> for HostToken {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xA0 => Ok(HostToken::Driver),
            0xB0 => Ok(HostToken::Tool),
            val => Ok(HostToken::VendorSpecific(val)),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwUpdateOfferResponse
pub struct FwUpdateOfferResponse {
    _reserved0: [u8; 3],                  // bytes 0-2
    pub token: HostToken,                 // byte 3
    _reserved1: [u8; 4],                  // bytes 4-7
    pub reject_reason: OfferRejectReason, // byte 8
    _reserved2: [u8; 3],                  // bytes 9-11
    pub status: OfferStatus,              // byte 12
    _reserved3: [u8; 3],                  // bytes 13-15
}

impl FwUpdateOfferResponse {
    pub fn new_accept(token: HostToken) -> Self {
        Self {
            token,
            reject_reason: OfferRejectReason::SwapPending, // not used for success cases
            status: OfferStatus::Accept,
            _reserved0: [0; 3],
            _reserved1: [0; 4],
            _reserved2: [0; 3],
            _reserved3: [0; 3],
        }
    }

    pub fn new_with_failure(token: HostToken, reject_reason: OfferRejectReason, status: OfferStatus) -> Self {
        Self {
            token,
            reject_reason,
            status,
            _reserved0: [0; 3],
            _reserved1: [0; 4],
            _reserved2: [0; 3],
            _reserved3: [0; 3],
        }
    }
}

// Convert to bytes
impl From<&FwUpdateOfferResponse> for [u8; 16] {
    fn from(response: &FwUpdateOfferResponse) -> Self {
        let mut buffer = [0u8; 16];

        // Initialize fields 0..3, 4..8, 9..12, 13..16 to 0.
        buffer[0..3].copy_from_slice(&[0; 3]); // response._reserved0 is reserved
        buffer[3] = response.token.into();
        buffer[4..8].copy_from_slice(&[0; 4]); // response._reserved1 is reserved
        buffer[8] = response.reject_reason.into();
        buffer[9..12].copy_from_slice(&[0; 3]); // response._reserved2 is reserved
        buffer[12] = response.status.into();
        buffer[13..16].copy_from_slice(&[0; 3]); // response._reserved3 is reserved
        buffer
    }
}

// Convert from bytes
impl TryFrom<[u8; 16]> for FwUpdateOfferResponse {
    type Error = ConversionError;

    fn try_from(buffer: [u8; 16]) -> Result<Self, Self::Error> {
        Ok(Self {
            token: HostToken::try_from(buffer[3]).map_err(|_| ConversionError::ByteConversionError)?,
            reject_reason: OfferRejectReason::try_from(buffer[8]).map_err(|_| ConversionError::ByteConversionError)?,
            status: OfferStatus::try_from(buffer[12]).map_err(|_| ConversionError::ByteConversionError)?,
            _reserved0: [0; 3],
            _reserved1: [0; 4],
            _reserved2: [0; 3],
            _reserved3: [0; 3],
        })
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OfferRejectReason {
    #[default]
    /// The Offer was rejected because the Major and Minor Version is not newer than the current image.
    OldFw = 0x00,
    /// The Offer was rejected due a mismatch of Component ID.
    InvalidComponent = 0x01,
    /// The Offer was rejected because a previous update has been downloaded but not yet applied.
    SwapPending = 0x02,
    /// Vendor specific extesions
    VendorSpecific(u8),
}

// Convert to byte
impl From<OfferRejectReason> for u8 {
    fn from(value: OfferRejectReason) -> Self {
        match value {
            OfferRejectReason::OldFw => 0x00,
            OfferRejectReason::InvalidComponent => 0x01,
            OfferRejectReason::SwapPending => 0x02,
            OfferRejectReason::VendorSpecific(val) => val,
        }
    }
}

// Convert from byte
impl TryFrom<u8> for OfferRejectReason {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(OfferRejectReason::OldFw),
            0x01 => Ok(OfferRejectReason::InvalidComponent),
            0x02 => Ok(OfferRejectReason::SwapPending),
            val if val >= 0xE0 => Ok(OfferRejectReason::VendorSpecific(val)),
            _ => Err(ConversionError::ValueOutOfRange),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum OfferStatus {
    #[default]
    /// Component has decided to skip the offer, host must offer it again later
    Skip = 0x00,
    /// Component has accepted the offer
    Accept = 0x01,
    /// Component has rejected the offer
    Reject = 0x02,
    /// Component is busy, host must wait until the component is ready
    Busy = 0x03,
    /// Issued after receipt of OFFER_NOTIFY_ON_READY request when the Primary Component is ready to accept Offers.
    CommandReady = 0x04,
    /// Not supported command
    CmdNotSupported = 0xFF,
}

// Convert to byte
impl From<OfferStatus> for u8 {
    fn from(value: OfferStatus) -> Self {
        match value {
            OfferStatus::Skip => 0x00,
            OfferStatus::Accept => 0x01,
            OfferStatus::Reject => 0x02,
            OfferStatus::Busy => 0x03,
            OfferStatus::CommandReady => 0x04,
            OfferStatus::CmdNotSupported => 0xFF,
        }
    }
}

// Convert from byte
impl TryFrom<u8> for OfferStatus {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(OfferStatus::Skip),
            0x01 => Ok(OfferStatus::Accept),
            0x02 => Ok(OfferStatus::Reject),
            0x03 => Ok(OfferStatus::Busy),
            0x04 => Ok(OfferStatus::CommandReady),
            0xFF => Ok(OfferStatus::CmdNotSupported),
            _ => Err(ConversionError::ByteConversionError),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum CfuUpdateContentResponseStatus {
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

// Convert to byte
impl From<CfuUpdateContentResponseStatus> for u8 {
    fn from(value: CfuUpdateContentResponseStatus) -> Self {
        match value {
            CfuUpdateContentResponseStatus::Success => 0x00,
            CfuUpdateContentResponseStatus::ErrorPrepare => 0x01,
            CfuUpdateContentResponseStatus::ErrorWrite => 0x02,
            CfuUpdateContentResponseStatus::ErrorComplete => 0x03,
            CfuUpdateContentResponseStatus::ErrorVerify => 0x04,
            CfuUpdateContentResponseStatus::ErrorCrc => 0x05,
            CfuUpdateContentResponseStatus::ErrorSignature => 0x06,
            CfuUpdateContentResponseStatus::ErrorVersion => 0x07,
            CfuUpdateContentResponseStatus::SwapPending => 0x08,
            CfuUpdateContentResponseStatus::ErrorInvalidAddr => 0x09,
            CfuUpdateContentResponseStatus::ErrorNoOffer => 0x0A,
            CfuUpdateContentResponseStatus::ErrorInvalid => 0x0B,
        }
    }
}

// Convert from byte
impl TryFrom<u8> for CfuUpdateContentResponseStatus {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(CfuUpdateContentResponseStatus::Success),
            0x01 => Ok(CfuUpdateContentResponseStatus::ErrorPrepare),
            0x02 => Ok(CfuUpdateContentResponseStatus::ErrorWrite),
            0x03 => Ok(CfuUpdateContentResponseStatus::ErrorComplete),
            0x04 => Ok(CfuUpdateContentResponseStatus::ErrorVerify),
            0x05 => Ok(CfuUpdateContentResponseStatus::ErrorCrc),
            0x06 => Ok(CfuUpdateContentResponseStatus::ErrorSignature),
            0x07 => Ok(CfuUpdateContentResponseStatus::ErrorVersion),
            0x08 => Ok(CfuUpdateContentResponseStatus::SwapPending),
            0x09 => Ok(CfuUpdateContentResponseStatus::ErrorInvalidAddr),
            0x0A => Ok(CfuUpdateContentResponseStatus::ErrorNoOffer),
            0x0B => Ok(CfuUpdateContentResponseStatus::ErrorInvalid),
            _ => Err(ConversionError::ByteConversionError),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LSB first Representation of FwUpdateContentResponse
pub struct FwUpdateContentResponse {
    pub sequence: u16,                          // bytes 0-1
    _reserved0: u16,                            // bytes 2-3
    pub status: CfuUpdateContentResponseStatus, // byte 4
    _reserved1: [u8; 11],                       // bytes 5-15
}

impl FwUpdateContentResponse {
    pub fn new(sequence: u16, status: CfuUpdateContentResponseStatus) -> Self {
        Self {
            sequence,
            status,
            _reserved0: 0,
            _reserved1: [0; 11],
        }
    }
}

// Convert to bytes
impl From<&FwUpdateContentResponse> for [u8; 16] {
    fn from(response: &FwUpdateContentResponse) -> Self {
        let mut buffer = [0u8; 16];
        buffer[0..2].copy_from_slice(&response.sequence.to_le_bytes());
        buffer[2..4].copy_from_slice(&[0; 2]); // response._reserved0 is reserved
        buffer[4] = response.status.into();
        buffer[5..16].copy_from_slice(&[0; 11]); // response._reserved1 is reserved
        buffer
    }
}

// Convert from bytes
impl TryFrom<[u8; 16]> for FwUpdateContentResponse {
    type Error = ConversionError;

    fn try_from(buffer: [u8; 16]) -> Result<Self, Self::Error> {
        Ok(Self {
            sequence: u16::from_le_bytes([buffer[0], buffer[1]]),
            _reserved0: 0, // [buffer[2], buffer[3]] is reserved
            status: CfuUpdateContentResponseStatus::try_from(buffer[4])
                .map_err(|_| ConversionError::ByteConversionError)?,
            _reserved1: [0; 11], // buffer[5..16] is reserved
        })
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
    /// ContentResponseError
    CfuContentUpdateResponseError(CfuUpdateContentResponseStatus),
    /// OfferStatusError
    CfuOfferStatusError(OfferStatus),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Serialiation and Deserialization tests for FwUpdateOffer
    #[test]
    fn test_fwupdate_offer_serialization_deserialization() {
        // Create an instance of FwUpdateOffer
        let offer_command_orig = FwUpdateOffer {
            component_info: UpdateOfferComponentInfo::new(HostToken::Driver, 1),
            firmware_version: FwVersion {
                major: 1,
                minor: 2,
                variant: 3,
            },
            vendor_specific: 0x2,
            misc_and_protocol_version: 0x87654321,
        };

        // Serialize the offer command to a byte array
        let offer_command_serialized: [u8; 32] = (&offer_command_orig).into();

        // Deserialize the byte array back to a FwUpdateOffer instance
        let offer_command_deserialized = FwUpdateOffer::try_from(&offer_command_serialized);

        // Compare both
        assert_eq!(offer_command_orig, offer_command_deserialized.unwrap());
    }

    // Serialiation and Deserialization tests for FwUpdateOfferExtended
    #[test]
    fn test_fwupdate_offer_extended_serialization_deserialization() {
        // Create an instance of FwUpdateOfferExtended
        let offer_extended_command_orig = FwUpdateOfferExtended::new(OfferExtendedComponentInfo::new(
            HostToken::Driver,
            SpecialComponentIds::Command,
            OfferCommandExtendedCodeValues::OfferNotifyOnReady,
        ));

        // Serialize the extended command to a byte array
        let offer_extended_command_serialized: [u8; 16] = (&offer_extended_command_orig).into();

        // Ensure reserved fields are all zeros
        assert_eq!(offer_extended_command_serialized[1], 0);
        assert_eq!(&offer_extended_command_serialized[4..8], &[0; 4]);
        assert_eq!(&offer_extended_command_serialized[8..12], &[0; 4]);
        assert_eq!(&offer_extended_command_serialized[12..16], &[0; 4]);

        // Deserialize the byte array back to a FwUpdateOfferExtended instance
        let offer_extended_command_deserialized = FwUpdateOfferExtended::try_from(&offer_extended_command_serialized);

        // Compare both
        assert_eq!(
            offer_extended_command_orig,
            offer_extended_command_deserialized.unwrap()
        );
    }

    // Serialiation and Deserialization tests for FwUpdateContentCommand
    #[test]
    fn test_fwupdate_content_command_serialization_deserialization() {
        // Create an instance of FwUpdateContentCommand
        let content_command_orig = FwUpdateContentCommand {
            header: FwUpdateContentHeader {
                flags: FW_UPDATE_FLAG_FIRST_BLOCK,
                data_length: DEFAULT_DATA_LENGTH as u8,
                sequence_num: 0x1234,
                firmware_address: 0x5678,
            },
            data: [0x01; DEFAULT_DATA_LENGTH],
        };

        // Serialize the content command to a byte array
        let content_command_serialized: [u8; 60] = (&content_command_orig).into();

        // Deserialize the byte array back to a FwUpdateContentCommand instance
        let content_command_deserialized = FwUpdateContentCommand::try_from(&content_command_serialized);

        // Compare both
        assert_eq!(content_command_orig, content_command_deserialized.unwrap());
    }

    // Serialization and Deserialization tests for FwUpdateOfferInformation
    #[test]
    fn test_fwupdate_offer_information_serialization_deserialization() {
        // Create an instance of FwUpdateOfferInformation
        let offer_info_orig = FwUpdateOfferInformation::new(OfferInformationComponentInfo::new(
            HostToken::VendorSpecific(0xFE), // use VendorSpecific to test the conversion
            SpecialComponentIds::Info,
            OfferInformationCodeValues::StartEntireTransaction,
        ));

        // Serialize the offer information to a byte array
        let offer_info_serialized: [u8; 16] = (&offer_info_orig).into();

        // Ensure reserved fields are all zeros
        assert_eq!(offer_info_serialized[1], 0);
        assert_eq!(&offer_info_serialized[4..8], &[0; 4]);
        assert_eq!(&offer_info_serialized[8..12], &[0; 4]);
        assert_eq!(&offer_info_serialized[12..16], &[0; 4]);

        // Deserialize the byte array back to a FwUpdateOfferInformation instance
        let offer_info_deserialized = FwUpdateOfferInformation::try_from(&offer_info_serialized);

        // Compare both
        assert_eq!(offer_info_orig, offer_info_deserialized.unwrap());
    }

    // Serialiation and Deserialization tests for GetFwVersionResponse
    #[test]
    fn test_get_fw_version_response_serialization_deserialization() {
        // Create an instance of GetFwVersionResponse
        let fw_version = FwVersion {
            major: 1,
            minor: 2,
            variant: 3,
        };

        let mut component_info = [FwVerComponentInfo::default(); MAX_CMPT_COUNT];
        component_info[0] = FwVerComponentInfo::new(fw_version, 1);
        component_info[1] = FwVerComponentInfo::new(fw_version, 2);
        component_info[2] = FwVerComponentInfo::new(fw_version, 3);

        let fwversion_response_orig = GetFwVersionResponse {
            header: GetFwVersionResponseHeader {
                component_count: 3,
                _reserved: 54,
                byte3: GetFwVerRespHeaderByte3::NoSpecialFlags,
            },
            component_info,
        };

        // Serialize the fwversion_response_orig to a byte array
        let fwversion_response_serialized: [u8; 60] = (&fwversion_response_orig).try_into().unwrap();

        // Deserialize the byte array back to a GetFwVersionResponse instance
        let fwversion_response_deserialized = GetFwVersionResponse::try_from(&fwversion_response_serialized);

        //compare both
        assert_eq!(fwversion_response_orig, fwversion_response_deserialized.unwrap());
    }

    // Serialization and Deserialization tests for FwUpdateOfferResponse
    #[test]
    fn test_fwupdate_offer_response_serialization_deserialization() {
        // Create an instance of FwUpdateOfferResponse
        let offer_response_orig = FwUpdateOfferResponse::new_accept(HostToken::Driver);

        // Serialize the offer_response_orig to a byte array
        let offer_response_serialized: [u8; 16] = (&offer_response_orig).into();

        // Ensure reserved fields are all zeros
        assert_eq!(&offer_response_serialized[0..3], &[0; 3]); // _reserved0 is reserved
        assert_eq!(&offer_response_serialized[4..8], &[0; 4]); // _reserved1 is reserved
        assert_eq!(&offer_response_serialized[9..12], &[0; 3]); // _reserved2 is reserved
        assert_eq!(&offer_response_serialized[13..16], &[0; 3]); // _reserved3 is reserved

        // Deserialize the byte array back to a FwUpdateOfferResponse instance
        let offer_response_deserialized = FwUpdateOfferResponse::try_from(offer_response_serialized).unwrap();

        // Compare both
        assert_eq!(offer_response_orig, offer_response_deserialized);
    }

    // Serialization and Deserialization tests for FwUpdateContentResponse
    #[test]
    fn test_fwupdate_content_response_serialization_deserialization() {
        // Create an instance of FwUpdateContentResponse
        let content_response_orig = FwUpdateContentResponse::new(0x1234, CfuUpdateContentResponseStatus::Success);

        // Serialize the content_response_orig to a byte array
        let content_response_serialized: [u8; 16] = (&content_response_orig).into();

        // Ensure reserved fields are all zeros
        assert_eq!(&content_response_serialized[2..4], &[0; 2]); // _reserved0 is reserved
        assert_eq!(&content_response_serialized[5..16], &[0; 11]); // _reserved1 is reserved

        // Deserialize the byte array back to a FwUpdateContentResponse instance
        let content_response_deserialized = FwUpdateContentResponse::try_from(content_response_serialized).unwrap();

        // Compare both
        assert_eq!(content_response_orig, content_response_deserialized);
    }
}
