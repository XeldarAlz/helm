---
name: match3
description: "Match-3 puzzle game architecture — grid system, tile matching, cascade/gravity, special tiles, combo chains, level objectives, lives/energy system."
globs: ["**/Match*.cs", "**/Grid*.cs", "**/Tile*.cs", "**/Board*.cs", "**/Puzzle*.cs"]
---

# Match-3 Puzzle Patterns

## Grid System

```csharp
public sealed class Board : MonoBehaviour
{
    [SerializeField] private int m_Width = 7;
    [SerializeField] private int m_Height = 9;
    [SerializeField] private float m_CellSize = 1f;
    [SerializeField] private TileDefinition[] m_TileTypes;

    private Tile[,] m_Grid;

    private void Awake()
    {
        m_Grid = new Tile[m_Width, m_Height];
    }

    public Vector3 GridToWorld(int x, int y)
    {
        float offsetX = (m_Width - 1) * 0.5f;
        float offsetY = (m_Height - 1) * 0.5f;
        return new Vector3((x - offsetX) * m_CellSize, (y - offsetY) * m_CellSize, 0f);
    }

    public bool IsValidPosition(int x, int y)
    {
        return x >= 0 && x < m_Width && y >= 0 && y < m_Height;
    }

    public Tile GetTile(int x, int y)
    {
        if (!IsValidPosition(x, y)) return null;
        return m_Grid[x, y];
    }

    public void SetTile(int x, int y, Tile tile)
    {
        m_Grid[x, y] = tile;
        if (tile != null)
        {
            tile.GridX = x;
            tile.GridY = y;
        }
    }
}
```

## Tile Definition

```csharp
[CreateAssetMenu(menuName = "Match3/Tile Definition")]
public sealed class TileDefinition : ScriptableObject
{
    [SerializeField] private string m_TileId;
    [SerializeField] private Sprite m_Sprite;
    [SerializeField] private Color m_Color = Color.white;
    [SerializeField] private TileType m_Type = TileType.Normal;

    public string TileId => m_TileId;
    public Sprite Sprite => m_Sprite;
    public Color Color => m_Color;
    public TileType Type => m_Type;
}

public enum TileType
{
    Normal,
    StripedHorizontal,
    StripedVertical,
    Wrapped,
    ColorBomb,
    Blocker,
    Ice,
    Chain
}
```

## Swap & Match Detection

```csharp
public sealed class MatchDetector
{
    private readonly Board m_Board;
    private readonly List<MatchResult> m_MatchBuffer = new(16);

    public MatchDetector(Board board) { m_Board = board; }

    public List<MatchResult> FindAllMatches()
    {
        m_MatchBuffer.Clear();
        FindHorizontalMatches();
        FindVerticalMatches();
        return m_MatchBuffer;
    }

    private void FindHorizontalMatches()
    {
        for (int y = 0; y < m_Board.Height; y++)
        {
            int matchStart = 0;
            for (int x = 1; x <= m_Board.Width; x++)
            {
                bool matches = x < m_Board.Width &&
                    m_Board.GetTile(x, y) != null &&
                    m_Board.GetTile(matchStart, y) != null &&
                    m_Board.GetTile(x, y).Definition.TileId ==
                    m_Board.GetTile(matchStart, y).Definition.TileId;

                if (!matches)
                {
                    int length = x - matchStart;
                    if (length >= 3)
                    {
                        MatchResult match = new MatchResult();
                        match.Direction = MatchDirection.Horizontal;
                        for (int mx = matchStart; mx < x; mx++)
                        {
                            match.Tiles.Add(m_Board.GetTile(mx, y));
                        }
                        m_MatchBuffer.Add(match);
                    }
                    matchStart = x;
                }
            }
        }
    }

    private void FindVerticalMatches()
    {
        for (int x = 0; x < m_Board.Width; x++)
        {
            int matchStart = 0;
            for (int y = 1; y <= m_Board.Height; y++)
            {
                bool matches = y < m_Board.Height &&
                    m_Board.GetTile(x, y) != null &&
                    m_Board.GetTile(x, matchStart) != null &&
                    m_Board.GetTile(x, y).Definition.TileId ==
                    m_Board.GetTile(x, matchStart).Definition.TileId;

                if (!matches)
                {
                    int length = y - matchStart;
                    if (length >= 3)
                    {
                        MatchResult match = new MatchResult();
                        match.Direction = MatchDirection.Vertical;
                        for (int my = matchStart; my < y; my++)
                        {
                            match.Tiles.Add(m_Board.GetTile(x, my));
                        }
                        m_MatchBuffer.Add(match);
                    }
                    matchStart = y;
                }
            }
        }
    }
}
```

## Cascade / Gravity

After matches are cleared:
1. Remove matched tiles (play destroy animation)
2. Gravity: tiles above empty spaces fall down
3. Refill: new tiles spawn from top
4. Re-check for new matches (chain combo)
5. Repeat until no matches remain

```csharp
// Coroutine-based cascade loop
private IEnumerator CascadeLoop()
{
    List<MatchResult> matches = m_Detector.FindAllMatches();
    int comboCount = 0;

    while (matches.Count > 0)
    {
        comboCount++;
        yield return StartCoroutine(DestroyMatches(matches, comboCount));
        yield return StartCoroutine(ApplyGravity());
        yield return StartCoroutine(RefillBoard());
        matches = m_Detector.FindAllMatches();
    }

    CheckLevelObjective();
    EnableInput();
}
```

## Special Tile Rules

| Match | Creates |
|-------|---------|
| 4 in a row | Striped tile (clears row or column) |
| L or T shape | Wrapped tile (explodes 3x3 area) |
| 5 in a row | Color bomb (clears all of one color) |

| Combo | Effect |
|-------|--------|
| Striped + Striped | Cross clear (row + column) |
| Striped + Wrapped | 3-row or 3-column clear |
| Wrapped + Wrapped | 5x5 explosion |
| Color bomb + any | Clears all tiles of that color |
| Color bomb + Color bomb | Clears entire board |

## Touch Input for Swap

- Drag threshold: 0.5 cells (half the cell size)
- Only allow horizontal/vertical swaps (snap to nearest direction)
- Invalid swap: animate swap then swap back
- Disable input during cascade animation

## Level Objectives

- **Score target:** reach N points in M moves
- **Clear blockers:** destroy all ice/chain tiles
- **Collect items:** match next to items to collect them
- **Reach bottom:** guide special tile to bottom row

## Lives / Energy System

```csharp
public sealed class LivesSystem : MonoBehaviour
{
    [SerializeField] private int m_MaxLives = 5;
    [SerializeField] private int m_RegenMinutes = 30;

    private int m_CurrentLives;
    private System.DateTime m_LastLifeLostTime;

    public int CurrentLives => m_CurrentLives;
    public bool HasLives => m_CurrentLives > 0;

    public void UseLive()
    {
        if (m_CurrentLives <= 0) return;
        m_CurrentLives--;
        if (m_CurrentLives < m_MaxLives)
        {
            m_LastLifeLostTime = System.DateTime.UtcNow;
        }
        Save();
    }

    public void CheckRegen()
    {
        if (m_CurrentLives >= m_MaxLives) return;
        System.TimeSpan elapsed = System.DateTime.UtcNow - m_LastLifeLostTime;
        int livesRegened = (int)(elapsed.TotalMinutes / m_RegenMinutes);
        if (livesRegened > 0)
        {
            m_CurrentLives = Mathf.Min(m_MaxLives, m_CurrentLives + livesRegened);
            m_LastLifeLostTime = m_LastLifeLostTime.AddMinutes(livesRegened * m_RegenMinutes);
            Save();
        }
    }

    private void Save()
    {
        PlayerPrefs.SetInt("Lives", m_CurrentLives);
        PlayerPrefs.SetString("LastLifeLost", m_LastLifeLostTime.ToBinary().ToString());
        PlayerPrefs.Save();
    }
}
```

## Performance Notes

- Pool all tiles — never Instantiate/Destroy during gameplay
- Use SpriteRenderer or UI Image, not 3D meshes
- Pre-calculate match possibilities for hint system
- Animate with DOTween for smooth tile movement

---

## Complete Game Loop

### Game State Machine

```csharp
public enum GameLoopState
{
    WaitingForInput,
    Swapping,
    Matching,
    Cascading,
    Refilling,
    Evaluating,
    GameOver
}
```

### Game Controller (Plain C# System, VContainer-Injected)

```csharp
public sealed class GameLoopSystem : IDisposable
{
    private readonly BoardModel m_Board;
    private readonly MatchDetector m_Detector;
    private readonly SwapSystem m_SwapSystem;
    private readonly CascadeSystem m_CascadeSystem;
    private readonly ObjectiveTracker m_ObjectiveTracker;
    private readonly IPublisher<GameStateChangedMessage> m_StatePublisher;
    private readonly CancellationTokenSource m_Cts = new();

    private GameLoopState m_CurrentState = GameLoopState.WaitingForInput;
    private int m_MovesRemaining;
    private int m_CascadeDepth;

    [Inject]
    public GameLoopSystem(
        BoardModel board,
        MatchDetector detector,
        SwapSystem swapSystem,
        CascadeSystem cascadeSystem,
        ObjectiveTracker objectiveTracker,
        IPublisher<GameStateChangedMessage> statePublisher)
    {
        m_Board = board;
        m_Detector = detector;
        m_SwapSystem = swapSystem;
        m_CascadeSystem = cascadeSystem;
        m_ObjectiveTracker = objectiveTracker;
        m_StatePublisher = statePublisher;
    }

    public async UniTaskVoid ProcessSwap(int fromX, int fromY, int toX, int toY)
    {
        if (m_CurrentState != GameLoopState.WaitingForInput) return;

        SetState(GameLoopState.Swapping);
        bool validSwap = await m_SwapSystem.TrySwap(fromX, fromY, toX, toY, m_Cts.Token);

        if (!validSwap)
        {
            // Animate swap-back, return to input
            await m_SwapSystem.AnimateSwapBack(fromX, fromY, toX, toY, m_Cts.Token);
            SetState(GameLoopState.WaitingForInput);
            return;
        }

        m_MovesRemaining--;
        m_CascadeDepth = 0;

        // Core loop: match → destroy → gravity → refill → repeat
        SetState(GameLoopState.Matching);
        List<MatchResult> matches = m_Detector.FindAllMatches();

        while (matches.Count > 0)
        {
            m_CascadeDepth++;

            // Destroy matched tiles, create special pieces if applicable
            SetState(GameLoopState.Cascading);
            await m_CascadeSystem.DestroyMatches(matches, m_CascadeDepth, m_Cts.Token);
            m_ObjectiveTracker.ReportMatches(matches, m_CascadeDepth);

            // Apply gravity — tiles fall to fill empty spaces
            await m_CascadeSystem.ApplyGravity(m_Cts.Token);

            // Refill from top
            SetState(GameLoopState.Refilling);
            await m_CascadeSystem.RefillBoard(m_Cts.Token);

            // Check for new matches created by gravity/refill
            SetState(GameLoopState.Matching);
            matches = m_Detector.FindAllMatches();
        }

        // Evaluate win/loss after cascade settles
        SetState(GameLoopState.Evaluating);
        if (m_ObjectiveTracker.AllObjectivesMet())
        {
            SetState(GameLoopState.GameOver);
            // Win — publish victory message
            return;
        }

        if (m_MovesRemaining <= 0)
        {
            SetState(GameLoopState.GameOver);
            // Loss — publish defeat message
            return;
        }

        SetState(GameLoopState.WaitingForInput);
    }

    private void SetState(GameLoopState newState)
    {
        m_CurrentState = newState;
        m_StatePublisher.Publish(new GameStateChangedMessage(newState));
    }

    public void Dispose() => m_Cts.Cancel();
}
```

### Objective Tracking

```csharp
public sealed class ObjectiveTracker
{
    private readonly List<LevelObjective> m_Objectives = new();
    private int m_TotalScore;

    public void LoadObjectives(LevelDefinition level)
    {
        m_Objectives.Clear();
        for (int objectiveIndex = 0; objectiveIndex < level.Objectives.Count; objectiveIndex++)
        {
            m_Objectives.Add(level.Objectives[objectiveIndex].Clone());
        }
    }

    public void ReportMatches(List<MatchResult> matches, int cascadeDepth)
    {
        int scoreMultiplier = cascadeDepth; // Chain bonus
        for (int matchIndex = 0; matchIndex < matches.Count; matchIndex++)
        {
            MatchResult match = matches[matchIndex];
            int matchScore = match.Tiles.Count * 10 * scoreMultiplier;
            m_TotalScore += matchScore;

            // Update collection objectives
            for (int objectiveIndex = 0; objectiveIndex < m_Objectives.Count; objectiveIndex++)
            {
                m_Objectives[objectiveIndex].ReportMatch(match);
            }
        }
    }

    public bool AllObjectivesMet()
    {
        for (int objectiveIndex = 0; objectiveIndex < m_Objectives.Count; objectiveIndex++)
        {
            if (!m_Objectives[objectiveIndex].IsMet) return false;
        }
        return true;
    }
}
```

The input system (View layer) must check `m_CurrentState == GameLoopState.WaitingForInput` before forwarding swap gestures to the GameLoopSystem. This prevents input during cascades.

---

## Advanced Combo Mechanics

### Chain Multiplier and Combo Counter

```csharp
public sealed class ComboSystem : IDisposable
{
    private readonly IPublisher<ComboMessage> m_ComboPublisher;

    private int m_ChainCount;
    private int m_ComboCounter;
    private float m_ComboTimer;
    private float m_ComboTimeout = 2f;

    [Inject]
    public ComboSystem(IPublisher<ComboMessage> comboPublisher)
    {
        m_ComboPublisher = comboPublisher;
    }

    // Called by GameLoopSystem for each cascade depth level
    public int GetScoreMultiplier(int cascadeDepth)
    {
        m_ChainCount = cascadeDepth;

        // Multiplier: 1x for first match, 2x for first cascade, 3x for second, etc.
        int multiplier = cascadeDepth;

        // Combo counter tracks consecutive moves that produce cascades
        if (cascadeDepth > 1)
        {
            m_ComboCounter++;
            m_ComboTimer = m_ComboTimeout;
            multiplier += m_ComboCounter / 3; // Bonus every 3 consecutive combo moves
        }

        return multiplier;
    }

    // Called from game tick to decay combo counter between moves
    public void UpdateComboTimer(float deltaTime)
    {
        if (m_ComboCounter <= 0) return;

        m_ComboTimer -= deltaTime;
        if (m_ComboTimer <= 0f)
        {
            m_ComboCounter = 0;
        }
    }

    public void ResetChain()
    {
        m_ChainCount = 0;
    }

    public void Dispose() { }
}
```

### Special Piece Creation from Chain Length

```csharp
public sealed class SpecialPieceFactory
{
    // Determine what special piece to create based on match shape and size
    public TileType DetermineSpecialPiece(MatchResult match, List<MatchResult> allMatches)
    {
        // Check for cross/L/T shapes first (intersection of horizontal + vertical)
        if (HasIntersection(match, allMatches))
        {
            return TileType.Wrapped;
        }

        return match.Tiles.Count switch
        {
            4 => match.Direction == MatchDirection.Horizontal
                ? TileType.StripedHorizontal
                : TileType.StripedVertical,
            >= 5 => TileType.ColorBomb,
            _ => TileType.Normal // No special piece for basic 3-match
        };
    }

    private bool HasIntersection(MatchResult match, List<MatchResult> allMatches)
    {
        for (int otherIndex = 0; otherIndex < allMatches.Count; otherIndex++)
        {
            MatchResult other = allMatches[otherIndex];
            if (other.Direction == match.Direction) continue;

            // Check if any tiles overlap between horizontal and vertical matches
            for (int tileA = 0; tileA < match.Tiles.Count; tileA++)
            {
                for (int tileB = 0; tileB < other.Tiles.Count; tileB++)
                {
                    if (match.Tiles[tileA].GridX == other.Tiles[tileB].GridX &&
                        match.Tiles[tileA].GridY == other.Tiles[tileB].GridY)
                    {
                        return true;
                    }
                }
            }
        }
        return false;
    }
}
```

### Board Juice on Big Combos

For cascades of depth 3+, trigger screen shake and particle bursts via MessagePipe:
- Publish `BigComboMessage` with cascade depth and center position
- The CameraShakeView subscribes and applies shake intensity proportional to combo depth
- The ParticleView subscribes and spawns pooled particle bursts at the match center
- Never trigger juice logic from the System — the View reacts to messages independently

---

## Tutorial System

### First-Time Tutorial Flow

```csharp
public sealed class TutorialSystem : IDisposable
{
    private readonly IPublisher<TutorialStepMessage> m_StepPublisher;
    private readonly CancellationTokenSource m_Cts = new();

    private int m_CurrentStep;
    private bool m_IsActive;
    private bool m_SkipRequested;

    [Inject]
    public TutorialSystem(IPublisher<TutorialStepMessage> stepPublisher)
    {
        m_StepPublisher = stepPublisher;
    }

    public async UniTask RunTutorial(TutorialDefinition definition, CancellationToken token)
    {
        m_IsActive = true;
        m_CurrentStep = 0;

        while (m_CurrentStep < definition.Steps.Count && !m_SkipRequested)
        {
            TutorialStep step = definition.Steps[m_CurrentStep];
            m_StepPublisher.Publish(new TutorialStepMessage(step));

            // Wait for the player to complete the required action
            await UniTask.WaitUntil(() => step.IsCompleted || m_SkipRequested, cancellationToken: token);

            m_CurrentStep++;
        }

        m_IsActive = false;
        // Persist tutorial completion so it never shows again
        PlayerPrefs.SetInt("TutorialComplete", 1);
        PlayerPrefs.Save();
    }

    public void RequestSkip()
    {
        m_SkipRequested = true;
    }

    public bool IsTutorialActive => m_IsActive;
    public void Dispose() => m_Cts.Cancel();
}
```

### Tutorial Step Definition

```csharp
[System.Serializable]
public sealed class TutorialStep
{
    public string InstructionText;
    public int HighlightTileX;      // Grid position to spotlight
    public int HighlightTileY;
    public SwapDirection RequiredSwap; // Direction the player must swipe
    public bool IsCompleted;
}
```

### Progressive Mechanic Unlock

Introduce special pieces over multiple levels rather than all at once:
- **Level 1-3:** Basic matching only, no special pieces generated
- **Level 4:** First striped tile tutorial — force a 4-match scenario, overlay explains the effect
- **Level 7:** First wrapped tile tutorial — pre-placed L-shape opportunity
- **Level 10:** Color bomb introduction — 5-in-a-row forced scenario

Store unlocked mechanic flags in save data. The `SpecialPieceFactory` checks these flags before creating special pieces, returning `TileType.Normal` for locked mechanics.

---

## Monetization Hooks

### Booster System

```csharp
public enum BoosterType
{
    ExtraMoves,         // Add 5 moves
    RowClear,           // Clear one row (player picks)
    ColorBomb,          // Grants a color bomb to place
    Shuffle,            // Rearrange all tiles (guaranteed matches)
    ColumnClear         // Clear one column (player picks)
}

public sealed class BoosterModel
{
    private readonly Dictionary<BoosterType, int> m_BoosterCounts = new();

    public int GetCount(BoosterType type)
    {
        return m_BoosterCounts.TryGetValue(type, out int count) ? count : 0;
    }

    public bool TryConsume(BoosterType type)
    {
        if (!m_BoosterCounts.TryGetValue(type, out int count) || count <= 0) return false;
        m_BoosterCounts[type] = count - 1;
        return true;
    }

    public void Add(BoosterType type, int amount)
    {
        if (!m_BoosterCounts.ContainsKey(type))
        {
            m_BoosterCounts[type] = 0;
        }
        m_BoosterCounts[type] += amount;
    }
}
```

### Continue / Retry Gate

When the player runs out of moves but has not met the objective:
1. Show a "Continue?" dialog with two options
2. **Option A:** Watch rewarded ad — grants 5 extra moves, game resumes
3. **Option B:** Spend premium currency — same effect, no ad
4. If the player declines both, the level is failed and a life is consumed

Limit continues to one per level attempt to prevent trivializing difficulty.

### Energy / Lives with Timer Refill

The `LivesSystem` above handles core regen logic. Additional monetization hooks:
- **Instant refill:** spend premium currency to fill all lives immediately
- **Watch ad for one life:** capped at 3 per day to prevent ad fatigue
- **Unlimited lives pass:** time-limited (30 min, 2 hours) purchasable item that bypasses the lives check

### Daily Reward Calendar

```csharp
public sealed class DailyRewardSystem
{
    private int m_CurrentDay;
    private System.DateTime m_LastClaimDate;

    public bool CanClaimToday()
    {
        return System.DateTime.UtcNow.Date > m_LastClaimDate.Date;
    }

    public DailyRewardEntry ClaimReward(DailyRewardCalendar calendar)
    {
        if (!CanClaimToday()) return null;

        DailyRewardEntry reward = calendar.GetReward(m_CurrentDay);
        m_CurrentDay = (m_CurrentDay + 1) % calendar.TotalDays;
        m_LastClaimDate = System.DateTime.UtcNow;
        return reward;
    }
}
```

---

## Save Integration

### Board State Serialization

```csharp
[System.Serializable]
public sealed class BoardSaveData
{
    public int Width;
    public int Height;
    public List<TileSaveEntry> Tiles;
}

[System.Serializable]
public sealed class TileSaveEntry
{
    public int GridX;
    public int GridY;
    public string TileDefinitionId;
    public TileType SpecialType;
}
```

### Level Progress and Stars

```csharp
[System.Serializable]
public sealed class PlayerProgressSaveData
{
    public int HighestLevelUnlocked;
    public Dictionary<int, int> LevelStars; // levelIndex → star count (1-3)
    public int TotalStars;

    // Currency
    public int Coins;
    public int PremiumCurrency;

    // Boosters
    public List<BoosterSaveEntry> Boosters;

    // Lives
    public int CurrentLives;
    public long LastLifeLostTimeBinary; // DateTime.ToBinary() for serialization

    // Daily rewards
    public int DailyRewardDay;
    public long LastDailyClaimBinary;
}

[System.Serializable]
public sealed class BoosterSaveEntry
{
    public BoosterType Type;
    public int Count;
}
```

### Offline Time Tracking for Energy Refill

On game launch, calculate elapsed time since last save:

```csharp
public sealed class OfflineProgressSystem
{
    public void ProcessOfflineTime(PlayerProgressSaveData save, int regenMinutes, int maxLives)
    {
        if (save.CurrentLives >= maxLives) return;

        System.DateTime lastLost = System.DateTime.FromBinary(save.LastLifeLostTimeBinary);
        System.TimeSpan elapsed = System.DateTime.UtcNow - lastLost;
        int livesRegened = (int)(elapsed.TotalMinutes / regenMinutes);

        if (livesRegened > 0)
        {
            save.CurrentLives = Mathf.Min(maxLives, save.CurrentLives + livesRegened);
            save.LastLifeLostTimeBinary = lastLost.AddMinutes(livesRegened * regenMinutes).ToBinary();
        }
    }
}
```

Save to `Application.persistentDataPath` using JSON serialization. Never use `PlayerPrefs` for large data — it has platform-specific size limits and is not designed for structured data. Use `PlayerPrefs` only for small flags like tutorial completion.

---

## Common Pitfalls

### Match Detection Missing L-Shaped and T-Shaped Patterns
The basic horizontal/vertical scan finds straight lines but misses L and T shapes. After finding all horizontal and vertical matches, check for tile intersections. If a tile belongs to both a horizontal and a vertical match, merge those matches into a single result and flag it for special piece creation (Wrapped tile). Without this merge step, the intersection tile gets counted in two separate matches and may be destroyed twice, causing index errors.

### Cascade Infinite Loop
If the refill step randomly generates tiles that always create new matches, the cascade loop never terminates. Guard against this by:
1. Setting a maximum cascade depth (e.g., 50 iterations) and force-breaking the loop
2. During refill, check if a tile placement would create an immediate match and pick a different tile type
3. Use a seeded random with a finite tile type pool to reduce the probability of perpetual matches

The anti-match check during refill should only prevent 3+ matches at the moment of placement. After gravity settles, new matches from shifting tiles are legitimate cascades and should proceed normally.

### Input Accepted During Cascade Animation
The GameLoopSystem state machine prevents this when implemented correctly, but the View layer must also disable touch/click raycasts during non-input states. A common bug: the View checks its own `m_IsInputEnabled` flag but forgets to set it when the state changes. Subscribe to `GameStateChangedMessage` via MessagePipe and toggle input in the handler — never poll the state in Update.

### Tile Pooling Race Condition on Rapid Swaps
If the player swaps two tiles very quickly in succession, the second swap may reference a tile that is mid-animation from the first swap. Prevent this by:
1. Locking tiles that are currently animating (set a `m_IsLocked` flag on the Tile)
2. Rejecting swaps that involve locked tiles
3. Clearing the lock when the animation completes via a callback

Never return a tile to the pool while its DOTween animation is still running. Kill the tween first with `DOTween.Kill(tileTransform)` before deactivating and returning to pool.

### Board Deadlock (No Valid Moves)
After every cascade settles, scan the board for at least one valid swap that would produce a match. If no valid moves exist, either shuffle the board automatically or spawn a hint booster. The scan must check every possible adjacent swap — a brute force check on a 7x9 board evaluates ~112 swaps, which is cheap enough to run synchronously. Cache the result and only re-scan after board state changes.

### Score Display Desync
Score updates arrive via MessagePipe messages, but the score display View may receive match messages out of order if multiple matches resolve in the same frame. Use a score queue in the View: accumulate incoming score deltas and animate them sequentially with a small delay between each increment. This ensures the displayed score always counts up smoothly even during fast cascades.
