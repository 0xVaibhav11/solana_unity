# Solana Unity SDK

A native Solana SDK for Unity that enables web3 functionality through a Rust-compiled DLL. This allows Unity games and applications to interact with the Solana blockchain in a performant way.

## Features

- Connect to Solana RPC endpoints
- Create and manage wallet accounts
- Generate or import keypairs
- Query account balances and data
- Send SOL and SPL token transfers
- Build and sign transactions
- Interact with smart contracts/programs
- Support for BIP39 mnemonic phrases (optional)

## Prerequisites

- Rust toolchain (rustc, cargo)
- .NET/Mono development tools
- Unity 2019.4 or newer
- For cross-platform builds:
  - Windows: Visual Studio with C++ support
  - macOS: Xcode command line tools
  - Linux: gcc/clang and required development libraries

## Building the Library

### Building Rust Library

1. Clone this repository
2. Build the Rust library:

```bash
cargo build --release
```

This will produce a dynamic library file:

- Windows: `target/release/solana_unity.dll`
- macOS: `target/release/libsolana_unity.dylib`
- Linux: `target/release/libsolana_unity.so`

To build with BIP39 support:

```bash
cargo build --release --features bip39
```

### Unity Integration

1. Copy the built DLL/dylib/so file to your Unity project's `Assets/Plugins` directory:

   - Windows: `Assets/Plugins/x86_64/solana_unity.dll`
   - macOS: `Assets/Plugins/x86_64/libsolana_unity.dylib` (rename to `solana_unity.bundle` for Unity)
   - Linux: `Assets/Plugins/x86_64/libsolana_unity.so`

2. Add the `SolanaUnity.cs` wrapper file to your Unity project.

3. (Optional) If you need BIP39 support, define the `UNITY_SOLANA_BIP39` symbol in your project settings or add it as a compiler directive.

## Usage

### Basic Setup

```csharp
using SolanaUnity;

// Create a client and connect to Solana
SolanaClient solanaClient = new SolanaClient("https://api.mainnet-beta.solana.com", "confirmed");

// Generate a new account
SolanaClient.Account account = new SolanaClient.Account();
string publicKey = account.GetPublicKey();

// Get account balance
ulong balance = solanaClient.GetBalance(publicKey);
Debug.Log($"Balance: {balance / 1_000_000_000.0f} SOL");
```

### Sending SOL

```csharp
// Create transaction
using (var transaction = new SolanaClient.Transaction(solanaClient))
{
    // Get recent blockhash
    string blockhash = solanaClient.GetLatestBlockhash();

    // Build transfer
    transaction.BuildTransfer(
        fromPublicKey,    // Sender
        toPublicKey,      // Recipient
        1_000_000_000,    // Amount in lamports (1 SOL)
        blockhash         // Recent blockhash
    );

    // Sign transaction
    transaction.Sign(privateKeyBytes);

    // Send and get signature
    string signature = transaction.Send();

    // Check confirmation
    bool confirmed = solanaClient.ConfirmTransaction(signature);
}
```

### Working with SPL Tokens

```csharp
// Get token account balance
ulong tokenBalance = solanaClient.GetTokenAccountBalance(tokenAccountAddress);

// Send tokens
using (var transaction = new SolanaClient.Transaction(solanaClient))
{
    string blockhash = solanaClient.GetLatestBlockhash();

    // Transfer tokens
    transaction.BuildTokenTransfer(
        "",                  // Token program ID (empty for default)
        sourceTokenAccount,  // Source token account
        destinationAccount,  // Destination token account
        ownerPublicKey,      // Token account owner
        1000,                // Amount (token units)
        blockhash            // Recent blockhash
    );

    transaction.Sign(privateKeyBytes);
    string signature = transaction.Send();
}
```

### Smart Contract Interaction

To interact with a custom program:

```csharp
using (var transaction = new SolanaClient.Transaction(solanaClient))
{
    string blockhash = solanaClient.GetLatestBlockhash();

    // Get program accounts
    string programAccountsJson = solanaClient.GetProgramAccounts(programId);

    // Prepare account metadata - (pubkey, is_signer, is_writable)
    var accounts = new (string, bool, bool)[]
    {
        (accountA, false, true),  // Account A (writable)
        (accountB, false, false), // Account B (read-only)
        (payerAccount, true, true) // Payer (signer, writable)
    };

    // Prepare instruction data
    byte[] data = new byte[] { 1, 2, 3, 4 }; // Your instruction data

    // Build program call
    // This would typically be done with a helper method in your codebase
    // that wraps the programId, accounts, and data specific to your program

    // Sign and send...
}
```

## Security Considerations

- **Private Keys**: Never expose private keys in your application. For production apps, use secure key storage mechanisms or allow users to sign via wallet adapters/extensions.
- **Error Handling**: Always wrap your Solana interactions in try-catch blocks to gracefully handle network issues.
- **RPC Endpoints**: For production use, consider using dedicated RPC endpoints rather than public ones to avoid rate limiting.

## Example Project

See the `SolanaExample.cs` file for a complete example of how to use the SDK in a Unity MonoBehaviour.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
