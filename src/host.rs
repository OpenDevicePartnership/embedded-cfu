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
