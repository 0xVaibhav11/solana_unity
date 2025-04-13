use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar};
use std::ptr;
use std::slice;

use crate::account::Account;
use crate::error::{error_to_c_string, free_c_string, SolanaUnityError};
use crate::instruction::{InstructionBuilder, TokenInstructions};
use crate::pda::ProgramDerivedAddress;
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

// Add new FFI functions for PDA

#[no_mangle]
pub extern "C" fn solana_find_program_address(
    seeds_ptr: *const *const c_char,
    seeds_len: usize,
    program_id: *const c_char,
    address_out: *mut *mut c_char,
    bump_out: *mut u8,
    error_out: *mut *mut c_char,
) -> c_int {
    if seeds_ptr.is_null() || program_id.is_null() || address_out.is_null() || bump_out.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null pointer(s) provided".to_string(),
                ));
            }
        }
        return 0;
    }

    let program_id_str = match unsafe { c_str_to_string(program_id) } {
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

    // Convert C array of strings to Rust Vec of byte arrays
    let mut seeds_vec = Vec::with_capacity(seeds_len);
    for i in 0..seeds_len {
        let seed_ptr = unsafe { *seeds_ptr.add(i) };
        let seed_str = match unsafe { c_str_to_string(seed_ptr) } {
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
        seeds_vec.push(seed_str.into_bytes());
    }

    // Convert Vec<Vec<u8>> to Vec<&[u8]>
    let seeds_slice: Vec<&[u8]> = seeds_vec.iter().map(|s| s.as_slice()).collect();

    match ProgramDerivedAddress::find_program_address(&seeds_slice, &program_id_str) {
        Ok((address, bump)) => {
            // Set the output address
            match CString::new(address) {
                Ok(c_address) => unsafe {
                    *address_out = c_address.into_raw();
                    *bump_out = bump;
                },
                Err(e) => {
                    if !error_out.is_null() {
                        unsafe {
                            *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                                "Failed to convert address to C string: {}",
                                e
                            )));
                        }
                    }
                    return 0;
                }
            }
            1
        }
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
pub extern "C" fn solana_find_associated_token_address(
    wallet_address: *const c_char,
    token_mint: *const c_char,
    address_out: *mut *mut c_char,
    error_out: *mut *mut c_char,
) -> c_int {
    if wallet_address.is_null() || token_mint.is_null() || address_out.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null pointer(s) provided".to_string(),
                ));
            }
        }
        return 0;
    }

    let wallet_str = match unsafe { c_str_to_string(wallet_address) } {
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

    let mint_str = match unsafe { c_str_to_string(token_mint) } {
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

    match ProgramDerivedAddress::find_associated_token_address(&wallet_str, &mint_str) {
        Ok(address) => {
            // Set the output address
            match CString::new(address) {
                Ok(c_address) => unsafe {
                    *address_out = c_address.into_raw();
                },
                Err(e) => {
                    if !error_out.is_null() {
                        unsafe {
                            *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                                "Failed to convert address to C string: {}",
                                e
                            )));
                        }
                    }
                    return 0;
                }
            }
            1
        }
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

// Add simulation function

#[no_mangle]
pub extern "C" fn solana_simulate_transaction(
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

    // Simulate the transaction
    match unsafe { (*client).simulate_transaction(tx) } {
        Ok(result) => match CString::new(result) {
            Ok(c_result) => c_result.into_raw(),
            Err(e) => {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::FfiError(format!(
                            "Failed to convert simulation result to C string: {}",
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

// Add instruction functions

#[no_mangle]
pub extern "C" fn solana_create_token_transfer_instruction(
    source: *const c_char,
    destination: *const c_char,
    owner: *const c_char,
    amount: u64,
    encoded_data_out: *mut *mut c_uchar,
    encoded_data_len_out: *mut usize,
    error_out: *mut *mut c_char,
) -> c_int {
    if source.is_null()
        || destination.is_null()
        || owner.is_null()
        || encoded_data_out.is_null()
        || encoded_data_len_out.is_null()
    {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null pointer(s) provided".to_string(),
                ));
            }
        }
        return 0;
    }

    let source_str = match unsafe { c_str_to_string(source) } {
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

    let destination_str = match unsafe { c_str_to_string(destination) } {
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

    let owner_str = match unsafe { c_str_to_string(owner) } {
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

    match TokenInstructions::transfer(&source_str, &destination_str, &owner_str, amount) {
        Ok(instruction) => {
            // Encode the instruction for returning to C#
            let encoded = match bincode::serialize(&instruction) {
                Ok(data) => data,
                Err(e) => {
                    if !error_out.is_null() {
                        unsafe {
                            *error_out = error_to_c_string(&SolanaUnityError::SerializationError(
                                format!("Failed to serialize instruction: {}", e),
                            ));
                        }
                    }
                    return 0;
                }
            };

            // Allocate memory for the instruction data
            let data_len = encoded.len();
            let data_ptr = unsafe { libc::malloc(data_len) } as *mut c_uchar;
            if data_ptr.is_null() {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                            "Failed to allocate memory for instruction data".to_string(),
                        ));
                    }
                }
                return 0;
            }

            // Copy the data
            unsafe {
                std::ptr::copy_nonoverlapping(encoded.as_ptr(), data_ptr, data_len);
                *encoded_data_out = data_ptr;
                *encoded_data_len_out = data_len;
            }

            1
        }
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
pub extern "C" fn solana_free_encoded_instruction(data_ptr: *mut c_uchar) {
    if !data_ptr.is_null() {
        unsafe {
            libc::free(data_ptr as *mut libc::c_void);
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_build_with_instructions(
    transaction: *mut Transaction,
    instructions_data: *const c_uchar,
    instructions_data_len: usize,
    instructions_count: usize,
    fee_payer: *const c_char,
    recent_blockhash: *const c_char,
    error_out: *mut *mut c_char,
) -> c_int {
    if transaction.is_null()
        || instructions_data.is_null()
        || fee_payer.is_null()
        || recent_blockhash.is_null()
    {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null pointer(s) provided".to_string(),
                ));
            }
        }
        return 0;
    }

    let fee_payer_str = match unsafe { c_str_to_string(fee_payer) } {
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

    // Deserialize the instructions
    let instructions_bytes =
        unsafe { slice::from_raw_parts(instructions_data, instructions_data_len) };
    let instructions: Vec<solana_sdk::instruction::Instruction> =
        match bincode::deserialize(instructions_bytes) {
            Ok(insts) => insts,
            Err(e) => {
                if !error_out.is_null() {
                    unsafe {
                        *error_out = error_to_c_string(&SolanaUnityError::SerializationError(
                            format!("Failed to deserialize instructions: {}", e),
                        ));
                    }
                }
                return 0;
            }
        };

    // Build the transaction
    match unsafe {
        (*transaction).build_with_instructions(&instructions, &fee_payer_str, &blockhash_str)
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

// Add multiple signatures support
#[no_mangle]
pub extern "C" fn solana_sign_transaction_with_keypairs(
    transaction: *mut Transaction,
    private_keys_data: *const *const c_uchar,
    private_keys_lengths: *const usize,
    private_keys_count: usize,
    error_out: *mut *mut c_char,
) -> c_int {
    if transaction.is_null() || private_keys_data.is_null() || private_keys_lengths.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null pointer(s) provided".to_string(),
                ));
            }
        }
        return 0;
    }

    // Convert C array of byte arrays to Rust Vec of &[u8]
    let mut private_keys = Vec::with_capacity(private_keys_count);
    for i in 0..private_keys_count {
        let key_ptr = unsafe { *private_keys_data.add(i) };
        let key_len = unsafe { *private_keys_lengths.add(i) };
        let key_slice = unsafe { slice::from_raw_parts(key_ptr, key_len) };
        private_keys.push(key_slice);
    }
    // Sign the transaction
    let key_slices: Vec<&[u8]> = private_keys.iter().map(|k| *k).collect();
    match unsafe { (*transaction).sign_with_keypairs(&key_slices) } {
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
pub extern "C" fn solana_account_get_private_key(
    account: *mut Account,
    error_out: *mut *mut c_char,
) -> *mut c_uchar {
    if account.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null account pointer".to_string(),
                ));
            }
        }
        return std::ptr::null_mut();
    }

    match unsafe { (*account).get_private_key() } {
        Ok(private_key) => {
            let len = private_key.len();
            let ptr = unsafe { libc::malloc(len) as *mut c_uchar };
            if !ptr.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(private_key.as_ptr(), ptr, len);
                }
            }
            ptr
        }
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_account_has_private_key(
    account: *mut Account,
    error_out: *mut *mut c_char,
) -> c_int {
    if account.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null account pointer".to_string(),
                ));
            }
        }
        return 0;
    }

    unsafe { (*account).has_private_key() as c_int }
}

#[no_mangle]
pub extern "C" fn solana_account_get_keypair(
    account: *mut Account,
    error_out: *mut *mut c_char,
) -> *mut std::os::raw::c_void {
    if account.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null account pointer".to_string(),
                ));
            }
        }
        return std::ptr::null_mut();
    }

    match unsafe { (*account).get_keypair() } {
        Ok(keypair) => {
            // Convert the keypair reference to a raw pointer
            keypair as *const _ as *mut std::os::raw::c_void
        }
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_build_program_call(
    transaction: *mut Transaction,
    program_id: *const c_char,
    accounts: *const *const c_char,
    accounts_is_signer: *const c_int,
    accounts_is_writable: *const c_int,
    accounts_count: usize,
    data: *const c_uchar,
    data_len: usize,
    recent_blockhash: *const c_char,
    fee_payer: *const c_char,
    error_out: *mut *mut c_char,
) -> c_int {
    if transaction.is_null()
        || program_id.is_null()
        || accounts.is_null()
        || data.is_null()
        || recent_blockhash.is_null()
        || fee_payer.is_null()
    {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null pointer(s) provided".to_string(),
                ));
            }
        }
        return 0;
    }

    let program_id_str = match unsafe { c_str_to_string(program_id) } {
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

    let fee_payer_str = match unsafe { c_str_to_string(fee_payer) } {
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

    // Convert C arrays to Rust Vec
    let mut accounts_vec = Vec::with_capacity(accounts_count);
    for i in 0..accounts_count {
        let account_ptr = unsafe { *accounts.add(i) };
        let account_str = match unsafe { c_str_to_string(account_ptr) } {
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
        let is_signer = unsafe { *accounts_is_signer.add(i) } != 0;
        let is_writable = unsafe { *accounts_is_writable.add(i) } != 0;
        accounts_vec.push((account_str, is_signer, is_writable));
    }

    // Convert data to Vec<u8>
    let data_vec = unsafe { slice::from_raw_parts(data, data_len) }.to_vec();

    match unsafe {
        (*transaction).build_program_call(
            &program_id_str,
            accounts_vec,
            data_vec,
            &blockhash_str,
            &fee_payer_str,
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

#[no_mangle]
pub extern "C" fn solana_get_account_data(
    client: *mut RpcClient,
    pubkey: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut c_uchar {
    if client.is_null() {
        if !error_out.is_null() {
            unsafe {
                *error_out = error_to_c_string(&SolanaUnityError::FfiError(
                    "Null client pointer".to_string(),
                ));
            }
        }
        return std::ptr::null_mut();
    }

    let pubkey_str = match unsafe { c_str_to_string(pubkey) } {
        Ok(s) => s,
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            return std::ptr::null_mut();
        }
    };

    match unsafe { (*client).get_account_data(&pubkey_str) } {
        Ok(data) => {
            let len = data.len();
            let ptr = unsafe { libc::malloc(len) as *mut c_uchar };
            if !ptr.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, len);
                }
            }
            ptr
        }
        Err(e) => {
            if !error_out.is_null() {
                unsafe {
                    *error_out = error_to_c_string(&e);
                }
            }
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn solana_confirm_transaction(
    client: *mut RpcClient,
    signature: *const c_char,
    error_out: *mut *mut c_char,
) -> c_int {
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

    let signature_str = match unsafe { c_str_to_string(signature) } {
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

    match unsafe { (*client).confirm_transaction(&signature_str) } {
        Ok(confirmed) => confirmed as c_int,
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
