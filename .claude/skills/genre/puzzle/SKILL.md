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
