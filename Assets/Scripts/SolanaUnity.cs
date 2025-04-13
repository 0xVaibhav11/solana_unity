using System;
using System.Runtime.InteropServices;
using System.Linq;

public class SolanaUnity
{
    [DllImport("solana_unity")]
    private static extern int solana_build_program_call(
        IntPtr transaction,
        string programId,
        string[] accounts,
        int[] accountsIsSigner,
        int[] accountsIsWritable,
        int accountsCount,
        byte[] data,
        int dataLen,
        string recentBlockhash,
        string feePayer,
        out IntPtr error
    );

    [DllImport("solana_unity")]
    private static extern IntPtr solana_get_account_data(
        IntPtr client,
        string pubkey,
        out IntPtr error
    );

    [DllImport("solana_unity")]
    private static extern int solana_confirm_transaction(
        IntPtr client,
        string signature,
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
        if (transactionPtr == IntPtr.Zero)
            throw new InvalidOperationException("Transaction not initialized");

        string[] accountPubkeys = accounts.Select(a => a.pubkey).ToArray();
        int[] isSigner = accounts.Select(a => a.isSigner ? 1 : 0).ToArray();
        int[] isWritable = accounts.Select(a => a.isWritable ? 1 : 0).ToArray();

        IntPtr error;
        int result = solana_build_program_call(
            transactionPtr,
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
            Marshal.FreeHGlobal(error);
            throw new Exception($"Failed to build program call: {errorMsg}");
        }
    }

    public byte[] GetAccountData(string pubkey)
    {
        if (clientPtr == IntPtr.Zero)
            throw new InvalidOperationException("Client not initialized");

        IntPtr error;
        IntPtr dataPtr = solana_get_account_data(clientPtr, pubkey, out error);

        if (dataPtr == IntPtr.Zero)
        {
            string errorMsg = Marshal.PtrToStringAnsi(error);
            Marshal.FreeHGlobal(error);
            throw new Exception($"Failed to get account data: {errorMsg}");
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
        if (clientPtr == IntPtr.Zero)
            throw new InvalidOperationException("Client not initialized");

        IntPtr error;
        int result = solana_confirm_transaction(clientPtr, signature, out error);

        if (result == 0)
        {
            string errorMsg = Marshal.PtrToStringAnsi(error);
            Marshal.FreeHGlobal(error);
            throw new Exception($"Failed to confirm transaction: {errorMsg}");
        }

        return result == 1;
    }
}