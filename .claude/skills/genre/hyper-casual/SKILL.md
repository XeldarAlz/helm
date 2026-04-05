---
name: hyper-casual
description: "Hyper-casual mobile game architecture — one-tap/swipe controls, instant onboarding, short sessions, ad monetization, minimalist visuals, level progression, score systems."
globs: ["**/HyperCasual*.cs", "**/Level*.cs", "**/GameManager*.cs"]
---

# Hyper-Casual Mobile Patterns

## Core Design Principles

- **One mechanic, perfected** — the entire game is one interaction
- **Instant onboarding** — player understands in 3 seconds, no tutorial screen
- **Short sessions** — 30-60 second rounds
- **Satisfying feedback** — haptics, screen shake, particle bursts, sound on every action
- **Endless or level-based** — simple progression keeps players coming back

## One-Tap Controller

```csharp
public sealed class TapController : MonoBehaviour
{
    [Header("Input")]
    [SerializeField] private float m_TapForce = 10f;
    [SerializeField] private float m_HoldForceMultiplier = 0.5f;

    private Rigidbody m_Rb;
    private bool m_IsTouching;

    private void Awake()
    {
        m_Rb = GetComponent<Rigidbody>();
    }

    private void Update()
    {
        if (UnityEngine.InputSystem.Touchscreen.current == null) return;

        UnityEngine.InputSystem.Controls.TouchControl touch =
            UnityEngine.InputSystem.Touchscreen.current.primaryTouch;

        if (touch.press.wasPressedThisFrame)
        {
            OnTap();
        }

        m_IsTouching = touch.press.isPressed;
    }

    private void FixedUpdate()
    {
        if (m_IsTouching)
        {
            m_Rb.AddForce(Vector3.up * m_HoldForceMultiplier, ForceMode.Acceleration);
        }
    }

    private void OnTap()
    {
        m_Rb.linearVelocity = Vector3.zero;
        m_Rb.AddForce(Vector3.up * m_TapForce, ForceMode.Impulse);
    }
}
```

## Swipe Controller

```csharp
public sealed class SwipeController : MonoBehaviour
{
    [SerializeField] private float m_SwipeThreshold = 50f;
    [SerializeField] private float m_MoveSpeed = 10f;
    [SerializeField] private float m_LaneSwitchDuration = 0.15f;

    private Vector2 m_TouchStartPos;
    private bool m_IsSwiping;

    private void Update()
    {
        if (UnityEngine.InputSystem.Touchscreen.current == null) return;

        UnityEngine.InputSystem.Controls.TouchControl touch =
            UnityEngine.InputSystem.Touchscreen.current.primaryTouch;

        if (touch.press.wasPressedThisFrame)
        {
            m_TouchStartPos = touch.position.ReadValue();
            m_IsSwiping = true;
        }

        if (touch.press.wasReleasedThisFrame && m_IsSwiping)
        {
            Vector2 delta = touch.position.ReadValue() - m_TouchStartPos;
            m_IsSwiping = false;

            if (delta.magnitude > m_SwipeThreshold)
            {
                if (Mathf.Abs(delta.x) > Mathf.Abs(delta.y))
                {
                    OnSwipeHorizontal(delta.x > 0f ? 1 : -1);
                }
                else
                {
                    OnSwipeVertical(delta.y > 0f ? 1 : -1);
                }
            }
            else
            {
                OnTap();
            }
        }
    }

    private void OnSwipeHorizontal(int direction) { /* lane switch or steer */ }
    private void OnSwipeVertical(int direction) { /* jump or duck */ }
    private void OnTap() { /* primary action */ }
}
```

## Level Progression System

```csharp
[CreateAssetMenu(menuName = "HyperCasual/Level Config")]
public sealed class LevelConfig : ScriptableObject
{
    [SerializeField] private int m_LevelNumber;
    [SerializeField] private float m_Speed = 5f;
    [SerializeField] private float m_ObstacleFrequency = 0.5f;
    [SerializeField] private Color m_ThemeColor = Color.white;
    [SerializeField] private int m_TargetScore;

    public int LevelNumber => m_LevelNumber;
    public float Speed => m_Speed;
    public float ObstacleFrequency => m_ObstacleFrequency;
    public Color ThemeColor => m_ThemeColor;
    public int TargetScore => m_TargetScore;
}

public sealed class LevelManager : MonoBehaviour
{
    [SerializeField] private LevelConfig[] m_Levels;

    private int m_CurrentLevelIndex;
    private const string k_LevelKey = "CurrentLevel";

    private void Awake()
    {
        m_CurrentLevelIndex = PlayerPrefs.GetInt(k_LevelKey, 0);
    }

    public LevelConfig GetCurrentLevel()
    {
        int index = m_CurrentLevelIndex % m_Levels.Length;
        return m_Levels[index];
    }

    public void CompleteLevel()
    {
        m_CurrentLevelIndex++;
        PlayerPrefs.SetInt(k_LevelKey, m_CurrentLevelIndex);
        PlayerPrefs.Save();
    }
}
```

## Game Loop (Hyper-Casual)

```
Splash → Menu (one big PLAY button)
    → Playing (auto-forward, player reacts)
        → Game Over (score, best, retry/home)
            → Rewarded Ad (continue?) → resume or Menu
```

- **No loading screens** — instant transitions
- **No settings menus** — sound toggle only
- **Retry = instant** — no delays, no confirmation

## Monetization Hooks

- **Interstitial:** show after every 3rd game over (not on first play)
- **Rewarded:** offer continue/2x score at game over
- **Banner:** bottom of menu screen only (never during gameplay)

```csharp
public interface IAdService
{
    void ShowInterstitial(System.Action onComplete);
    void ShowRewarded(System.Action<bool> onResult);
    void ShowBanner();
    void HideBanner();
}
```

## Visual Style

- Solid colors, no textures (smallest build size)
- Primitives: cubes, spheres, cylinders
- Color palette: 3-4 colors max per level
- Camera: fixed angle or auto-follow, never player-controlled

## Performance Budget

- **< 30 draw calls** — hyper-casual games should be trivially simple to render
- **< 50MB build size** — critical for ad network installs
- **30fps** — saves battery, users don't notice for casual games
- **Zero allocations** in gameplay loop
