#![no_std]
use core::future::Future;

use embedded_io_async::{Read, ReadExactError, Seek, SeekFrom};

use crate::components::*;
use crate::protocol_definitions::*;

pub mod client;
pub mod components;
pub mod fmt;
pub mod host;
pub mod protocol_definitions;

// re-export the error enum
pub use CfuProtocolError::*;

/// Trait to define the characteristics of a CFU image that will be sent by a Cfu Host
pub trait CfuImage: Read + Seek + Copy {
    /// Gets the total size in bytes of an image
    fn get_total_size(self) -> usize;
    /// converts the image into a slice of bytes
    /// get bytes for a chunk because full image might not fit for some components
    fn get_bytes_for_chunk(
        self,
        buf: &mut [u8],
        offset: usize,
    ) -> impl Future<Output = Result<(), ReadExactError<Self::Error>>>;
}

/// Helper function to read from an image at the offset matching the sequence number
pub async fn read_from_exact<I: CfuImage>(
    image: &mut I,
    seq_num: usize,
    buf: &mut [u8],
) -> Result<(), ReadExactError<I::Error>> {
    let offset = seq_num * DEFAULT_DATA_LENGTH;
    image
        .seek(SeekFrom::Start(offset as u64))
        .await
        .map_err(ReadExactError::Other)?;
    image.read_exact(buf).await
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CfuWriterError {
    StorageError,
    ByteConversionError,
    Other,
}

/// Trait to define R/W behavior for driver that can talk to a CFU component or client
pub trait CfuWriter {
    /// writes a chunk of data to a component and reads back to another buffer
    fn cfu_write_read(
        &self,
        mem_offset: Option<usize>,
        data: &[u8],
        read: &mut [u8],
    ) -> impl Future<Output = Result<(), CfuWriterError>>;
    /// Fills a given buffer with data from the component
    fn cfu_read(&self, mem_offset: Option<usize>, read: &mut [u8]) -> impl Future<Output = Result<(), CfuWriterError>>;
    /// Writes a given buffer of data to a component
    fn cfu_write(&self, mem_offset: Option<usize>, data: &[u8]) -> impl Future<Output = Result<(), CfuWriterError>>;
}

pub type DataChunk = [u8; DEFAULT_DATA_LENGTH];

#[derive(Copy, Clone)]
pub struct CfuWriterDefault {}

impl CfuWriterDefault {
    /// Create new instance
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CfuWriterDefault {
    /// Creates default instance of CfuWriterDefault
    fn default() -> Self {
        Self::new()
    }
}

impl CfuWriter for CfuWriterDefault {
    /// writes a chunk of data to a component and reads back to another buffer
    async fn cfu_write_read(&self, offset: Option<usize>, write: &[u8], read: &mut [u8]) -> Result<(), CfuWriterError> {
        // TODO: add with_timeout to these calls
        self.cfu_write(offset, write).await?;
        self.cfu_read(offset, read).await?;
        Ok(())
    }
    /// Fills a given buffer with 0xBE 0xEF alternating bytes
    async fn cfu_read(&self, _offset: Option<usize>, read: &mut [u8]) -> Result<(), CfuWriterError> {
        info!("Fake reading from component");
        for (i, byte) in read.iter_mut().enumerate() {
            if i % 2 == 0 {
                *byte = 0xEF;
            } else {
                *byte = 0xBE;
            }
        }
        Ok(())
    }
    async fn cfu_write(&self, _offset: Option<usize>, write: &[u8]) -> Result<(), CfuWriterError> {
        info!("Fake writing to component: {:?}", write);
        Ok(())
    }
}
