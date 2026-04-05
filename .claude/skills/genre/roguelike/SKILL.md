---
name: Roguelike
description: Roguelike/roguelite patterns — procedural dungeons, permadeath, meta-progression, loot systems, turn-based or real-time
globs: ["**/Dungeon*.cs", "**/Rogue*.cs", "**/Procedural*.cs", "**/Loot*.cs", "**/Permadeath*.cs", "**/Floor*.cs", "**/Room*.cs"]
---

# Roguelike / Roguelite Patterns

## Overview

The roguelike core loop: the player enters a procedurally generated dungeon, explores floor by floor, fights enemies, collects loot, and inevitably dies. Death is permanent for that run -- all progress within the dungeon is lost. Between runs, meta-currency earned based on performance unlocks permanent bonuses that make future runs slightly easier or more varied. The tension between run-specific power and permanent progression is the heart of the genre.

Key systems: dungeon generation, turn management (or real-time action), loot and item management, run state with permadeath, meta-progression persistence, and enemy encounter scaling. Each system is a plain C# class registered in VContainer. MonoBehaviours are thin Views.

---

## Dungeon Generation

### Room-Based BSP Generation

Binary Space Partitioning creates a tree of rectangular regions, then places rooms within each leaf node. This guarantees rooms do not overlap and produces natural-looking dungeon layouts.

```csharp
public enum RoomType { Combat, Treasure, Shop, Boss, Rest, Start, Exit }

public sealed class RoomData
{
    public RectInt Bounds { get; }
    public RoomType Type { get; set; }
    public List<Vector2Int> DoorPositions { get; } = new();
    public bool IsVisited { get; set; }
    public bool IsRevealed { get; set; }

    public RoomData(RectInt bounds, RoomType type)
    {
        Bounds = bounds;
        Type = type;
    }

    public Vector2Int Center => new(
        Bounds.x + Bounds.width / 2,
        Bounds.y + Bounds.height / 2
    );
}

public sealed class DungeonModel
{
    public int Width { get; }
    public int Height { get; }
    public int Floor { get; set; }
    public int Seed { get; }
    public List<RoomData> Rooms { get; } = new();
    public TileType[,] Tiles { get; }

    public DungeonModel(int width, int height, int seed)
    {
        Width = width;
        Height = height;
        Seed = seed;
        Tiles = new TileType[width, height];
    }
}

public enum TileType { Wall, Floor, Door, StairsDown, StairsUp, Trap }
```

### BSP Generator System

```csharp
public sealed class DungeonGeneratorSystem : IDisposable
{
    private readonly DungeonConfig m_Config;

    [Inject]
    public DungeonGeneratorSystem(DungeonConfig config)
    {
        m_Config = config;
    }

    public DungeonModel Generate(int floor, int seed)
    {
        var random = new System.Random(seed + floor * 1000);
        var dungeon = new DungeonModel(m_Config.Width, m_Config.Height, seed);
        dungeon.Floor = floor;

        // Fill with walls
        for (int x = 0; x < dungeon.Width; x++)
        {
            for (int y = 0; y < dungeon.Height; y++)
            {
                dungeon.Tiles[x, y] = TileType.Wall;
            }
        }

        // BSP split
        var rootNode = new BSPNode(new RectInt(1, 1, dungeon.Width - 2, dungeon.Height - 2));
        SplitNode(rootNode, random, 0);

        // Create rooms in leaf nodes
        List<BSPNode> leaves = new();
        CollectLeaves(rootNode, leaves);

        for (int roomIndex = 0; roomIndex < leaves.Count; roomIndex++)
        {
            BSPNode leaf = leaves[roomIndex];
            RectInt roomBounds = ShrinkRect(leaf.Bounds, random);
            var room = new RoomData(roomBounds, RoomType.Combat);
            dungeon.Rooms.Add(room);
            CarveRoom(dungeon, roomBounds);
        }

        // Connect rooms with corridors
        ConnectRooms(dungeon, rootNode, random);

        // Assign special room types
        AssignRoomTypes(dungeon, random, floor);

        return dungeon;
    }

    private void SplitNode(BSPNode node, System.Random random, int depth)
    {
        if (depth >= m_Config.MaxSplitDepth)
        {
            return;
        }

        if (node.Bounds.width < m_Config.MinRoomSize * 2
            || node.Bounds.height < m_Config.MinRoomSize * 2)
        {
            return;
        }

        bool splitHorizontal = random.Next(2) == 0;
        if (node.Bounds.width > node.Bounds.height * 1.5f)
        {
            splitHorizontal = false;
        }
        else if (node.Bounds.height > node.Bounds.width * 1.5f)
        {
            splitHorizontal = true;
        }

        if (splitHorizontal)
        {
            int splitY = random.Next(
                node.Bounds.y + m_Config.MinRoomSize,
                node.Bounds.y + node.Bounds.height - m_Config.MinRoomSize);
            node.Left = new BSPNode(new RectInt(
                node.Bounds.x, node.Bounds.y,
                node.Bounds.width, splitY - node.Bounds.y));
            node.Right = new BSPNode(new RectInt(
                node.Bounds.x, splitY,
                node.Bounds.width, node.Bounds.y + node.Bounds.height - splitY));
        }
        else
        {
            int splitX = random.Next(
                node.Bounds.x + m_Config.MinRoomSize,
                node.Bounds.x + node.Bounds.width - m_Config.MinRoomSize);
            node.Left = new BSPNode(new RectInt(
                node.Bounds.x, node.Bounds.y,
                splitX - node.Bounds.x, node.Bounds.height));
            node.Right = new BSPNode(new RectInt(
                splitX, node.Bounds.y,
                node.Bounds.x + node.Bounds.width - splitX, node.Bounds.height));
        }

        SplitNode(node.Left, random, depth + 1);
        SplitNode(node.Right, random, depth + 1);
    }

    private void CollectLeaves(BSPNode node, List<BSPNode> leaves)
    {
        if (node.Left == null && node.Right == null)
        {
            leaves.Add(node);
            return;
        }
        if (node.Left != null) CollectLeaves(node.Left, leaves);
        if (node.Right != null) CollectLeaves(node.Right, leaves);
    }

    private void CarveRoom(DungeonModel dungeon, RectInt bounds)
    {
        for (int x = bounds.x; x < bounds.x + bounds.width; x++)
        {
            for (int y = bounds.y; y < bounds.y + bounds.height; y++)
            {
                dungeon.Tiles[x, y] = TileType.Floor;
            }
        }
    }

    private RectInt ShrinkRect(RectInt bounds, System.Random random)
    {
        int shrinkX = random.Next(0, Mathf.Max(1, bounds.width - m_Config.MinRoomSize));
        int shrinkY = random.Next(0, Mathf.Max(1, bounds.height - m_Config.MinRoomSize));
        return new RectInt(
            bounds.x + shrinkX / 2,
            bounds.y + shrinkY / 2,
            Mathf.Max(m_Config.MinRoomSize, bounds.width - shrinkX),
            Mathf.Max(m_Config.MinRoomSize, bounds.height - shrinkY));
    }

    private void ConnectRooms(DungeonModel dungeon, BSPNode node, System.Random random)
    {
        if (node.Left == null || node.Right == null)
        {
            return;
        }

        ConnectRooms(dungeon, node.Left, random);
        ConnectRooms(dungeon, node.Right, random);

        Vector2Int leftCenter = GetNodeCenter(node.Left);
        Vector2Int rightCenter = GetNodeCenter(node.Right);
        CarveCorridor(dungeon, leftCenter, rightCenter, random);
    }

    private Vector2Int GetNodeCenter(BSPNode node)
    {
        return new Vector2Int(
            node.Bounds.x + node.Bounds.width / 2,
            node.Bounds.y + node.Bounds.height / 2);
    }

    private void CarveCorridor(DungeonModel dungeon, Vector2Int from, Vector2Int to, System.Random random)
    {
        Vector2Int current = from;

        // L-shaped corridor: horizontal then vertical (or vice versa)
        bool horizontalFirst = random.Next(2) == 0;
        if (horizontalFirst)
        {
            while (current.x != to.x)
            {
                dungeon.Tiles[current.x, current.y] = TileType.Floor;
                current.x += current.x < to.x ? 1 : -1;
            }
            while (current.y != to.y)
            {
                dungeon.Tiles[current.x, current.y] = TileType.Floor;
                current.y += current.y < to.y ? 1 : -1;
            }
        }
        else
        {
            while (current.y != to.y)
            {
                dungeon.Tiles[current.x, current.y] = TileType.Floor;
                current.y += current.y < to.y ? 1 : -1;
            }
            while (current.x != to.x)
            {
                dungeon.Tiles[current.x, current.y] = TileType.Floor;
                current.x += current.x < to.x ? 1 : -1;
            }
        }
        dungeon.Tiles[to.x, to.y] = TileType.Floor;
    }

    private void AssignRoomTypes(DungeonModel dungeon, System.Random random, int floor)
    {
        if (dungeon.Rooms.Count == 0)
        {
            return;
        }

        dungeon.Rooms[0].Type = RoomType.Start;
        dungeon.Rooms[0].IsVisited = true;
        dungeon.Rooms[0].IsRevealed = true;

        dungeon.Rooms[dungeon.Rooms.Count - 1].Type = floor % m_Config.FloorsPerBoss == 0
            ? RoomType.Boss
            : RoomType.Exit;

        // Place stairs in the last room
        RoomData exitRoom = dungeon.Rooms[dungeon.Rooms.Count - 1];
        Vector2Int exitCenter = exitRoom.Center;
        dungeon.Tiles[exitCenter.x, exitCenter.y] = TileType.StairsDown;

        // Assign special rooms among the middle rooms
        for (int roomIndex = 1; roomIndex < dungeon.Rooms.Count - 1; roomIndex++)
        {
            int roll = random.Next(100);
            if (roll < 10)
            {
                dungeon.Rooms[roomIndex].Type = RoomType.Treasure;
            }
            else if (roll < 18)
            {
                dungeon.Rooms[roomIndex].Type = RoomType.Shop;
            }
            else if (roll < 25)
            {
                dungeon.Rooms[roomIndex].Type = RoomType.Rest;
            }
        }
    }

    public void Dispose() { }
}

public sealed class BSPNode
{
    public RectInt Bounds { get; }
    public BSPNode Left { get; set; }
    public BSPNode Right { get; set; }

    public BSPNode(RectInt bounds) { Bounds = bounds; }
}
```

### Dungeon Config (ScriptableObject)

```csharp
[CreateAssetMenu(menuName = "Roguelike/Dungeon Config")]
public sealed class DungeonConfig : ScriptableObject
{
    [SerializeField] private int m_Width = 64;
    [SerializeField] private int m_Height = 64;
    [SerializeField] private int m_MinRoomSize = 5;
    [SerializeField] private int m_MaxSplitDepth = 5;
    [SerializeField] private int m_FloorsPerBoss = 5;

    public int Width => m_Width;
    public int Height => m_Height;
    public int MinRoomSize => m_MinRoomSize;
    public int MaxSplitDepth => m_MaxSplitDepth;
    public int FloorsPerBoss => m_FloorsPerBoss;
}
```

**Seed-based reproducibility.** Always pass the seed to `System.Random`. The seed is stored in `DungeonModel` so the dungeon can be regenerated identically. Combine the run seed with the floor number to get a unique seed per floor: `seed + floor * 1000`.

---

## Turn System

### Turn-Based Architecture

The turn manager controls action order. Each actor (player and enemies) has an energy pool. Acting costs energy. Actors with higher speed accumulate energy faster and act more often.

```csharp
public sealed class TurnModel
{
    public int TurnNumber { get; set; }
    public bool IsPlayerTurn { get; set; } = true;
    public bool IsWaitingForInput { get; set; }
}

public interface ITurnActor
{
    int Speed { get; }
    int Energy { get; set; }
    bool IsAlive { get; }
    UniTask TakeAction(CancellationToken token);
}

public sealed class TurnSystem : IDisposable
{
    private readonly TurnModel m_Model;
    private readonly List<ITurnActor> m_Actors = new();
    private readonly IPublisher<TurnAdvancedMessage> m_TurnPublisher;
    private readonly CancellationTokenSource m_Cts = new();
    private static readonly int k_EnergyThreshold = 100;

    [Inject]
    public TurnSystem(
        TurnModel model,
        IPublisher<TurnAdvancedMessage> turnPublisher)
    {
        m_Model = model;
        m_TurnPublisher = turnPublisher;
    }

    public void RegisterActor(ITurnActor actor)
    {
        m_Actors.Add(actor);
    }

    public void UnregisterActor(ITurnActor actor)
    {
        m_Actors.Remove(actor);
    }

    public async UniTask RunTurnLoop(CancellationToken token)
    {
        while (!token.IsCancellationRequested)
        {
            // Grant energy to all actors
            for (int actorIndex = 0; actorIndex < m_Actors.Count; actorIndex++)
            {
                ITurnActor actor = m_Actors[actorIndex];
                if (actor.IsAlive)
                {
                    actor.Energy += actor.Speed;
                }
            }

            // Process actors that have enough energy (highest energy first)
            bool anyActed = true;
            while (anyActed)
            {
                anyActed = false;
                ITurnActor bestActor = null;
                int bestEnergy = k_EnergyThreshold;

                for (int actorIndex = 0; actorIndex < m_Actors.Count; actorIndex++)
                {
                    ITurnActor actor = m_Actors[actorIndex];
                    if (actor.IsAlive && actor.Energy >= k_EnergyThreshold && actor.Energy > bestEnergy)
                    {
                        bestEnergy = actor.Energy;
                        bestActor = actor;
                    }
                }

                if (bestActor != null)
                {
                    bestActor.Energy -= k_EnergyThreshold;
                    await bestActor.TakeAction(token);
                    anyActed = true;
                }
            }

            m_Model.TurnNumber++;
            m_TurnPublisher.Publish(new TurnAdvancedMessage(m_Model.TurnNumber));
        }
    }

    public void Dispose() => m_Cts.Cancel();
}

public readonly struct TurnAdvancedMessage
{
    public readonly int TurnNumber;
    public TurnAdvancedMessage(int turnNumber) { TurnNumber = turnNumber; }
}
```

### Real-Time Alternative

For action roguelikes (Hades-style), replace the turn system with cooldown-based actions. Each action has a cooldown timer. The player acts freely, and enemies act on their own timers via `ITickable`.

```csharp
public sealed class ActionCooldown
{
    public float BaseCooldown { get; }
    public float Remaining { get; set; }
    public bool IsReady => Remaining <= 0f;

    public ActionCooldown(float baseCooldown)
    {
        BaseCooldown = baseCooldown;
    }

    public void Trigger()
    {
        Remaining = BaseCooldown;
    }

    public void Tick(float dt)
    {
        if (Remaining > 0f)
        {
            Remaining -= dt;
        }
    }
}
```

---

## Loot and Item System

### Item Definition

```csharp
public enum ItemRarity { Common, Uncommon, Rare, Epic, Legendary, Cursed }

[CreateAssetMenu(menuName = "Roguelike/Item Definition")]
public sealed class ItemDefinition : ScriptableObject
{
    [SerializeField] private string m_ItemId;
    [SerializeField] private string m_DisplayName;
    [TextArea] [SerializeField] private string m_Description;
    [SerializeField] private Sprite m_Icon;
    [SerializeField] private ItemRarity m_Rarity;
    [SerializeField] private StatModifier[] m_StatModifiers;
    [SerializeField] private bool m_IsCursed;
    [SerializeField] private string m_CurseDescription;
    [SerializeField] private int m_ShopPrice;

    public string ItemId => m_ItemId;
    public string DisplayName => m_DisplayName;
    public string Description => m_Description;
    public Sprite Icon => m_Icon;
    public ItemRarity Rarity => m_Rarity;
    public IReadOnlyList<StatModifier> StatModifiers => m_StatModifiers;
    public bool IsCursed => m_IsCursed;
    public string CurseDescription => m_CurseDescription;
    public int ShopPrice => m_ShopPrice;
}
```

### Drop Table

```csharp
[System.Serializable]
public sealed class DropEntry
{
    [SerializeField] private ItemDefinition m_Item;
    [SerializeField] private float m_Weight;

    public ItemDefinition Item => m_Item;
    public float Weight => m_Weight;
}

[CreateAssetMenu(menuName = "Roguelike/Drop Table")]
public sealed class DropTable : ScriptableObject
{
    [SerializeField] private DropEntry[] m_Entries;
    [SerializeField] private float m_NothingWeight;

    public ItemDefinition Roll(System.Random random)
    {
        float totalWeight = m_NothingWeight;
        for (int entryIndex = 0; entryIndex < m_Entries.Length; entryIndex++)
        {
            totalWeight += m_Entries[entryIndex].Weight;
        }

        float roll = (float)(random.NextDouble() * totalWeight);
        float cumulative = m_NothingWeight;

        if (roll < cumulative)
        {
            return null;
        }

        for (int entryIndex = 0; entryIndex < m_Entries.Length; entryIndex++)
        {
            cumulative += m_Entries[entryIndex].Weight;
            if (roll < cumulative)
            {
                return m_Entries[entryIndex].Item;
            }
        }

        return null;
    }
}
```

### Inventory Model

```csharp
public sealed class InventoryModel
{
    private readonly ItemDefinition[] m_Slots;
    private readonly int m_MaxSlots;

    public int MaxSlots => m_MaxSlots;
    public ReactiveProperty<int> ItemCount { get; } = new(0);

    public InventoryModel(int maxSlots)
    {
        m_MaxSlots = maxSlots;
        m_Slots = new ItemDefinition[maxSlots];
    }

    public bool IsFull => ItemCount.Value >= m_MaxSlots;

    public ItemDefinition GetItem(int slotIndex)
    {
        return m_Slots[slotIndex];
    }

    public bool TryAddItem(ItemDefinition item)
    {
        for (int slotIndex = 0; slotIndex < m_MaxSlots; slotIndex++)
        {
            if (m_Slots[slotIndex] == null)
            {
                m_Slots[slotIndex] = item;
                ItemCount.Value++;
                return true;
            }
        }
        return false;
    }

    public ItemDefinition RemoveItem(int slotIndex)
    {
        ItemDefinition item = m_Slots[slotIndex];
        if (item != null)
        {
            m_Slots[slotIndex] = null;
            ItemCount.Value--;
        }
        return item;
    }

    public void Clear()
    {
        for (int slotIndex = 0; slotIndex < m_MaxSlots; slotIndex++)
        {
            m_Slots[slotIndex] = null;
        }
        ItemCount.Value = 0;
    }
}
```

### Loot System

The loot system handles item pickup, stat application, and cursed item behavior. When an item is picked up, its stat modifiers are applied to the player. Cursed items cannot be unequipped until the curse is removed (at a rest room or via a specific consumable).

```csharp
public readonly struct ItemPickedUpMessage
{
    public readonly ItemDefinition Item;
    public ItemPickedUpMessage(ItemDefinition item) { Item = item; }
}

public sealed class LootSystem : IDisposable
{
    private readonly InventoryModel m_Inventory;
    private readonly PlayerStatsModel m_PlayerStats;
    private readonly IPublisher<ItemPickedUpMessage> m_PickupPublisher;

    [Inject]
    public LootSystem(
        InventoryModel inventory,
        PlayerStatsModel playerStats,
        IPublisher<ItemPickedUpMessage> pickupPublisher)
    {
        m_Inventory = inventory;
        m_PlayerStats = playerStats;
        m_PickupPublisher = pickupPublisher;
    }

    public bool TryPickupItem(ItemDefinition item)
    {
        if (m_Inventory.IsFull)
        {
            return false;
        }

        if (!m_Inventory.TryAddItem(item))
        {
            return false;
        }

        ApplyModifiers(item);
        m_PickupPublisher.Publish(new ItemPickedUpMessage(item));
        return true;
    }

    private void ApplyModifiers(ItemDefinition item)
    {
        IReadOnlyList<StatModifier> modifiers = item.StatModifiers;
        for (int modIndex = 0; modIndex < modifiers.Count; modIndex++)
        {
            m_PlayerStats.AddModifier(modifiers[modIndex]);
        }
    }

    public void Dispose() { }
}
```

---

## Permadeath and Run State

### Run Model

The RunModel tracks all per-run state. It is created fresh at the start of each run and destroyed on death.

```csharp
public sealed class RunModel
{
    public int Seed { get; }
    public int CurrentFloor { get; set; } = 1;
    public int TotalKills { get; set; }
    public int TotalGoldEarned { get; set; }
    public int Gold { get; set; }
    public float ElapsedTime { get; set; }
    public bool IsActive { get; set; } = true;
    public string DeathCause { get; set; }

    public RunModel(int seed)
    {
        Seed = seed;
    }

    public int CalculateScore()
    {
        int floorBonus = CurrentFloor * 500;
        int killBonus = TotalKills * 10;
        int timeBonus = Mathf.Max(0, 10000 - Mathf.RoundToInt(ElapsedTime));
        return floorBonus + killBonus + TotalGoldEarned + timeBonus;
    }
}
```

### Run Manager System

```csharp
public readonly struct RunEndedMessage
{
    public readonly RunModel Run;
    public readonly bool Victory;
    public RunEndedMessage(RunModel run, bool victory) { Run = run; Victory = victory; }
}

public sealed class RunManagerSystem : IDisposable
{
    private RunModel m_CurrentRun;
    private readonly MetaProgressionModel m_Meta;
    private readonly IPublisher<RunEndedMessage> m_RunEndPublisher;

    [Inject]
    public RunManagerSystem(
        MetaProgressionModel meta,
        IPublisher<RunEndedMessage> runEndPublisher)
    {
        m_Meta = meta;
        m_RunEndPublisher = runEndPublisher;
    }

    public RunModel StartNewRun()
    {
        int seed = System.Environment.TickCount;
        m_CurrentRun = new RunModel(seed);
        return m_CurrentRun;
    }

    public void EndRun(bool victory, string deathCause = null)
    {
        if (m_CurrentRun == null || !m_CurrentRun.IsActive)
        {
            return;
        }

        m_CurrentRun.IsActive = false;
        m_CurrentRun.DeathCause = deathCause;

        int score = m_CurrentRun.CalculateScore();
        int metaCurrency = CalculateMetaCurrency(score, victory);
        m_Meta.MetaCurrency.Value += metaCurrency;

        if (score > m_Meta.HighScore.Value)
        {
            m_Meta.HighScore.Value = score;
        }

        m_RunEndPublisher.Publish(new RunEndedMessage(m_CurrentRun, victory));
        m_CurrentRun = null;
    }

    private int CalculateMetaCurrency(int score, bool victory)
    {
        float multiplier = victory ? 2f : 1f;
        return Mathf.RoundToInt(score * 0.01f * multiplier);
    }

    public void Dispose() { }
}
```

### Death Screen

The death screen View subscribes to `RunEndedMessage` and displays: floor reached, kill count, gold earned, elapsed time, final score, meta-currency earned, and cause of death. A "Try Again" button calls `RunManagerSystem.StartNewRun()`.

---

## Meta-Progression

### Meta-Progression Model

This model persists across runs. It is saved to disk via the save system.

```csharp
public sealed class MetaProgressionModel
{
    public ReactiveProperty<int> MetaCurrency { get; } = new(0);
    public ReactiveProperty<int> HighScore { get; } = new(0);
    public HashSet<string> UnlockedUpgradeIds { get; } = new();
    public HashSet<string> UnlockedCharacterIds { get; } = new();
    public int TotalRuns { get; set; }
    public int TotalDeaths { get; set; }
    public int BestFloor { get; set; }
}
```

### Unlock Definition

```csharp
public enum UnlockCategory { StartingItem, PassiveBonus, CharacterClass, NewEnemy, NewItem }

[CreateAssetMenu(menuName = "Roguelike/Unlock Definition")]
public sealed class UnlockDefinition : ScriptableObject
{
    [SerializeField] private string m_UnlockId;
    [SerializeField] private string m_DisplayName;
    [TextArea] [SerializeField] private string m_Description;
    [SerializeField] private Sprite m_Icon;
    [SerializeField] private UnlockCategory m_Category;
    [SerializeField] private int m_Cost;
    [SerializeField] private UnlockDefinition[] m_Prerequisites;
    [SerializeField] private StatModifier[] m_PassiveBonuses;

    public string UnlockId => m_UnlockId;
    public string DisplayName => m_DisplayName;
    public int Cost => m_Cost;
    public UnlockCategory Category => m_Category;
    public IReadOnlyList<UnlockDefinition> Prerequisites => m_Prerequisites;
    public IReadOnlyList<StatModifier> PassiveBonuses => m_PassiveBonuses;
}
```

### Meta-Progression System

```csharp
public readonly struct UnlockPurchasedMessage
{
    public readonly UnlockDefinition Unlock;
    public UnlockPurchasedMessage(UnlockDefinition unlock) { Unlock = unlock; }
}

public sealed class MetaProgressionSystem : IDisposable
{
    private readonly MetaProgressionModel m_Model;
    private readonly UnlockDefinition[] m_AllUnlocks;
    private readonly IPublisher<UnlockPurchasedMessage> m_UnlockPublisher;

    [Inject]
    public MetaProgressionSystem(
        MetaProgressionModel model,
        UnlockDefinition[] allUnlocks,
        IPublisher<UnlockPurchasedMessage> unlockPublisher)
    {
        m_Model = model;
        m_AllUnlocks = allUnlocks;
        m_UnlockPublisher = unlockPublisher;
    }

    public bool CanAfford(UnlockDefinition unlock)
    {
        return m_Model.MetaCurrency.Value >= unlock.Cost;
    }

    public bool HasPrerequisites(UnlockDefinition unlock)
    {
        IReadOnlyList<UnlockDefinition> prereqs = unlock.Prerequisites;
        for (int prereqIndex = 0; prereqIndex < prereqs.Count; prereqIndex++)
        {
            if (!m_Model.UnlockedUpgradeIds.Contains(prereqs[prereqIndex].UnlockId))
            {
                return false;
            }
        }
        return true;
    }

    public bool TryPurchaseUnlock(UnlockDefinition unlock)
    {
        if (m_Model.UnlockedUpgradeIds.Contains(unlock.UnlockId))
        {
            return false;
        }

        if (!CanAfford(unlock) || !HasPrerequisites(unlock))
        {
            return false;
        }

        m_Model.MetaCurrency.Value -= unlock.Cost;
        m_Model.UnlockedUpgradeIds.Add(unlock.UnlockId);
        m_UnlockPublisher.Publish(new UnlockPurchasedMessage(unlock));
        return true;
    }

    public void ApplyPassiveBonuses(PlayerStatsModel playerStats)
    {
        for (int unlockIndex = 0; unlockIndex < m_AllUnlocks.Length; unlockIndex++)
        {
            UnlockDefinition unlock = m_AllUnlocks[unlockIndex];
            if (!m_Model.UnlockedUpgradeIds.Contains(unlock.UnlockId))
            {
                continue;
            }

            IReadOnlyList<StatModifier> bonuses = unlock.PassiveBonuses;
            for (int bonusIndex = 0; bonusIndex < bonuses.Count; bonusIndex++)
            {
                playerStats.AddModifier(bonuses[bonusIndex]);
            }
        }
    }

    public void Dispose() { }
}
```

**Balancing meta-progression.** Passive bonuses should provide convenience, not trivialize the game. Cap total meta-bonus per stat (e.g., max +25% health from unlocks). Gate the strongest unlocks behind high costs and deep prerequisite chains. Test by playing a fresh save to ensure the base game is completable without meta-progression.

---

## Enemy Encounter Design

### Enemy Scaling

Enemy stats increase per floor. Use a simple scaling formula rather than hand-tuning every floor.

```csharp
[CreateAssetMenu(menuName = "Roguelike/Enemy Definition")]
public sealed class EnemyDefinition : ScriptableObject
{
    [SerializeField] private string m_EnemyId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private int m_BaseHealth;
    [SerializeField] private int m_BaseDamage;
    [SerializeField] private int m_BaseSpeed;
    [SerializeField] private int m_XPReward;
    [SerializeField] private DropTable m_DropTable;
    [SerializeField] private int m_MinFloor;

    public string EnemyId => m_EnemyId;
    public int BaseHealth => m_BaseHealth;
    public int BaseDamage => m_BaseDamage;
    public int BaseSpeed => m_BaseSpeed;
    public int XPReward => m_XPReward;
    public DropTable DropTable => m_DropTable;
    public int MinFloor => m_MinFloor;

    public int GetScaledHealth(int floor)
    {
        return Mathf.RoundToInt(m_BaseHealth * (1f + (floor - 1) * 0.15f));
    }

    public int GetScaledDamage(int floor)
    {
        return Mathf.RoundToInt(m_BaseDamage * (1f + (floor - 1) * 0.1f));
    }
}
```

### Elite Modifiers

Elite enemies are regular enemies with one or more modifiers applied. Modifiers are data-driven.

```csharp
public enum EliteModifier { Fast, Armored, Splitting, Regenerating, Teleporting }

[CreateAssetMenu(menuName = "Roguelike/Elite Config")]
public sealed class EliteConfig : ScriptableObject
{
    [SerializeField] private float m_HealthMultiplier = 2f;
    [SerializeField] private float m_DamageMultiplier = 1.5f;
    [SerializeField] private int m_ExtraXPReward = 50;
    [SerializeField] private EliteModifier[] m_PossibleModifiers;

    public float HealthMultiplier => m_HealthMultiplier;
    public float DamageMultiplier => m_DamageMultiplier;
    public int ExtraXPReward => m_ExtraXPReward;

    public EliteModifier RollModifier(System.Random random)
    {
        return m_PossibleModifiers[random.Next(m_PossibleModifiers.Length)];
    }
}
```

### Boss Encounters

Boss rooms lock the player in (doors close) and spawn a boss with phase-based behavior. Each phase has different attack patterns and triggers at health thresholds.

```csharp
[System.Serializable]
public sealed class BossPhase
{
    [SerializeField] private float m_HealthThreshold;
    [SerializeField] private string m_AttackPattern;
    [SerializeField] private float m_SpeedMultiplier = 1f;
    [SerializeField] private bool m_SpawnsMinions;
    [SerializeField] private EnemyDefinition m_MinionType;
    [SerializeField] private int m_MinionCount;

    public float HealthThreshold => m_HealthThreshold;
    public string AttackPattern => m_AttackPattern;
    public float SpeedMultiplier => m_SpeedMultiplier;
    public bool SpawnsMinions => m_SpawnsMinions;
    public EnemyDefinition MinionType => m_MinionType;
    public int MinionCount => m_MinionCount;
}

[CreateAssetMenu(menuName = "Roguelike/Boss Definition")]
public sealed class BossDefinition : ScriptableObject
{
    [SerializeField] private EnemyDefinition m_BaseEnemy;
    [SerializeField] private BossPhase[] m_Phases;
    [SerializeField] private string m_IntroDialogue;

    public EnemyDefinition BaseEnemy => m_BaseEnemy;
    public IReadOnlyList<BossPhase> Phases => m_Phases;
}
```

---

## Minimap and Fog of War

### Room-Based Visibility

Visibility works at the room level. Rooms have three states: hidden (not on minimap), revealed (shown as outline), and visited (fully visible with contents).

```csharp
public sealed class MinimapModel
{
    private readonly RoomVisibility[] m_RoomStates;

    public MinimapModel(int roomCount)
    {
        m_RoomStates = new RoomVisibility[roomCount];
        for (int roomIndex = 0; roomIndex < roomCount; roomIndex++)
        {
            m_RoomStates[roomIndex] = RoomVisibility.Hidden;
        }
    }

    public RoomVisibility GetVisibility(int roomIndex)
    {
        return m_RoomStates[roomIndex];
    }

    public void RevealRoom(int roomIndex)
    {
        if (m_RoomStates[roomIndex] == RoomVisibility.Hidden)
        {
            m_RoomStates[roomIndex] = RoomVisibility.Revealed;
        }
    }

    public void VisitRoom(int roomIndex)
    {
        m_RoomStates[roomIndex] = RoomVisibility.Visited;
    }
}

public enum RoomVisibility { Hidden, Revealed, Visited }
```

When the player enters a room, mark it as visited and reveal all adjacent rooms (connected by doors). The minimap View reads room bounds from `DungeonModel` and tints each room tile based on its visibility state. Use a separate render texture or UI overlay for the minimap.

---

## Common Pitfalls

**Unsolvable layouts.** BSP generation can produce rooms that are not connected if the corridor carving fails to reach a room. Always validate connectivity after generation: flood-fill from the start room and confirm every room is reachable. If not, regenerate with a different seed offset.

**Loot balance.** Too many item drops per floor makes the player overpowered by floor 5. Too few makes runs feel unrewarding. Start with one guaranteed drop per combat room (from the room's drop table) and one from elite/boss enemies. Tune from there via playtesting.

**Meta-progression trivializing runs.** If passive bonuses stack too high, early floors become boring. Cap total meta-stat bonuses. Alternatively, scale dungeon difficulty slightly based on unlocked meta-upgrades so early floors remain challenging.

**Turn order with status effects.** Status effects that modify speed (haste, slow) can change turn order mid-combat, causing confusing behavior. Recalculate energy gains at the start of each turn cycle, not mid-cycle. Apply speed changes at the beginning of the next full turn.

**Cursed items softlocking.** If the inventory is full of cursed items the player cannot drop, they are stuck. Always provide an escape: rest rooms that remove one curse, a consumable that uncurses, or a "sacrifice health to drop" mechanic.

**Dead-end rooms with no loot.** If a room type is combat but the drop table rolls "nothing" and the room has no other reward, the player feels punished for exploring. Guarantee at least a small gold drop from every combat room. Reserve "nothing" drops for bonus chests and destructible props.

---

## Performance

### Generate Rooms on Demand

Do not instantiate the entire dungeon at once. Generate and render only the current room and its immediate neighbors. When the player moves to a new room, destroy or pool the tiles from rooms two or more rooms away.

```csharp
public sealed class DungeonRenderSystem : ITickable, IDisposable
{
    private int m_LastRenderedRoomIndex = -1;
    private readonly DungeonModel m_Dungeon;
    private readonly TilePoolSystem m_TilePool;

    [Inject]
    public DungeonRenderSystem(DungeonModel dungeon, TilePoolSystem tilePool)
    {
        m_Dungeon = dungeon;
        m_TilePool = tilePool;
    }

    public void Tick()
    {
        int currentRoom = GetCurrentRoomIndex();
        if (currentRoom == m_LastRenderedRoomIndex)
        {
            return;
        }

        // Return tiles from distant rooms to pool
        for (int roomIndex = 0; roomIndex < m_Dungeon.Rooms.Count; roomIndex++)
        {
            if (Mathf.Abs(roomIndex - currentRoom) > 1)
            {
                m_TilePool.ReturnRoomTiles(roomIndex);
            }
        }

        // Render current room and adjacent rooms
        RenderRoom(currentRoom);
        List<int> adjacentRooms = GetAdjacentRoomIndices(currentRoom);
        for (int adjIndex = 0; adjIndex < adjacentRooms.Count; adjIndex++)
        {
            RenderRoom(adjacentRooms[adjIndex]);
        }

        m_LastRenderedRoomIndex = currentRoom;
    }

    private void RenderRoom(int roomIndex)
    {
        // Pull tiles from pool and position them according to DungeonModel.Tiles
        m_TilePool.RenderRoom(m_Dungeon, roomIndex);
    }

    private int GetCurrentRoomIndex() { return 0; }
    private List<int> GetAdjacentRoomIndices(int roomIndex) { return new List<int>(); }

    public void Dispose() { }
}
```

### Tile-Based Rendering Optimization

Use a tilemap (Unity 2D Tilemap or a custom mesh-based solution for 3D) rather than individual GameObjects per tile. For 3D roguelikes, batch all floor tiles into a single mesh and all wall tiles into another. Rebuild only when the player changes rooms.

### Pool Everything

Pool enemies, projectiles, VFX, loot pickup objects, and tile GameObjects. Size pools based on the largest room dimensions. Pre-warm pools during the loading screen between floors.

### Enemy AI Budget

Limit the number of enemies that run pathfinding per frame. Use a staggered update where only a fraction of enemies recalculate their path each tick. Enemies outside the current room should be frozen entirely (no AI, no rendering, no physics).
