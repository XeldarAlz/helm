---
name: puzzle
description: "Mobile puzzle game architecture — grid/board logic, undo system, hint system, level packs, star ratings, touch drag-and-drop, tutorial overlays."
globs: ["**/Puzzle*.cs", "**/Board*.cs", "**/Grid*.cs", "**/Hint*.cs", "**/Undo*.cs"]
---

# Mobile Puzzle Patterns

## Undo System (Command Pattern)

```csharp
public interface IGameCommand
{
    void Execute();
    void Undo();
}

public sealed class UndoManager
{
    private readonly Stack<IGameCommand> m_UndoStack = new();
    private readonly int m_MaxUndoSteps;

    public UndoManager(int maxSteps = 50)
    {
        m_MaxUndoSteps = maxSteps;
    }

    public int UndoCount => m_UndoStack.Count;

    public void Execute(IGameCommand command)
    {
        command.Execute();
        m_UndoStack.Push(command);
        if (m_UndoStack.Count > m_MaxUndoSteps)
        {
            // Trim oldest — would need a different data structure for efficiency
        }
    }

    public bool Undo()
    {
        if (m_UndoStack.Count == 0) return false;
        IGameCommand command = m_UndoStack.Pop();
        command.Undo();
        return true;
    }

    public void Clear()
    {
        m_UndoStack.Clear();
    }
}

// Example: move a piece
public sealed class MovePieceCommand : IGameCommand
{
    private readonly Piece m_Piece;
    private readonly Vector2Int m_FromPos;
    private readonly Vector2Int m_ToPos;

    public MovePieceCommand(Piece piece, Vector2Int from, Vector2Int to)
    {
        m_Piece = piece;
        m_FromPos = from;
        m_ToPos = to;
    }

    public void Execute() { m_Piece.MoveTo(m_ToPos); }
    public void Undo() { m_Piece.MoveTo(m_FromPos); }
}
```

## Level Pack System

```csharp
[CreateAssetMenu(menuName = "Puzzle/Level Pack")]
public sealed class LevelPack : ScriptableObject
{
    [SerializeField] private string m_PackId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private Sprite m_Icon;
    [SerializeField] private PuzzleLevel[] m_Levels;
    [SerializeField] private bool m_IsLocked;
    [SerializeField] private int m_StarsToUnlock;

    public string PackId => m_PackId;
    public string DisplayName => m_DisplayName;
    public IReadOnlyList<PuzzleLevel> Levels => m_Levels;
    public bool IsLocked => m_IsLocked;
    public int StarsToUnlock => m_StarsToUnlock;
}

[CreateAssetMenu(menuName = "Puzzle/Level")]
public sealed class PuzzleLevel : ScriptableObject
{
    [SerializeField] private string m_LevelId;
    [SerializeField] private int m_ParMoves; // 3 stars if completed in this many moves
    [SerializeField] private int m_MaxMoves; // fail if exceeded (0 = unlimited)
    [SerializeField] private float m_ParTime; // 3 stars if completed in this time
    [SerializeField] private TextAsset m_LevelData; // JSON or custom format

    public string LevelId => m_LevelId;
    public int ParMoves => m_ParMoves;
    public int MaxMoves => m_MaxMoves;
}
```

## Star Rating

```csharp
public sealed class StarCalculator
{
    public static int Calculate(PuzzleLevel level, int movesTaken, float timeTaken)
    {
        int stars = 1; // completing = 1 star minimum

        if (level.ParMoves > 0 && movesTaken <= level.ParMoves)
        {
            stars = 3;
        }
        else if (level.ParMoves > 0 && movesTaken <= level.ParMoves * 1.5f)
        {
            stars = 2;
        }

        return stars;
    }

    public static int GetTotalStars(string packId)
    {
        // Sum all stars earned across levels in pack
        int total = 0;
        // Read from save data...
        return total;
    }
}
```

## Hint System

```csharp
public sealed class HintSystem : MonoBehaviour
{
    [SerializeField] private float m_AutoHintDelay = 15f; // show hint after N seconds idle
    [SerializeField] private int m_FreeHints = 3;

    private int m_HintsRemaining;
    private float m_IdleTimer;
    private bool m_HintShowing;

    public event System.Action<HintData> OnShowHint;
    public event System.Action OnHideHint;

    private void Update()
    {
        if (m_HintShowing) return;

        m_IdleTimer += Time.deltaTime;
        if (m_IdleTimer >= m_AutoHintDelay)
        {
            ShowAutoHint();
        }
    }

    public void OnPlayerAction()
    {
        m_IdleTimer = 0f;
        if (m_HintShowing)
        {
            m_HintShowing = false;
            OnHideHint?.Invoke();
        }
    }

    public bool UseHint()
    {
        if (m_HintsRemaining <= 0) return false;
        m_HintsRemaining--;
        ShowExplicitHint();
        return true;
    }

    private void ShowAutoHint()
    {
        // Subtle hint — highlight possible move
        m_HintShowing = true;
    }

    private void ShowExplicitHint()
    {
        // Obvious hint — animate the solution move
        m_HintShowing = true;
    }
}
```

## Touch Drag-and-Drop

```csharp
public sealed class DragHandler : MonoBehaviour
{
    [SerializeField] private Camera m_Camera;
    [SerializeField] private LayerMask m_DraggableLayer;
    [SerializeField] private float m_DragOffset = 0.5f; // lift piece while dragging

    private Piece m_DraggedPiece;
    private Vector3 m_DragStartWorldPos;
    private Vector2Int m_DragStartGridPos;

    private void Update()
    {
        if (UnityEngine.InputSystem.Touchscreen.current == null) return;

        UnityEngine.InputSystem.Controls.TouchControl touch =
            UnityEngine.InputSystem.Touchscreen.current.primaryTouch;

        if (touch.press.wasPressedThisFrame)
        {
            TryStartDrag(touch.position.ReadValue());
        }
        else if (touch.press.isPressed && m_DraggedPiece != null)
        {
            UpdateDrag(touch.position.ReadValue());
        }
        else if (touch.press.wasReleasedThisFrame && m_DraggedPiece != null)
        {
            EndDrag(touch.position.ReadValue());
        }
    }

    private void TryStartDrag(Vector2 screenPos)
    {
        Ray ray = m_Camera.ScreenPointToRay(screenPos);
        if (Physics2D.Raycast(ray.origin, ray.direction, 100f, m_DraggableLayer))
        {
            // Start dragging the hit piece
        }
    }

    private void UpdateDrag(Vector2 screenPos)
    {
        Vector3 worldPos = m_Camera.ScreenToWorldPoint(new Vector3(screenPos.x, screenPos.y, 10f));
        m_DraggedPiece.transform.position = new Vector3(worldPos.x, worldPos.y + m_DragOffset, 0f);
    }

    private void EndDrag(Vector2 screenPos)
    {
        // Snap to nearest valid grid position or return to start
        m_DraggedPiece = null;
    }
}
```

## Tutorial Overlay

- **First-time only** — check PlayerPrefs flag per tutorial step
- **Dim background** — semi-transparent overlay, spotlight on target element
- **Animated hand** — show tap/drag gesture on the target
- **Progressive** — teach one mechanic per level, not all at once
- **Skippable** — always allow dismissing

## Puzzle Game Types

| Type | Core Mechanic | Examples |
|------|--------------|----------|
| Slide puzzle | Move tiles to solve | 15-puzzle, Unblock Me |
| Match puzzle | Match/connect similar | Match-3, Dots |
| Physics puzzle | Aim/launch objects | Angry Birds, Cut the Rope |
| Word puzzle | Form words from letters | Wordle, Word Cookies |
| Logic puzzle | Deduce solution | Sudoku, Nonograms |
| Spatial puzzle | Fit shapes | Tetris, Block Puzzle |

## Performance

- Puzzle games are rarely GPU-bound — focus on clean input and smooth animations
- Pool popup effects and particles
- Pre-calculate valid moves for hint system at level start
- Save progress per-level to avoid data loss on app kill

## Board and Grid System

A generic grid is the backbone of most puzzle games. It handles coordinate mapping, neighbor queries, and state snapshots for undo.

```csharp
public sealed class Grid<T>
{
    private readonly T[] m_Cells;
    private readonly int m_Width;
    private readonly int m_Height;

    public int Width => m_Width;
    public int Height => m_Height;

    public Grid(int width, int height)
    {
        m_Width = width;
        m_Height = height;
        m_Cells = new T[width * height];
    }

    public T Get(int column, int row)
    {
        return m_Cells[row * m_Width + column];
    }

    public void Set(int column, int row, T value)
    {
        m_Cells[row * m_Width + column] = value;
    }

    public T Get(Vector2Int pos) => Get(pos.x, pos.y);
    public void Set(Vector2Int pos, T value) => Set(pos.x, pos.y, value);

    public bool IsInBounds(int column, int row)
    {
        return column >= 0 && column < m_Width && row >= 0 && row < m_Height;
    }

    public bool IsInBounds(Vector2Int pos) => IsInBounds(pos.x, pos.y);

    // World position to grid coordinate
    public Vector2Int WorldToGrid(Vector3 worldPos, Vector3 gridOrigin, float cellSize)
    {
        int column = Mathf.FloorToInt((worldPos.x - gridOrigin.x) / cellSize);
        int row = Mathf.FloorToInt((worldPos.y - gridOrigin.y) / cellSize);
        return new Vector2Int(column, row);
    }

    // Grid coordinate to world center position
    public Vector3 GridToWorld(Vector2Int pos, Vector3 gridOrigin, float cellSize)
    {
        float worldX = gridOrigin.x + pos.x * cellSize + cellSize * 0.5f;
        float worldY = gridOrigin.y + pos.y * cellSize + cellSize * 0.5f;
        return new Vector3(worldX, worldY, 0f);
    }

    // 4-directional neighbors (up, down, left, right)
    private static readonly Vector2Int[] k_Neighbors4 =
    {
        new(0, 1), new(0, -1), new(-1, 0), new(1, 0)
    };

    // 8-directional neighbors (includes diagonals)
    private static readonly Vector2Int[] k_Neighbors8 =
    {
        new(0, 1), new(0, -1), new(-1, 0), new(1, 0),
        new(-1, 1), new(1, 1), new(-1, -1), new(1, -1)
    };

    // Get valid neighbors — writes to a pre-allocated buffer, returns count
    public int GetNeighbors4(Vector2Int pos, Vector2Int[] buffer)
    {
        int count = 0;
        for (int neighborIndex = 0; neighborIndex < k_Neighbors4.Length; neighborIndex++)
        {
            Vector2Int neighbor = pos + k_Neighbors4[neighborIndex];
            if (IsInBounds(neighbor))
            {
                buffer[count++] = neighbor;
            }
        }
        return count;
    }

    public int GetNeighbors8(Vector2Int pos, Vector2Int[] buffer)
    {
        int count = 0;
        for (int neighborIndex = 0; neighborIndex < k_Neighbors8.Length; neighborIndex++)
        {
            Vector2Int neighbor = pos + k_Neighbors8[neighborIndex];
            if (IsInBounds(neighbor))
            {
                buffer[count++] = neighbor;
            }
        }
        return count;
    }

    // Snapshot for undo — copies all cell data
    public T[] CreateSnapshot()
    {
        T[] snapshot = new T[m_Cells.Length];
        System.Array.Copy(m_Cells, snapshot, m_Cells.Length);
        return snapshot;
    }

    public void RestoreSnapshot(T[] snapshot)
    {
        System.Array.Copy(snapshot, m_Cells, m_Cells.Length);
    }
}
```

Key points:
- Use `Vector2Int` for grid coordinates everywhere — never raw `int x, int y` pairs
- Neighbor buffers are pre-allocated (max size 8) and reused to avoid allocations
- Snapshot/restore for undo uses `System.Array.Copy` for speed
- World-to-grid conversion centralizes the math so Views never calculate positions themselves

## Move Validation and Execution

Moves go through a validation-then-execute pipeline. Always snapshot state before execution for undo support.

```csharp
public enum MoveType
{
    Swap,
    Place,
    Slide,
    Rotate
}

public readonly struct MoveRequest
{
    public readonly Vector2Int From;
    public readonly Vector2Int To;
    public readonly MoveType Type;

    public MoveRequest(Vector2Int from, Vector2Int to, MoveType type)
    {
        From = from;
        To = to;
        Type = type;
    }
}

public sealed class MoveModel
{
    public ReactiveProperty<int> MovesUsed { get; } = new(0);
    public int MoveLimit { get; set; }
    public bool HasMovesRemaining => MoveLimit <= 0 || MovesUsed.Value < MoveLimit;
}

public sealed class MoveSystem : IDisposable
{
    private readonly Grid<int> m_Grid;
    private readonly MoveModel m_Model;
    private readonly UndoManager m_UndoManager;
    private readonly IPublisher<MoveExecutedMessage> m_MovePublisher;
    private readonly IPublisher<InvalidMoveMessage> m_InvalidMovePublisher;

    [Inject]
    public MoveSystem(
        Grid<int> grid,
        MoveModel model,
        UndoManager undoManager,
        IPublisher<MoveExecutedMessage> movePublisher,
        IPublisher<InvalidMoveMessage> invalidMovePublisher)
    {
        m_Grid = grid;
        m_Model = model;
        m_UndoManager = undoManager;
        m_MovePublisher = movePublisher;
        m_InvalidMovePublisher = invalidMovePublisher;
    }

    public bool IsLegalMove(MoveRequest request)
    {
        if (!m_Model.HasMovesRemaining) return false;
        if (!m_Grid.IsInBounds(request.From)) return false;
        if (!m_Grid.IsInBounds(request.To)) return false;

        return request.Type switch
        {
            MoveType.Swap => ValidateSwap(request),
            MoveType.Place => ValidatePlace(request),
            MoveType.Slide => ValidateSlide(request),
            MoveType.Rotate => ValidateRotate(request),
            _ => false
        };
    }

    public bool TryExecute(MoveRequest request)
    {
        if (!IsLegalMove(request))
        {
            m_InvalidMovePublisher.Publish(new InvalidMoveMessage(request));
            return false;
        }

        // Snapshot state before execution for undo
        int[] snapshot = m_Grid.CreateSnapshot();
        int movesBefore = m_Model.MovesUsed.Value;

        ExecuteMove(request);
        m_Model.MovesUsed.Value++;

        // Push undo command with captured state
        m_UndoManager.Execute(new GridRestoreCommand(m_Grid, snapshot, m_Model, movesBefore));
        m_MovePublisher.Publish(new MoveExecutedMessage(request));
        return true;
    }

    private void ExecuteMove(MoveRequest request)
    {
        // Swap two cells as the default implementation
        int temp = m_Grid.Get(request.From);
        m_Grid.Set(request.From, m_Grid.Get(request.To));
        m_Grid.Set(request.To, temp);
    }

    private bool ValidateSwap(MoveRequest request) => m_Grid.Get(request.From) != 0;
    private bool ValidatePlace(MoveRequest request) => m_Grid.Get(request.To) == 0;
    private bool ValidateSlide(MoveRequest request) => m_Grid.Get(request.To) == 0;
    private bool ValidateRotate(MoveRequest request) => true;

    public void Dispose() { }
}

public readonly struct MoveExecutedMessage
{
    public readonly MoveRequest Request;
    public MoveExecutedMessage(MoveRequest request) { Request = request; }
}

public readonly struct InvalidMoveMessage
{
    public readonly MoveRequest Request;
    public InvalidMoveMessage(MoveRequest request) { Request = request; }
}
```

On invalid move, the View should play a short shake animation on the selected piece and an error sound. Never block input — let the player try again immediately.

## Win Condition Detection

```csharp
public sealed class WinConditionModel
{
    public ReactiveProperty<bool> IsComplete { get; } = new(false);
    public ReactiveProperty<float> Progress { get; } = new(0f);
    public ReactiveProperty<int> StarsEarned { get; } = new(0);
}

public sealed class WinConditionSystem : IDisposable
{
    private readonly Grid<int> m_Grid;
    private readonly WinConditionModel m_Model;
    private readonly MoveModel m_MoveModel;
    private readonly IPublisher<LevelCompleteMessage> m_CompletePublisher;

    [Inject]
    public WinConditionSystem(
        Grid<int> grid,
        WinConditionModel model,
        MoveModel moveModel,
        IPublisher<LevelCompleteMessage> completePublisher)
    {
        m_Grid = grid;
        m_Model = model;
        m_MoveModel = moveModel;
        m_CompletePublisher = completePublisher;
    }

    // Call after every move execution
    public void CheckWinCondition(PuzzleLevel level)
    {
        float progress = CalculateProgress();
        m_Model.Progress.Value = progress;

        if (progress >= 1f)
        {
            int stars = CalculateStars(level);
            m_Model.StarsEarned.Value = stars;
            m_Model.IsComplete.Value = true;
            m_CompletePublisher.Publish(new LevelCompleteMessage(stars));
        }
    }

    private float CalculateProgress()
    {
        // Count cleared/matched cells vs total target
        int cleared = 0;
        int total = m_Grid.Width * m_Grid.Height;
        for (int row = 0; row < m_Grid.Height; row++)
        {
            for (int column = 0; column < m_Grid.Width; column++)
            {
                if (m_Grid.Get(column, row) == 0)
                {
                    cleared++;
                }
            }
        }
        return (float)cleared / total;
    }

    private int CalculateStars(PuzzleLevel level)
    {
        int movesUsed = m_MoveModel.MovesUsed.Value;
        if (level.ParMoves > 0 && movesUsed <= level.ParMoves) return 3;
        if (level.ParMoves > 0 && movesUsed <= Mathf.CeilToInt(level.ParMoves * 1.5f)) return 2;
        return 1;
    }

    public void Dispose() { }
}

public readonly struct LevelCompleteMessage
{
    public readonly int Stars;
    public LevelCompleteMessage(int stars) { Stars = stars; }
}
```

Victory sequence in the View:
1. Delay 0.3s after the final move lands (let the player see the completed board)
2. Play a celebration particle burst
3. Animate all pieces outward with staggered scale-to-zero
4. Show results panel with star animation

## Animation and Feedback

### Piece Movement Tweening

Use DOTween Sequences for multi-step animations. Chain them so matching, clearing, and refilling play in order.

```csharp
public sealed class PieceAnimationView : MonoBehaviour
{
    [SerializeField] private float m_MoveDuration = 0.2f;
    [SerializeField] private float m_PopDuration = 0.15f;
    [SerializeField] private float m_LandSquashAmount = 0.2f;
    [SerializeField] private Ease m_MoveEase = Ease.OutQuad;

    private Vector3 m_OriginalScale;

    private void Awake()
    {
        m_OriginalScale = transform.localScale;
    }

    // Move piece to target world position
    public Tween MoveTo(Vector3 target)
    {
        return transform.DOMove(target, m_MoveDuration).SetEase(m_MoveEase);
    }

    // Pop animation for match/clear
    public Sequence PopAndClear()
    {
        Sequence sequence = DOTween.Sequence();
        sequence.Append(transform.DOScale(m_OriginalScale * 1.2f, m_PopDuration * 0.5f));
        sequence.Append(transform.DOScale(Vector3.zero, m_PopDuration * 0.5f));
        return sequence;
    }

    // Squash on landing after a drop
    public Sequence LandSquash()
    {
        Sequence sequence = DOTween.Sequence();
        Vector3 squashed = new Vector3(
            m_OriginalScale.x * (1f + m_LandSquashAmount),
            m_OriginalScale.y * (1f - m_LandSquashAmount),
            m_OriginalScale.z);
        sequence.Append(transform.DOScale(squashed, 0.08f).SetEase(Ease.OutQuad));
        sequence.Append(transform.DOScale(m_OriginalScale, 0.12f).SetEase(Ease.OutBounce));
        return sequence;
    }

    // Stagger delay for chain reactions — each step waits for the previous
    public static async UniTask PlayChainAsync(
        Sequence[] steps,
        float delayBetween,
        CancellationToken token)
    {
        for (int stepIndex = 0; stepIndex < steps.Length; stepIndex++)
        {
            steps[stepIndex].Play();
            await steps[stepIndex].AsyncWaitForCompletion().AsUniTask()
                .AttachExternalCancellation(token);
            if (delayBetween > 0f)
            {
                await UniTask.Delay(
                    TimeSpan.FromSeconds(delayBetween),
                    cancellationToken: token);
            }
        }
    }
}
```

### Completion Celebration

```csharp
public sealed class CelebrationView : MonoBehaviour
{
    [SerializeField] private ParticleSystem m_ConfettiParticles;
    [SerializeField] private float m_PieceExitStagger = 0.03f;

    public async UniTask PlayCelebrationAsync(
        PieceAnimationView[] pieces,
        CancellationToken token)
    {
        m_ConfettiParticles.Play();

        // Staggered scale-out for all pieces
        for (int pieceIndex = 0; pieceIndex < pieces.Length; pieceIndex++)
        {
            if (pieces[pieceIndex] == null) continue;

            pieces[pieceIndex].transform.DOScale(Vector3.zero, 0.2f)
                .SetDelay(pieceIndex * m_PieceExitStagger)
                .SetEase(Ease.InBack);
        }

        await UniTask.Delay(
            TimeSpan.FromSeconds(pieces.Length * m_PieceExitStagger + 0.5f),
            cancellationToken: token);
    }
}
```

Key animation rules:
- Chain reaction delays (0.1-0.15s between steps) let the player see each cascade step
- Squash on landing sells the physicality of falling pieces
- Always kill running tweens before starting new ones on the same transform (`transform.DOKill()`)
- Use `SetEase(Ease.OutBounce)` sparingly — it works for landing but looks wrong on movement

## Timer and Scoring

```csharp
public sealed class PuzzleTimerModel
{
    public ReactiveProperty<float> TimeRemaining { get; } = new(0f);
    public ReactiveProperty<bool> IsRunning { get; } = new(false);
    public float InitialTime { get; set; }
}

public sealed class PuzzleTimerSystem : ITickable, IDisposable
{
    private readonly PuzzleTimerModel m_Model;
    private readonly IPublisher<TimerExpiredMessage> m_TimerPublisher;

    [Inject]
    public PuzzleTimerSystem(
        PuzzleTimerModel model,
        IPublisher<TimerExpiredMessage> timerPublisher)
    {
        m_Model = model;
        m_TimerPublisher = timerPublisher;
    }

    public void StartTimer(float seconds)
    {
        m_Model.InitialTime = seconds;
        m_Model.TimeRemaining.Value = seconds;
        m_Model.IsRunning.Value = true;
    }

    public void Tick()
    {
        if (!m_Model.IsRunning.Value) return;

        m_Model.TimeRemaining.Value -= Time.deltaTime;
        if (m_Model.TimeRemaining.Value <= 0f)
        {
            m_Model.TimeRemaining.Value = 0f;
            m_Model.IsRunning.Value = false;
            m_TimerPublisher.Publish(new TimerExpiredMessage());
        }
    }

    // Pause when app goes to background
    public void OnApplicationPause(bool paused)
    {
        if (m_Model.IsRunning.Value)
        {
            m_Model.IsRunning.Value = !paused;
        }
    }

    // Time bonus: percentage of time remaining maps to bonus points
    public int CalculateTimeBonus(int basePoints)
    {
        if (m_Model.InitialTime <= 0f) return 0;
        float ratio = m_Model.TimeRemaining.Value / m_Model.InitialTime;
        return Mathf.RoundToInt(basePoints * ratio);
    }

    public void Dispose() { }
}

public readonly struct TimerExpiredMessage { }

// Score breakdown for results screen
public sealed class PuzzleScoreBreakdown
{
    public int BaseScore { get; set; }
    public int TimeBonus { get; set; }
    public int MoveBonus { get; set; }
    public int TotalScore => BaseScore + TimeBonus + MoveBonus;

    public static PuzzleScoreBreakdown Calculate(
        PuzzleLevel level,
        int movesUsed,
        float timeRemaining,
        float initialTime)
    {
        int baseScore = 1000;
        int moveBonus = level.ParMoves > 0
            ? Mathf.Max(0, (level.ParMoves - movesUsed) * 50)
            : 0;
        float timeRatio = initialTime > 0f ? timeRemaining / initialTime : 0f;
        int timeBonus = Mathf.RoundToInt(500 * timeRatio);

        return new PuzzleScoreBreakdown
        {
            BaseScore = baseScore,
            TimeBonus = timeBonus,
            MoveBonus = moveBonus
        };
    }
}
```

## Progression and Level Unlock

```csharp
public sealed class ProgressionModel
{
    // Stars earned per level, keyed by level ID
    public Dictionary<string, int> LevelStars { get; } = new();
    public int TotalStars { get; set; }
}

public sealed class ProgressionSystem : IDisposable
{
    private readonly ProgressionModel m_Model;
    private readonly LevelPack[] m_Packs;

    private const string k_StarsPrefix = "Stars_";
    private const string k_TotalStarsKey = "TotalStars";

    [Inject]
    public ProgressionSystem(ProgressionModel model, LevelPack[] packs)
    {
        m_Model = model;
        m_Packs = packs;
        LoadProgress();
    }

    private void LoadProgress()
    {
        m_Model.TotalStars = PlayerPrefs.GetInt(k_TotalStarsKey, 0);
    }

    public void RecordLevelComplete(string levelId, int stars)
    {
        int previous = 0;
        if (m_Model.LevelStars.TryGetValue(levelId, out int existing))
        {
            previous = existing;
        }

        // Only update if new star count is higher
        if (stars > previous)
        {
            m_Model.LevelStars[levelId] = stars;
            m_Model.TotalStars += stars - previous;
            PlayerPrefs.SetInt(k_StarsPrefix + levelId, stars);
            PlayerPrefs.SetInt(k_TotalStarsKey, m_Model.TotalStars);
            PlayerPrefs.Save();
        }
    }

    public bool IsPackUnlocked(LevelPack pack)
    {
        if (!pack.IsLocked) return true;
        return m_Model.TotalStars >= pack.StarsToUnlock;
    }

    // Daily challenge uses date as seed for deterministic generation
    public int GetDailySeed()
    {
        System.DateTime today = System.DateTime.UtcNow.Date;
        return today.Year * 10000 + today.Month * 100 + today.Day;
    }

    public void Dispose() { }
}
```

Progression View patterns:
- World map shows level packs as nodes — locked packs display a lock icon and required star count
- Unlocked packs animate (scale pulse, glow) when the player first has enough stars
- Daily challenge appears as a special node that refreshes each day
- Difficulty curve: each pack introduces exactly one new mechanic (e.g., pack 2 adds rotatable pieces, pack 3 adds bombs)

## Common Pitfalls

**Undo stack memory:**
Cap the undo stack at a fixed size (50 is generous). Each snapshot stores the full grid state. For large boards (10x10+), this adds up fast. Consider delta-based undo (store only changed cells) for boards larger than 8x8.

**Grid coordinate vs world position confusion:**
Always convert through the Grid class methods (`WorldToGrid`, `GridToWorld`). Never manually calculate `position / cellSize` in View code. A single off-by-one or rounding difference between two Views causes pieces to land in wrong cells.

**Animation timing blocking input:**
Use a state machine for the board: `Idle`, `Animating`, `Checking`. Block input during `Animating` state. If you allow input during animations, players can break the board state by moving pieces that are mid-tween.

**Level data not serializable:**
Store level layouts in ScriptableObjects or TextAsset (JSON/CSV), never as hardcoded arrays in C#. Designers need to edit levels without touching code. Use a custom editor tool or external level editor that exports to the TextAsset format.

**Forgetting to check for cascades:**
After clearing matched pieces and dropping remaining pieces, scan the board again. Matches can chain. Use a loop: clear -> drop -> scan -> repeat until no matches found.

**Pooling piece GameObjects:**
Puzzle boards create and destroy many piece objects. Pool them. When a piece is "cleared," deactivate and return to pool. When new pieces spawn (gravity fill from top), pull from pool. Set the pool initial size to `boardWidth * boardHeight`.
