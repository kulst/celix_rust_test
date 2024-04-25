use crate::celix_sys;
use crate::celix_sys::celix_status_t;

pub const CELIX_SUCCESS: celix_status_t = celix_sys::CELIX_SUCCESS as celix_status_t;

//Note compile-time defined constants are not available in rust generated bindings, so
//these are defined with literal values.
pub const BUNDLE_EXCEPTION: celix_status_t = 70001;

pub enum Error {
    BundleException,
    CelixStatusError(celix_status_t), // Represent not explicitly mapped celix_status_t values
}

impl From<celix_status_t> for Error {
    fn from(status: celix_status_t) -> Self {
        match status {
            BUNDLE_EXCEPTION => Error::BundleException,
            _ => Error::CelixStatusError(status),
        }
    }
}

impl Into<celix_status_t> for Error {
    fn into(self) -> celix_status_t {
        match self {
            Error::BundleException => BUNDLE_EXCEPTION,
            Error::CelixStatusError(status) => status,
        }
    }
}
