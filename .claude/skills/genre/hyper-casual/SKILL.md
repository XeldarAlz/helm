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

## Game State Machine

Hyper-casual games use a minimal state machine with instant transitions. No loading screens, no complex menus.

```csharp
public enum GameState
{
    Splash,
    Menu,
    Playing,
    GameOver,
    Results
}

public sealed class GameStateModel
{
    public ReactiveProperty<GameState> Current { get; } = new(GameState.Splash);
    public bool IsFirstRun { get; set; }
}

public sealed class GameStateSystem : IDisposable
{
    private readonly GameStateModel m_Model;
    private readonly IPublisher<GameStateChangedMessage> m_StatePublisher;
    private const string k_FirstRunKey = "HasPlayedBefore";

    [Inject]
    public GameStateSystem(
        GameStateModel model,
        IPublisher<GameStateChangedMessage> statePublisher)
    {
        m_Model = model;
        m_StatePublisher = statePublisher;
        m_Model.IsFirstRun = PlayerPrefs.GetInt(k_FirstRunKey, 0) == 0;
    }

    public void TransitionTo(GameState next)
    {
        GameState previous = m_Model.Current.Value;
        m_Model.Current.Value = next;
        m_StatePublisher.Publish(new GameStateChangedMessage(previous, next));
    }

    // Single tap from Menu -> Playing (no button required)
    public void OnTapAnywhere()
    {
        GameState current = m_Model.Current.Value;
        if (current == GameState.Menu)
        {
            if (m_Model.IsFirstRun)
            {
                m_Model.IsFirstRun = false;
                PlayerPrefs.SetInt(k_FirstRunKey, 1);
                PlayerPrefs.Save();
            }
            TransitionTo(GameState.Playing);
        }
        else if (current == GameState.Results)
        {
            // Auto-restart: tap after results -> instant retry
            TransitionTo(GameState.Playing);
        }
    }

    public void OnPlayerDied()
    {
        TransitionTo(GameState.GameOver);
    }

    public void Dispose() { }
}

public readonly struct GameStateChangedMessage
{
    public readonly GameState Previous;
    public readonly GameState Next;

    public GameStateChangedMessage(GameState previous, GameState next)
    {
        Previous = previous;
        Next = next;
    }
}
```

Key patterns:
- Single tap to start from Menu — no "Play" button interaction needed
- Tap after Results -> instant retry with zero delay
- First-run detection via PlayerPrefs flag — use this to trigger tutorial overlay on first session
- All transitions publish a message so Views react independently (UI fade, camera reset, score clear)

## Score System

```csharp
public sealed class ScoreModel
{
    public ReactiveProperty<int> Current { get; } = new(0);
    public ReactiveProperty<int> Best { get; } = new(0);
    public ReactiveProperty<int> SessionTotal { get; } = new(0);
    public ReactiveProperty<int> ComboMultiplier { get; } = new(1);
    public float ComboTimeRemaining { get; set; }
}

public sealed class ScoreSystem : ITickable, IDisposable
{
    private readonly ScoreModel m_Model;
    private readonly IPublisher<ScoreChangedMessage> m_ScorePublisher;
    private readonly IPublisher<NewHighScoreMessage> m_HighScorePublisher;

    private const string k_BestScoreKey = "BestScore";
    private const float k_ComboDuration = 2f;
    private const int k_MaxCombo = 8;

    [Inject]
    public ScoreSystem(
        ScoreModel model,
        IPublisher<ScoreChangedMessage> scorePublisher,
        IPublisher<NewHighScoreMessage> highScorePublisher)
    {
        m_Model = model;
        m_ScorePublisher = scorePublisher;
        m_HighScorePublisher = highScorePublisher;
        m_Model.Best.Value = PlayerPrefs.GetInt(k_BestScoreKey, 0);
    }

    public void AddScore(int basePoints)
    {
        int points = basePoints * m_Model.ComboMultiplier.Value;
        m_Model.Current.Value += points;
        m_Model.SessionTotal.Value += points;
        m_Model.ComboTimeRemaining = k_ComboDuration;

        if (m_Model.ComboMultiplier.Value < k_MaxCombo)
        {
            m_Model.ComboMultiplier.Value++;
        }

        m_ScorePublisher.Publish(new ScoreChangedMessage(points, m_Model.Current.Value));

        if (m_Model.Current.Value > m_Model.Best.Value)
        {
            m_Model.Best.Value = m_Model.Current.Value;
            PlayerPrefs.SetInt(k_BestScoreKey, m_Model.Best.Value);
            PlayerPrefs.Save();
            m_HighScorePublisher.Publish(new NewHighScoreMessage());
        }
    }

    public void Tick()
    {
        if (m_Model.ComboTimeRemaining > 0f)
        {
            m_Model.ComboTimeRemaining -= Time.deltaTime;
            if (m_Model.ComboTimeRemaining <= 0f)
            {
                m_Model.ComboMultiplier.Value = 1;
            }
        }
    }

    // Star rating for level completion (1-3 stars)
    public int CalculateStars(int targetScore)
    {
        int score = m_Model.Current.Value;
        if (score >= targetScore * 2) return 3;
        if (score >= targetScore) return 2;
        return 1;
    }

    public void Reset()
    {
        m_Model.Current.Value = 0;
        m_Model.ComboMultiplier.Value = 1;
        m_Model.ComboTimeRemaining = 0f;
    }

    public void Dispose() { }
}
```

Score popup View should use a pooled TextMeshPro object that floats upward and fades. Never instantiate new GameObjects for score popups.

## Visual Juice and Feedback

Hyper-casual games live or die on feedback. Every player action needs an immediate, satisfying response.

### Screen Shake

Use Cinemachine Impulse for camera shake. Keep it subtle on mobile — large shakes cause nausea.

```csharp
public sealed class ScreenShakeView : MonoBehaviour
{
    [SerializeField] private CinemachineImpulseSource m_ImpulseSource;
    [SerializeField] private float m_LightIntensity = 0.2f;
    [SerializeField] private float m_HeavyIntensity = 0.5f;

    public void ShakeLight()
    {
        m_ImpulseSource.GenerateImpulse(m_LightIntensity);
    }

    public void ShakeHeavy()
    {
        m_ImpulseSource.GenerateImpulse(m_HeavyIntensity);
    }
}
```

### Scale Punch on Collect/Hit

Pop an object up then back to original size. Use DOTween Sequence, not Update-based lerp.

```csharp
public sealed class ScalePunchView : MonoBehaviour
{
    [SerializeField] private float m_PunchScale = 1.3f;
    [SerializeField] private float m_Duration = 0.2f;

    private Vector3 m_OriginalScale;

    private void Awake()
    {
        m_OriginalScale = transform.localScale;
    }

    public void Punch()
    {
        transform.DOKill();
        transform.localScale = m_OriginalScale;
        transform.DOPunchScale(m_OriginalScale * (m_PunchScale - 1f), m_Duration, 1, 0.5f);
    }
}
```

### Color Flash on Damage

Use MaterialPropertyBlock to avoid creating new material instances. Shared materials stay shared.

```csharp
public sealed class DamageFlashView : MonoBehaviour
{
    [SerializeField] private Renderer m_Renderer;
    [SerializeField] private Color m_FlashColor = Color.white;
    [SerializeField] private float m_FlashDuration = 0.1f;

    private static readonly int k_ColorProperty = Shader.PropertyToID("_BaseColor");
    private MaterialPropertyBlock m_PropertyBlock;
    private Color m_OriginalColor;

    private void Awake()
    {
        m_PropertyBlock = new MaterialPropertyBlock();
        m_Renderer.GetPropertyBlock(m_PropertyBlock);
        m_OriginalColor = m_Renderer.sharedMaterial.GetColor(k_ColorProperty);
    }

    public async UniTaskVoid FlashAsync(CancellationToken token)
    {
        m_PropertyBlock.SetColor(k_ColorProperty, m_FlashColor);
        m_Renderer.SetPropertyBlock(m_PropertyBlock);

        await UniTask.Delay(
            TimeSpan.FromSeconds(m_FlashDuration),
            cancellationToken: token);

        m_PropertyBlock.SetColor(k_ColorProperty, m_OriginalColor);
        m_Renderer.SetPropertyBlock(m_PropertyBlock);
    }
}
```

### Trail Renderer

Add a TrailRenderer component to the player prefab. Set Time to 0.2-0.4s, Width Curve from 0.3 to 0. Use the level theme color. No code needed — configure on the prefab.

### Confetti Particle Burst

Pre-warm a ParticleSystem on the level-complete prefab. Trigger via `m_Confetti.Play()` from the results View. Set `maxParticles` to 50 or less on mobile.

### Time Slow on Near-Miss

```csharp
public sealed class TimeSlowSystem : IDisposable
{
    private readonly CancellationTokenSource m_Cts = new();

    private const float k_SlowScale = 0.3f;
    private const float k_SlowDuration = 0.15f;
    private const float k_ReturnSpeed = 8f;

    public async UniTaskVoid TriggerSlowMotion()
    {
        Time.timeScale = k_SlowScale;

        await UniTask.Delay(
            TimeSpan.FromSeconds(k_SlowDuration * k_SlowScale),
            ignoreTimeScale: true,
            cancellationToken: m_Cts.Token);

        // Lerp back to normal
        while (Time.timeScale < 0.99f)
        {
            Time.timeScale = Mathf.MoveTowards(
                Time.timeScale, 1f, k_ReturnSpeed * Time.unscaledDeltaTime);
            await UniTask.Yield(PlayerLoopTiming.Update, m_Cts.Token);
        }

        Time.timeScale = 1f;
    }

    public void Dispose()
    {
        m_Cts.Cancel();
        Time.timeScale = 1f;
    }
}
```

## Audio Integration

Keep audio simple. One music track, pooled one-shot SFX, and a single mute toggle.

```csharp
public sealed class AudioSystem : IDisposable
{
    private readonly AudioSource m_MusicSource;
    private readonly AudioSource[] m_SfxPool;
    private int m_NextSfxIndex;
    private bool m_IsMuted;

    private const string k_MuteKey = "AudioMuted";
    private const float k_PitchVariation = 0.1f;

    [Inject]
    public AudioSystem(AudioSource musicSource, AudioSource[] sfxPool)
    {
        m_MusicSource = musicSource;
        m_SfxPool = sfxPool;
        m_IsMuted = PlayerPrefs.GetInt(k_MuteKey, 0) == 1;
        ApplyMuteState();
    }

    public void PlayMusic(AudioClip clip)
    {
        m_MusicSource.clip = clip;
        m_MusicSource.loop = true;
        m_MusicSource.Play();
    }

    public void PlaySfx(AudioClip clip)
    {
        if (m_IsMuted) return;

        AudioSource source = m_SfxPool[m_NextSfxIndex];
        m_NextSfxIndex = (m_NextSfxIndex + 1) % m_SfxPool.Length;

        // Pitch variation prevents repetitive feel on rapid triggers
        source.pitch = 1f + UnityEngine.Random.Range(-k_PitchVariation, k_PitchVariation);
        source.PlayOneShot(clip);
    }

    public void ToggleMute()
    {
        m_IsMuted = !m_IsMuted;
        PlayerPrefs.SetInt(k_MuteKey, m_IsMuted ? 1 : 0);
        PlayerPrefs.Save();
        ApplyMuteState();
    }

    private void ApplyMuteState()
    {
        m_MusicSource.mute = m_IsMuted;
        for (int sourceIndex = 0; sourceIndex < m_SfxPool.Length; sourceIndex++)
        {
            m_SfxPool[sourceIndex].mute = m_IsMuted;
        }
    }

    public void Dispose() { }
}
```

Audio pool should have 4-6 AudioSource components on a single persistent GameObject. Register via VContainer in the RootLifetimeScope. No crossfade — music is a single looping track that plays instantly.

## Difficulty Progression

Difficulty should ramp within a session and across levels. Use AnimationCurve for smooth ramping and ScriptableObject for per-level configuration.

```csharp
[CreateAssetMenu(menuName = "HyperCasual/Difficulty Config")]
public sealed class DifficultyConfig : ScriptableObject
{
    [SerializeField] private AnimationCurve m_SpeedCurve;
    [SerializeField] private AnimationCurve m_SpawnFrequencyCurve;
    [SerializeField] private int m_ObstacleUnlockInterval = 5;
    [SerializeField] private int m_ConsecutiveDeathsToReduce = 3;
    [SerializeField] private float m_DifficultyReductionFactor = 0.8f;

    // Evaluate speed at a normalized progress (0 = level start, 1 = level end)
    public float GetSpeed(float progress) => m_SpeedCurve.Evaluate(progress);
    public float GetSpawnFrequency(float progress) => m_SpawnFrequencyCurve.Evaluate(progress);
    public int ObstacleUnlockInterval => m_ObstacleUnlockInterval;
    public int ConsecutiveDeathsToReduce => m_ConsecutiveDeathsToReduce;
    public float DifficultyReductionFactor => m_DifficultyReductionFactor;
}

public sealed class DifficultySystem : IDisposable
{
    private readonly DifficultyConfig m_Config;
    private readonly DifficultyModel m_Model;

    [Inject]
    public DifficultySystem(DifficultyConfig config, DifficultyModel model)
    {
        m_Config = config;
        m_Model = model;
    }

    // Number of distinct obstacle types available at this level
    public int GetUnlockedObstacleCount(int levelNumber)
    {
        return 1 + (levelNumber / m_Config.ObstacleUnlockInterval);
    }

    public float GetCurrentSpeed(float levelProgress)
    {
        float baseSpeed = m_Config.GetSpeed(levelProgress);
        return baseSpeed * m_Model.DifficultyMultiplier;
    }

    // Call on death — reduces difficulty after consecutive failures
    public void RecordDeath()
    {
        m_Model.ConsecutiveDeaths++;
        if (m_Model.ConsecutiveDeaths >= m_Config.ConsecutiveDeathsToReduce)
        {
            m_Model.DifficultyMultiplier *= m_Config.DifficultyReductionFactor;
            m_Model.ConsecutiveDeaths = 0;
        }
    }

    // Call on level complete — resets death counter and restores difficulty
    public void RecordSuccess()
    {
        m_Model.ConsecutiveDeaths = 0;
        m_Model.DifficultyMultiplier = Mathf.MoveTowards(
            m_Model.DifficultyMultiplier, 1f, 0.1f);
    }

    public void Dispose() { }
}

public sealed class DifficultyModel
{
    public int ConsecutiveDeaths { get; set; }
    public float DifficultyMultiplier { get; set; } = 1f;
}
```

Key points:
- AnimationCurve for speed/frequency ramp — designers tweak curves in the Inspector, not code
- Level-based difficulty via LevelConfig (already defined above), not continuous
- New obstacle types unlock every N levels so early levels stay simple
- Fail-safe: after N consecutive deaths, reduce the difficulty multiplier so players are not stuck

## Tutorial System

No text. Visual-only. Gesture hints overlay the game during the first 3 levels.

```csharp
public sealed class TutorialModel
{
    public ReactiveProperty<bool> IsShowingHint { get; } = new(false);
    public int CompletedSteps { get; set; }
}

public sealed class TutorialSystem : IDisposable
{
    private readonly TutorialModel m_Model;
    private const string k_TutorialCompleteKey = "TutorialDone";
    private const int k_TutorialLevelCount = 3;

    [Inject]
    public TutorialSystem(TutorialModel model)
    {
        m_Model = model;
        m_Model.CompletedSteps = PlayerPrefs.GetInt(k_TutorialCompleteKey, 0);
    }

    public bool ShouldShowTutorial(int levelNumber)
    {
        return levelNumber < k_TutorialLevelCount
            && m_Model.CompletedSteps <= levelNumber;
    }

    public void ShowGestureHint()
    {
        m_Model.IsShowingHint.Value = true;
    }

    public void OnPlayerFirstInput()
    {
        // Player understood — dismiss the hint
        m_Model.IsShowingHint.Value = false;
    }

    public void CompleteStep(int step)
    {
        m_Model.CompletedSteps = step + 1;
        PlayerPrefs.SetInt(k_TutorialCompleteKey, m_Model.CompletedSteps);
        PlayerPrefs.Save();
    }

    public bool IsTutorialComplete => m_Model.CompletedSteps >= k_TutorialLevelCount;

    public void Dispose() { }
}
```

Tutorial View pattern:
- Animated hand sprite (looping position tween from DOTween) showing tap or swipe gesture
- Arrow pointing at the interactive element
- Semi-transparent overlay dims everything except the target area
- Hint auto-dismisses on first correct player input
- Levels 0-2 each teach one mechanic, then the flag is set and tutorials stop

## Common Pitfalls

**Particle overdraw on low-end mobile:**
Keep particle `maxParticles` under 50 per emitter. Use additive blending sparingly — it doubles overdraw. Test on a low-end Android device early, not just in the Editor.

**Input lag from Update-based polling:**
Use Input System action callbacks (`performed`, `canceled`) instead of polling `Touchscreen.current` every frame. Polling misses fast taps that start and end within the same frame.

**Object pooling not aggressive enough:**
Everything that appears and disappears must be pooled: score popups, particle effects, obstacles, collectibles. A single `Instantiate` call during gameplay causes a GC spike visible to players as a frame hitch.

**Time.timeScale affecting UI animations:**
When using time slow for near-miss effects, UI animations that use scaled time will also slow down. Use `Time.unscaledDeltaTime` for UI tweens, or set DOTween sequences to `SetUpdate(true)` (unscaled).

**PlayerPrefs not saved on mobile kill:**
Call `PlayerPrefs.Save()` immediately after every write. Mobile OS can kill the app without calling `OnApplicationQuit`. Never batch PlayerPrefs writes.

**Physics jitter at low frame rates:**
Hyper-casual games often target 30fps. Rigidbody interpolation must be set to `Interpolate` to avoid visible stutter. Set this on the prefab, not in code.

**Trail renderers on pooled objects:**
When returning a pooled object with a TrailRenderer, call `trail.Clear()` before reactivating. Otherwise the trail draws a line from the last deactivation position to the new spawn position.
