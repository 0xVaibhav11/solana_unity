extern crate solana_unity;

use solana_unity::{Account, RpcClient, SolanaUnityError, Transaction};

// Real Solana RPC endpoint for testing
const TEST_RPC_URL: &str = "https://api.devnet.solana.com";

mod account_tests {
    use super::*;

    #[test]
    fn create_and_verify_account() {
        // Create a new account using our library
        let account = Account::generate();

        // Get the public key
        let pubkey = account.get_pubkey().unwrap();
        println!("Created account with pubkey: {}", pubkey);
        assert!(!pubkey.is_empty());

        // Verify we can get the private key
        let private_key = account.get_private_key().unwrap();
        assert_eq!(private_key.len(), 64); // Ed25519 keypair is 64 bytes

        // Verify we have a private key
        assert!(account.has_private_key());
    }

    #[test]
    fn restore_account_from_private_key() {
        // Create an original account
        let original_account = Account::generate();
        let original_pubkey = original_account.get_pubkey().unwrap();
        let private_key = original_account.get_private_key().unwrap();

        // Restore the account from private key
        let restored_account = Account::from_private_key(&private_key).unwrap();

        // Verify the public key matches
        let restored_pubkey = restored_account.get_pubkey().unwrap();
        assert_eq!(restored_pubkey, original_pubkey);

        // Verify the restored account has a private key
        assert!(restored_account.has_private_key());
    }

    #[test]
    fn create_account_from_pubkey() {
        // Create a read-only account from a public key
        let pubkey_str = "11111111111111111111111111111111";
        let account = Account::from_pubkey(pubkey_str).unwrap();

        // Verify the pubkey was stored correctly
        assert_eq!(account.get_pubkey().unwrap(), pubkey_str);

        // Verify it's read-only (no private key)
        assert!(!account.has_private_key());
        assert!(account.get_private_key().is_err());
    }
}

mod rpc_tests {
    use super::*;

    #[test]
    fn connect_to_rpc_endpoint() {
        // Create a connection using our RPC client
        let rpc_client = RpcClient::new(TEST_RPC_URL, "confirmed").unwrap();

        // Get blockhash to verify connection works
        let blockhash = rpc_client.get_latest_blockhash();
        assert!(blockhash.is_ok(), "Failed to get latest blockhash");

        let blockhash_str = blockhash.unwrap();
        println!("Got real blockhash: {}", blockhash_str);
        assert!(!blockhash_str.is_empty());
    }

    #[test]
    fn get_account_details() {
        // Create our RPC client
        let rpc_client = RpcClient::new(TEST_RPC_URL, "confirmed").unwrap();

        // Query system program account
        let system_program_id = "11111111111111111111111111111111";

        // Get account info as JSON
        let account_info = rpc_client.get_account_info(system_program_id);
        assert!(account_info.is_ok(), "Failed to get system program account");

        let account_json = account_info.unwrap();
        println!("Got account info: {}", account_json);

        // Verify we got some data back
        assert!(!account_json.is_empty());

        // Validate it's proper JSON
        let json_result = serde_json::from_str::<serde_json::Value>(&account_json);
        assert!(json_result.is_ok(), "Invalid JSON returned");
    }

    #[test]
    fn get_balance() {
        // Create our RPC client
        let rpc_client = RpcClient::new(TEST_RPC_URL, "confirmed").unwrap();

        // Create a new account
        let account = Account::generate();
        let pubkey = account.get_pubkey().unwrap();

        // The account won't have any balance, but the call should succeed
        let balance = rpc_client.get_balance(&pubkey);
        assert!(balance.is_ok(), "Failed to get balance");

        // Balance should be zero or very small
        let lamports = balance.unwrap();
        println!("Account balance: {} lamports", lamports);

        // New account should have 0 balance
        assert_eq!(lamports, 0);
    }
}

mod transaction_tests {
    use super::*;

    #[test]
    fn build_and_serialize_transaction() {
        // Create our RPC client to get a real blockhash
        let rpc_client = RpcClient::new(TEST_RPC_URL, "confirmed").unwrap();
        let blockhash = rpc_client.get_latest_blockhash().unwrap();

        // Create sender and recipient accounts
        let from_account = Account::generate();
        let from_pubkey = from_account.get_pubkey().unwrap();
        let to_account = Account::generate();
        let to_pubkey = to_account.get_pubkey().unwrap();
        let lamports = 1000;

        // Build a transfer transaction
        let mut tx = Transaction::new();
        let result = tx.build_transfer(&from_pubkey, &to_pubkey, lamports, &blockhash);
        assert!(result.is_ok(), "Failed to build transfer transaction");

        // Sign the transaction
        let private_key = from_account.get_private_key().unwrap();
        let sign_result = tx.sign(&private_key);
        assert!(sign_result.is_ok(), "Failed to sign transaction");

        // Serialize the transaction
        let serialized = tx.serialize();
        assert!(serialized.is_ok(), "Failed to serialize transaction");

        let serialized_data = serialized.unwrap();
        assert!(
            !serialized_data.is_empty(),
            "Serialized transaction is empty"
        );

        // Deserialize the transaction
        let mut new_tx = Transaction::new();
        let deserialize_result = new_tx.from_serialized(&serialized_data);
        assert!(
            deserialize_result.is_ok(),
            "Failed to deserialize transaction"
        );

        println!("Successfully built, signed, and serialized a real transaction");
    }

    #[test]
    fn build_token_transfer() {
        // Create our RPC client to get a real blockhash
        let rpc_client = RpcClient::new(TEST_RPC_URL, "confirmed").unwrap();
        let blockhash = rpc_client.get_latest_blockhash().unwrap();

        // Create sender, recipient, and owner accounts
        let from_account = Account::generate();
        let from_pubkey = from_account.get_pubkey().unwrap();
        let to_account = Account::generate();
        let to_pubkey = to_account.get_pubkey().unwrap();
        let owner_account = Account::generate();
        let owner_pubkey = owner_account.get_pubkey().unwrap();
        let amount = 1000;

        // Build a token transfer transaction
        let mut tx = Transaction::new();
        let result = tx.build_token_transfer(
            "", // Use default token program
            &from_pubkey,
            &to_pubkey,
            &owner_pubkey,
            amount,
            &blockhash,
        );
        assert!(result.is_ok(), "Failed to build token transfer transaction");

        // Sign the transaction
        let private_key = owner_account.get_private_key().unwrap();
        let sign_result = tx.sign(&private_key);
        assert!(sign_result.is_ok(), "Failed to sign transaction");

        println!("Successfully built and signed a token transfer transaction");
    }

    #[test]
    fn sign_and_submit_transaction() {
        // Create our RPC client to get a real blockhash
        let rpc_client = RpcClient::new(TEST_RPC_URL, "confirmed").unwrap();
        let blockhash = rpc_client.get_latest_blockhash().unwrap();

        // Create sender and recipient accounts
        let from_account = Account::generate();
        let from_pubkey = from_account.get_pubkey().unwrap();
        let to_account = Account::generate();
        let to_pubkey = to_account.get_pubkey().unwrap();
        let lamports = 1000; // We don't actually have funds, so this will fail on network, but API test should work

        // Build a transfer transaction
        let mut tx = Transaction::new();
        let result = tx.build_transfer(&from_pubkey, &to_pubkey, lamports, &blockhash);
        assert!(result.is_ok(), "Failed to build transfer transaction");

        // Sign the transaction
        let private_key = from_account.get_private_key().unwrap();
        let sign_result = tx.sign(&private_key);
        assert!(sign_result.is_ok(), "Failed to sign transaction");

        // Note: We expect this to fail on the network since we're using a newly generated account
        // with no funds, but our API should handle it correctly
        let submit_result = rpc_client.send_transaction(tx.get_transaction().unwrap());
        match submit_result {
            Ok(signature) => {
                println!("Transaction submitted with signature: {}", signature);
                // This would be unexpected since our account has no funds
                assert!(!signature.is_empty(), "Empty signature returned");
            }
            Err(e) => {
                // Expected error about insufficient funds
                println!("Expected error (no funds): {}", e);
                match e {
                    SolanaUnityError::RpcError(_) => {
                        // This is expected - newly created account has no SOL
                    }
                    _ => {
                        panic!("Unexpected error type: {:?}", e);
                    }
                }
            }
        }

        println!("Transaction API tested successfully");
    }
}
