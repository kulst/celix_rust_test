use crate::bundle_context::{BundleContext, BundleContextInternal};
use crate::details::*;
use crate::error::Error;
use std::marker::PhantomData;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

#[allow(dead_code)]
pub struct BundleStopSignal {
    pub(crate) inner: (),
}

pub trait BundleActivator {
    fn thread_fn(
    ) -> impl FnOnce(BundleContext, Receiver<BundleStopSignal>) -> Result<(), Error> + Send + 'static;
}

struct InnerBundleActivator<T> {
    thread_util: Option<ThreadUtil>,
    phantom: PhantomData<T>,
}

struct ThreadUtil {
    handle: JoinHandle<Result<(), Error>>,
    stopper: Sender<BundleStopSignal>,
}

#[doc(hidden)]
#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn celix_bundle_activator_create<T: BundleActivator>(
    _ctx: *mut CBundleContext,
    out: *mut *mut ::std::ffi::c_void,
) -> Result<(), Error> {
    let activator = Box::new(InnerBundleActivator::<T> {
        thread_util: None,
        phantom: PhantomData,
    });
    unsafe { *out = Box::into_raw(activator) as *mut std::ffi::c_void };

    Ok(())
}

#[doc(hidden)]
#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn celix_bundle_activator_start<T: BundleActivator>(
    handle: *mut std::ffi::c_void,
    ctx: *mut CBundleContext,
) -> Result<(), Error> {
    if ctx.is_null() {
        return Err(Error::BundleException);
    }
    let context = BundleContextInternal(ctx);
    let activator = unsafe {
        (handle as *mut InnerBundleActivator<T>)
            .as_mut()
            .ok_or(Error::BundleException)?
    };
    let thread_fn = <T as BundleActivator>::thread_fn();
    let (sender, receiver) = std::sync::mpsc::channel();
    activator.thread_util = Some(ThreadUtil {
        handle: std::thread::spawn(move || {
            thread_fn(BundleContext(&context), receiver)?;
            Ok(())
        }),
        stopper: sender,
    });
    Ok(())
}

#[doc(hidden)]
#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn celix_bundle_activator_stop<T: BundleActivator>(
    handle: *mut std::ffi::c_void,
    _ctx: *mut CBundleContext,
) -> Result<(), Error> {
    let activator = unsafe {
        (handle as *mut InnerBundleActivator<T>)
            .as_mut()
            .ok_or(Error::BundleException)?
    };
    let util = activator.thread_util.take().unwrap();
    let _ = util.stopper.send(BundleStopSignal { inner: () });
    util.handle.join().map_err(|_| Error::BundleException)?
}

#[doc(hidden)]
#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn celix_bundle_activator_destroy<T: BundleActivator>(
    handle: *mut std::ffi::c_void,
    _ctx: *mut CBundleContext,
) -> Result<(), Error> {
    if handle.is_null() {
        return Err(Error::BundleException);
    }
    let activator = unsafe { Box::from_raw(handle as *mut InnerBundleActivator<T>) };
    drop(activator);
    Ok(())
}

#[macro_export]
macro_rules! generate_bundle_activator {
    ($activator:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn celix_bundleActivator_create(
            ctx: *mut $crate::details::CBundleContext,
            out: *mut *mut ::std::ffi::c_void,
        ) -> $crate::details::CStatus {
            match $crate::bundle_activator::celix_bundle_activator_create::<$activator>(ctx, out) {
                Ok(_) => CELIX_SUCCESS,
                Err(e) => e.into(),
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn celix_bundleActivator_start(
            handle: *mut ::std::ffi::c_void,
            ctx: *mut $crate::details::CBundleContext,
        ) -> $crate::details::CStatus {
            match $crate::bundle_activator::celix_bundle_activator_start::<$activator>(handle, ctx)
            {
                Ok(_) => CELIX_SUCCESS,
                Err(e) => e.into(),
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn celix_bundleActivator_stop(
            handle: *mut ::std::ffi::c_void,
            ctx: *mut $crate::details::CBundleContext,
        ) -> $crate::details::CStatus {
            match $crate::bundle_activator::celix_bundle_activator_stop::<$activator>(handle, ctx) {
                Ok(_) => CELIX_SUCCESS,
                Err(e) => e.into(),
            }
        }

        #[no_mangle]
        pub unsafe extern "C" fn celix_bundleActivator_destroy(
            handle: *mut ::std::ffi::c_void,
            ctx: *mut $crate::details::CBundleContext,
        ) -> $crate::details::CStatus {
            match $crate::bundle_activator::celix_bundle_activator_destroy::<$activator>(
                handle, ctx,
            ) {
                Ok(_) => CELIX_SUCCESS,
                Err(e) => e.into(),
            }
        }
    };
}
