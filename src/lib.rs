mod celix_sys;

#[doc(hidden)]
pub mod details {
    pub use crate::celix_sys::celix_bundle_context_t as CBundleContext;
    pub use crate::celix_sys::celix_status_t as CStatus;
}

pub mod bundle_activator;
pub mod bundle_context;
pub mod error;

mod test {
    use std::sync::mpsc::Receiver;

    use crate::bundle_activator::*;
    use crate::bundle_context::*;
    use crate::error::*;

    struct OwnActivator;

    impl BundleActivator for OwnActivator {
        fn thread_fn(
        ) -> impl FnOnce(BundleContext, Receiver<BundleStopSignal>) -> Result<(), Error> + Send + 'static
        {
            own_thread_fn
        }
    }

    fn own_thread_fn(_: BundleContext, _: Receiver<BundleStopSignal>) -> Result<(), Error> {
        Ok(())
    }

    crate::generate_bundle_activator!(OwnActivator);
}
