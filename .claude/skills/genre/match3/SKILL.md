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
