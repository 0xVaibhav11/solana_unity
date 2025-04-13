use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::error::SolanaUnityError;

pub struct InstructionBuilder {
    program_id: String,
    accounts: Vec<AccountMetaInfo>,
    data: Vec<u8>,
}

pub struct AccountMetaInfo {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

impl InstructionBuilder {
    pub fn new(program_id: &str) -> Self {
        Self {
            program_id: program_id.to_string(),
            accounts: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn add_account(&mut self, pubkey: &str, is_signer: bool, is_writable: bool) -> &mut Self {
        self.accounts.push(AccountMetaInfo {
            pubkey: pubkey.to_string(),
            is_signer,
            is_writable,
        });
        self
    }

    pub fn set_data(&mut self, data: Vec<u8>) -> &mut Self {
        self.data = data;
        self
    }

    pub fn build(&self) -> Result<Instruction, SolanaUnityError> {
        let program_id = Pubkey::from_str(&self.program_id)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid program ID: {}", e)))?;

        let mut account_metas = Vec::with_capacity(self.accounts.len());
        for account in &self.accounts {
            let pubkey = Pubkey::from_str(&account.pubkey).map_err(|e| {
                SolanaUnityError::InvalidInput(format!("Invalid account pubkey: {}", e))
            })?;

            account_metas.push(AccountMeta {
                pubkey,
                is_signer: account.is_signer,
                is_writable: account.is_writable,
            });
        }

        Ok(Instruction {
            program_id,
            accounts: account_metas,
            data: self.data.clone(),
        })
    }
}

// SPL Token Program Instructions
pub struct TokenInstructions {}

impl TokenInstructions {
    // SPL Token Program ID
    pub const TOKEN_PROGRAM_ID: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    // Associated Token Program ID
    pub const ASSOCIATED_TOKEN_PROGRAM_ID: &'static str =
        "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    // Command indices for various token operations
    const TOKEN_TRANSFER_INDEX: u8 = 3;
    const TOKEN_APPROVE_INDEX: u8 = 4;
    const TOKEN_REVOKE_INDEX: u8 = 5;
    const TOKEN_MINT_TO_INDEX: u8 = 7;
    const TOKEN_BURN_INDEX: u8 = 8;
    const TOKEN_CLOSE_ACCOUNT_INDEX: u8 = 9;

    // Build a token transfer instruction
    pub fn transfer(
        source: &str,
        destination: &str,
        owner: &str,
        amount: u64,
    ) -> Result<Instruction, SolanaUnityError> {
        let mut data = Vec::with_capacity(9);
        data.push(Self::TOKEN_TRANSFER_INDEX);
        data.extend_from_slice(&amount.to_le_bytes());

        let mut builder = InstructionBuilder::new(Self::TOKEN_PROGRAM_ID);
        builder
            .add_account(source, false, true)
            .add_account(destination, false, true)
            .add_account(owner, true, false)
            .set_data(data);

        builder.build()
    }

    // Build a token approve instruction
    pub fn approve(
        source: &str,
        delegate: &str,
        owner: &str,
        amount: u64,
    ) -> Result<Instruction, SolanaUnityError> {
        let mut data = Vec::with_capacity(9);
        data.push(Self::TOKEN_APPROVE_INDEX);
        data.extend_from_slice(&amount.to_le_bytes());

        let mut builder = InstructionBuilder::new(Self::TOKEN_PROGRAM_ID);
        builder
            .add_account(source, false, true)
            .add_account(delegate, false, false)
            .add_account(owner, true, false)
            .set_data(data);

        builder.build()
    }

    // Build a token revoke instruction
    pub fn revoke(source: &str, owner: &str) -> Result<Instruction, SolanaUnityError> {
        let mut data = Vec::with_capacity(1);
        data.push(Self::TOKEN_REVOKE_INDEX);

        let mut builder = InstructionBuilder::new(Self::TOKEN_PROGRAM_ID);
        builder
            .add_account(source, false, true)
            .add_account(owner, true, false)
            .set_data(data);

        builder.build()
    }

    // Build a token mint-to instruction
    pub fn mint_to(
        mint: &str,
        destination: &str,
        authority: &str,
        amount: u64,
    ) -> Result<Instruction, SolanaUnityError> {
        let mut data = Vec::with_capacity(9);
        data.push(Self::TOKEN_MINT_TO_INDEX);
        data.extend_from_slice(&amount.to_le_bytes());

        let mut builder = InstructionBuilder::new(Self::TOKEN_PROGRAM_ID);
        builder
            .add_account(mint, false, true)
            .add_account(destination, false, true)
            .add_account(authority, true, false)
            .set_data(data);

        builder.build()
    }

    // Build a token burn instruction
    pub fn burn(
        account: &str,
        mint: &str,
        owner: &str,
        amount: u64,
    ) -> Result<Instruction, SolanaUnityError> {
        let mut data = Vec::with_capacity(9);
        data.push(Self::TOKEN_BURN_INDEX);
        data.extend_from_slice(&amount.to_le_bytes());

        let mut builder = InstructionBuilder::new(Self::TOKEN_PROGRAM_ID);
        builder
            .add_account(account, false, true)
            .add_account(mint, false, true)
            .add_account(owner, true, false)
            .set_data(data);

        builder.build()
    }

    // Build a token close account instruction
    pub fn close_account(
        account: &str,
        destination: &str,
        owner: &str,
    ) -> Result<Instruction, SolanaUnityError> {
        let mut data = Vec::with_capacity(1);
        data.push(Self::TOKEN_CLOSE_ACCOUNT_INDEX);

        let mut builder = InstructionBuilder::new(Self::TOKEN_PROGRAM_ID);
        builder
            .add_account(account, false, true)
            .add_account(destination, false, true)
            .add_account(owner, true, false)
            .set_data(data);

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_instruction_builder() {
        let program_id = Pubkey::new_unique().to_string();
        let account1 = Pubkey::new_unique().to_string();
        let account2 = Pubkey::new_unique().to_string();

        let data = vec![1, 2, 3, 4];

        let mut builder = InstructionBuilder::new(&program_id);
        builder
            .add_account(&account1, true, false)
            .add_account(&account2, false, true)
            .set_data(data.clone());

        let instruction = builder.build().unwrap();

        assert_eq!(
            instruction.program_id,
            Pubkey::from_str(&program_id).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(
            instruction.accounts[0].pubkey,
            Pubkey::from_str(&account1).unwrap()
        );
        assert_eq!(instruction.accounts[0].is_signer, true);
        assert_eq!(instruction.accounts[0].is_writable, false);
        assert_eq!(
            instruction.accounts[1].pubkey,
            Pubkey::from_str(&account2).unwrap()
        );
        assert_eq!(instruction.accounts[1].is_signer, false);
        assert_eq!(instruction.accounts[1].is_writable, true);
        assert_eq!(instruction.data, data);
    }

    #[test]
    fn test_token_transfer_instruction() {
        let source = Pubkey::new_unique().to_string();
        let destination = Pubkey::new_unique().to_string();
        let owner = Pubkey::new_unique().to_string();
        let amount = 1000;

        let instruction =
            TokenInstructions::transfer(&source, &destination, &owner, amount).unwrap();

        assert_eq!(
            instruction.program_id,
            Pubkey::from_str(TokenInstructions::TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(
            instruction.accounts[0].pubkey,
            Pubkey::from_str(&source).unwrap()
        );
        assert_eq!(
            instruction.accounts[1].pubkey,
            Pubkey::from_str(&destination).unwrap()
        );
        assert_eq!(
            instruction.accounts[2].pubkey,
            Pubkey::from_str(&owner).unwrap()
        );

        // Check instruction data
        assert_eq!(instruction.data[0], TokenInstructions::TOKEN_TRANSFER_INDEX);
        let amount_from_data = u64::from_le_bytes([
            instruction.data[1],
            instruction.data[2],
            instruction.data[3],
            instruction.data[4],
            instruction.data[5],
            instruction.data[6],
            instruction.data[7],
            instruction.data[8],
        ]);
        assert_eq!(amount_from_data, amount);
    }

    #[test]
    fn test_token_approve_instruction() {
        let source = Pubkey::new_unique().to_string();
        let delegate = Pubkey::new_unique().to_string();
        let owner = Pubkey::new_unique().to_string();
        let amount = 500;

        let instruction = TokenInstructions::approve(&source, &delegate, &owner, amount).unwrap();

        assert_eq!(
            instruction.program_id,
            Pubkey::from_str(TokenInstructions::TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 3);

        // Check instruction data
        assert_eq!(instruction.data[0], TokenInstructions::TOKEN_APPROVE_INDEX);
        let amount_from_data = u64::from_le_bytes([
            instruction.data[1],
            instruction.data[2],
            instruction.data[3],
            instruction.data[4],
            instruction.data[5],
            instruction.data[6],
            instruction.data[7],
            instruction.data[8],
        ]);
        assert_eq!(amount_from_data, amount);
    }
}
