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
