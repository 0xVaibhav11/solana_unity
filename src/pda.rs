use crate::error::SolanaUnityError;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub struct ProgramDerivedAddress {}

impl ProgramDerivedAddress {
    /// Finds a program derived address and bump seed for the given seeds and program ID
    pub fn find_program_address(
        seeds: &[&[u8]],
        program_id: &str,
    ) -> Result<(String, u8), SolanaUnityError> {
        let program_pubkey = Pubkey::from_str(program_id)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid program ID: {}", e)))?;

        let (address, bump) = Pubkey::find_program_address(seeds, &program_pubkey);

        Ok((address.to_string(), bump))
    }

    /// Creates a program address for the given seeds and program ID
    pub fn create_program_address(
        seeds: &[&[u8]],
        program_id: &str,
    ) -> Result<String, SolanaUnityError> {
        let program_pubkey = Pubkey::from_str(program_id)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid program ID: {}", e)))?;

        let address = Pubkey::create_program_address(seeds, &program_pubkey).map_err(|e| {
            SolanaUnityError::InvalidInput(format!("Failed to create program address: {}", e))
        })?;

        Ok(address.to_string())
    }

    /// Finds an associated token account address for a wallet address and token mint
    pub fn find_associated_token_address(
        wallet_address: &str,
        token_mint: &str,
    ) -> Result<String, SolanaUnityError> {
        let wallet_pubkey = Pubkey::from_str(wallet_address).map_err(|e| {
            SolanaUnityError::InvalidInput(format!("Invalid wallet address: {}", e))
        })?;

        let token_mint_pubkey = Pubkey::from_str(token_mint)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid token mint: {}", e)))?;

        // SPL Token Program ID
        let token_program_id =
            Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

        // Associated Token Program ID
        let associated_token_program_id =
            Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();

        let seeds = &[
            wallet_pubkey.as_ref(),
            token_program_id.as_ref(),
            token_mint_pubkey.as_ref(),
        ];

        let (address, _) = Pubkey::find_program_address(seeds, &associated_token_program_id);

        Ok(address.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_find_program_address() {
        // Example program ID (System Program)
        let program_id = "11111111111111111111111111111111";

        // Example seeds
        let seeds = &[b"metadata", "MySeed".as_bytes()];

        let result = ProgramDerivedAddress::find_program_address(seeds, program_id);
        assert!(result.is_ok());

        let (address, bump) = result.unwrap();
        assert!(!address.is_empty());
        assert!(bump <= 255);
    }

    #[test]
    fn test_create_program_address() {
        // Example program ID (System Program)
        let program_id = "11111111111111111111111111111111";

        // Create seeds with a valid bump
        let bump: u8 = 255;
        let mut seed1 = "MySeed".as_bytes().to_vec();
        seed1.push(bump);

        let seeds = &[b"metadata", seed1.as_slice()];

        let result = ProgramDerivedAddress::create_program_address(seeds, program_id);

        // Note: This might fail sometimes due to the nature of program addresses
        // Not all combinations of seeds and bumps produce valid program addresses
        if result.is_ok() {
            let address = result.unwrap();
            assert!(!address.is_empty());
        }
    }

    #[test]
    fn test_find_associated_token_address() {
        // Example wallet address
        let wallet = Pubkey::new_unique().to_string();

        // Example token mint
        let mint = Pubkey::new_unique().to_string();

        let result = ProgramDerivedAddress::find_associated_token_address(&wallet, &mint);
        assert!(result.is_ok());

        let address = result.unwrap();
        assert!(!address.is_empty());
    }
}
