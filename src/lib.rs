//mod bundle_context;
mod celix_bindings;
mod error;

use std::marker::PhantomData;
use std::mem::drop;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;

use celix_bindings::celix_bundle_context_t;
use celix_bindings::celix_status_t;
use error::*;

struct BundleContext(*mut celix_bundle_context_t);

unsafe impl Send for BundleContext {}
unsafe impl Sync for BundleContext {}

#[derive(Clone)]
pub struct BundleContextHandle<'a>(&'a BundleContext);

pub struct StopThreadSignal;

pub trait ThreadedUser {
    type ThreadFunction: FnOnce(BundleContextHandle, Receiver<StopThreadSignal>) -> Result<(), Error>
        + Send
        + 'static;

    fn provide_thread_fn() -> Self::ThreadFunction;
}

struct ThreadedBundleActivator<T> {
    thread_util: Option<ThreadUtil>,
    phantom: PhantomData<T>,
}

struct ThreadUtil {
    handle: JoinHandle<Result<(), Error>>,
    stopper: Sender<StopThreadSignal>,
}

pub fn celix_bundle_activator_create<T: ThreadedUser>(
    _ctx: *mut celix_bundle_context_t,
    out: *mut *mut ::std::ffi::c_void,
) -> Result<(), Error> {
    let activator = Box::new(ThreadedBundleActivator::<T> {
        thread_util: None,
        phantom: PhantomData,
    });
    unsafe { *out = Box::into_raw(activator) as *mut std::ffi::c_void };

    Ok(())
}

pub fn celix_bundle_activator_start<T: ThreadedUser>(
    handle: *mut std::ffi::c_void,
    ctx: *mut celix_bundle_context_t,
) -> Result<(), Error> {
    if ctx.is_null() {
        return Err(Error::BundleException);
    }
    let context = BundleContext(ctx);
    let activator = unsafe {
        (handle as *mut ThreadedBundleActivator<T>)
            .as_mut()
            .ok_or(Error::BundleException)?
    };
    let thread_fn = <T as ThreadedUser>::provide_thread_fn();
    let (sender, receiver) = std::sync::mpsc::channel();
    activator.thread_util = Some(ThreadUtil {
        handle: std::thread::spawn(move || {
            thread_fn(BundleContextHandle(&context), receiver)?;
            Ok(())
        }),
        stopper: sender,
    });
    Ok(())
}

pub fn celix_bundle_activator_stop<T: ThreadedUser>(
    handle: *mut std::ffi::c_void,
    _ctx: *mut celix_bundle_context_t,
) -> Result<(), Error> {
    let activator = unsafe {
        (handle as *mut ThreadedBundleActivator<T>)
            .as_mut()
            .ok_or(Error::BundleException)?
    };
    let util = activator.thread_util.take().unwrap();
    let _ = util.stopper.send(StopThreadSignal);
    util.handle.join().map_err(|_| Error::BundleException)?
}

pub fn celix_bundle_activator_destroy<T: ThreadedUser>(
    handle: *mut std::ffi::c_void,
    _ctx: *mut celix_bundle_context_t,
) -> Result<(), Error> {
    if handle.is_null() {
        return Err(Error::BundleException);
    }
    let activator = unsafe { Box::from_raw(handle as *mut ThreadedBundleActivator<T>) };
    drop(activator);
    Ok(())
}

#[macro_export]
macro_rules! generate_bundle_activator {
    ($activator:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn celix_bundleActivator_create(
            ctx: *mut $crate::celix_bindings::celix_bundle_context,
            out: *mut *mut ::std::ffi::c_void,
        ) -> $crate::celix_status_t {
            match celix_bundle_activator_create::<$activator>(ctx, out) {
                Ok(_) => CELIX_SUCCESS,
                Err(e) => e.into(),
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn celix_bundleActivator_start(
            handle: *mut ::std::ffi::c_void,
            ctx: *mut $crate::celix_bindings::celix_bundle_context,
        ) -> $crate::celix_status_t {
            match celix_bundle_activator_start::<$activator>(handle, ctx) {
                Ok(_) => CELIX_SUCCESS,
                Err(e) => e.into(),
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn celix_bundleActivator_stop(
            handle: *mut ::std::ffi::c_void,
            ctx: *mut $crate::celix_bindings::celix_bundle_context,
        ) -> $crate::celix_status_t {
            match celix_bundle_activator_stop::<$activator>(handle, ctx) {
                Ok(_) => CELIX_SUCCESS,
                Err(e) => e.into(),
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn celix_bundleActivator_destroy(
            handle: *mut ::std::ffi::c_void,
            ctx: *mut $crate::celix_bindings::celix_bundle_context,
        ) -> $crate::celix_status_t {
            match celix_bundle_activator_destroy::<$activator>(handle, ctx) {
                Ok(_) => CELIX_SUCCESS,
                Err(e) => e.into(),
            }
        }
    };
}

mod test {
    use crate::*;

    struct OwnActivator;

    impl ThreadedUser for OwnActivator {
        type ThreadFunction =
            fn(BundleContextHandle, Receiver<StopThreadSignal>) -> Result<(), Error>;

        fn provide_thread_fn() -> Self::ThreadFunction {
            own_thread_fn
        }
    }

    fn own_thread_fn(_: BundleContextHandle, _: Receiver<StopThreadSignal>) -> Result<(), Error> {
        Ok(())
    }

    generate_bundle_activator!(OwnActivator);
}
