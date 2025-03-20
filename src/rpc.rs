use serde_json::Value;
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
}
