use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use std::str::FromStr;

use crate::error::SolanaUnityError;

// Account wrapper for Solana accounts
pub struct Account {
    pubkey: Option<Pubkey>,
    keypair: Option<Keypair>,
}

impl Account {
    // Create a new empty account
    pub fn new() -> Self {
        Self {
            pubkey: None,
            keypair: None,
        }
    }

    // Create a new account from public key
    pub fn from_pubkey(pubkey_str: &str) -> Result<Self, SolanaUnityError> {
        let pubkey = Pubkey::from_str(pubkey_str)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid pubkey: {}", e)))?;

        Ok(Self {
            pubkey: Some(pubkey),
            keypair: None,
        })
    }

    // Create a new account from private key
    pub fn from_private_key(private_key: &[u8]) -> Result<Self, SolanaUnityError> {
        let keypair = Keypair::from_bytes(private_key)
            .map_err(|e| SolanaUnityError::WalletError(format!("Invalid keypair: {}", e)))?;

        let pubkey = keypair.pubkey();

        Ok(Self {
            pubkey: Some(pubkey),
            keypair: Some(keypair),
        })
    }

    // Derive a keypair from a BIP39 mnemonic phrase
    #[cfg(feature = "bip39")]
    pub fn from_mnemonic(
        mnemonic: &str,
        passphrase: &str,
        derivation_path: &str,
    ) -> Result<Self, SolanaUnityError> {
        use solana_sdk::derivation_path::DerivationPath;
        use tiny_bip39::{Language, Mnemonic, Seed};

        // Validate the mnemonic
        let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| SolanaUnityError::WalletError(format!("Invalid mnemonic: {}", e)))?;

        // Generate seed from mnemonic and optional passphrase
        let seed = Seed::new(&mnemonic, passphrase).as_bytes().to_vec();

        // Parse the derivation path
        let derivation_path = if derivation_path.is_empty() {
            DerivationPath::default()
        } else {
            DerivationPath::from_str(derivation_path).map_err(|e| {
                SolanaUnityError::WalletError(format!("Invalid derivation path: {}", e))
            })?
        };

        // Derive keypair from seed and path
        let keypair =
            Keypair::from_seed_and_derivation_path(seed, derivation_path).map_err(|e| {
                SolanaUnityError::WalletError(format!("Keypair derivation failed: {}", e))
            })?;

        let pubkey = keypair.pubkey();

        Ok(Self {
            pubkey: Some(pubkey),
            keypair: Some(keypair),
        })
    }

    // Generate a new random keypair
    pub fn generate() -> Self {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();

        Self {
            pubkey: Some(pubkey),
            keypair: Some(keypair),
        }
    }

    // Get the account's public key as a string
    pub fn get_pubkey(&self) -> Result<String, SolanaUnityError> {
        self.pubkey
            .as_ref()
            .map(|pk| pk.to_string())
            .ok_or_else(|| SolanaUnityError::WalletError("No public key available".to_string()))
    }

    // Get the private key bytes
    pub fn get_private_key(&self) -> Result<Vec<u8>, SolanaUnityError> {
        self.keypair
            .as_ref()
            .map(|kp| kp.to_bytes().to_vec())
            .ok_or_else(|| SolanaUnityError::WalletError("No keypair available".to_string()))
    }

    // Check if the account has a private key
    pub fn has_private_key(&self) -> bool {
        self.keypair.is_some()
    }

    // Get the Solana keypair
    pub fn get_keypair(&self) -> Result<&Keypair, SolanaUnityError> {
        self.keypair
            .as_ref()
            .ok_or_else(|| SolanaUnityError::WalletError("No keypair available".to_string()))
    }
}
