pub mod account;
pub mod error;
pub mod ffi;
pub mod instruction;
pub mod pda;
pub mod rpc;
pub mod transaction;

pub use account::Account;
pub use error::SolanaUnityError;
pub use instruction::{InstructionBuilder, TokenInstructions};
pub use pda::ProgramDerivedAddress;
pub use rpc::RpcClient;
pub use transaction::Transaction;

// Re-export the FFI functions for use in Unity
pub use ffi::*;

#[cfg(test)]
mod tests {
    // Unit tests for library integration
    use crate::account::Account;
    use crate::instruction::TokenInstructions;
    use crate::pda::ProgramDerivedAddress;
    use crate::transaction::Transaction;

    const TEST_URL: &str = "https://api.devnet.solana.com";

    #[test]
    fn test_account_creation() {
        let account = Account::generate();
        assert!(account.has_private_key());
        let pubkey = account.get_pubkey();
        assert!(pubkey.is_ok());
    }

    #[test]
    fn test_account_from_pubkey() {
        let pubkey_str = "11111111111111111111111111111111";
        let account = Account::from_pubkey(pubkey_str);
        assert!(account.is_ok());
    }

    #[test]
    fn test_account_from_private_key() {
        let original_account = Account::generate();
        let private_key = original_account.get_private_key().unwrap();
        let restored_account = Account::from_private_key(&private_key);
        assert!(restored_account.is_ok());
    }

    #[test]
    fn test_transaction_basics() {
        let mut tx = Transaction::new();
        assert!(tx.get_transaction().is_err()); // No transaction created yet

        // For more comprehensive tests, see transaction module tests
    }

    #[test]
    fn test_pda_and_instruction_integration() {
        // Generate test accounts
        let account1 = Account::generate();
        let account2 = Account::generate();
        let pubkey1 = account1.get_pubkey().unwrap();
        let pubkey2 = account2.get_pubkey().unwrap();

        // Test token transfer instruction
        let result = TokenInstructions::transfer(&pubkey1, &pubkey2, &pubkey1, 1000);
        assert!(result.is_ok());

        // Test PDA
        let program_id = "11111111111111111111111111111111"; // System program
        let seeds = &[b"test", pubkey1.as_bytes()];
        let result = ProgramDerivedAddress::find_program_address(seeds, program_id);
        assert!(result.is_ok());
    }
}
