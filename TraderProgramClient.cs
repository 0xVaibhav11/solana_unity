using System;
using System.Collections.Generic;
using System.Linq;
using System.Numerics;
using System.Text;
using UnityEngine;
using Newtonsoft.Json.Linq;
using SolanaUnity;

namespace TraderProgramClient
{
  // / <summary>
  // / Client for the Trader contract: allows creating NPCs and trading items
  // / </summary>
  public class TraderClient : MonoBehaviour
  {
    // Program ID from your IDL
    private const string PROGRAM_ID = "NPCaMhiQD8oAbQYypLEe8rk7Y1QMVQzjThCbcw6c4Xo";

    // Fixed PDA seeds
    private static readonly byte[] NPC_SEED = Encoding.UTF8.GetBytes("npc");
    private static readonly byte[] ITEM_SEED = Encoding.UTF8.GetBytes("item");

    // Solana connection
    private SolanaClient _client;
    private ProgramClient _programClient;

    // User's wallet for signing transactions
    private SolanaClient.Account _wallet;

    private void Awake()
    {
      // Initialize Solana client
      _client = new SolanaClient("https://api.devnet.solana.com", "confirmed");

      // Load the trader program IDL from the JSON file
      TextAsset idlAsset = Resources.Load<TextAsset>("trader_contract");
      string idlJson = idlAsset.text;

      // Create program client
      var idlParser = new IdlParser(idlJson, PROGRAM_ID);
      _programClient = idlParser.CreateProgramClient(_client);

      Debug.Log($"Trader contract client initialized: {_programClient.ProgramName}");
    }

    private void OnDestroy()
    {
      _client?.Dispose();
      _wallet?.Dispose();
    }

    /// <summary>
    /// Set the wallet to use for signing transactions
    /// </summary>
    public void SetWallet(byte[] privateKey)
    {
      _wallet?.Dispose();
      _wallet = new SolanaClient.Account(privateKey);
      Debug.Log($"Wallet set: {_wallet.GetPublicKey()}");
    }

    /// <summary>
    /// Initialize a new NPC for the game
    /// </summary>
    public async void InitializeNpc(ulong gameInstanceId, Action<string> onSuccess, Action<string> onError)
    {
      try
      {
        if (_wallet == null)
        {
          onError?.Invoke("Wallet not set");
          return;
        }

        string walletAddress = _wallet.GetPublicKey();

        // Find the NPC PDA
        (string npcAddress, _) = FindNpcPda(gameInstanceId);

        // Prepare accounts
        var accounts = new List<AccountMeta>
                {
                    AccountMeta.Writable(walletAddress, true),  // authority
                    AccountMeta.Writable(npcAddress, false),    // npc (PDA)
                    AccountMeta.ReadOnly("11111111111111111111111111111111", false)  // system program
                };

        // Prepare args
        var args = new Dictionary<string, object>
                {
                    { "game_instance_id", gameInstanceId }
                };

        // Simulate first
        string simulationResult = _programClient.SimulateTransaction(
            "initialize_npc",
            args,
            accounts,
            walletAddress
        );

        // Check simulation result
        if (simulationResult.Contains("\"err\":null"))
        {
          // Send the transaction
          string signature = _programClient.SendTransaction(
              "initialize_npc",
              args,
              accounts,
              _wallet.GetPrivateKey()
          );

          onSuccess?.Invoke(signature);
          Debug.Log($"NPC initialized with signature: {signature}");
        }
        else
        {
          onError?.Invoke($"Simulation failed: {simulationResult}");
        }
      }
      catch (Exception e)
      {
        onError?.Invoke($"Error initializing NPC: {e.Message}");
        Debug.LogError($"Error initializing NPC: {e.Message}");
      }
    }

    /// <summary>
    /// Add an item to the NPC's inventory
    /// </summary>
    public async void AddItem(ulong gameInstanceId, ulong itemId, ulong price, ulong quantity,
        Action<string> onSuccess, Action<string> onError)
    {
      try
      {
        if (_wallet == null)
        {
          onError?.Invoke("Wallet not set");
          return;
        }

        string walletAddress = _wallet.GetPublicKey();

        // Find the NPC PDA
        (string npcAddress, _) = FindNpcPda(gameInstanceId);

        // Find the Item PDA
        (string itemAddress, _) = FindItemPda(npcAddress, itemId);

        // Prepare accounts
        var accounts = new List<AccountMeta>
                {
                    AccountMeta.Writable(walletAddress, true),  // authority
                    AccountMeta.ReadOnly(npcAddress, false),    // npc
                    AccountMeta.Writable(itemAddress, false),   // item
                    AccountMeta.ReadOnly("11111111111111111111111111111111", false)  // system program
                };

        // Prepare args
        var args = new Dictionary<string, object>
                {
                    { "item_id", itemId },
                    { "price", price },
                    { "quantity", quantity }
                };

        // Simulate first
        string simulationResult = _programClient.SimulateTransaction(
            "add_item",
            args,
            accounts,
            walletAddress
        );

        // Check simulation result
        if (simulationResult.Contains("\"err\":null"))
        {
          // Send the transaction
          string signature = _programClient.SendTransaction(
              "add_item",
              args,
              accounts,
              _wallet.GetPrivateKey()
          );

          onSuccess?.Invoke(signature);
          Debug.Log($"Item added with signature: {signature}");
        }
        else
        {
          onError?.Invoke($"Simulation failed: {simulationResult}");
        }
      }
      catch (Exception e)
      {
        onError?.Invoke($"Error adding item: {e.Message}");
        Debug.LogError($"Error adding item: {e.Message}");
      }
    }

    /// <summary>
    /// Buy an item from the NPC
    /// </summary>
    public async void BuyItem(ulong gameInstanceId, ulong itemId, ulong quantity, ulong maxPrice,
        Action<string> onSuccess, Action<string> onError)
    {
      try
      {
        if (_wallet == null)
        {
          onError?.Invoke("Wallet not set");
          return;
        }

        string walletAddress = _wallet.GetPublicKey();

        // Find the NPC PDA
        (string npcAddress, _) = FindNpcPda(gameInstanceId);

        // Find the Item PDA
        (string itemAddress, _) = FindItemPda(npcAddress, itemId);

        // Prepare accounts
        var accounts = new List<AccountMeta>
                {
                    AccountMeta.Writable(walletAddress, true),  // buyer
                    AccountMeta.Writable(npcAddress, false),    // npc
                    AccountMeta.Writable(itemAddress, false),   // item
                    AccountMeta.ReadOnly("11111111111111111111111111111111", false)  // system program
                };

        // Prepare args
        var args = new Dictionary<string, object>
                {
                    { "item_id", itemId },
                    { "quantity", quantity },
                    { "max_price", maxPrice }
                };

        // Simulate first
        string simulationResult = _programClient.SimulateTransaction(
            "buy_item",
            args,
            accounts,
            walletAddress
        );

        // Check simulation result
        if (simulationResult.Contains("\"err\":null"))
        {
          // Send the transaction
          string signature = _programClient.SendTransaction(
              "buy_item",
              args,
              accounts,
              _wallet.GetPrivateKey()
          );

          onSuccess?.Invoke(signature);
          Debug.Log($"Item purchased with signature: {signature}");
        }
        else
        {
          onError?.Invoke($"Simulation failed: {simulationResult}");
        }
      }
      catch (Exception e)
      {
        onError?.Invoke($"Error buying item: {e.Message}");
        Debug.LogError($"Error buying item: {e.Message}");
      }
    }

    /// <summary>
    /// Sell an item to the NPC
    /// </summary>
    public async void SellItem(ulong gameInstanceId, ulong itemId, ulong quantity, ulong minPrice,
        Action<string> onSuccess, Action<string> onError)
    {
      try
      {
        if (_wallet == null)
        {
          onError?.Invoke("Wallet not set");
          return;
        }

        string walletAddress = _wallet.GetPublicKey();

        // Find the NPC PDA
        (string npcAddress, _) = FindNpcPda(gameInstanceId);

        // Find the Item PDA
        (string itemAddress, _) = FindItemPda(npcAddress, itemId);

        // Prepare accounts
        var accounts = new List<AccountMeta>
                {
                    AccountMeta.Writable(walletAddress, true),  // seller
                    AccountMeta.Writable(npcAddress, false),    // npc
                    AccountMeta.Writable(itemAddress, false),   // item
                    AccountMeta.ReadOnly("11111111111111111111111111111111", false)  // system program
                };

        // Prepare args
        var args = new Dictionary<string, object>
                {
                    { "item_id", itemId },
                    { "quantity", quantity },
                    { "min_price", minPrice }
                };

        // Simulate first
        string simulationResult = _programClient.SimulateTransaction(
            "sell_item",
            args,
            accounts,
            walletAddress
        );

        // Check simulation result
        if (simulationResult.Contains("\"err\":null"))
        {
          // Send the transaction
          string signature = _programClient.SendTransaction(
              "sell_item",
              args,
              accounts,
              _wallet.GetPrivateKey()
          );

          onSuccess?.Invoke(signature);
          Debug.Log($"Item sold with signature: {signature}");
        }
        else
        {
          onError?.Invoke($"Simulation failed: {simulationResult}");
        }
      }
      catch (Exception e)
      {
        onError?.Invoke($"Error selling item: {e.Message}");
        Debug.LogError($"Error selling item: {e.Message}");
      }
    }

    /// <summary>
    /// Fetch NPC data from the blockchain
    /// </summary>
    public async void FetchNpc(ulong gameInstanceId, Action<NpcData> onSuccess, Action<string> onError)
    {
      try
      {
        // Find the NPC PDA
        (string npcAddress, _) = FindNpcPda(gameInstanceId);

        // Get account info
        string accountJson = _client.GetAccountInfo(npcAddress);

        // Parse account data
        JObject account = JObject.Parse(accountJson);
        JObject data = (JObject)account["data"];

        if (data != null && data.ContainsKey("parsed"))
        {
          var npcData = new NpcData
          {
            GameInstanceId = gameInstanceId,
            Authority = data["parsed"]["authority"].ToString(),
            Bump = byte.Parse(data["parsed"]["bump"].ToString()),
            IsInitialized = bool.Parse(data["parsed"]["is_initialized"].ToString())
          };

          onSuccess?.Invoke(npcData);
        }
        else
        {
          onError?.Invoke("NPC account not found or not properly initialized");
        }
      }
      catch (Exception e)
      {
        onError?.Invoke($"Error fetching NPC: {e.Message}");
        Debug.LogError($"Error fetching NPC: {e.Message}");
      }
    }

    /// <summary>
    /// Fetch item data from the blockchain
    /// </summary>
    public async void FetchItem(ulong gameInstanceId, ulong itemId, Action<ItemData> onSuccess, Action<string> onError)
    {
      try
      {
        // Find the NPC PDA
        (string npcAddress, _) = FindNpcPda(gameInstanceId);

        // Find the Item PDA
        (string itemAddress, _) = FindItemPda(npcAddress, itemId);

        // Get account info
        string accountJson = _client.GetAccountInfo(itemAddress);

        // Parse account data
        JObject account = JObject.Parse(accountJson);
        JObject data = (JObject)account["data"];

        if (data != null && data.ContainsKey("parsed"))
        {
          var itemData = new ItemData
          {
            ItemId = itemId,
            Price = ulong.Parse(data["parsed"]["price"].ToString()),
            Quantity = ulong.Parse(data["parsed"]["quantity"].ToString()),
            Npc = data["parsed"]["npc"].ToString(),
            IsInitialized = bool.Parse(data["parsed"]["is_initialized"].ToString())
          };

          onSuccess?.Invoke(itemData);
        }
        else
        {
          onError?.Invoke("Item account not found or not properly initialized");
        }
      }
      catch (Exception e)
      {
        onError?.Invoke($"Error fetching item: {e.Message}");
        Debug.LogError($"Error fetching item: {e.Message}");
      }
    }

    /// <summary>
    /// Get all items for an NPC
    /// </summary>
    public async void GetAllNpcItems(ulong gameInstanceId, Action<List<ItemData>> onSuccess, Action<string> onError)
    {
      try
      {
        // Find the NPC PDA
        (string npcAddress, _) = FindNpcPda(gameInstanceId);

        // Get program accounts filtered by NPC
        string accountsJson = _client.GetProgramAccounts(PROGRAM_ID);
        JArray accounts = JArray.Parse(accountsJson);

        var items = new List<ItemData>();

        foreach (JObject account in accounts)
        {
          try
          {
            JObject data = (JObject)account["data"];
            if (data != null && data.ContainsKey("parsed"))
            {
              // Check if this is an Item account
              if (data["parsed"].ContainsKey("npc"))
              {
                string accountNpc = data["parsed"]["npc"].ToString();
                if (accountNpc == npcAddress)
                {
                  var itemData = new ItemData
                  {
                    ItemId = ulong.Parse(data["parsed"]["item_id"].ToString()),
                    Price = ulong.Parse(data["parsed"]["price"].ToString()),
                    Quantity = ulong.Parse(data["parsed"]["quantity"].ToString()),
                    Npc = accountNpc,
                    IsInitialized = bool.Parse(data["parsed"]["is_initialized"].ToString())
                  };

                  items.Add(itemData);
                }
              }
            }
          }
          catch (Exception)
          {
            // Skip accounts that don't match our format
            continue;
          }
        }

        onSuccess?.Invoke(items);
      }
      catch (Exception e)
      {
        onError?.Invoke($"Error fetching items: {e.Message}");
        Debug.LogError($"Error fetching items: {e.Message}");
      }
    }

    /// <summary>
    /// Find the NPC's PDA for a game instance
    /// </summary>
    public (string, byte) FindNpcPda(ulong gameInstanceId)
    {
      // Prepare seeds
      var gameInstanceBytes = BitConverter.GetBytes(gameInstanceId);

      // Call the client's FindProgramAddress method
      return _client.FindProgramAddress(
          new string[] {
                    Convert.ToBase64String(NPC_SEED),
                    Convert.ToBase64String(gameInstanceBytes)
          },
          PROGRAM_ID
      );
    }

    /// <summary>
    /// Find the Item's PDA for an NPC and item ID
    /// </summary>
    public (string, byte) FindItemPda(string npcAddress, ulong itemId)
    {
      // Prepare seeds
      var itemIdBytes = BitConverter.GetBytes(itemId);

      // Call the client's FindProgramAddress method
      return _client.FindProgramAddress(
          new string[] {
                    Convert.ToBase64String(ITEM_SEED),
                    npcAddress,
                    Convert.ToBase64String(itemIdBytes)
          },
          PROGRAM_ID
      );
    }
  }

  /// <summary>
  /// Data structure representing an NPC
  /// </summary>
  [System.Serializable]
  public class NpcData
  {
    public ulong GameInstanceId { get; set; }
    public string Authority { get; set; }
    public byte Bump { get; set; }
    public bool IsInitialized { get; set; }
  }

  /// <summary>
  /// Data structure representing an Item
  /// </summary>
  [System.Serializable]
  public class ItemData
  {
    public ulong ItemId { get; set; }
    public ulong Price { get; set; }
    public ulong Quantity { get; set; }
    public string Npc { get; set; }
    public bool IsInitialized { get; set; }
  }
}
