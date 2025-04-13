using System;
using System.Collections.Generic;
using UnityEngine;

namespace SolanaUnity.Examples
{
    public class SolanaUsageExample : MonoBehaviour
    {
        // Example RPC URLs
        private const string MAINNET_URL = "https://api.mainnet-beta.solana.com";
        private const string DEVNET_URL = "https://api.devnet.solana.com";
        private const string TESTNET_URL = "https://api.testnet.solana.com";

        // Using devnet for examples
        private string _rpcUrl = DEVNET_URL;

        private SolanaClient _client;

        void Start()
        {
            // Initialize the client
            _client = new SolanaClient(_rpcUrl, "confirmed");

            // Run a few examples
            try
            {
                // Basic transaction example
                BasicTransactionExample();

                // PDA examples
                PdaExamples();

                // Token instruction examples
                TokenInstructionExamples();

                // Transaction simulation example
                TransactionSimulationExample();

                // Multi-signature example
                MultiSignatureExample();
            }
            catch (SolanaException e)
            {
                Debug.LogError($"Solana operation failed: {e.Message}");
            }
            finally
            {
                // Clean up resources
                _client.Dispose();
            }
        }

        private void BasicTransactionExample()
        {
            Debug.Log("Running basic transaction example...");

            // Create a new account
            using (var sender = new SolanaClient.Account())
            {
                // Get the public key
                string senderPubkey = sender.GetPublicKey();
                Debug.Log($"Sender pubkey: {senderPubkey}");

                // Create a recipient account
                using (var recipient = new SolanaClient.Account())
                {
                    string recipientPubkey = recipient.GetPublicKey();
                    Debug.Log($"Recipient pubkey: {recipientPubkey}");

                    // Get recent blockhash
                    string blockhash = _client.GetLatestBlockhash();

                    // Create and send a transaction (this would fail on devnet without funds)
                    using (var transaction = new SolanaClient.Transaction(_client))
                    {
                        // Build a transfer transaction (1 SOL = 1000000000 lamports)
                        transaction.BuildTransfer(senderPubkey, recipientPubkey, 100000, blockhash);

                        // Sign the transaction (would fail without funds)
                        try
                        {
                            // Get private key bytes from the sender's account
                            // (This is just an example - in real code you'd need a funded account)
                            byte[] privateKey = new byte[64]; // This is a placeholder

                            transaction.Sign(privateKey);

                            // Send the transaction (would fail without funds)
                            // string signature = transaction.Send();
                            // Debug.Log($"Transaction sent with signature: {signature}");

                            // Simulate the transaction instead
                            string simulationResult = transaction.Simulate();
                            Debug.Log($"Transaction simulation result: {simulationResult}");
                        }
                        catch (SolanaException)
                        {
                            // Expected to fail on devnet without funds
                            Debug.Log("Transaction signing/sending skipped (no funds)");
                        }
                    }
                }
            }
        }

        private void PdaExamples()
        {
            Debug.Log("Running PDA examples...");

            // Create an account to work with
            using (var account = new SolanaClient.Account())
            {
                string pubkey = account.GetPublicKey();

                // Example 1: Find a program derived address
                string programId = "11111111111111111111111111111111"; // System program
                string[] seeds = new string[] { "metadata", pubkey };

                (string pda, byte bump) = _client.FindProgramAddress(seeds, programId);
                Debug.Log($"Found PDA: {pda} with bump seed: {bump}");

                // Example 2: Find an associated token account
                string mintAddress = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"; // USDC on mainnet

                string associatedTokenAccount = _client.FindAssociatedTokenAddress(pubkey, mintAddress);
                Debug.Log($"Associated token account for {pubkey} and mint {mintAddress}: {associatedTokenAccount}");
            }
        }

        private void TokenInstructionExamples()
        {
            Debug.Log("Running token instruction examples...");

            // Create two accounts for the example
            using (var owner = new SolanaClient.Account())
            using (var recipient = new SolanaClient.Account())
            {
                string ownerPubkey = owner.GetPublicKey();
                string recipientPubkey = recipient.GetPublicKey();

                // For this example, we'll pretend we have a token account and mint
                string tokenMint = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"; // Random mint for example
                string ownerTokenAccount = _client.FindAssociatedTokenAddress(ownerPubkey, tokenMint);
                string recipientTokenAccount = _client.FindAssociatedTokenAddress(recipientPubkey, tokenMint);

                // Get the latest blockhash
                string blockhash = _client.GetLatestBlockhash();

                // Example 1: Create a token transfer instruction
                Instruction transferInstruction = Instruction.CreateTokenTransfer(
                    ownerTokenAccount,
                    recipientTokenAccount,
                    ownerPubkey,
                    1000000 // 1 token with 6 decimals
                );

                // Example 2: Create a token approve instruction
                Instruction approveInstruction = InstructionFactory.CreateTokenApprove(
                    ownerTokenAccount,
                    recipientPubkey, // Delegate
                    ownerPubkey,
                    500000 // 0.5 token with 6 decimals
                );

                // Example 3: Create a token revoke instruction
                Instruction revokeInstruction = InstructionFactory.CreateTokenRevoke(
                    ownerTokenAccount,
                    ownerPubkey
                );

                // Example 4: Create a custom instruction
                List<AccountMeta> accounts = new List<AccountMeta>
                {
                    AccountMeta.Writable(ownerTokenAccount, false),
                    AccountMeta.Writable(recipientTokenAccount, false),
                    AccountMeta.ReadOnly(ownerPubkey, true)
                };

                // Sample data for a token transfer (this is simplified)
                byte[] customData = new byte[] { 3, 0, 0, 0, 0, 0, 0, 0, 100 }; // Command 3 = transfer, amount = 100

                Instruction customInstruction = InstructionFactory.CreateCustomInstruction(
                    Instruction.TokenProgramId,
                    accounts,
                    customData
                );

                // Build a transaction with multiple instructions
                using (var transaction = new SolanaClient.Transaction(_client))
                {
                    // Combine two instructions in one transaction
                    Instruction[] instructions = new Instruction[] { transferInstruction, approveInstruction };

                    transaction.BuildWithInstructions(instructions, ownerPubkey, blockhash);

                    // Simulate the transaction
                    string simulationResult = transaction.Simulate();
                    Debug.Log($"Multiple instruction transaction simulation result: {simulationResult}");
                }
            }
        }

        private void TransactionSimulationExample()
        {
            Debug.Log("Running transaction simulation example...");

            // Create accounts for the example
            using (var sender = new SolanaClient.Account())
            using (var recipient = new SolanaClient.Account())
            {
                string senderPubkey = sender.GetPublicKey();
                string recipientPubkey = recipient.GetPublicKey();

                // Get recent blockhash
                string blockhash = _client.GetLatestBlockhash();

                // Create a transaction
                using (var transaction = new SolanaClient.Transaction(_client))
                {
                    // Build a transfer transaction
                    transaction.BuildTransfer(senderPubkey, recipientPubkey, 50000000, blockhash); // 0.05 SOL

                    // Simulate the transaction
                    string simulationResult = transaction.Simulate();
                    Debug.Log($"Simulation result (transfer): {simulationResult}");

                    // You would typically parse the simulation result to check for errors
                    // or get other information before actually sending the transaction
                }
            }
        }

        private void MultiSignatureExample()
        {
            Debug.Log("Running multi-signature example...");

            // Create multiple accounts for the example
            using (var account1 = new SolanaClient.Account())
            using (var account2 = new SolanaClient.Account())
            using (var account3 = new SolanaClient.Account())
            {
                string pubkey1 = account1.GetPublicKey();
                string pubkey2 = account2.GetPublicKey();
                string pubkey3 = account3.GetPublicKey();

                // This is just an example - in a real app, you'd have proper private keys
                byte[] privateKey1 = new byte[64]; // Placeholder
                byte[] privateKey2 = new byte[64]; // Placeholder
                byte[] privateKey3 = new byte[64]; // Placeholder

                // Get recent blockhash
                string blockhash = _client.GetLatestBlockhash();

                // Create a transaction that requires multiple signatures
                using (var transaction = new SolanaClient.Transaction(_client))
                {
                    // Create a list of accounts for a custom instruction
                    List<AccountMeta> accounts = new List<AccountMeta>
                    {
                        // All three accounts are signers for this example
                        AccountMeta.Writable(pubkey1, true),
                        AccountMeta.Writable(pubkey2, true),
                        AccountMeta.Writable(pubkey3, true)
                    };

                    // Sample data for the instruction
                    byte[] data = new byte[] { 0, 1, 2, 3 };

                    // Create a custom instruction
                    Instruction instruction = InstructionFactory.CreateCustomInstruction(
                        "11111111111111111111111111111111", // System program 
                        accounts,
                        data
                    );

                    // Build the transaction
                    transaction.BuildWithInstructions(
                        new Instruction[] { instruction },
                        pubkey1, // Fee payer
                        blockhash
                    );

                    // Sign with multiple keypairs (would fail with placeholders)
                    try
                    {
                        transaction.SignWithKeypairs(new byte[][] { privateKey1, privateKey2, privateKey3 });

                        // Simulate the transaction
                        string simulationResult = transaction.Simulate();
                        Debug.Log($"Multi-signature transaction simulation result: {simulationResult}");

                        // In a real app, you would send the transaction if simulation succeeds
                        // string signature = transaction.Send();
                        // Debug.Log($"Multi-signature transaction sent with signature: {signature}");
                    }
                    catch (SolanaException)
                    {
                        // Expected to fail with placeholder keys
                        Debug.Log("Multi-signature transaction signing skipped (placeholder keys)");
                    }
                }
            }
        }
    }
}