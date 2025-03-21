mod account;
mod error;
mod ffi;
mod rpc;
mod transaction;

pub use account::Account;
pub use error::SolanaUnityError;
pub use rpc::RpcClient;
pub use transaction::Transaction;

// Re-export the FFI functions for use in Unity
pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;
    use solana_sdk::signer::Signer;
    

    const TEST_URL: &str = "https://api.devnet.solana.com";

    #[test]
    fn test_account_creation() {
        let account = Account::new();
        assert!(account.get_pubkey().is_err());
        assert!(!account.has_private_key());

        let account = Account::generate();
        assert!(account.get_pubkey().is_ok());
        assert!(account.has_private_key());

        let pubkey = account.get_pubkey().unwrap();
        assert!(!pubkey.is_empty());
    }

    #[test]
    fn test_account_from_pubkey() {
        let keypair = Keypair::new();
        let pubkey_str = keypair.pubkey().to_string();

        let account = Account::from_pubkey(&pubkey_str).unwrap();
        assert!(account.get_pubkey().is_ok());
        assert!(!account.has_private_key());

        let recovered_pubkey = account.get_pubkey().unwrap();
        assert_eq!(recovered_pubkey, pubkey_str);
    }

    #[test]
    fn test_account_from_private_key() {
        let keypair = Keypair::new();
        let pubkey_str = keypair.pubkey().to_string();
        let private_key = keypair.to_bytes();

        let account = Account::from_private_key(&private_key).unwrap();
        assert!(account.get_pubkey().is_ok());
        assert!(account.has_private_key());

        let recovered_pubkey = account.get_pubkey().unwrap();
        assert_eq!(recovered_pubkey, pubkey_str);
    }

    #[test]
    fn test_transaction_basics() {
        let tx = Transaction::new();
        assert!(tx.serialize().is_err());
        assert!(tx.get_transaction().is_err());
    }
}
