use core::future::Future;

use crate::components::CfuComponentTraits;

/// CfuReceiveContent trait defines behavior needed for a Cfu client (receiver) to process CFU commands
/// E is an error type that can be defined by the implementor
/// C is a command type that can be defined by the implementor
/// T is a generic args type that can be defined by the implementor to pass any additional info to the methods
pub trait CfuReceiveContent<T, C, E: Default> {
    /// receives a CFU command from a Host and processes the contents
    /// Typestates here allow for flexible implementations
    fn process_command(&self, args: Option<T>, cmd: C) -> impl Future<Output = Result<(), E>>;

    /// For all components, run their storage_prepare() method
    /// Typestates here allow for flexible implementations
    fn prepare_components(
        &self,
        args: Option<T>,
        primary_component: impl CfuComponentTraits,
    ) -> impl Future<Output = Result<(), E>>;
}
