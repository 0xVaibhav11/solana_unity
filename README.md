# Solana Unity SDK

A comprehensive SDK for interacting with the Solana blockchain from Unity applications. This SDK provides a complete set of tools for building Solana-powered games and applications.

## Features

- **Account Management**: Create, import, and manage Solana accounts
- **Transaction Building**: Create and manage SOL transfers and more complex transactions
- **Token Operations**: Full SPL Token support for transfers, minting, burning, and approvals
- **Program Derived Addresses (PDAs)**: Generate PDAs and find associated token accounts
- **Instruction Building**: Create custom instructions and compose complex transactions
- **Transaction Simulation**: Test transactions without committing them to the blockchain
- **Multi-signature Support**: Build and sign transactions with multiple signers
- **RPC Client**: Comprehensive RPC methods for interacting with Solana nodes

## Installation

1. Import the SDK package into your Unity project
2. Add the necessary DLLs to your project's Plugins folder
3. Import the namespace in your scripts: `using SolanaUnity;`

## Basic Usage

### Creating an RPC Client

```csharp
// Initialize a client with a connection to a Solana RPC node
SolanaClient client = new SolanaClient("https://api.devnet.solana.com", "confirmed");

// Always dispose the client when you're done
client.Dispose();
```

### Creating Accounts

```csharp
// Generate a new account
using (var account = new SolanaClient.Account())
{
    string publicKey = account.GetPublicKey();
    // Use the account...
}

// Create from an existing public key (read-only)
using (var account = new SolanaClient.Account("PUBLIC_KEY_HERE"))
{
    // Use the account...
}

// Create from a private key
using (var account = new SolanaClient.Account(privateKeyBytes))
{
    // Use the account...
}
```

### Basic Transfers

```csharp
// Get a recent blockhash
string blockhash = client.GetLatestBlockhash();

// Create and send a transaction
using (var transaction = new SolanaClient.Transaction(client))
{
    // Build a transfer transaction (lamports = SOL × 10⁹)
    transaction.BuildTransfer(fromPubkey, toPubkey, 1000000000, blockhash); // 1 SOL

    // Sign the transaction
    transaction.Sign(privateKeyBytes);

    // Send the transaction
    string signature = transaction.Send();

    // Check if the transaction is confirmed
    bool confirmed = client.ConfirmTransaction(signature);
}
```

### Working with PDAs

```csharp
// Find a program derived address
string programId = "11111111111111111111111111111111"; // Example program ID
string[] seeds = new string[] { "metadata", "pubkey" };

(string pda, byte bump) = client.FindProgramAddress(seeds, programId);

// Find an associated token account
string walletAddress = "...";
string tokenMint = "...";
string associatedTokenAccount = client.FindAssociatedTokenAddress(walletAddress, tokenMint);
```

### Token Operations

```csharp
// Get token balance
ulong balance = client.GetTokenAccountBalance(tokenAccountAddress);

// Create a token transfer instruction
Instruction transferInstruction = Instruction.CreateTokenTransfer(
    sourceAccount,
    destinationAccount,
    ownerAddress,
    amount
);

// More token instructions
Instruction approveInstruction = InstructionFactory.CreateTokenApprove(
    tokenAccount,
    delegateAddress,
    ownerAddress,
    amount
);

Instruction revokeInstruction = InstructionFactory.CreateTokenRevoke(
    tokenAccount,
    ownerAddress
);

Instruction mintToInstruction = InstructionFactory.CreateTokenMintTo(
    mintAddress,
    destinationAccount,
    mintAuthorityAddress,
    amount
);

Instruction burnInstruction = InstructionFactory.CreateTokenBurn(
    tokenAccount,
    mintAddress,
    ownerAddress,
    amount
);
```

### Building Transactions with Multiple Instructions

```csharp
// Create several instructions
Instruction instruction1 = ...;
Instruction instruction2 = ...;

// Build a transaction with multiple instructions
using (var transaction = new SolanaClient.Transaction(client))
{
    // Combine instructions in one transaction
    Instruction[] instructions = new Instruction[] { instruction1, instruction2 };

    transaction.BuildWithInstructions(instructions, feePayer, blockhash);

    // Sign and send as usual
    transaction.Sign(privateKeyBytes);
    string signature = transaction.Send();
}
```

### Custom Instructions

```csharp
// Create a list of accounts for a custom instruction
List<AccountMeta> accounts = new List<AccountMeta>
{
    AccountMeta.Writable(account1, true),   // Writable and signer
    AccountMeta.ReadOnly(account2, false)   // Read-only and not signer
};

// Create instruction data
byte[] data = new byte[] { 0, 1, 2, 3 };  // Custom data format depends on the program

// Create a custom instruction
Instruction customInstruction = InstructionFactory.CreateCustomInstruction(
    programId,
    accounts,
    data
);
```

### Transaction Simulation

```csharp
// Build a transaction as normal
using (var transaction = new SolanaClient.Transaction(client))
{
    transaction.BuildTransfer(fromPubkey, toPubkey, lamports, blockhash);

    // Simulate the transaction without sending it
    string simulationResult = transaction.Simulate();

    // Check the simulation result before actually sending
    if (simulationResult.Contains("\"err\":null"))
    {
        // Simulation succeeded, now sign and send
        transaction.Sign(privateKeyBytes);
        string signature = transaction.Send();
    }
}
```

### Multi-signature Transactions

```csharp
// Create a transaction requiring multiple signatures
using (var transaction = new SolanaClient.Transaction(client))
{
    // Build the transaction...

    // Sign with multiple keypairs
    byte[][] privateKeys = new byte[][] { privateKey1, privateKey2, privateKey3 };
    transaction.SignWithKeypairs(privateKeys);

    // Send the transaction
    string signature = transaction.Send();
}
```

## Advanced Features

### Program Accounts Query

```csharp
// Get all accounts owned by a program
string programId = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // SPL Token program
string accountsJson = client.GetProgramAccounts(programId);

// Process the JSON result...
```

### Account Information

```csharp
// Get account data
string accountInfo = client.GetAccountInfo(address);

// Process the JSON result...
```

## Error Handling

All methods in the SDK can throw a `SolanaException` if anything goes wrong. Make sure to wrap your calls in try-catch blocks:

```csharp
try
{
    // Solana operations...
}
catch (SolanaException e)
{
    Debug.LogError($"Solana operation failed: {e.Message}");
}
finally
{
    // Clean up resources
    client.Dispose();
}
```

## Memory Management

The SDK uses native resources that must be properly disposed:

- Always dispose `SolanaClient` instances
- Use `using` statements with `Account` and `Transaction` objects
- The SDK handles cleanup for other resources like instruction data

## Thread Safety

The SDK is not thread-safe. Do not use the same client, account, or transaction objects across multiple threads simultaneously.

## Performance Considerations

- For high-performance applications, reuse the same `SolanaClient` instance
- Consider batching operations for better performance
- Use transaction simulation to validate transactions before sending them

## License

This SDK is distributed under the MIT license.
