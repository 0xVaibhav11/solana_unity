using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Numerics;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;
using UnityEngine;

namespace SolanaUnity
{
    /// <summary>
    /// Class for handling Anchor IDLs (Interface Definition Language) and generating program clients
    /// </summary>
    public class IdlParser
    {
        private JObject _idl;
        private string _programId;

        public IdlParser(string idlJson, string programId)
        {
            _idl = JObject.Parse(idlJson);
            _programId = programId;

            ValidateIdl();
        }

        private void ValidateIdl()
        {
            if (_idl == null)
            {
                throw new ArgumentException("IDL is null or empty");
            }

            if (!_idl.ContainsKey("instructions"))
            {
                throw new ArgumentException("IDL is missing instructions field");
            }

            if (!_idl.ContainsKey("name"))
            {
                throw new ArgumentException("IDL is missing name field");
            }
        }

        /// <summary>
        /// Creates a program client from the IDL
        /// </summary>
        public ProgramClient CreateProgramClient(SolanaClient client)
        {
            return new ProgramClient(client, _idl, _programId);
        }
    }

    /// <summary>
    /// Client for interacting with a Solana program using its IDL
    /// </summary>
    public class ProgramClient
    {
        private SolanaClient _client;
        private JObject _idl;
        private string _programId;
        private Dictionary<string, ProgramInstruction> _instructions;
        private Dictionary<string, ProgramAccount> _accounts;

        public string ProgramId => _programId;
        public string ProgramName => _idl["name"].ToString();

        public ProgramClient(SolanaClient client, JObject idl, string programId)
        {
            _client = client;
            _idl = idl;
            _programId = programId;

            ParseInstructions();
            ParseAccounts();
        }

        private void ParseInstructions()
        {
            _instructions = new Dictionary<string, ProgramInstruction>();

            var instructionsArray = _idl["instructions"] as JArray;
            if (instructionsArray == null) return;

            foreach (JObject instruction in instructionsArray)
            {
                string name = instruction["name"].ToString();
                _instructions[name] = new ProgramInstruction(instruction, _programId);
            }
        }

        private void ParseAccounts()
        {
            _accounts = new Dictionary<string, ProgramAccount>();

            if (!_idl.ContainsKey("accounts")) return;

            var accountsArray = _idl["accounts"] as JArray;
            if (accountsArray == null) return;

            foreach (JObject account in accountsArray)
            {
                string name = account["name"].ToString();
                _accounts[name] = new ProgramAccount(account);
            }
        }

        /// <summary>
        /// Gets the named instruction from the IDL
        /// </summary>
        public ProgramInstruction GetInstruction(string name)
        {
            if (!_instructions.ContainsKey(name))
            {
                throw new ArgumentException($"Instruction '{name}' not found in IDL");
            }

            return _instructions[name];
        }

        /// <summary>
        /// Gets the named account from the IDL
        /// </summary>
        public ProgramAccount GetAccount(string name)
        {
            if (!_accounts.ContainsKey(name))
            {
                throw new ArgumentException($"Account '{name}' not found in IDL");
            }

            return _accounts[name];
        }

        /// <summary>
        /// Creates an instruction with the given name and arguments
        /// </summary>
        public Instruction CreateInstruction(string name, Dictionary<string, object> args, List<AccountMeta> accounts)
        {
            var instruction = GetInstruction(name);
            return instruction.BuildInstruction(args, accounts);
        }

        /// <summary>
        /// Builds and sends a transaction with a single instruction
        /// </summary>
        public string SendTransaction(string name, Dictionary<string, object> args, List<AccountMeta> accounts, byte[] signerPrivateKey)
        {
            var instruction = CreateInstruction(name, args, accounts);
            string blockhash = _client.GetLatestBlockhash();

            using (var transaction = new SolanaClient.Transaction(_client))
            {
                // Get signer pubkey from private key
                var tempAccount = new SolanaClient.Account(signerPrivateKey);
                string signerPubkey = tempAccount.GetPublicKey();

                transaction.BuildWithInstructions(new[] { instruction }, signerPubkey, blockhash);
                transaction.Sign(signerPrivateKey);
                return transaction.Send();
            }
        }

        /// <summary>
        /// Simulates a transaction with a single instruction
        /// </summary>
        public string SimulateTransaction(string name, Dictionary<string, object> args, List<AccountMeta> accounts, string feePayer)
        {
            var instruction = CreateInstruction(name, args, accounts);
            string blockhash = _client.GetLatestBlockhash();

            using (var transaction = new SolanaClient.Transaction(_client))
            {
                transaction.BuildWithInstructions(new[] { instruction }, feePayer, blockhash);
                return transaction.Simulate();
            }
        }
    }

    /// <summary>
    /// Represents an instruction in a Solana program
    /// </summary>
    public class ProgramInstruction
    {
        private JObject _instruction;
        private string _programId;
        private string _name;
        private List<IdlInstructionArg> _args;
        private List<IdlInstructionAccount> _accounts;

        public string Name => _name;
        public List<IdlInstructionArg> Args => _args;
        public List<IdlInstructionAccount> Accounts => _accounts;

        public ProgramInstruction(JObject instruction, string programId)
        {
            _instruction = instruction;
            _programId = programId;
            _name = instruction["name"].ToString();

            ParseArgs();
            ParseAccounts();
        }

        private void ParseArgs()
        {
            _args = new List<IdlInstructionArg>();

            if (!_instruction.ContainsKey("args")) return;

            var argsArray = _instruction["args"] as JArray;
            if (argsArray == null) return;

            foreach (JObject arg in argsArray)
            {
                _args.Add(new IdlInstructionArg(arg));
            }
        }

        private void ParseAccounts()
        {
            _accounts = new List<IdlInstructionAccount>();

            if (!_instruction.ContainsKey("accounts")) return;

            var accountsArray = _instruction["accounts"] as JArray;
            if (accountsArray == null) return;

            foreach (JObject account in accountsArray)
            {
                _accounts.Add(new IdlInstructionAccount(account));
            }
        }

        /// <summary>
        /// Builds the instruction with the given arguments and accounts
        /// </summary>
        public Instruction BuildInstruction(Dictionary<string, object> args, List<AccountMeta> accountMetas)
        {
            // Validate accounts count
            if (accountMetas.Count < _accounts.Count)
            {
                throw new ArgumentException($"Not enough accounts provided. Expected at least {_accounts.Count}, got {accountMetas.Count}");
            }

            // Build the instruction data
            byte[] data = EncodeInstructionData(args);

            // Create the instruction
            return InstructionFactory.CreateCustomInstruction(_programId, accountMetas, data);
        }

        /// <summary>
        /// Encodes the instruction data based on the IDL specification
        /// </summary>
        private byte[] EncodeInstructionData(Dictionary<string, object> args)
        {
            // First byte is the instruction discriminator (index or hash)
            using (var stream = new System.IO.MemoryStream())
            {
                // Write 8-byte discriminator based on instruction name
                byte[] discriminator = ComputeInstructionDiscriminator(_name);
                stream.Write(discriminator, 0, discriminator.Length);

                // Write each argument according to its type
                foreach (var arg in _args)
                {
                    if (!args.ContainsKey(arg.Name))
                    {
                        throw new ArgumentException($"Missing argument '{arg.Name}'");
                    }

                    WriteArgument(stream, args[arg.Name], arg.Type);
                }

                return stream.ToArray();
            }
        }

        /// <summary>
        /// Computes the 8-byte instruction discriminator from the instruction name
        /// </summary>
        private byte[] ComputeInstructionDiscriminator(string name)
        {
            // In Anchor, the discriminator is the first 8 bytes of the SHA256 hash of "global:name"
            var nameBytes = Encoding.UTF8.GetBytes($"global:{name}");
            var hash = System.Security.Cryptography.SHA256.Create().ComputeHash(nameBytes);
            var result = new byte[8];
            Array.Copy(hash, result, 8);
            return result;
        }

        /// <summary>
        /// Writes an argument to the stream based on its type
        /// </summary>
        private void WriteArgument(System.IO.Stream stream, object value, string type)
        {
            if (type == "u8")
            {
                stream.WriteByte(Convert.ToByte(value));
            }
            else if (type == "u16")
            {
                var bytes = BitConverter.GetBytes(Convert.ToUInt16(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "u32")
            {
                var bytes = BitConverter.GetBytes(Convert.ToUInt32(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "u64")
            {
                var bytes = BitConverter.GetBytes(Convert.ToUInt64(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "i8")
            {
                var bytes = BitConverter.GetBytes(Convert.ToSByte(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "i16")
            {
                var bytes = BitConverter.GetBytes(Convert.ToInt16(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "i32")
            {
                var bytes = BitConverter.GetBytes(Convert.ToInt32(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "i64")
            {
                var bytes = BitConverter.GetBytes(Convert.ToInt64(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "f32")
            {
                var bytes = BitConverter.GetBytes(Convert.ToSingle(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "f64")
            {
                var bytes = BitConverter.GetBytes(Convert.ToDouble(value));
                stream.Write(bytes, 0, bytes.Length);
            }
            else if (type == "bool")
            {
                stream.WriteByte(Convert.ToBoolean(value) ? (byte)1 : (byte)0);
            }
            else if (type == "string" || type.StartsWith("string["))
            {
                var stringValue = value.ToString();
                var stringBytes = Encoding.UTF8.GetBytes(stringValue);

                // Write string length as u32
                var lengthBytes = BitConverter.GetBytes((UInt32)stringBytes.Length);
                stream.Write(lengthBytes, 0, lengthBytes.Length);

                // Write string bytes
                stream.Write(stringBytes, 0, stringBytes.Length);
            }
            else if (type.StartsWith("publicKey") || type.StartsWith("pubkey"))
            {
                // Public keys are 32 bytes
                var pubkeyStr = value.ToString();
                byte[] pubkeyBytes;

                if (pubkeyStr.StartsWith("0x"))
                {
                    // Handle hex format
                    pubkeyStr = pubkeyStr.Substring(2);
                    pubkeyBytes = new byte[32];
                    for (int i = 0; i < 32; i++)
                    {
                        pubkeyBytes[i] = Convert.ToByte(pubkeyStr.Substring(i * 2, 2), 16);
                    }
                }
                else
                {
                    // Get public key bytes - should be base58 encoded
                    pubkeyBytes = Base58.Decode(pubkeyStr);
                }

                stream.Write(pubkeyBytes, 0, pubkeyBytes.Length);
            }
            else if (type.Contains("[") && type.Contains("]"))
            {
                // Handle array types
                var arrayType = type.Substring(0, type.IndexOf('['));
                var arrayValues = value as IEnumerable<object>;

                if (arrayValues == null)
                {
                    throw new ArgumentException($"Expected array for type {type}");
                }

                // Write array length as u32
                var lengthBytes = BitConverter.GetBytes((UInt32)arrayValues.Count());
                stream.Write(lengthBytes, 0, lengthBytes.Length);

                // Write each array element
                foreach (var arrayValue in arrayValues)
                {
                    WriteArgument(stream, arrayValue, arrayType);
                }
            }
            else
            {
                throw new ArgumentException($"Unsupported type: {type}");
            }
        }
    }

    /// <summary>
    /// Represents an argument for a program instruction
    /// </summary>
    public class IdlInstructionArg
    {
        private JObject _arg;

        public string Name { get; private set; }
        public string Type { get; private set; }

        public IdlInstructionArg(JObject arg)
        {
            _arg = arg;
            Name = arg["name"].ToString();
            Type = arg["type"].ToString();
        }
    }

    /// <summary>
    /// Represents an account used in a program instruction
    /// </summary>
    public class IdlInstructionAccount
    {
        private JObject _account;

        public string Name { get; private set; }
        public bool IsMut { get; private set; }
        public bool IsSigner { get; private set; }

        public IdlInstructionAccount(JObject account)
        {
            _account = account;
            Name = account["name"].ToString();
            IsMut = account.ContainsKey("isMut") && (bool)account["isMut"];
            IsSigner = account.ContainsKey("isSigner") && (bool)account["isSigner"];
        }
    }

    /// <summary>
    /// Represents an account type defined in the IDL
    /// </summary>
    public class ProgramAccount
    {
        private JObject _account;

        public string Name { get; private set; }
        public List<IdlAccountField> Fields { get; private set; }

        public ProgramAccount(JObject account)
        {
            _account = account;
            Name = account["name"].ToString();
            Fields = new List<IdlAccountField>();

            if (account.ContainsKey("type") && account["type"]["kind"].ToString() == "struct")
            {
                var fields = account["type"]["fields"] as JArray;
                if (fields != null)
                {
                    foreach (JObject field in fields)
                    {
                        Fields.Add(new IdlAccountField(field));
                    }
                }
            }
        }
    }

    /// <summary>
    /// Represents a field in a program account
    /// </summary>
    public class IdlAccountField
    {
        private JObject _field;

        public string Name { get; private set; }
        public string Type { get; private set; }

        public IdlAccountField(JObject field)
        {
            _field = field;
            Name = field["name"].ToString();
            Type = field["type"].ToString();
        }
    }

    /// <summary>
    /// Utility for Base58 encoding/decoding (used for public keys)
    /// </summary>
    public static class Base58
    {
        private const string ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        private static readonly byte[] INDEXES = BuildIndexes();

        private static byte[] BuildIndexes()
        {
            byte[] indexes = new byte[128];
            for (int i = 0; i < indexes.Length; i++)
            {
                indexes[i] = 255;
            }

            for (int i = 0; i < ALPHABET.Length; i++)
            {
                indexes[ALPHABET[i]] = (byte)i;
            }

            return indexes;
        }

        public static byte[] Decode(string base58)
        {
            if (string.IsNullOrEmpty(base58))
                return new byte[0];

            byte[] input58 = new byte[base58.Length];
            for (int i = 0; i < base58.Length; i++)
            {
                char c = base58[i];
                int digit = c < 128 ? INDEXES[c] : 255;
                if (digit == 255)
                    throw new FormatException($"Invalid Base58 character '{c}' at position {i}");
                input58[i] = (byte)digit;
            }

            // Count leading zeros
            int zeros = 0;
            while (zeros < base58.Length && base58[zeros] == '1')
            {
                zeros++;
            }

            // Convert from Base58 to binary
            byte[] decoded = new byte[base58.Length];
            int outputStart = decoded.Length;

            for (int inputStart = zeros; inputStart < base58.Length;)
            {
                decoded[--outputStart] = DivideAndConquer(input58, inputStart);
                if (input58[inputStart] == 0)
                {
                    inputStart++;
                }
            }

            // Ignore excess leading zeroes in the decoded result
            while (outputStart < decoded.Length && decoded[outputStart] == 0)
            {
                outputStart++;
            }

            // Copy the result
            byte[] result = new byte[zeros + (decoded.Length - outputStart)];
            Array.Copy(decoded, outputStart, result, zeros, decoded.Length - outputStart);

            return result;
        }

        private static byte DivideAndConquer(byte[] number, int firstDigit)
        {
            int remainder = 0;
            for (int i = firstDigit; i < number.Length; i++)
            {
                int digit = number[i] & 0xFF;
                int temp = remainder * 58 + digit;
                number[i] = (byte)(temp / 256);
                remainder = temp % 256;
            }
            return (byte)remainder;
        }

        public static string Encode(byte[] data)
        {
            if (data.Length == 0)
                return "";

            // Count leading zeros
            int zeros = 0;
            while (zeros < data.Length && data[zeros] == 0)
            {
                zeros++;
            }

            // Convert to base58
            byte[] input = new byte[data.Length];
            Array.Copy(data, input, data.Length);

            char[] encoded = new char[input.Length * 2]; // Rough estimate of space needed
            int outputStart = encoded.Length;

            for (int inputStart = zeros; inputStart < input.Length;)
            {
                encoded[--outputStart] = ALPHABET[DivideAndRemainder(input, inputStart)];
                if (input[inputStart] == 0)
                {
                    inputStart++;
                }
            }

            // Add '1' for each leading zero
            while (zeros-- > 0)
            {
                encoded[--outputStart] = '1';
            }

            return new string(encoded, outputStart, encoded.Length - outputStart);
        }

        private static byte DivideAndRemainder(byte[] number, int firstDigit)
        {
            int remainder = 0;
            for (int i = firstDigit; i < number.Length; i++)
            {
                int digit = number[i] & 0xFF;
                int temp = remainder * 256 + digit;
                number[i] = (byte)(temp / 58);
                remainder = temp % 58;
            }
            return (byte)remainder;
        }
    }
}
