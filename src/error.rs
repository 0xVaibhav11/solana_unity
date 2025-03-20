use libc::c_char;
use std::ffi::CString;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolanaUnityError {
    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Transaction error: {0}")]
    TransactionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Wallet error: {0}")]
    WalletError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("FFI error: {0}")]
    FfiError(String),
}

// Convert error to C string for FFI
pub fn error_to_c_string(error: &SolanaUnityError) -> *mut c_char {
    let error_string = error.to_string();
    let c_error = CString::new(error_string)
        .unwrap_or_else(|_| CString::new("Error converting error message").unwrap());
    c_error.into_raw()
}

// Free C string (to be called from C#)
pub unsafe fn free_c_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
