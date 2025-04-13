use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use std::str::FromStr;

use crate::error::SolanaUnityError;

pub struct Account {
    pubkey: Option<Pubkey>,
    keypair: Option<Keypair>,
}

impl Account {
    pub fn new() -> Self {
        Self {
            pubkey: None,
            keypair: None,
        }
    }

    pub fn from_pubkey(pubkey_str: &str) -> Result<Self, SolanaUnityError> {
        let pubkey = Pubkey::from_str(pubkey_str)
            .map_err(|e| SolanaUnityError::InvalidInput(format!("Invalid pubkey: {}", e)))?;

        Ok(Self {
            pubkey: Some(pubkey),
            keypair: None,
        })
    }

    pub fn from_private_key(private_key: &[u8]) -> Result<Self, SolanaUnityError> {
        let keypair = Keypair::from_bytes(private_key)
            .map_err(|e| SolanaUnityError::WalletError(format!("Invalid keypair: {}", e)))?;

        let pubkey = keypair.pubkey();

        Ok(Self {
            pubkey: Some(pubkey),
            keypair: Some(keypair),
        })
    }

    #[cfg(feature = "bip39")]
    pub fn from_mnemonic(
        mnemonic: &str,
        passphrase: &str,
        derivation_path: &str,
    ) -> Result<Self, SolanaUnityError> {
        use solana_sdk::derivation_path::DerivationPath;
        use tiny_bip39::{Language, Mnemonic, Seed};

        let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| SolanaUnityError::WalletError(format!("Invalid mnemonic: {}", e)))?;

        let seed = Seed::new(&mnemonic, passphrase).as_bytes().to_vec();

        let derivation_path = if derivation_path.is_empty() {
            DerivationPath::default()
        } else {
            DerivationPath::from_str(derivation_path).map_err(|e| {
                SolanaUnityError::WalletError(format!("Invalid derivation path: {}", e))
            })?
        };

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

    pub fn generate() -> Self {
        let keypair = Keypair::new();
        let pubkey = keypair.pubkey();

        Self {
            pubkey: Some(pubkey),
            keypair: Some(keypair),
        }
    }

    pub fn get_pubkey(&self) -> Result<String, SolanaUnityError> {
        self.pubkey
            .as_ref()
            .map(|pk| pk.to_string())
            .ok_or_else(|| SolanaUnityError::WalletError("No public key available".to_string()))
    }

    pub fn get_private_key(&self) -> Result<Vec<u8>, SolanaUnityError> {
        self.keypair
            .as_ref()
            .map(|kp| kp.to_bytes().to_vec())
            .ok_or_else(|| SolanaUnityError::WalletError("No keypair available".to_string()))
    }

    pub fn has_private_key(&self) -> bool {
        self.keypair.is_some()
    }

    pub fn get_keypair(&self) -> Result<&Keypair, SolanaUnityError> {
        self.keypair
            .as_ref()
            .ok_or_else(|| SolanaUnityError::WalletError("No keypair available".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;
    use solana_sdk::signer::Signer;

    #[test]
    fn test_account_new() {
        let account = Account::new();
        assert!(!account.has_private_key());
        assert!(account.get_pubkey().is_err());
        assert!(account.get_private_key().is_err());
    }

    #[test]
    fn test_account_from_pubkey() {
        let pubkey_str = "GsbwXfJraMomNxBcjK7tY82aT7ZUJNf6BA9wRx4GfDHP";
        let result = Account::from_pubkey(pubkey_str);
        assert!(result.is_ok());

        let account = result.unwrap();
        assert!(!account.has_private_key());

        let recovered_pubkey = account.get_pubkey().unwrap();
        assert_eq!(recovered_pubkey, pubkey_str);

        assert!(account.get_private_key().is_err());
        assert!(account.get_keypair().is_err());
    }

    #[test]
    fn test_account_from_private_key() {
        let keypair = Keypair::new();
        let private_key = keypair.to_bytes();
        let expected_pubkey = keypair.pubkey().to_string();

        let account = Account::from_private_key(&private_key);
        assert!(account.is_ok());

        let account = account.unwrap();
        assert!(account.has_private_key());

        let pubkey = account.get_pubkey().unwrap();
        assert_eq!(pubkey, expected_pubkey);

        let recovered_private_key = account.get_private_key().unwrap();
        assert_eq!(recovered_private_key, private_key);

        let keypair_ref = account.get_keypair();
        assert!(keypair_ref.is_ok());
        assert_eq!(keypair_ref.unwrap().pubkey().to_string(), expected_pubkey);
    }

    #[test]
    fn test_account_generate() {
        let account = Account::generate();
        assert!(account.has_private_key());

        let pubkey = account.get_pubkey();
        assert!(pubkey.is_ok());
        assert!(!pubkey.unwrap().is_empty());

        let private_key = account.get_private_key();
        assert!(private_key.is_ok());
        assert_eq!(private_key.unwrap().len(), 64);

        let keypair_ref = account.get_keypair();
        assert!(keypair_ref.is_ok());
    }

    #[test]
    fn test_invalid_pubkey() {
        let invalid_pubkeys = [
            "",
            "not-a-valid-pubkey",
            "tooshort",
            "TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG_TOOLONG",
        ];

        for &pubkey in &invalid_pubkeys {
            let result = Account::from_pubkey(pubkey);
            assert!(
                result.is_err(),
                "Expected error for invalid pubkey: {}",
                pubkey
            );

            match result {
                Err(SolanaUnityError::InvalidInput(_)) => {}
                _ => panic!("Expected InvalidInput error for pubkey: {}", pubkey),
            }
        }
    }

    #[test]
    fn test_invalid_private_key() {
        let invalid_key = vec![1, 2, 3, 4];
        let result = Account::from_private_key(&invalid_key);
        assert!(result.is_err());

        let invalid_key = vec![0; 32];
        let result = Account::from_private_key(&invalid_key);
        assert!(result.is_err());

        match result {
            Err(SolanaUnityError::WalletError(_)) => {}
            _ => panic!("Expected WalletError for invalid private key"),
        }
    }

    #[test]
    fn test_keypair_consistency() {
        let account = Account::generate();
        let pubkey = account.get_pubkey().unwrap();
        let private_key = account.get_private_key().unwrap();

        let restored_account = Account::from_private_key(&private_key).unwrap();
        let restored_pubkey = restored_account.get_pubkey().unwrap();

        assert_eq!(pubkey, restored_pubkey);
    }

    #[test]
    fn test_real_keypair_generation() {
        let account = Account::generate();
        assert!(account.has_private_key());

        let pubkey = account.get_pubkey();
        assert!(pubkey.is_ok());

        let pubkey_str = pubkey.unwrap();
        println!("Generated public key: {}", pubkey_str);

        let parsed_pubkey = solana_sdk::pubkey::Pubkey::from_str(&pubkey_str);
        assert!(parsed_pubkey.is_ok(), "Generated invalid pubkey");

        let private_key = account.get_private_key();
        assert!(private_key.is_ok());
        assert_eq!(private_key.unwrap().len(), 64);
    }

    #[test]
    fn test_real_keypair_from_private_key() {
        let keypair = solana_sdk::signer::keypair::Keypair::new();
        let expected_pubkey = keypair.pubkey().to_string();
        let bytes = keypair.to_bytes();

        let account = Account::from_private_key(&bytes);
        assert!(account.is_ok());

        let account = account.unwrap();
        let pubkey = account.get_pubkey();
        assert!(pubkey.is_ok());
        assert_eq!(pubkey.unwrap(), expected_pubkey);
        println!("Successfully restored account from private key");
    }

    #[test]
    fn test_real_account_from_pubkey() {
        let pubkey_str = "11111111111111111111111111111111";

        let account = Account::from_pubkey(pubkey_str);
        assert!(account.is_ok());

        let account = account.unwrap();
        let pubkey = account.get_pubkey();
        assert!(pubkey.is_ok());
        assert_eq!(pubkey.unwrap(), pubkey_str);
        assert!(!account.has_private_key());

        println!("Successfully created read-only account from public key");
    }
}
