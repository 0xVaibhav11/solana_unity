using System;
using System.Runtime.InteropServices;
using System.Collections.Generic;

namespace SolanaUnity
{
    // Extension class with more complete instruction implementations
    public static class InstructionFactory
    {
        // Create instruction for token approve (delegate)
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_create_token_approve_instruction(
            [MarshalAs(UnmanagedType.LPStr)] string source,
            [MarshalAs(UnmanagedType.LPStr)] string delegate_,
            [MarshalAs(UnmanagedType.LPStr)] string owner,
            ulong amount,
            out IntPtr encodedDataOut,
            out IntPtr encodedDataLenOut,
            out IntPtr error);

        // Create instruction for token revoke
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_create_token_revoke_instruction(
            [MarshalAs(UnmanagedType.LPStr)] string source,
            [MarshalAs(UnmanagedType.LPStr)] string owner,
            out IntPtr encodedDataOut,
            out IntPtr encodedDataLenOut,
            out IntPtr error);

        // Create instruction for token mint to
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_create_token_mint_to_instruction(
            [MarshalAs(UnmanagedType.LPStr)] string mint,
            [MarshalAs(UnmanagedType.LPStr)] string destination,
            [MarshalAs(UnmanagedType.LPStr)] string authority,
            ulong amount,
            out IntPtr encodedDataOut,
            out IntPtr encodedDataLenOut,
            out IntPtr error);

        // Create instruction for token burn
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_create_token_burn_instruction(
            [MarshalAs(UnmanagedType.LPStr)] string account,
            [MarshalAs(UnmanagedType.LPStr)] string mint,
            [MarshalAs(UnmanagedType.LPStr)] string owner,
            ulong amount,
            out IntPtr encodedDataOut,
            out IntPtr encodedDataLenOut,
            out IntPtr error);

        // Create instruction for token close account
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_create_token_close_account_instruction(
            [MarshalAs(UnmanagedType.LPStr)] string account,
            [MarshalAs(UnmanagedType.LPStr)] string destination,
            [MarshalAs(UnmanagedType.LPStr)] string owner,
            out IntPtr encodedDataOut,
            out IntPtr encodedDataLenOut,
            out IntPtr error);

        // Create a instruction builder for custom instructions
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr solana_create_instruction_builder(
            [MarshalAs(UnmanagedType.LPStr)] string programId,
            out IntPtr error);

        // Add account to instruction builder
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_instruction_builder_add_account(
            IntPtr builder,
            [MarshalAs(UnmanagedType.LPStr)] string pubkey,
            bool isSigner,
            bool isWritable,
            out IntPtr error);

        // Set data for instruction builder
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_instruction_builder_set_data(
            IntPtr builder,
            byte[] data,
            int dataLen,
            out IntPtr error);

        // Build instruction from instruction builder
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern int solana_instruction_builder_build(
            IntPtr builder,
            out IntPtr encodedDataOut,
            out IntPtr encodedDataLenOut,
            out IntPtr error);

        // Destroy instruction builder
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_destroy_instruction_builder(IntPtr builder);

        // Free function for encoded instruction data
        [DllImport("solana_unity", CallingConvention = CallingConvention.Cdecl)]
        private static extern void solana_free_encoded_instruction(IntPtr dataPtr);

        // Free function for strings
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

        // Create token approve instruction
        public static Instruction CreateTokenApprove(string source, string delegate_, string owner, ulong amount)
        {
            IntPtr errorPtr;
            IntPtr encodedDataPtr;
            IntPtr encodedDataLenPtr;

            int result = solana_create_token_approve_instruction(
                source,
                delegate_,
                owner,
                amount,
                out encodedDataPtr,
                out encodedDataLenPtr,
                out errorPtr);

            CheckError(errorPtr);

            if (result == 0)
            {
                throw new SolanaException("Failed to create token approve instruction");
            }

            // Convert to managed byte array
            int dataLen = (int)encodedDataLenPtr;
            byte[] data = new byte[dataLen];
            Marshal.Copy(encodedDataPtr, data, 0, dataLen);

            // Free the native memory
            solana_free_encoded_instruction(encodedDataPtr);

            return new Instruction(data);
        }

        // Create token revoke instruction
        public static Instruction CreateTokenRevoke(string source, string owner)
        {
            IntPtr errorPtr;
            IntPtr encodedDataPtr;
            IntPtr encodedDataLenPtr;

            int result = solana_create_token_revoke_instruction(
                source,
                owner,
                out encodedDataPtr,
                out encodedDataLenPtr,
                out errorPtr);

            CheckError(errorPtr);

            if (result == 0)
            {
                throw new SolanaException("Failed to create token revoke instruction");
            }

            // Convert to managed byte array
            int dataLen = (int)encodedDataLenPtr;
            byte[] data = new byte[dataLen];
            Marshal.Copy(encodedDataPtr, data, 0, dataLen);

            // Free the native memory
            solana_free_encoded_instruction(encodedDataPtr);

            return new Instruction(data);
        }

        // Create token mint-to instruction
        public static Instruction CreateTokenMintTo(string mint, string destination, string authority, ulong amount)
        {
            IntPtr errorPtr;
            IntPtr encodedDataPtr;
            IntPtr encodedDataLenPtr;

            int result = solana_create_token_mint_to_instruction(
                mint,
                destination,
                authority,
                amount,
                out encodedDataPtr,
                out encodedDataLenPtr,
                out errorPtr);

            CheckError(errorPtr);

            if (result == 0)
            {
                throw new SolanaException("Failed to create token mint-to instruction");
            }

            // Convert to managed byte array
            int dataLen = (int)encodedDataLenPtr;
            byte[] data = new byte[dataLen];
            Marshal.Copy(encodedDataPtr, data, 0, dataLen);

            // Free the native memory
            solana_free_encoded_instruction(encodedDataPtr);

            return new Instruction(data);
        }

        // Create token burn instruction
        public static Instruction CreateTokenBurn(string account, string mint, string owner, ulong amount)
        {
            IntPtr errorPtr;
            IntPtr encodedDataPtr;
            IntPtr encodedDataLenPtr;

            int result = solana_create_token_burn_instruction(
                account,
                mint,
                owner,
                amount,
                out encodedDataPtr,
                out encodedDataLenPtr,
                out errorPtr);

            CheckError(errorPtr);

            if (result == 0)
            {
                throw new SolanaException("Failed to create token burn instruction");
            }

            // Convert to managed byte array
            int dataLen = (int)encodedDataLenPtr;
            byte[] data = new byte[dataLen];
            Marshal.Copy(encodedDataPtr, data, 0, dataLen);

            // Free the native memory
            solana_free_encoded_instruction(encodedDataPtr);

            return new Instruction(data);
        }

        // Create token close account instruction
        public static Instruction CreateTokenCloseAccount(string account, string destination, string owner)
        {
            IntPtr errorPtr;
            IntPtr encodedDataPtr;
            IntPtr encodedDataLenPtr;

            int result = solana_create_token_close_account_instruction(
                account,
                destination,
                owner,
                out encodedDataPtr,
                out encodedDataLenPtr,
                out errorPtr);

            CheckError(errorPtr);

            if (result == 0)
            {
                throw new SolanaException("Failed to create token close account instruction");
            }

            // Convert to managed byte array
            int dataLen = (int)encodedDataLenPtr;
            byte[] data = new byte[dataLen];
            Marshal.Copy(encodedDataPtr, data, 0, dataLen);

            // Free the native memory
            solana_free_encoded_instruction(encodedDataPtr);

            return new Instruction(data);
        }

        // Create a custom instruction using the instruction builder
        public static Instruction CreateCustomInstruction(string programId, List<AccountMeta> accounts, byte[] data)
        {
            IntPtr errorPtr;
            IntPtr builderPtr = solana_create_instruction_builder(programId, out errorPtr);
            CheckError(errorPtr);

            try
            {
                // Add accounts
                foreach (var account in accounts)
                {
                    int addResult = solana_instruction_builder_add_account(
                        builderPtr,
                        account.PublicKey,
                        account.IsSigner,
                        account.IsWritable,
                        out errorPtr);
                    CheckError(errorPtr);

                    if (addResult == 0)
                    {
                        throw new SolanaException("Failed to add account to instruction builder");
                    }
                }

                // Set data
                int setDataResult = solana_instruction_builder_set_data(
                    builderPtr,
                    data,
                    data.Length,
                    out errorPtr);
                CheckError(errorPtr);

                if (setDataResult == 0)
                {
                    throw new SolanaException("Failed to set data for instruction builder");
                }

                // Build the instruction
                IntPtr encodedDataPtr;
                IntPtr encodedDataLenPtr;

                int buildResult = solana_instruction_builder_build(
                    builderPtr,
                    out encodedDataPtr,
                    out encodedDataLenPtr,
                    out errorPtr);
                CheckError(errorPtr);

                if (buildResult == 0)
                {
                    throw new SolanaException("Failed to build instruction");
                }

                // Convert to managed byte array
                int dataLen = (int)encodedDataLenPtr;
                byte[] encodedData = new byte[dataLen];
                Marshal.Copy(encodedDataPtr, encodedData, 0, dataLen);

                // Free the native memory
                solana_free_encoded_instruction(encodedDataPtr);

                return new Instruction(encodedData);
            }
            finally
            {
                // Always clean up the builder
                solana_destroy_instruction_builder(builderPtr);
            }
        }
    }

    // Account meta information for custom instructions
    public class AccountMeta
    {
        public string PublicKey { get; private set; }
        public bool IsSigner { get; private set; }
        public bool IsWritable { get; private set; }

        public AccountMeta(string publicKey, bool isSigner, bool isWritable)
        {
            PublicKey = publicKey;
            IsSigner = isSigner;
            IsWritable = isWritable;
        }

        // Helper methods for common account meta types
        public static AccountMeta ReadOnly(string publicKey, bool isSigner)
        {
            return new AccountMeta(publicKey, isSigner, false);
        }

        public static AccountMeta Writable(string publicKey, bool isSigner)
        {
            return new AccountMeta(publicKey, isSigner, true);
        }
    }
}