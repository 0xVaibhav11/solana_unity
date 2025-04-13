use solana_client::rpc_client::RpcClient as SolanaRpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcSendTransactionConfig};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::Transaction as SolanaTransaction;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;
use std::sync::Arc;

use crate::error::SolanaUnityError;

pub struct RpcClient {
    client: Arc<SolanaRpcClient>,
    commitment: CommitmentConfig,
}

impl RpcClient {
    pub fn new(url: &str, commitment: &str) -> Result<Self, SolanaUnityError> {
        let commitment = match commitment {
            "processed" => CommitmentConfig::processed(),
            "confirmed" => CommitmentConfig::confirmed(),
            "finalized" => CommitmentConfig::finalized(),
            _ => CommitmentConfig::default(),
        };

        let client = SolanaRpcClient::new_with_commitment(url.to_string(), commitment);
        Ok(Self {
            client: Arc::new(client),
            commitment,
        })
    }

    pub fn get_balance(&self, pubkey_str: &str) -> Result<u64, SolanaUnityError> {
        let pubkey = solana_sdk::pubkey::Pubkey::from_str(pubkey_str)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid pubkey: {}", e)))?;

        self.client
            .get_balance(&pubkey)
            .map_err(|e| SolanaUnityError::RpcError(e.to_string()))
    }

    pub fn get_latest_blockhash(&self) -> Result<String, SolanaUnityError> {
        let blockhash = self
            .client
            .get_latest_blockhash()
            .map_err(|e| SolanaUnityError::RpcError(e.to_string()))?;

        Ok(blockhash.to_string())
    }

    pub fn send_transaction(
        &self,
        transaction: &SolanaTransaction,
    ) -> Result<String, SolanaUnityError> {
        let config = RpcSendTransactionConfig {
            skip_preflight: false,
            preflight_commitment: Some(self.commitment.commitment),
            encoding: None,
            max_retries: None,
            min_context_slot: None,
        };

        self.client
            .send_transaction_with_config(transaction, config)
            .map_err(|e| SolanaUnityError::RpcError(e.to_string()))
            .map(|sig| sig.to_string())
    }

    pub fn get_account_data(&self, pubkey_str: &str) -> Result<Vec<u8>, SolanaUnityError> {
        let pubkey = solana_sdk::pubkey::Pubkey::from_str(pubkey_str)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid pubkey: {}", e)))?;

        let account = self
            .client
            .get_account_with_commitment(&pubkey, self.commitment)
            .map_err(|e| SolanaUnityError::RpcError(e.to_string()))?
            .value
            .ok_or_else(|| SolanaUnityError::RpcError("Account not found".to_string()))?;

        Ok(account.data.clone())
    }

    pub fn confirm_transaction(&self, signature_str: &str) -> Result<bool, SolanaUnityError> {
        let signature = Signature::from_str(signature_str)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid signature: {}", e)))?;

        self.client
            .confirm_transaction(&signature)
            .map_err(|e| SolanaUnityError::RpcError(e.to_string()))
    }

    // Get token account balance
    pub fn get_token_account_balance(&self, token_account: &str) -> Result<u64, SolanaUnityError> {
        let pubkey = solana_sdk::pubkey::Pubkey::from_str(token_account)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid pubkey: {}", e)))?;

        let token_balance = self
            .client
            .get_token_account_balance(&pubkey)
            .map_err(|e| {
                SolanaUnityError::RpcError(format!("Failed to get token balance: {}", e))
            })?;

        // Parse the UI amount string to lamports
        match token_balance.amount.parse::<u64>() {
            Ok(amount) => Ok(amount),
            Err(e) => Err(SolanaUnityError::RpcError(format!(
                "Failed to parse token amount: {}",
                e
            ))),
        }
    }

    // Get account info
    pub fn get_account_info(&self, pubkey_str: &str) -> Result<String, SolanaUnityError> {
        let pubkey = solana_sdk::pubkey::Pubkey::from_str(pubkey_str)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid pubkey: {}", e)))?;

        let config = RpcAccountInfoConfig {
            encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
            commitment: Some(self.commitment),
            data_slice: None,
            min_context_slot: None,
        };

        let account = self
            .client
            .get_account_with_config(&pubkey, config)
            .map_err(|e| SolanaUnityError::RpcError(e.to_string()))?
            .value
            .ok_or_else(|| SolanaUnityError::RpcError("Account not found".to_string()))?;

        // Convert account to JSON
        let json = serde_json::to_string(&account).map_err(|e| {
            SolanaUnityError::SerializationError(format!("Failed to serialize account: {}", e))
        })?;

        Ok(json)
    }

    // Get program accounts
    pub fn get_program_accounts(&self, program_id: &str) -> Result<String, SolanaUnityError> {
        let pubkey = solana_sdk::pubkey::Pubkey::from_str(program_id)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid program ID: {}", e)))?;

        let accounts = self.client.get_program_accounts(&pubkey).map_err(|e| {
            SolanaUnityError::RpcError(format!("Failed to get program accounts: {}", e))
        })?;

        // Convert to JSON
        let json = serde_json::to_string(&accounts).map_err(|e| {
            SolanaUnityError::SerializationError(format!("Failed to serialize accounts: {}", e))
        })?;

        Ok(json)
    }

    // Get transaction status
    pub fn get_transaction_status(&self, signature_str: &str) -> Result<String, SolanaUnityError> {
        let signature = Signature::from_str(signature_str)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid signature: {}", e)))?;

        let tx_status = self
            .client
            .get_transaction(&signature, UiTransactionEncoding::Json)
            .map_err(|e| SolanaUnityError::RpcError(format!("Failed to get transaction: {}", e)))?;

        // Convert to JSON
        let json = serde_json::to_string(&tx_status).map_err(|e| {
            SolanaUnityError::SerializationError(format!("Failed to serialize transaction: {}", e))
        })?;

        Ok(json)
    }

    // Add transaction simulation method
    pub fn simulate_transaction(
        &self,
        transaction: &SolanaTransaction,
    ) -> Result<String, SolanaUnityError> {
        let config = solana_client::rpc_config::RpcSimulateTransactionConfig {
            sig_verify: false,
            replace_recent_blockhash: false,
            commitment: Some(self.commitment),
            encoding: Some(solana_transaction_status::UiTransactionEncoding::Json),
            accounts: None,
            min_context_slot: None,
            inner_instructions: true,
        };

        let result = self
            .client
            .simulate_transaction_with_config(transaction, config)
            .map_err(|e| SolanaUnityError::RpcError(format!("Simulation failed: {}", e)))?;

        // Convert to JSON
        let json = serde_json::to_string(&result).map_err(|e| {
            SolanaUnityError::SerializationError(format!(
                "Failed to serialize simulation result: {}",
                e
            ))
        })?;

        Ok(json)
    }

    // Add method to get multiple accounts
    pub fn get_multiple_accounts(&self, pubkeys: &[&str]) -> Result<String, SolanaUnityError> {
        // Convert pubkey strings to Pubkey objects
        let mut pubkey_objects = Vec::with_capacity(pubkeys.len());
        for pubkey_str in pubkeys {
            let pubkey = solana_sdk::pubkey::Pubkey::from_str(pubkey_str)
                .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid pubkey: {}", e)))?;
            pubkey_objects.push(pubkey);
        }

        // Get accounts
        let accounts = self
            .client
            .get_multiple_accounts(&pubkey_objects)
            .map_err(|e| SolanaUnityError::RpcError(format!("Failed to get accounts: {}", e)))?;

        // Convert to JSON
        let json = serde_json::to_string(&accounts).map_err(|e| {
            SolanaUnityError::SerializationError(format!("Failed to serialize accounts: {}", e))
        })?;

        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    // For more comprehensive tests, we should use mockall
    // Let's create a set of tests that don't require network connectivity

    #[test]
    fn test_create_client() {
        let url = "https://api.devnet.solana.com";

        let client = RpcClient::new(url, "confirmed");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.commitment, CommitmentConfig::confirmed());

        let client = RpcClient::new(url, "processed");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.commitment, CommitmentConfig::processed());

        let client = RpcClient::new(url, "finalized");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.commitment, CommitmentConfig::finalized());

        // Test invalid commitment level falls back to default
        let client = RpcClient::new(url, "invalid");
        assert!(client.is_ok());
    }

    #[test]
    fn test_invalid_pubkey() {
        let url = "https://api.devnet.solana.com";
        let client = RpcClient::new(url, "confirmed").unwrap();

        // Test a variety of invalid pubkeys
        let invalid_pubkeys = [
            "",
            "not-a-valid-pubkey",
            "tooshort",
            "TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG",
        ];

        for &invalid_pubkey in &invalid_pubkeys {
            // Test get_balance
            let result = client.get_balance(invalid_pubkey);
            assert!(
                result.is_err(),
                "Expected error for invalid pubkey: {}",
                invalid_pubkey
            );

            match result {
                Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
                _ => panic!(
                    "Expected InvalidInput error for pubkey in get_balance: {}",
                    invalid_pubkey
                ),
            }

            // Test get_account_data
            let result = client.get_account_data(invalid_pubkey);
            assert!(result.is_err());

            match result {
                Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
                _ => panic!("Expected InvalidInput error for pubkey in get_account_data"),
            }

            // Test get_token_account_balance
            let result = client.get_token_account_balance(invalid_pubkey);
            assert!(result.is_err());

            match result {
                Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
                _ => panic!("Expected InvalidInput error for pubkey in get_token_account_balance"),
            }

            // Test get_account_info
            let result = client.get_account_info(invalid_pubkey);
            assert!(result.is_err());

            match result {
                Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
                _ => panic!("Expected InvalidInput error for pubkey in get_account_info"),
            }
        }
    }

    #[test]
    fn test_invalid_signature() {
        let url = "https://api.devnet.solana.com";
        let client = RpcClient::new(url, "confirmed").unwrap();

        // Test a variety of invalid signatures
        let invalid_signatures = [
            "",
            "not-a-valid-signature",
            "tooshort",
            "TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG",
        ];

        for &invalid_sig in &invalid_signatures {
            // Test confirm_transaction
            let result = client.confirm_transaction(invalid_sig);
            assert!(result.is_err());

            match result {
                Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
                _ => panic!("Expected InvalidInput error for signature in confirm_transaction"),
            }

            // Test get_transaction_status
            let result = client.get_transaction_status(invalid_sig);
            assert!(result.is_err());

            match result {
                Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
                _ => panic!("Expected InvalidInput error for signature in get_transaction_status"),
            }
        }
    }

    #[test]
    fn test_client_methods_validation() {
        let url = "https://api.devnet.solana.com";
        let client = RpcClient::new(url, "confirmed").unwrap();

        // Valid pubkey for testing
        let valid_pubkey = Pubkey::new_unique().to_string();

        // We can't test the actual RPC calls without a mock, but we can test that the
        // validation part of our methods work correctly

        // For get_program_accounts
        let result = client.get_program_accounts("not-a-valid-program-id");
        assert!(result.is_err());

        match result {
            Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
            _ => panic!("Expected InvalidInput error for program ID"),
        }
    }

    // Real network connectivity tests
    #[test]
    fn test_get_balance_with_connection() {
        let url = "https://api.devnet.solana.com";
        let client = RpcClient::new(url, "confirmed").unwrap();

        // Known account with balance - replace with a real Solana account if testing
        let pubkey = "Ey9yot9JRj8RDjrTk1nxES1EA5Pig7PUMNhtC2xpxuPr";

        let result = client.get_balance(pubkey);
        assert!(result.is_ok());

        let balance = result.unwrap();
        assert!(balance >= 0); // Balance should be non-negative
        println!("Account balance: {} lamports", balance);
    }

    #[test]
    fn test_get_latest_blockhash_with_connection() {
        let url = "https://api.devnet.solana.com";
        let client = RpcClient::new(url, "confirmed").unwrap();

        let result = client.get_latest_blockhash();
        assert!(result.is_ok());

        let blockhash = result.unwrap();
        assert!(!blockhash.is_empty());
        println!("Latest blockhash: {}", blockhash);

        // Blockhash should be 32 bytes encoded as base58, typically around 44 chars
        assert!(blockhash.len() >= 32);
    }

    #[test]
    fn test_get_account_info_with_connection() {
        let url = "https://api.devnet.solana.com";
        let client = RpcClient::new(url, "confirmed").unwrap();

        // Solana token program is a well-known account that should always exist
        let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

        let result = client.get_account_info(token_program);
        assert!(result.is_ok());

        let account_info = result.unwrap();
        assert!(!account_info.is_empty());

        // Verify it's valid JSON
        let json_result = serde_json::from_str::<serde_json::Value>(&account_info);
        assert!(json_result.is_ok());

        println!("Account info retrieved successfully");
    }

    #[test]
    fn test_get_program_accounts_with_connection() {
        let url = "https://api.devnet.solana.com";
        let client = RpcClient::new(url, "confirmed").unwrap();

        // System program is a well-known program that should always exist and have accounts
        let system_program = "11111111111111111111111111111111";

        let result = client.get_program_accounts(system_program);

        // This might be slow and return a lot of accounts, so we'll accept either:
        // - Success (some accounts found)
        // - RPC error due to timeout or response too large
        match result {
            Ok(accounts_json) => {
                assert!(!accounts_json.is_empty());
                println!("Program accounts retrieved successfully");
            }
            Err(e) => {
                println!(
                    "Note: System program has too many accounts, expected error: {}",
                    e
                );
                // This is acceptable - system program has millions of accounts
                match e {
                    SolanaUnityError::RpcError(_) => {} // Expected
                    _ => panic!("Unexpected error type: {:?}", e),
                }
            }
        }
    }
}
