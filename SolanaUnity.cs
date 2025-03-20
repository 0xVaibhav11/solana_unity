using System;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;

namespace SolanaUnity
{
    public class SolanaException : Exception
    {
        public SolanaException(string message) : base(message) { }
    }

    public class SolanaClient
    {
        private IntPtr _clientPtr;
        private bool _disposed = false;

        // RPC Client functions
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_create_rpc_client(
            [MarshalAs(UnmanagedType.LPStr)] string url,
            [MarshalAs(UnmanagedType.LPStr)] string commitment,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_destroy_rpc_client(IntPtr client);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern ulong solana_get_balance(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPStr)] string pubkey,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_get_latest_blockhash(
            IntPtr client,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern ulong solana_get_token_account_balance(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPStr)] string tokenAccount,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_get_account_info(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPStr)] string pubkey,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_get_program_accounts(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPStr)] string programId,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_get_transaction_status(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPStr)] string signature,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_confirm_transaction(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPStr)] string signature,
            out IntPtr error);

        // Transaction functions
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_create_transaction();

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_destroy_transaction(IntPtr transaction);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_build_transfer(
            IntPtr transaction,
            [MarshalAs(UnmanagedType.LPStr)] string fromPubkey,
            [MarshalAs(UnmanagedType.LPStr)] string toPubkey,
            ulong lamports,
            [MarshalAs(UnmanagedType.LPStr)] string recentBlockhash,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_build_token_transfer(
            IntPtr transaction,
            [MarshalAs(UnmanagedType.LPStr)] string tokenProgramId,
            [MarshalAs(UnmanagedType.LPStr)] string sourcePubkey,
            [MarshalAs(UnmanagedType.LPStr)] string destinationPubkey,
            [MarshalAs(UnmanagedType.LPStr)] string ownerPubkey,
            ulong amount,
            [MarshalAs(UnmanagedType.LPStr)] string recentBlockhash,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_sign_transaction(
            IntPtr transaction,
            byte[] privateKey,
            int privateKeyLen,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_send_transaction(
            IntPtr client,
            IntPtr transaction,
            out IntPtr error);

        // Account functions
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_create_account();

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_destroy_account(IntPtr account);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_account_from_pubkey(
            [MarshalAs(UnmanagedType.LPStr)] string pubkey,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_account_from_private_key(
            byte[] privateKey,
            int privateKeyLen,
            out IntPtr error);

        #if UNITY_SOLANA_BIP39
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_account_from_mnemonic(
            [MarshalAs(UnmanagedType.LPStr)] string mnemonic,
            [MarshalAs(UnmanagedType.LPStr)] string passphrase,
            [MarshalAs(UnmanagedType.LPStr)] string derivationPath,
            out IntPtr error);
        #endif

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_account_generate();

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_account_get_pubkey(
            IntPtr account,
            out IntPtr error);

        // Helper functions
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_free_string(IntPtr ptr);

        // Helper to handle errors
        private static void CheckError(IntPtr errorPtr)
        {
            if (errorPtr != IntPtr.Zero)
            {
                string errorMessage = Marshal.PtrToStringAnsi(errorPtr);
                solana_free_string(errorPtr);
                throw new SolanaException(errorMessage);
            }
        }

        // Helper to convert IntPtr to string and free the memory
        private static string PtrToStringAndFree(IntPtr ptr)
        {
            if (ptr == IntPtr.Zero)
                return null;

            string result = Marshal.PtrToStringAnsi(ptr);
            solana_free_string(ptr);
            return result;
        }

        // Constructor
        public SolanaClient(string url, string commitment = "confirmed")
        {
            IntPtr errorPtr;
            _clientPtr = solana_create_rpc_client(url, commitment, out errorPtr);
            CheckError(errorPtr);
        }

        // Destructor
        ~SolanaClient()
        {
            Dispose(false);
        }

        // Dispose pattern
        public void Dispose()
        {
            Dispose(true);
            GC.SuppressFinalize(this);
        }

        protected virtual void Dispose(bool disposing)
        {
            if (!_disposed)
            {
                if (_clientPtr != IntPtr.Zero)
                {
                    solana_destroy_rpc_client(_clientPtr);
                    _clientPtr = IntPtr.Zero;
                }
                _disposed = true;
            }
        }

        // API Methods
        public ulong GetBalance(string pubkey)
        {
            IntPtr errorPtr;
            ulong balance = solana_get_balance(_clientPtr, pubkey, out errorPtr);
            CheckError(errorPtr);
            return balance;
        }

        public string GetLatestBlockhash()
        {
            IntPtr errorPtr;
            IntPtr blockhashPtr = solana_get_latest_blockhash(_clientPtr, out errorPtr);
            CheckError(errorPtr);
            return PtrToStringAndFree(blockhashPtr);
        }

        public ulong GetTokenAccountBalance(string tokenAccount)
        {
            IntPtr errorPtr;
            ulong balance = solana_get_token_account_balance(_clientPtr, tokenAccount, out errorPtr);
            CheckError(errorPtr);
            return balance;
        }

        public string GetAccountInfo(string pubkey)
        {
            IntPtr errorPtr;
            IntPtr infoPtr = solana_get_account_info(_clientPtr, pubkey, out errorPtr);
            CheckError(errorPtr);
            return PtrToStringAndFree(infoPtr);
        }

        public string GetProgramAccounts(string programId)
        {
            IntPtr errorPtr;
            IntPtr accountsPtr = solana_get_program_accounts(_clientPtr, programId, out errorPtr);
            CheckError(errorPtr);
            return PtrToStringAndFree(accountsPtr);
        }

        public string GetTransactionStatus(string signature)
        {
            IntPtr errorPtr;
            IntPtr statusPtr = solana_get_transaction_status(_clientPtr, signature, out errorPtr);
            CheckError(errorPtr);
            return PtrToStringAndFree(statusPtr);
        }

        public bool ConfirmTransaction(string signature)
        {
            IntPtr errorPtr;
            int result = solana_confirm_transaction(_clientPtr, signature, out errorPtr);
            CheckError(errorPtr);
            return result != 0;
        }

        // Transaction class wrapper
        public class Transaction : IDisposable
        {
            private IntPtr _transactionPtr;
            private bool _disposed = false;
            private SolanaClient _client;

            public Transaction(SolanaClient client)
            {
                _client = client;
                _transactionPtr = solana_create_transaction();
            }

            ~Transaction()
            {
                Dispose(false);
            }

            public void Dispose()
            {
                Dispose(true);
                GC.SuppressFinalize(this);
            }

            protected virtual void Dispose(bool disposing)
            {
                if (!_disposed)
                {
                    if (_transactionPtr != IntPtr.Zero)
                    {
                        solana_destroy_transaction(_transactionPtr);
                        _transactionPtr = IntPtr.Zero;
                    }
                    _disposed = true;
                }
            }

            public void BuildTransfer(string fromPubkey, string toPubkey, ulong lamports, string recentBlockhash)
            {
                IntPtr errorPtr;
                int result = solana_build_transfer(
                    _transactionPtr,
                    fromPubkey,
                    toPubkey,
                    lamports,
                    recentBlockhash,
                    out errorPtr);

                CheckError(errorPtr);
                if (result == 0)
                {
                    throw new SolanaException("Failed to build transfer transaction");
                }
            }

            public void BuildTokenTransfer(
                string tokenProgramId,
                string sourcePubkey,
                string destinationPubkey,
                string ownerPubkey,
                ulong amount,
                string recentBlockhash)
            {
                IntPtr errorPtr;
                int result = solana_build_token_transfer(
                    _transactionPtr,
                    tokenProgramId,
                    sourcePubkey,
                    destinationPubkey,
                    ownerPubkey,
                    amount,
                    recentBlockhash,
                    out errorPtr);

                CheckError(errorPtr);
                if (result == 0)
                {
                    throw new SolanaException("Failed to build token transfer transaction");
                }
            }

            public void Sign(byte[] privateKey)
            {
                IntPtr errorPtr;
                int result = solana_sign_transaction(
                    _transactionPtr,
                    privateKey,
                    privateKey.Length,
                    out errorPtr);

                CheckError(errorPtr);
                if (result == 0)
                {
                    throw new SolanaException("Failed to sign transaction");
                }
            }

            public string Send()
            {
                IntPtr errorPtr;
                IntPtr signaturePtr = solana_send_transaction(
                    _client._clientPtr,
                    _transactionPtr,
                    out errorPtr);

                CheckError(errorPtr);
                return PtrToStringAndFree(signaturePtr);
            }
        }

        // Account class wrapper
        public class Account : IDisposable
        {
            private IntPtr _accountPtr;
            private bool _disposed = false;

            // Generate a new random account
            public Account()
            {
                _accountPtr = solana_account_generate();
            }

            // Create from public key (read-only)
            public Account(string pubkey)
            {
                IntPtr errorPtr;
                _accountPtr = solana_account_from_pubkey(pubkey, out errorPtr);
                CheckError(errorPtr);
            }

            // Create from private key
            public Account(byte[] privateKey)
            {
                IntPtr errorPtr;
                _accountPtr = solana_account_from_private_key(privateKey, privateKey.Length, out errorPtr);
                CheckError(errorPtr);
            }

            #if UNITY_SOLANA_BIP39
            // Create from mnemonic (when BIP39 feature is enabled)
            public Account(string mnemonic, string passphrase = "", string derivationPath = "")
            {
                IntPtr errorPtr;
                _accountPtr = solana_account_from_mnemonic(mnemonic, passphrase, derivationPath, out errorPtr);
                CheckError(errorPtr);
            }
            #endif

            ~Account()
            {
                Dispose(false);
            }

            public void Dispose()
            {
                Dispose(true);
                GC.SuppressFinalize(this);
            }

            protected virtual void Dispose(bool disposing)
            {
                if (!_disposed)
                {
                    if (_accountPtr != IntPtr.Zero)
                    {
                        solana_destroy_account(_accountPtr);
                        _accountPtr = IntPtr.Zero;
                    }
                    _disposed = true;
                }
            }

            public string GetPublicKey()
            {
                IntPtr errorPtr;
                IntPtr pubkeyPtr = solana_account_get_pubkey(_accountPtr, out errorPtr);
                CheckError(errorPtr);
                return PtrToStringAndFree(pubkeyPtr);
            }
        }
    }
} 