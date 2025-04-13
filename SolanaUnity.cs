using System;
using System.Runtime.InteropServices;
using System.Text;
using UnityEngine;
using System.Linq;

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

        // New RPC methods
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_simulate_transaction(
            IntPtr client,
            IntPtr transaction,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_get_multiple_accounts(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 2)] string[] pubkeys,
            int pubkeysCount,
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

        // New transaction methods
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_sign_transaction_with_keypairs(
            IntPtr transaction,
            IntPtr[] privateKeysData,
            IntPtr[] privateKeysLengths,
            int privateKeysCount,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_build_with_instructions(
            IntPtr transaction,
            IntPtr instructionsData,
            int instructionsDataLen,
            int instructionsCount,
            [MarshalAs(UnmanagedType.LPStr)] string feePayer,
            [MarshalAs(UnmanagedType.LPStr)] string recentBlockhash,
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

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_account_get_private_key(
            IntPtr account,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_account_has_private_key(
            IntPtr account,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_account_get_keypair(
            IntPtr account,
            out IntPtr error);

        // PDA functions
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_find_program_address(
            IntPtr[] seeds,
            int seedsLen,
            [MarshalAs(UnmanagedType.LPStr)] string programId,
            out IntPtr addressOut,
            out byte bumpOut,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_find_associated_token_address(
            [MarshalAs(UnmanagedType.LPStr)] string walletAddress,
            [MarshalAs(UnmanagedType.LPStr)] string tokenMint,
            out IntPtr addressOut,
            out IntPtr error);

        // Instruction building functions
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_create_token_transfer_instruction(
            [MarshalAs(UnmanagedType.LPStr)] string source,
            [MarshalAs(UnmanagedType.LPStr)] string destination,
            [MarshalAs(UnmanagedType.LPStr)] string owner,
            ulong amount,
            out IntPtr encodedDataOut,
            out IntPtr encodedDataLenOut,
            out IntPtr error);

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_free_encoded_instruction(IntPtr dataPtr);

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

        // PDA methods
        public (string address, byte bump) FindProgramAddress(string[] seeds, string programId)
        {
            IntPtr errorPtr;
            IntPtr addressPtr;
            byte bump;

            IntPtr[] seedsPtr = new IntPtr[seeds.Length];
            for (int i = 0; i < seeds.Length; i++)
            {
                seedsPtr[i] = Marshal.StringToHGlobalAnsi(seeds[i]);
            }

            int result = solana_find_program_address(seedsPtr, seeds.Length, programId, out addressPtr, out bump, out errorPtr);

            // Free the allocated memory
            foreach (IntPtr ptr in seedsPtr)
            {
                Marshal.FreeHGlobal(ptr);
            }

            CheckError(errorPtr);

            if (result == 0)
            {
                throw new SolanaException("Failed to find program address");
            }

            string address = PtrToStringAndFree(addressPtr);
            return (address, bump);
        }

        public string FindAssociatedTokenAddress(string walletAddress, string tokenMint)
        {
            IntPtr errorPtr;
            IntPtr addressPtr;

            int result = solana_find_associated_token_address(walletAddress, tokenMint, out addressPtr, out errorPtr);
            CheckError(errorPtr);

            if (result == 0)
            {
                throw new SolanaException("Failed to find associated token address");
            }

            return PtrToStringAndFree(addressPtr);
        }

        // New method for multiple accounts
        public string GetMultipleAccounts(string[] pubkeys)
        {
            // TODO: Implement this method once the FFI function is updated to handle string arrays
            throw new NotImplementedException("GetMultipleAccounts is not yet implemented");
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

            // New method for multi-signature
            public void SignWithKeypairs(byte[][] privateKeys)
            {
                IntPtr errorPtr;

                IntPtr[] keyPtrs = new IntPtr[privateKeys.Length];
                IntPtr[] keyLengths = new IntPtr[privateKeys.Length];

                for (int i = 0; i < privateKeys.Length; i++)
                {
                    byte[] key = privateKeys[i];
                    IntPtr keyPtr = Marshal.AllocHGlobal(key.Length);
                    Marshal.Copy(key, 0, keyPtr, key.Length);
                    keyPtrs[i] = keyPtr;
                    keyLengths[i] = (IntPtr)key.Length;
                }

                int result = solana_sign_transaction_with_keypairs(
                    _transactionPtr,
                    keyPtrs,
                    keyLengths,
                    privateKeys.Length,
                    out errorPtr);

                // Free allocated memory
                for (int i = 0; i < keyPtrs.Length; i++)
                {
                    Marshal.FreeHGlobal(keyPtrs[i]);
                }

                CheckError(errorPtr);
                if (result == 0)
                {
                    throw new SolanaException("Failed to sign transaction with multiple keypairs");
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

            // New method for transaction simulation
            public string Simulate()
            {
                IntPtr errorPtr;
                IntPtr resultPtr = solana_simulate_transaction(
                    _client._clientPtr,
                    _transactionPtr,
                    out errorPtr);

                CheckError(errorPtr);
                return PtrToStringAndFree(resultPtr);
            }

            // New method to build with instructions
            public void BuildWithInstructions(Instruction[] instructions, string feePayer, string recentBlockhash)
            {
                if (instructions == null || instructions.Length == 0)
                {
                    throw new SolanaException("No instructions provided for transaction");
                }

                IntPtr errorPtr;

                // Prepare the serialized instructions
                using (var memoryStream = new System.IO.MemoryStream())
                {
                    foreach (var instruction in instructions)
                    {
                        var data = instruction.GetEncodedData();
                        memoryStream.Write(data, 0, data.Length);
                    }

                    byte[] instructionsData = memoryStream.ToArray();

                    IntPtr dataPtr = Marshal.AllocHGlobal(instructionsData.Length);
                    try
                    {
                        Marshal.Copy(instructionsData, 0, dataPtr, instructionsData.Length);

                        int result = solana_build_with_instructions(
                            _transactionPtr,
                            dataPtr,
                            instructionsData.Length,
                            instructions.Length,
                            feePayer,
                            recentBlockhash,
                            out errorPtr);

                        CheckError(errorPtr);

                        if (result == 0)
                        {
                            throw new SolanaException("Failed to build transaction with instructions");
                        }
                    }
                    finally
                    {
                        Marshal.FreeHGlobal(dataPtr);
                    }
                }
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

            public byte[] GetPrivateKey()
            {
                IntPtr errorPtr;
                IntPtr privateKeyPtr = solana_account_get_private_key(_accountPtr, out errorPtr);
                CheckError(errorPtr);

                if (privateKeyPtr == IntPtr.Zero)
                {
                    throw new SolanaException("No private key available");
                }

                // Convert the native byte array to managed byte array
                byte[] privateKey = new byte[64]; // Ed25519 keypair is 64 bytes
                Marshal.Copy(privateKeyPtr, privateKey, 0, 64);
                solana_free_string(privateKeyPtr);

                return privateKey;
            }

            public bool HasPrivateKey()
            {
                IntPtr errorPtr;
                int result = solana_account_has_private_key(_accountPtr, out errorPtr);
                CheckError(errorPtr);
                return result != 0;
            }

            public IntPtr GetKeypair()
            {
                IntPtr errorPtr;
                IntPtr keypairPtr = solana_account_get_keypair(_accountPtr, out errorPtr);
                CheckError(errorPtr);
                return keypairPtr;
            }
        }

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_build_program_call(
            IntPtr transaction,
            [MarshalAs(UnmanagedType.LPStr)] string programId,
            [MarshalAs(UnmanagedType.LPArray, ArraySubType = UnmanagedType.LPStr, SizeParamIndex = 5)] string[] accounts,
            [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 5)] int[] accountsIsSigner,
            [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 5)] int[] accountsIsWritable,
            int accountsCount,
            [MarshalAs(UnmanagedType.LPArray, SizeParamIndex = 7)] byte[] data,
            int dataLen,
            [MarshalAs(UnmanagedType.LPStr)] string recentBlockhash,
            [MarshalAs(UnmanagedType.LPStr)] string feePayer,
            out IntPtr error
        );

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_get_account_data(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPStr)] string pubkey,
            out IntPtr error
        );

        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_confirm_transaction(
            IntPtr client,
            [MarshalAs(UnmanagedType.LPStr)] string signature,
            out IntPtr error
        );

        public void BuildProgramCall(
            string programId,
            (string pubkey, bool isSigner, bool isWritable)[] accounts,
            byte[] data,
            string recentBlockhash,
            string feePayer
        )
        {
            if (_clientPtr == IntPtr.Zero)
                throw new InvalidOperationException("Client not initialized");

            string[] accountPubkeys = accounts.Select(a => a.pubkey).ToArray();
            int[] isSigner = accounts.Select(a => a.isSigner ? 1 : 0).ToArray();
            int[] isWritable = accounts.Select(a => a.isWritable ? 1 : 0).ToArray();

            IntPtr error;
            int result = solana_build_program_call(
                _clientPtr,
                programId,
                accountPubkeys,
                isSigner,
                isWritable,
                accounts.Length,
                data,
                data.Length,
                recentBlockhash,
                feePayer,
                out error
            );

            if (result == 0)
            {
                string errorMsg = Marshal.PtrToStringAnsi(error);
                solana_free_string(error);
                throw new SolanaException($"Failed to build program call: {errorMsg}");
            }
        }

        public byte[] GetAccountData(string pubkey)
        {
            if (_clientPtr == IntPtr.Zero)
                throw new InvalidOperationException("Client not initialized");

            IntPtr error;
            IntPtr dataPtr = solana_get_account_data(_clientPtr, pubkey, out error);

            if (dataPtr == IntPtr.Zero)
            {
                string errorMsg = Marshal.PtrToStringAnsi(error);
                solana_free_string(error);
                throw new SolanaException($"Failed to get account data: {errorMsg}");
            }

            // Get data length from first 8 bytes
            byte[] lenBytes = new byte[8];
            Marshal.Copy(dataPtr, lenBytes, 0, 8);
            int dataLen = BitConverter.ToInt32(lenBytes, 0);

            // Get actual data
            byte[] data = new byte[dataLen];
            Marshal.Copy(IntPtr.Add(dataPtr, 8), data, 0, dataLen);

            // Free native memory
            Marshal.FreeHGlobal(dataPtr);

            return data;
        }

        public bool ConfirmTransaction(string signature)
        {
            if (_clientPtr == IntPtr.Zero)
                throw new InvalidOperationException("Client not initialized");

            IntPtr error;
            int result = solana_confirm_transaction(_clientPtr, signature, out error);

            if (result == 0)
            {
                string errorMsg = Marshal.PtrToStringAnsi(error);
                solana_free_string(error);
                throw new SolanaException($"Failed to confirm transaction: {errorMsg}");
            }

            return result == 1;
        }
    }

    // Instruction class to handle building instructions
    public class Instruction
    {
        private byte[] _encodedData;

        // Token Program IDs as constants
        public const string TokenProgramId = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
        public const string AssociatedTokenProgramId = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

        // Internal constructor - use factory methods instead
        internal Instruction(byte[] encodedData)
        {
            _encodedData = encodedData;
        }

        // Create a token transfer instruction
        public static Instruction CreateTokenTransfer(string source, string destination, string owner, ulong amount)
        {
            IntPtr errorPtr;
            IntPtr encodedDataPtr;
            IntPtr encodedDataLenPtr;

            int result = solana_create_token_transfer_instruction(
                source,
                destination,
                owner,
                amount,
                out encodedDataPtr,
                out encodedDataLenPtr,
                out errorPtr);

            CheckError(errorPtr);

            if (result == 0)
            {
                throw new SolanaException("Failed to create token transfer instruction");
            }

            // Convert to managed byte array
            int dataLen = (int)encodedDataLenPtr;
            byte[] data = new byte[dataLen];
            Marshal.Copy(encodedDataPtr, data, 0, dataLen);

            // Free the native memory
            solana_free_encoded_instruction(encodedDataPtr);

            return new Instruction(data);
        }

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

        // Free function for encoded instruction data
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_free_encoded_instruction(IntPtr dataPtr);

        // Free function for strings
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_free_string(IntPtr ptr);

        // Method to get encoded data
        internal byte[] GetEncodedData()
        {
            return _encodedData;
        }
    }
}