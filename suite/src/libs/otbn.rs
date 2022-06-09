//! FFI Code for the opentitan big number library
#![allow(dead_code)]

#[repr(C)]
pub enum otbn_error_t {
    /// No errors.
    OtbnErrorOk = 0,
    /// Invalid argument provided to OTBN interface function.
    OtbnErrorInvalidArgument = 1,
    /// Invalid offset provided.
    OtbnErrorBadOffsetLen = 2,
    /// OTBN internal error; use otbn_get_err_bits for specific error codes.
    OtbnErrorExecutionFailed = 3,
    /// Attempt to interact with OTBN while it was unavailable.
    OtbnErrorUnavailable = 4,
}

#[link(name = "sw_lib_crypto_otbn")]
extern "C" {}

#[link(name = "sw_lib_crypto_otbn_util")]
extern "C" {}
