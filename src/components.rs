use core::future::Future;

use crate::protocol_definitions::*;
use crate::{CfuWriter, CfuWriterError};

pub trait CfuComponentInfo {
    /// Gets the current fw version of the component
    fn get_fw_version(&self) -> impl Future<Output = Result<FwVersion, CfuProtocolError>>;
    /// Gets the component's id
    /// Not async as this should be an element of struct that implements this trait
    fn get_component_id(&self) -> ComponentId;
    /// Validate the CFU offer for the component
    /// returns a CfuOfferResponseStatus with additional info on Reject Reason in the Err case.
    fn is_offer_valid(
        &self,
    ) -> impl Future<Output = Result<CfuOfferResponseStatus, (CfuOfferResponseStatus, RejectReason)>>;
    /// Returns whether or not this component is a primary component
    /// Not async as this should be an element of struct that implements this trait
    /// Default implementation returns false,
    fn is_primary_component(&self) -> bool {
        false
    }
    /// Returns whether or not this component has a dual-bank memory layout
    /// Not async as this should be an element of struct that implements this trait
    fn is_dual_bank(&self) -> bool;
    /// Returns sub-component ids if this component has any
    /// Not async as this should be an element of struct that implements this trait
    fn get_subcomponents(&self) -> [Option<ComponentId>; MAX_SUBCMPT_COUNT];
}

pub trait CfuComponentStorage: CfuWriter {
    fn storage_prepare(&self) -> impl Future<Output = Result<(), CfuWriterError>>;
    fn storage_write(&self) -> impl Future<Output = Result<(), CfuWriterError>>;
    fn storage_finalize(&self) -> impl Future<Output = Result<(), CfuWriterError>>;
    fn get_storage_offset(&self) -> usize {
        0
    }
}

pub trait CfuAccessoryComponent {
    /// Accessories need to be able to auto-reject offers if we're already mid-update
    /// Default implementation returns false
    fn is_midupdate(&self) -> impl Future<Output = Result<bool, CfuProtocolError>> {
        async {
            Ok(false)
            // Error case would use
            // Err(CfuProtocolError::Timeout)
        }
    }
}

pub trait CfuComponentFinalize {
    /// Handles any post-update requirements like delay before reset, or setting boot flags
    /// Default implementation is do nothing
    fn on_update_complete<T, RT: Default, E: Default>(&self, args: Option<T>) -> impl Future<Output = Result<RT, E>> {
        async move {
            if args.is_some() {
                use crate::trace;
                trace!("unexpected args to on_update_complete function");
                trace!("potentially missing implementation of on_update_complete in CfuComponentFinalize trait");
                return Err(E::default());
            }
            Ok(RT::default())
        }
    }
}

pub trait CfuComponentTraits: CfuComponentInfo + CfuComponentStorage + Default {}
