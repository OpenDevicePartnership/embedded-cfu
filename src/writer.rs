//! This module defines traits use to read and write data to CFU component or client.

use core::future::Future;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CfuWriterError {
    StorageError,
    ByteConversionError,
    Other,
}

/// Trait to define R/W behavior for driver that can talk to a CFU component or client
pub trait CfuWriterAsync {
    /// writes a chunk of data to a component and reads back to another buffer
    fn cfu_write_read(
        &self,
        mem_offset: Option<usize>,
        data: &[u8],
        read: &mut [u8],
    ) -> impl Future<Output = Result<(), CfuWriterError>> + Send;

    /// Fills a given buffer with data from the component
    fn cfu_read(&self, mem_offset: Option<usize>, read: &mut [u8]) -> impl Future<Output = Result<(), CfuWriterError>> + Send;

    /// Writes a given buffer of data to a component
    fn cfu_write(&self, mem_offset: Option<usize>, data: &[u8]) -> impl Future<Output = Result<(), CfuWriterError>> + Send;

    /// Manages erasing sectors and writing pages into flash based on the CFU offset
    fn cfu_storage(&mut self, mem_offset: usize, data: &[u8]) -> impl Future<Output = Result<(), CfuWriterError>> + Send;
}

/// Trait to define R/W behavior for driver that can talk to a CFU component or client
pub trait CfuWriterSync {
    /// writes a chunk of data to a component and reads back to another buffer
    fn cfu_write_read(&self, mem_offset: Option<usize>, data: &[u8], read: &mut [u8]) -> Result<(), CfuWriterError>;

    /// Fills a given buffer with data from the component
    fn cfu_read(&self, mem_offset: Option<usize>, read: &mut [u8]) -> Result<(), CfuWriterError>;

    /// Writes a given buffer of data to a component
    fn cfu_write(&self, mem_offset: Option<usize>, data: &[u8]) -> Result<(), CfuWriterError>;

    /// Manages erasing sectors and writing pages into flash based on the CFU offset
    fn cfu_storage(&mut self, mem_offset: usize, data: &[u8]) -> Result<(), CfuWriterError>;
}
