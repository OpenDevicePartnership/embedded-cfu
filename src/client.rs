use super::*;

/// CfuReceiveContent trait defines behavior needed for a Cfu client (receiver) to process CFU commands
/// E is an error type that can be defined by the implementor
/// C is a command type that can be defined by the implementor
/// T is a generic args type that can be defined by the implementor to pass any additional info to the methods
pub trait CfuReceiveContent<T, C, E: Default> {
    /// receives a CFU command from a Host and processes the contents
    /// Typestates here allow for flexible implementations
    fn process_command(&self, args: Option<T>, cmd: C) -> impl Future<Output = Result<(), E>> {
        default_process_command::<T, C, E>(args, cmd)
    }
    /// For all components, run their storage_prepare() method
    /// Typestates here allow for flexible implementations
    fn prepare_components(
        &self,
        args: Option<T>,
        primary_component: impl CfuComponentTraits,
    ) -> impl Future<Output = Result<(), E>> {
        default_prepare_components::<T, E>(args, primary_component)
    }
}

/// Helper function to provide a default implementation for process_command
async fn default_process_command<T, C, E: Default>(args: Option<T>, _cmd: C) -> Result<(), E> {
    if args.is_some() {
        trace!("unexpected args to default_process_command function");
        trace!("potentially missing implementation of process_command in CfuReceiveContent trait")
    }
    Ok(())
}

/// Helper function to provide a default implementation for prepare_components
async fn default_prepare_components<T, E: Default>(
    args: Option<T>,
    _primary_component: impl CfuComponentTraits,
) -> Result<(), E> {
    if args.is_some() {
        trace!("unexpected args to default_prepare_components function");
        trace!("potentially missing implementation of prepare_components in CfuReceiveContent trait")
    }
    Ok(())
}
