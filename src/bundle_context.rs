use crate::celix_sys::celix_bundleContext_getBundleId;
use crate::celix_sys::celix_bundle_context_t;
use std::ffi::c_long;

pub(crate) struct BundleContextInternal(pub(crate) *mut celix_bundle_context_t);

unsafe impl Send for BundleContextInternal {}
unsafe impl Sync for BundleContextInternal {}

#[derive(Clone)]
pub struct BundleContext<'a>(pub(crate) &'a BundleContextInternal);

impl<'a> BundleContext<'a> {
    pub fn register_service<T>(&self) -> ServiceRegistrationBuilder<T> {
        ServiceRegistrationBuilder::new(self.clone())
    }

    pub fn get_bundle_id(&self) -> c_long {
        unsafe { celix_bundleContext_getBundleId(self.0 .0) }
    }
}

pub struct ServiceRegistrationBuilder<'a, 'b, T> {
    ctx: BundleContext<'a>,
    svc: Option<Service<'b, T>>,
}

impl<'a, 'b, T> ServiceRegistrationBuilder<'a, 'b, T> {
    pub fn new(ctx: BundleContext<'a>) -> Self {
        ServiceRegistrationBuilder { ctx, svc: None }
    }

    pub fn with_owned_service(&mut self, svc: T) -> &mut Self {
        self.svc = Some(Service::Owned(Box::new(svc)));
        self
    }

    pub fn with_borrowed_service(&mut self, svc: &'b T) -> &mut Self {
        self.svc = Some(Service::Borrowed(svc));
        self
    }
}

enum Service<'a, T: ?Sized> {
    Owned(Box<T>),
    Borrowed(&'a T),
}
