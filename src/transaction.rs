use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction as SolanaTransaction;
use std::str::FromStr;

use crate::error::SolanaUnityError;

// Add SPL token program ID
const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

pub struct Transaction {
    tx: Option<SolanaTransaction>,
}

impl Transaction {
    pub fn new() -> Self {
        Self { tx: None }
    }

    // Build a transfer transaction
    pub fn build_transfer(
        &mut self,
        from_pubkey: &str,
        to_pubkey: &str,
        lamports: u64,
        recent_blockhash: &str,
    ) -> Result<(), SolanaUnityError> {
        let from = Pubkey::from_str(from_pubkey)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid from pubkey: {}", e)))?;

        let to = Pubkey::from_str(to_pubkey)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid to pubkey: {}", e)))?;

        let instruction = solana_sdk::system_instruction::transfer(&from, &to, lamports);

        let blockhash = Hash::from_str(recent_blockhash)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid blockhash: {}", e)))?;

        let message = Message::new(&[instruction], Some(&from));
        let tx = SolanaTransaction::new_unsigned(message);

        self.tx = Some(tx);
        Ok(())
    }

    // Build an SPL token transfer transaction
    pub fn build_token_transfer(
        &mut self,
        token_program_id: &str,
        source_pubkey: &str,
        destination_pubkey: &str,
        owner_pubkey: &str,
        amount: u64,
        recent_blockhash: &str,
    ) -> Result<(), SolanaUnityError> {
        let token_program = if token_program_id.is_empty() {
            Pubkey::from_str(TOKEN_PROGRAM_ID).map_err(|e| {
                SolanaUnityError::InvalidInput(format!("Invalid token program: {}", e))
            })?
        } else {
            Pubkey::from_str(token_program_id).map_err(|e| {
                SolanaUnityError::InvalidInput(format!("Invalid token program: {}", e))
            })?
        };

        let source = Pubkey::from_str(source_pubkey)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid source pubkey: {}", e)))?;

        let destination = Pubkey::from_str(destination_pubkey).map_err(|e| {
            SolanaUnityError::InvalidInput(format!("Invalid destination pubkey: {}", e))
        })?;

        let owner = Pubkey::from_str(owner_pubkey)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid owner pubkey: {}", e)))?;

        // Create transfer instruction
        // spl_token::instruction::transfer requires spl_token dependency, here we'll create proper instruction data
        // Transfer instruction has index 3 and amount as u64 LE bytes
        let mut data = Vec::with_capacity(9);
        data.push(3); // Transfer instruction index
        data.extend_from_slice(&amount.to_le_bytes()); // Amount as 8-byte LE

        let accounts = vec![
            (source_pubkey, false, true),      // source
            (destination_pubkey, false, true), // destination
            (owner_pubkey, true, false),       // owner
        ];

        self.build_program_call(
            &token_program.to_string(),
            accounts,
            data,
            recent_blockhash,
            owner_pubkey,
        )
    }

    // Build a transaction to call a program
    pub fn build_program_call(
        &mut self,
        program_id: &str,
        accounts: Vec<(&str, bool, bool)>, // pubkey, is_signer, is_writable
        data: Vec<u8>,
        recent_blockhash: &str,
        fee_payer: &str,
    ) -> Result<(), SolanaUnityError> {
        let program = Pubkey::from_str(program_id)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid program id: {}", e)))?;

        let fee_payer_pubkey = Pubkey::from_str(fee_payer)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid fee payer: {}", e)))?;

        let mut account_metas = Vec::new();
        for (pubkey_str, is_signer, is_writable) in accounts {
            let pubkey = Pubkey::from_str(pubkey_str).map_err(|e| {
                SolanaUnityError::InvalidInput(format!("Invalid account pubkey: {}", e))
            })?;

            account_metas.push(solana_sdk::instruction::AccountMeta {
                pubkey,
                is_signer,
                is_writable,
            });
        }

        let instruction = Instruction {
            program_id: program,
            accounts: account_metas,
            data,
        };

        let blockhash = Hash::from_str(recent_blockhash)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid blockhash: {}", e)))?;

        let message = Message::new(&[instruction], Some(&fee_payer_pubkey));
        let tx = SolanaTransaction::new_unsigned(message);

        self.tx = Some(tx);
        Ok(())
    }

    // Sign transaction with a keypair
    pub fn sign(&mut self, private_key: &[u8]) -> Result<(), SolanaUnityError> {
        let keypair = match Keypair::from_bytes(private_key) {
            Ok(kp) => kp,
            Err(e) => {
                return Err(SolanaUnityError::WalletError(format!(
                    "Invalid keypair: {}",
                    e
                )));
            }
        };

        let mut tx = self.tx.take().ok_or_else(|| {
            SolanaUnityError::TransactionError("No transaction to sign".to_string())
        })?;

        tx.try_sign(&[&keypair], tx.message.recent_blockhash)
            .map_err(|e| {
                SolanaUnityError::TransactionError(format!("Failed to sign transaction: {}", e))
            })?;

        self.tx = Some(tx);
        Ok(())
    }

    // Get the serialized transaction for sending
    pub fn serialize(&self) -> Result<Vec<u8>, SolanaUnityError> {
        let tx = self.tx.as_ref().ok_or_else(|| {
            SolanaUnityError::TransactionError("No transaction to serialize".to_string())
        })?;

        bincode::serialize(tx).map_err(|e| {
            SolanaUnityError::SerializationError(format!("Failed to serialize transaction: {}", e))
        })
    }

    // Get the transaction from serialized bytes
    pub fn from_serialized(&mut self, data: &[u8]) -> Result<(), SolanaUnityError> {
        let tx: SolanaTransaction = bincode::deserialize(data).map_err(|e| {
            SolanaUnityError::SerializationError(format!(
                "Failed to deserialize transaction: {}",
                e
            ))
        })?;

        self.tx = Some(tx);
        Ok(())
    }

    // Get transaction as a Solana transaction
    pub fn get_transaction(&self) -> Result<&SolanaTransaction, SolanaUnityError> {
        self.tx.as_ref().ok_or_else(|| {
            SolanaUnityError::TransactionError("No transaction available".to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;
    use solana_sdk::signer::Signer;

    #[test]
    fn test_new_transaction() {
        let tx = Transaction::new();
        assert!(tx.get_transaction().is_err());
    }

    #[test]
    fn test_build_transfer() {
        let mut tx = Transaction::new();
        let from = Keypair::new();
        let from_pubkey = from.pubkey().to_string();
        let to_pubkey = Keypair::new().pubkey().to_string();
        let blockhash = Hash::default().to_string();

        let result = tx.build_transfer(&from_pubkey, &to_pubkey, 1000, &blockhash);
        assert!(result.is_ok());
        assert!(tx.get_transaction().is_ok());

        // Check that the transaction has the right structure
        let tx_obj = tx.get_transaction().unwrap();
        assert_eq!(tx_obj.signatures.len(), 0); // Not signed yet
        assert_eq!(tx_obj.message.instructions.len(), 1); // One instruction (transfer)
    }

    #[test]
    fn test_build_token_transfer() {
        let mut tx = Transaction::new();
        let from = Keypair::new();
        let from_pubkey = from.pubkey().to_string();
        let to_pubkey = Keypair::new().pubkey().to_string();
        let owner_pubkey = from.pubkey().to_string();
        let blockhash = Hash::default().to_string();

        // Empty string should use default token program
        let result = tx.build_token_transfer(
            "",
            &from_pubkey,
            &to_pubkey,
            &owner_pubkey,
            1000,
            &blockhash,
        );
        assert!(result.is_ok());
        assert!(tx.get_transaction().is_ok());

        // Check that the transaction has the right structure
        let tx_obj = tx.get_transaction().unwrap();
        assert_eq!(tx_obj.signatures.len(), 0); // Not signed yet
        assert_eq!(tx_obj.message.instructions.len(), 1); // One instruction (token transfer)

        // Extract and verify instruction data
        let instruction = &tx_obj.message.instructions[0];
        let data = &instruction.data;
        assert_eq!(data[0], 3); // Index 3 is token transfer

        // First byte is the instruction index (3), next 8 bytes are the amount (1000 as u64)
        let amount_bytes = &data[1..9];
        let amount = u64::from_le_bytes([
            amount_bytes[0],
            amount_bytes[1],
            amount_bytes[2],
            amount_bytes[3],
            amount_bytes[4],
            amount_bytes[5],
            amount_bytes[6],
            amount_bytes[7],
        ]);
        assert_eq!(amount, 1000);
    }

    #[test]
    fn test_build_program_call() {
        let mut tx = Transaction::new();
        let program_id = Keypair::new().pubkey().to_string();
        let fee_payer = Keypair::new().pubkey().to_string();
        let account1 = Keypair::new().pubkey().to_string();
        let account2 = Keypair::new().pubkey().to_string();
        let blockhash = Hash::default().to_string();

        // Create accounts vector
        let accounts = vec![
            (&account1 as &str, true, false),
            (&account2 as &str, false, true),
        ];

        // Create instruction data
        let data = vec![0, 1, 2, 3];

        let result =
            tx.build_program_call(&program_id, accounts, data.clone(), &blockhash, &fee_payer);
        assert!(result.is_ok());
        assert!(tx.get_transaction().is_ok());

        // Check that the transaction has the right structure
        let tx_obj = tx.get_transaction().unwrap();
        assert_eq!(tx_obj.signatures.len(), 0); // Not signed yet
        assert_eq!(tx_obj.message.instructions.len(), 1); // One instruction

        // Extract and verify instruction data
        let instruction = &tx_obj.message.instructions[0];
        assert_eq!(instruction.data, data);
        assert_eq!(instruction.accounts.len(), 2); // Two accounts included
    }

    #[test]
    fn test_serialization() {
        let mut tx = Transaction::new();
        let from = Keypair::new();
        let from_pubkey = from.pubkey().to_string();
        let to_pubkey = Keypair::new().pubkey().to_string();
        let blockhash = Hash::default().to_string();

        tx.build_transfer(&from_pubkey, &to_pubkey, 1000, &blockhash)
            .unwrap();

        let serialized = tx.serialize();
        assert!(serialized.is_ok());

        let serialized_data = serialized.unwrap();
        assert!(!serialized_data.is_empty());

        let mut new_tx = Transaction::new();
        let result = new_tx.from_serialized(&serialized_data);
        assert!(result.is_ok());
        assert!(new_tx.get_transaction().is_ok());

        // The serialized and deserialized transactions should match
        let original_tx = tx.get_transaction().unwrap();
        let deserialized_tx = new_tx.get_transaction().unwrap();

        assert_eq!(
            original_tx.message.recent_blockhash,
            deserialized_tx.message.recent_blockhash
        );
        assert_eq!(
            original_tx.message.instructions.len(),
            deserialized_tx.message.instructions.len()
        );
    }

    #[test]
    fn test_sign_transaction() {
        let mut tx = Transaction::new();
        let keypair = Keypair::new();
        let from_pubkey = keypair.pubkey().to_string();
        let to_pubkey = Keypair::new().pubkey().to_string();
        let blockhash = Hash::default().to_string();

        // Build a transfer transaction
        tx.build_transfer(&from_pubkey, &to_pubkey, 1000, &blockhash)
            .unwrap();

        // Sign it
        let result = tx.sign(&keypair.to_bytes());
        assert!(result.is_ok());

        // Check that it's signed
        let tx_obj = tx.get_transaction().unwrap();
        assert_eq!(tx_obj.signatures.len(), 1);
        assert_ne!(
            tx_obj.signatures[0],
            solana_sdk::signature::Signature::default()
        );
    }

    #[test]
    fn test_invalid_pubkey() {
        let mut tx = Transaction::new();
        let invalid_pubkey = "not-a-valid-pubkey";
        let to_pubkey = Keypair::new().pubkey().to_string();
        let blockhash = Hash::default().to_string();

        let result = tx.build_transfer(invalid_pubkey, &to_pubkey, 1000, &blockhash);
        assert!(result.is_err());

        match result {
            Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
            _ => panic!("Expected InvalidInput error for invalid pubkey"),
        }

        // Also test invalid recipient
        let from_pubkey = Keypair::new().pubkey().to_string();
        let result = tx.build_transfer(&from_pubkey, invalid_pubkey, 1000, &blockhash);
        assert!(result.is_err());

        match result {
            Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
            _ => panic!("Expected InvalidInput error for invalid pubkey"),
        }
    }

    #[test]
    fn test_invalid_blockhash() {
        let mut tx = Transaction::new();
        let from_pubkey = Keypair::new().pubkey().to_string();
        let to_pubkey = Keypair::new().pubkey().to_string();

        // Test with invalid blockhash
        let invalid_blockhash = "not-a-valid-blockhash";
        let result = tx.build_transfer(&from_pubkey, &to_pubkey, 1000, invalid_blockhash);
        assert!(result.is_err());

        match result {
            Err(SolanaUnityError::InvalidInput(_)) => {} // Expected
            _ => panic!("Expected InvalidInput error for invalid blockhash"),
        }
    }

    #[test]
    fn test_invalid_signing() {
        let mut tx = Transaction::new();
        let keypair = Keypair::new();
        let from_pubkey = keypair.pubkey().to_string();
        let to_pubkey = Keypair::new().pubkey().to_string();
        let blockhash = Hash::default().to_string();

        // Try to sign without building a transaction
        let result = tx.sign(&keypair.to_bytes());
        assert!(result.is_err());

        match result {
            Err(SolanaUnityError::TransactionError(_)) => {} // Expected
            _ => panic!("Expected TransactionError when signing empty transaction"),
        }

        // Build transaction
        tx.build_transfer(&from_pubkey, &to_pubkey, 1000, &blockhash)
            .unwrap();

        // Try to sign with invalid keypair
        let invalid_keypair = vec![0; 32]; // Wrong length
        let result = tx.sign(&invalid_keypair);
        assert!(result.is_err());

        match result {
            Err(SolanaUnityError::WalletError(_)) => {} // Expected
            _ => panic!("Expected WalletError when signing with invalid keypair"),
        }
    }
}
