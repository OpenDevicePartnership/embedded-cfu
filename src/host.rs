use binary_serde::{BinarySerde, DeserializeError, Endianness};

use super::*;

/// CfuHostStates trait defines behavior needed for a Cfu Host to process available Cfu Offers
/// and send the appropriate commands to the Cfu Client to update the components
pub trait CfuHostStates {
    /// Notifies that the host is now initialized and has identified the offers to send
    fn start_transaction<W: CfuWriter>(
        self,
        writer: &mut W,
    ) -> impl Future<Output = Result<FwUpdateOfferResponse, CfuProtocolError>>;
    /// Notifies the primary component that the host is ready to start sending offers
    fn notify_start_offer_list<W: CfuWriter>(
        self,
        writer: &mut W,
    ) -> impl Future<Output = Result<FwUpdateOfferResponse, CfuProtocolError>>;
    /// Notifies the primary component that the host has sent all offers
    fn notify_end_offer_list<W: CfuWriter>(
        self,
        writer: &mut W,
    ) -> impl Future<Output = Result<FwUpdateOfferResponse, CfuProtocolError>>;
    /// For a slice of responses, determine if any components have not finished updating
    fn verify_all_updates_completed(
        offer_responses: &[FwUpdateOfferResponse],
    ) -> impl Future<Output = Result<bool, CfuProtocolError>>;
}

/// CfuUpdateContent trait defines behavior needed for a Cfu Host to send the contents of an accepted offer to a component via sending commands to a Cfu Client
pub trait CfuUpdateContent<W>
where
    W: CfuWriter,
{
    /// Write all chunks of an image
    fn write_data_chunks(
        &mut self,
        writer: &mut W,
        image: impl CfuImage,
        cmpt_id: ComponentId,
        base_offset: usize,
    ) -> impl Future<Output = Result<FwUpdateContentResponse, CfuProtocolError>>;
    /// Build and send UpdateOfferContent command with first block flag
    fn process_first_data_block(
        &mut self,
        w: &mut W,
        chunk: DataChunk,
    ) -> impl Future<Output = Result<FwUpdateContentResponse, CfuWriterError>>;
    /// Build and send UpdateOfferContent command, no special flags
    fn process_middle_data_block(
        &mut self,
        w: &mut W,
        chunk: DataChunk,
        seq_num: usize,
    ) -> impl Future<Output = Result<FwUpdateContentResponse, CfuWriterError>>;
    /// Build and send UpdateOfferContent command with last block flag
    fn process_last_data_block(
        &mut self,
        w: &mut W,
        chunk: DataChunk,
        seq_num: usize,
    ) -> impl Future<Output = Result<FwUpdateContentResponse, CfuWriterError>>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CfuUpdater {}

impl<W: CfuWriter> CfuUpdateContent<W> for CfuUpdater {
    /// Write all chunks of an image
    async fn write_data_chunks(
        &mut self,
        writer: &mut W,
        image: impl CfuImage,
        cmpt_id: ComponentId,
        base_offset: usize,
    ) -> Result<FwUpdateContentResponse, CfuProtocolError> {
        // Build update offer command
        let updateoffercmd_bytes = [0u8; 16];
        let mut offer_resp = [0u8; 16];
        writer
            .cfu_write_read(Some(base_offset), &updateoffercmd_bytes, &mut offer_resp)
            .await
            .map_err(CfuProtocolError::WriterError)?;

        let deser = FwUpdateContentResponse::binary_deserialize(&offer_resp, binary_serde::Endianness::Little)
            .map_err(|DeserializeError::InvalidEnumValue { enum_name }| {
                error!("deserializing error for: {:?}", enum_name);
                CfuProtocolError::WriterError(CfuWriterError::ByteConversionError)
            })?;
        let status = deser.status;
        if status != CfuOfferResponseStatus::Success {
            return Err(CfuProtocolError::CfuResponseError(status));
        }

        let total_bytes: usize = image.get_total_size();
        let chunk_size = DEFAULT_DATA_LENGTH;
        let num_chunks = total_bytes / chunk_size;
        let remainder = total_bytes % chunk_size;

        // Read and process data in chunks so as to not over-burden memory resources
        let mut resp = FwUpdateContentResponse::new(0, CfuOfferResponseStatus::ErrorInvalid);
        for i in 0..num_chunks {
            let mut chunk = [0u8; DEFAULT_DATA_LENGTH];
            let address_offset = i * DEFAULT_DATA_LENGTH + base_offset;
            let r = match i {
                0 => {
                    image
                        .get_bytes_for_chunk(&mut chunk, address_offset)
                        .await
                        .map_err(|_| CfuProtocolError::WriterError(CfuWriterError::StorageError))?;
                    self.process_first_data_block(writer, chunk).await
                }
                num if (num < num_chunks) => {
                    image
                        .get_bytes_for_chunk(&mut chunk, address_offset)
                        .await
                        .map_err(|_| CfuProtocolError::WriterError(CfuWriterError::StorageError))?;
                    self.process_middle_data_block(writer, chunk, i).await
                }
                _ => {
                    image
                        .get_bytes_for_chunk(&mut chunk[..remainder], address_offset)
                        .await
                        .map_err(|_| CfuProtocolError::WriterError(CfuWriterError::StorageError))?;
                    self.process_last_data_block(writer, chunk, i).await
                }
            }
            .map_err(CfuProtocolError::WriterError)?;
            // if no errors in processing the data block, check the response
            if r.status != CfuOfferResponseStatus::Success {
                return Err(CfuProtocolError::UpdateError(cmpt_id));
            }
            resp = r;
        }

        if resp.sequence != num_chunks as u16 {
            trace!("final sequence number does not match expected number of chunks");
            return Err(CfuProtocolError::InvalidBlockTransition);
        }

        Ok(resp)
    }

    /// Build and send UpdateOfferContent command with first block flag
    async fn process_first_data_block(
        &mut self,
        w: &mut W,
        chunk: DataChunk,
    ) -> Result<FwUpdateContentResponse, CfuWriterError> {
        let cmd = FwUpdateContentCommand {
            header: FwUpdateContentHeader {
                sequence_num: 0,
                data_length: DEFAULT_DATA_LENGTH as u8,
                flags: FwUpdateFlags::FirstBlock,
                firmware_address: 0,
            },
            data: chunk,
        };
        let mut cmd_bytes = [0u8; FwUpdateContentCommand::SERIALIZED_SIZE];
        cmd.binary_serialize(&mut cmd_bytes, Endianness::Little);
        let offset = 0;
        let mut resp_buf = [0u8; FwUpdateContentResponse::SERIALIZED_SIZE];
        w.cfu_write_read(Some(offset), &cmd_bytes, &mut resp_buf)
            .await
            .map_err(|_| CfuWriterError::StorageError)?;

        FwUpdateContentResponse::binary_deserialize(&resp_buf, Endianness::Little)
            .map_err(|_| CfuWriterError::ByteConversionError)
    }
    /// Build and send UpdateOfferContent command, no special flags
    async fn process_middle_data_block(
        &mut self,
        w: &mut W,
        chunk: DataChunk,
        seq_num: usize,
    ) -> Result<FwUpdateContentResponse, CfuWriterError> {
        let cmd = FwUpdateContentCommand {
            header: FwUpdateContentHeader {
                sequence_num: seq_num as u16,
                data_length: DEFAULT_DATA_LENGTH as u8,
                flags: FwUpdateFlags::None,
                firmware_address: 0,
            },
            data: chunk,
        };
        let mut cmd_bytes = [0u8; FwUpdateContentCommand::SERIALIZED_SIZE];
        cmd.binary_serialize(&mut cmd_bytes, Endianness::Little);
        let offset = seq_num * DEFAULT_DATA_LENGTH;
        let mut resp_buf = [0u8; FwUpdateContentResponse::SERIALIZED_SIZE];
        w.cfu_write_read(Some(offset), &cmd_bytes, &mut resp_buf)
            .await
            .map_err(|_| CfuWriterError::StorageError)?;

        FwUpdateContentResponse::binary_deserialize(&resp_buf, Endianness::Little)
            .map_err(|_| CfuWriterError::ByteConversionError)
    }
    /// Build and send UpdateOfferContent command with last block flag
    async fn process_last_data_block(
        &mut self,
        w: &mut W,
        chunk: DataChunk,
        seq_num: usize,
    ) -> Result<FwUpdateContentResponse, CfuWriterError> {
        let cmd = FwUpdateContentCommand {
            header: FwUpdateContentHeader {
                sequence_num: seq_num as u16,
                data_length: DEFAULT_DATA_LENGTH as u8,
                flags: FwUpdateFlags::LastBlock,
                firmware_address: 0,
            },
            data: chunk,
        };
        let mut cmd_bytes = [0u8; FwUpdateContentCommand::SERIALIZED_SIZE];
        cmd.binary_serialize(&mut cmd_bytes, Endianness::Little);
        let offset = seq_num * DEFAULT_DATA_LENGTH;
        let mut resp_buf = [0u8; FwUpdateContentResponse::SERIALIZED_SIZE];
        w.cfu_write_read(Some(offset), &cmd_bytes, &mut resp_buf)
            .await
            .map_err(|_| CfuWriterError::StorageError)?;

        FwUpdateContentResponse::binary_deserialize(&resp_buf, Endianness::Little)
            .map_err(|_| CfuWriterError::ByteConversionError)
    }
}
