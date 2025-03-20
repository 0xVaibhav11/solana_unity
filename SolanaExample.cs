using System;
using System.Collections;
using System.Text;
using UnityEngine;
using UnityEngine.UI;
using SolanaUnity;

public class SolanaExample : MonoBehaviour
{
    [Header("Solana Settings")]
    [SerializeField] private string rpcUrl = "https://api.devnet.solana.com";
    [SerializeField] private string commitment = "confirmed";
    
    [Header("UI Elements")]
    [SerializeField] private Text statusText;
    [SerializeField] private Text balanceText;
    [SerializeField] private Text pubkeyText;
    [SerializeField] private Button sendButton;
    [SerializeField] private InputField recipientInput;
    [SerializeField] private InputField amountInput;
    
    private SolanaClient solanaClient;
    private SolanaClient.Account account;
    private string pubkey;
    
    void Start()
    {
        try
        {
            // Initialize Solana client
            solanaClient = new SolanaClient(rpcUrl, commitment);
            
            // Generate a new account
            account = new SolanaClient.Account();
            pubkey = account.GetPublicKey();
            
            // Display public key
            if (pubkeyText != null)
                pubkeyText.text = $"Public Key: {pubkey}";
            
            // Set up send button
            if (sendButton != null)
                sendButton.onClick.AddListener(OnSendButtonClicked);
            
            UpdateBalance();
            
            Log("Solana client initialized successfully");
        }
        catch (Exception e)
        {
            Log($"Error initializing Solana: {e.Message}");
        }
    }
    
    void OnDestroy()
    {
        // Clean up resources
        account?.Dispose();
        solanaClient?.Dispose();
    }
    
    public void UpdateBalance()
    {
        try
        {
            ulong balance = solanaClient.GetBalance(pubkey);
            
            if (balanceText != null)
                balanceText.text = $"Balance: {balance / 1_000_000_000.0f} SOL";
            
            Log($"Balance updated: {balance / 1_000_000_000.0f} SOL");
        }
        catch (Exception e)
        {
            Log($"Error updating balance: {e.Message}");
        }
    }
    
    public void OnSendButtonClicked()
    {
        StartCoroutine(SendTransaction());
    }
    
    private IEnumerator SendTransaction()
    {
        try
        {
            // Get input values
            string recipient = recipientInput.text;
            string amountText = amountInput.text;
            
            if (string.IsNullOrEmpty(recipient) || string.IsNullOrEmpty(amountText))
            {
                Log("Please enter recipient and amount");
                yield break;
            }
            
            if (!float.TryParse(amountText, out float solAmount))
            {
                Log("Invalid amount");
                yield break;
            }
            
            // Convert SOL to lamports
            ulong lamports = (ulong)(solAmount * 1_000_000_000);
            
            Log($"Sending {solAmount} SOL to {recipient}...");
            
            // Get the latest blockhash
            string blockhash = solanaClient.GetLatestBlockhash();
            
            // Create and sign transaction
            using (var transaction = new SolanaClient.Transaction(solanaClient))
            {
                // Build transfer transaction
                transaction.BuildTransfer(pubkey, recipient, lamports, blockhash);
                
                // Get private key and sign
                byte[] privateKey = new byte[64]; // In a real app, securely get the private key
                transaction.Sign(privateKey);
                
                // Send transaction
                string signature = transaction.Send();
                
                Log($"Transaction sent! Signature: {signature}");
                
                // Wait a bit for confirmation
                yield return new WaitForSeconds(1);
                
                // Check confirmation
                bool confirmed = solanaClient.ConfirmTransaction(signature);
                Log(confirmed ? "Transaction confirmed!" : "Transaction not yet confirmed");
                
                // Update balance
                UpdateBalance();
            }
        }
        catch (Exception e)
        {
            Log($"Error sending transaction: {e.Message}");
        }
    }
    
    // Helper to demonstrate token transfer
    public IEnumerator SendTokenTransaction(string tokenAccount, string recipient, ulong amount)
    {
        try
        {
            Log($"Sending {amount} tokens to {recipient}...");
            
            // Get the latest blockhash
            string blockhash = solanaClient.GetLatestBlockhash();
            
            // Create and sign transaction
            using (var transaction = new SolanaClient.Transaction(solanaClient))
            {
                // SPL Token program ID (use empty string for default)
                string tokenProgramId = "";
                
                // Build token transfer transaction
                transaction.BuildTokenTransfer(
                    tokenProgramId,
                    tokenAccount,
                    recipient,
                    pubkey,
                    amount,
                    blockhash);
                
                // Get private key and sign
                byte[] privateKey = new byte[64]; // In a real app, securely get the private key
                transaction.Sign(privateKey);
                
                // Send transaction
                string signature = transaction.Send();
                
                Log($"Token transaction sent! Signature: {signature}");
                
                // Wait a bit for confirmation
                yield return new WaitForSeconds(1);
                
                // Check confirmation
                bool confirmed = solanaClient.ConfirmTransaction(signature);
                Log(confirmed ? "Token transaction confirmed!" : "Token transaction not yet confirmed");
            }
        }
        catch (Exception e)
        {
            Log($"Error sending token transaction: {e.Message}");
        }
    }
    
    private void Log(string message)
    {
        Debug.Log(message);
        if (statusText != null)
            statusText.text = message;
    }
} 