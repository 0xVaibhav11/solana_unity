use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar};
use std::ptr;
use std::slice;

use crate::account::Account;
use crate::error::{error_to_c_string, free_c_string, SolanaUnityError};
use crate::rpc::RpcClient;
use crate::transaction::Transaction;

// Helper to convert C string to Rust string
unsafe fn c_str_to_string(c_str: *const c_char) -> Result<String, SolanaUnityError> {
    if c_str.is_null() {
        return Err(SolanaUnityError::FfiError(
            "Null pointer provided".to_string(),
        ));
    }

    unsafe {
        CStr::from_ptr(c_str)
            .to_str()
            .map(|s| s.to_string())
            .map_err(|e| SolanaUnityError::FfiError(format!("Invalid UTF-8 string: {}", e)))
    }
}

// Helper to convert Rust result to C result with error
fn handle_result<T>(result: Result<T, SolanaUnityError>, error_out: *mut *mut c_char) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(err) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&err);
                }
            }
            None
        }
    }
}

// RPC Client functions

#[no_mangle]
pub extern "C" fn solana_create_rpc_client(
    url: *const c_char,
    commitment: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut RpcClient {
    let url_str = match unsafe { c_str_to_string(url) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    let commitment_str = match unsafe { c_str_to_string(commitment) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    match RpcClient::new(&url_str, &commitment_str) {
        Ok(client) => Box::into_raw(Box::new(client)),
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_destroy_rpc_client(client: *mut RpcClient) {
    if !client.is_null() {
        unsafe {
            let _ = Box::from_raw(client);
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_get_balance(
    client: *mut RpcClient,
    pubkey: *const c_char,
    error_out: *mut *mut c_char,
) -> u64 {
    if client.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null client pointer".to_string(),
                ));
            }
        }
        return 0;
    }

    let pubkey_str = match unsafe { c_str_to_string(pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    match unsafe { (*client).get_balance(&pubkey_str) } {
        Ok(balance) => balance,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_get_latest_blockhash(
    client: *mut RpcClient,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if client.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null client pointer".to_string(),
                ));
            }
        }
        return ptr::null_mut();
    }

    match unsafe { (*client).get_latest_blockhash() } {
        Ok(blockhash) => match CString::new(blockhash) {
            Ok(c_blockhash) => c_blockhash.into_raw(),
            Err(e) => {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                            "Failed to convert blockhash to C string: {}",
                            e
                        )));
                    }
                }
                ptr::null_mut()
            }
        },
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

// Transaction functions

#[no_mangle]
pub extern "C" fn solana_create_transaction() -> *mut Transaction {
    Box::into_raw(Box::new(Transaction::new()))
}

#[no_mangle]
pub extern "C" fn solana_destroy_transaction(transaction: *mut Transaction) {
    if !transaction.is_null() {
        unsafe {
            let _ = Box::from_raw(transaction);
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_build_transfer(
    transaction: *mut Transaction,
    from_pubkey: *const c_char,
    to_pubkey: *const c_char,
    lamports: u64,
    recent_blockhash: *const c_char,
    error_out: *mut *mut c_char,
) -> c_int {
    if transaction.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null transaction pointer".to_string(),
                ));
            }
        }
        return 0;
    }

    let from_str = match unsafe { c_str_to_string(from_pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    let to_str = match unsafe { c_str_to_string(to_pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    let blockhash_str = match unsafe { c_str_to_string(recent_blockhash) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    match unsafe { (*transaction).build_transfer(&from_str, &to_str, lamports, &blockhash_str) } {
        Ok(_) => 1,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_sign_transaction(
    transaction: *mut Transaction,
    private_key_bytes: *const c_uchar,
    private_key_len: usize,
    error_out: *mut *mut c_char,
) -> c_int {
    if transaction.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null transaction pointer".to_string(),
                ));
            }
        }
        return 0;
    }

    let private_key = unsafe { slice::from_raw_parts(private_key_bytes, private_key_len) };

    match unsafe { (*transaction).sign(private_key) } {
        Ok(_) => 1,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_send_transaction(
    client: *mut RpcClient,
    transaction: *mut Transaction,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if client.is_null() || transaction.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null pointer(s) provided".to_string(),
                ));
            }
        }
        return ptr::null_mut();
    }

    // Get transaction
    let tx_result = unsafe { (*transaction).get_transaction() };
    let tx = match tx_result {
        Ok(tx) => tx,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    // Send the transaction
    match unsafe { (*client).send_transaction(tx) } {
        Ok(signature) => match CString::new(signature) {
            Ok(c_signature) => c_signature.into_raw(),
            Err(e) => {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                            "Failed to convert signature to C string: {}",
                            e
                        )));
                    }
                }
                ptr::null_mut()
            }
        },
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

// Account functions

#[no_mangle]
pub extern "C" fn solana_create_account() -> *mut Account {
    Box::into_raw(Box::new(Account::new()))
}

#[no_mangle]
pub extern "C" fn solana_destroy_account(account: *mut Account) {
    if !account.is_null() {
        unsafe {
            let _ = Box::from_raw(account);
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_account_from_pubkey(
    pubkey: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut Account {
    let pubkey_str = match unsafe { c_str_to_string(pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    match Account::from_pubkey(&pubkey_str) {
        Ok(account) => Box::into_raw(Box::new(account)),
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_account_from_private_key(
    private_key_bytes: *const c_uchar,
    private_key_len: usize,
    error_out: *mut *mut c_char,
) -> *mut Account {
    let private_key = unsafe { slice::from_raw_parts(private_key_bytes, private_key_len) };

    match Account::from_private_key(private_key) {
        Ok(account) => Box::into_raw(Box::new(account)),
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_account_generate() -> *mut Account {
    Box::into_raw(Box::new(Account::generate()))
}

#[no_mangle]
pub extern "C" fn solana_account_get_pubkey(
    account: *const Account,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if account.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null account pointer".to_string(),
                ));
            }
        }
        return ptr::null_mut();
    }

    match unsafe { (*account).get_pubkey() } {
        Ok(pubkey) => match CString::new(pubkey) {
            Ok(c_pubkey) => c_pubkey.into_raw(),
            Err(e) => {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                            "Failed to convert pubkey to C string: {}",
                            e
                        )));
                    }
                }
                ptr::null_mut()
            }
        },
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

// Free C string (exported for Unity to clean up strings)
#[no_mangle]
pub extern "C" fn solana_free_string(ptr: *mut c_char) {
    unsafe {
        free_c_string(ptr);
    }
}

#[no_mangle]
pub extern "C" fn solana_build_token_transfer(
    transaction: *mut Transaction,
    token_program_id: *const c_char,
    source_pubkey: *const c_char,
    destination_pubkey: *const c_char,
    owner_pubkey: *const c_char,
    amount: u64,
    recent_blockhash: *const c_char,
    error_out: *mut *mut c_char,
) -> c_int {
    if transaction.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null transaction pointer".to_string(),
                ));
            }
        }
        return 0;
    }

    let token_program_str = match unsafe { c_str_to_string(token_program_id) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    let source_str = match unsafe { c_str_to_string(source_pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    let destination_str = match unsafe { c_str_to_string(destination_pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    let owner_str = match unsafe { c_str_to_string(owner_pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    let blockhash_str = match unsafe { c_str_to_string(recent_blockhash) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    match unsafe {
        (*transaction).build_token_transfer(
            &token_program_str,
            &source_str,
            &destination_str,
            &owner_str,
            amount,
            &blockhash_str,
        )
    } {
        Ok(_) => 1,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            0
        }
    }
}

#[cfg(feature = "bip39")]
#[no_mangle]
pub extern "C" fn solana_account_from_mnemonic(
    mnemonic: *const c_char,
    passphrase: *const c_char,
    derivation_path: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut Account {
    let mnemonic_str = match unsafe { c_str_to_string(mnemonic) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    let passphrase_str = match unsafe { c_str_to_string(passphrase) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    let path_str = match unsafe { c_str_to_string(derivation_path) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    match Account::from_mnemonic(&mnemonic_str, &passphrase_str, &path_str) {
        Ok(account) => Box::into_raw(Box::new(account)),
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_get_token_account_balance(
    client: *mut RpcClient,
    token_account: *const c_char,
    error_out: *mut *mut c_char,
) -> u64 {
    if client.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null client pointer".to_string(),
                ));
            }
        }
        return 0;
    }

    let token_account_str = match unsafe { c_str_to_string(token_account) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return 0;
        }
    };

    match unsafe { (*client).get_token_account_balance(&token_account_str) } {
        Ok(balance) => balance,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_get_account_info(
    client: *mut RpcClient,
    pubkey: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if client.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null client pointer".to_string(),
                ));
            }
        }
        return ptr::null_mut();
    }

    let pubkey_str = match unsafe { c_str_to_string(pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    match unsafe { (*client).get_account_info(&pubkey_str) } {
        Ok(info) => match CString::new(info) {
            Ok(c_info) => c_info.into_raw(),
            Err(e) => {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                            "Failed to convert account info to C string: {}",
                            e
                        )));
                    }
                }
                ptr::null_mut()
            }
        },
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_get_program_accounts(
    client: *mut RpcClient,
    program_id: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if client.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null client pointer".to_string(),
                ));
            }
        }
        return ptr::null_mut();
    }

    let program_id_str = match unsafe { c_str_to_string(program_id) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    match unsafe { (*client).get_program_accounts(&program_id_str) } {
        Ok(accounts) => match CString::new(accounts) {
            Ok(c_accounts) => c_accounts.into_raw(),
            Err(e) => {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                            "Failed to convert program accounts to C string: {}",
                            e
                        )));
                    }
                }
                ptr::null_mut()
            }
        },
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_get_transaction_status(
    client: *mut RpcClient,
    signature: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_char {
    if client.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null client pointer".to_string(),
                ));
            }
        }
        return ptr::null_mut();
    }

    let signature_str = match unsafe { c_str_to_string(signature) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return ptr::null_mut();
        }
    };

    match unsafe { (*client).get_transaction_status(&signature_str) } {
        Ok(status) => match CString::new(status) {
            Ok(c_status) => c_status.into_raw(),
            Err(e) => {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                            "Failed to convert transaction status to C string: {}",
                            e
                        )));
                    }
                }
                ptr::null_mut()
            }
        },
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            ptr::null_mut()
        }
    }
}
