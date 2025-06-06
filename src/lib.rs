#![no_std]
use core::future::Future;

use embedded_io_async::{Read, ReadExactError, Seek, SeekFrom};

use crate::protocol_definitions::*;

pub mod client;
pub mod components;
pub mod fmt;
pub mod host;
pub mod protocol_definitions;
pub mod writer;

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

pub type DataChunk = [u8; DEFAULT_DATA_LENGTH];
