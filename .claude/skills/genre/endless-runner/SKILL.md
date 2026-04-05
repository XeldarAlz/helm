---
name: endless-runner
description: "Endless runner architecture — procedural chunk spawning, lane-based or free movement, obstacle patterns, speed ramping, coin/collectible systems, distance scoring."
globs: ["**/Runner*.cs", "**/Endless*.cs", "**/Chunk*.cs", "**/Obstacle*.cs", "**/Lane*.cs"]
---

# Endless Runner Patterns

## Chunk-Based Level Generation

```csharp
public sealed class ChunkSpawner : MonoBehaviour
{
    [SerializeField] private GameObject[] m_ChunkPrefabs;
    [SerializeField] private float m_ChunkLength = 20f;
    [SerializeField] private int m_ActiveChunkCount = 5;
    [SerializeField] private Transform m_Player;

    private readonly Queue<GameObject> m_ActiveChunks = new();
    private float m_SpawnZ;
    private ObjectPool<GameObject>[] m_ChunkPools;

    private void Awake()
    {
        m_SpawnZ = 0f;
        // Initialize pools for each chunk type
    }

    private void Update()
    {
        float playerZ = m_Player.position.z;
        float despawnZ = playerZ - m_ChunkLength;

        // Recycle chunks behind player
        while (m_ActiveChunks.Count > 0)
        {
            GameObject oldest = m_ActiveChunks.Peek();
            if (oldest.transform.position.z < despawnZ)
            {
                m_ActiveChunks.Dequeue();
                oldest.SetActive(false); // return to pool
            }
            else break;
        }

        // Spawn chunks ahead
        float spawnThreshold = playerZ + m_ChunkLength * m_ActiveChunkCount;
        while (m_SpawnZ < spawnThreshold)
        {
            SpawnChunk();
        }
    }

    private void SpawnChunk()
    {
        int index = Random.Range(0, m_ChunkPrefabs.Length);
        GameObject chunk = GetFromPool(index);
        chunk.transform.position = new Vector3(0f, 0f, m_SpawnZ);
        chunk.SetActive(true);
        m_ActiveChunks.Enqueue(chunk);
        m_SpawnZ += m_ChunkLength;
    }

    private GameObject GetFromPool(int index)
    {
        // Use ObjectPool<T> or custom pool
        return Instantiate(m_ChunkPrefabs[index]); // placeholder — use pool
    }
}
```

## Lane-Based Movement (3-Lane)

```csharp
public sealed class LaneRunner : MonoBehaviour
{
    [Header("Lanes")]
    [SerializeField] private float m_LaneWidth = 2.5f;
    [SerializeField] private float m_LaneSwitchSpeed = 15f;

    [Header("Jump")]
    [SerializeField] private float m_JumpForce = 10f;
    [SerializeField] private float m_Gravity = -30f;

    [Header("Slide")]
    [SerializeField] private float m_SlideDuration = 0.5f;

    private int m_CurrentLane; // -1, 0, 1
    private float m_TargetX;
    private float m_VerticalVelocity;
    private bool m_IsGrounded = true;
    private bool m_IsSliding;
    private CharacterController m_Controller;

    private void Awake()
    {
        m_Controller = GetComponent<CharacterController>();
        m_CurrentLane = 0;
    }

    public void SwitchLane(int direction) // -1 left, +1 right
    {
        m_CurrentLane = Mathf.Clamp(m_CurrentLane + direction, -1, 1);
        m_TargetX = m_CurrentLane * m_LaneWidth;
    }

    public void Jump()
    {
        if (!m_IsGrounded) return;
        m_VerticalVelocity = m_JumpForce;
        m_IsGrounded = false;
    }

    public void Slide()
    {
        if (m_IsSliding) return;
        StartCoroutine(SlideCoroutine());
    }

    private IEnumerator SlideCoroutine()
    {
        m_IsSliding = true;
        m_Controller.height = 0.5f;
        m_Controller.center = new Vector3(0f, 0.25f, 0f);

        yield return new WaitForSeconds(m_SlideDuration);

        m_Controller.height = 2f;
        m_Controller.center = new Vector3(0f, 1f, 0f);
        m_IsSliding = false;
    }

    private void Update()
    {
        // Lateral movement
        float currentX = transform.position.x;
        float newX = Mathf.MoveTowards(currentX, m_TargetX, m_LaneSwitchSpeed * Time.deltaTime);

        // Vertical
        if (m_IsGrounded && m_VerticalVelocity < 0f)
        {
            m_VerticalVelocity = -1f;
        }
        m_VerticalVelocity += m_Gravity * Time.deltaTime;

        Vector3 move = new Vector3(newX - currentX, m_VerticalVelocity * Time.deltaTime, 0f);
        m_Controller.Move(move);

        m_IsGrounded = m_Controller.isGrounded;
    }
}
```

## Touch Input Mapping

```csharp
public sealed class RunnerInput : MonoBehaviour
{
    [SerializeField] private LaneRunner m_Runner;
    [SerializeField] private float m_SwipeThreshold = 50f;

    private Vector2 m_TouchStart;

    private void Update()
    {
        if (UnityEngine.InputSystem.Touchscreen.current == null) return;

        UnityEngine.InputSystem.Controls.TouchControl touch =
            UnityEngine.InputSystem.Touchscreen.current.primaryTouch;

        if (touch.press.wasPressedThisFrame)
        {
            m_TouchStart = touch.position.ReadValue();
        }

        if (touch.press.wasReleasedThisFrame)
        {
            Vector2 delta = touch.position.ReadValue() - m_TouchStart;

            if (delta.magnitude > m_SwipeThreshold)
            {
                if (Mathf.Abs(delta.x) > Mathf.Abs(delta.y))
                {
                    m_Runner.SwitchLane(delta.x > 0f ? 1 : -1);
                }
                else if (delta.y > 0f)
                {
                    m_Runner.Jump();
                }
                else
                {
                    m_Runner.Slide();
                }
            }
        }
    }
}
```

## Speed Ramping

```csharp
public sealed class SpeedManager : MonoBehaviour
{
    [SerializeField] private float m_StartSpeed = 8f;
    [SerializeField] private float m_MaxSpeed = 25f;
    [SerializeField] private float m_AccelerationPerSecond = 0.1f;

    private float m_CurrentSpeed;
    private float m_PlayTime;

    public float CurrentSpeed => m_CurrentSpeed;

    private void Update()
    {
        m_PlayTime += Time.deltaTime;
        m_CurrentSpeed = Mathf.Min(m_StartSpeed + m_AccelerationPerSecond * m_PlayTime, m_MaxSpeed);
    }

    public void ResetSpeed()
    {
        m_PlayTime = 0f;
        m_CurrentSpeed = m_StartSpeed;
    }
}
```

## Scoring

- **Distance score:** increases with time x speed
- **Coin multiplier:** collected coins multiply final score
- **Combo bonus:** consecutive collectibles without missing

## Obstacle Design Patterns

- **Low barrier:** jump over
- **High barrier:** slide under
- **Side barrier:** switch lanes
- **Combined:** low + side forces specific lane + jump
- **Moving obstacle:** timing-based avoidance

## Performance

- Pool ALL chunks, obstacles, collectibles, and effects
- Only 3-5 chunks active at any time
- Disable renderers/colliders when chunks are pooled
- Use LOD or disable distant chunk details
- Move world toward player (or keep player stationary and move world) to avoid floating-point precision issues at large Z values

---

## Complete Movement Controller

Full lane-switching controller with smooth interpolation, variable-height jump, slide with collider resize, and animation state sync. Uses UniTask instead of coroutines, and a plain C# system for logic.

```csharp
// RunnerMovementModel — pure C# state, no Unity API
public sealed class RunnerMovementModel
{
    public int CurrentLane;       // -1, 0, 1
    public float TargetX;
    public float VerticalVelocity;
    public bool IsGrounded = true;
    public bool IsSliding;
    public bool IsJumping;
    public bool IsStumbling;
    public bool IsDead;
    public float JumpHoldTime;    // tracks how long jump is held
    public float SlideTimer;
    public float StumbleTimer;
}

// RunnerMovementSystem — all logic, injected via VContainer
public sealed class RunnerMovementSystem : IDisposable
{
    private readonly RunnerMovementModel m_Model;
    private readonly RunnerConfig m_Config;
    private readonly IPublisher<RunnerDiedMessage> m_DiedPublisher;

    [Inject]
    public RunnerMovementSystem(
        RunnerMovementModel model,
        RunnerConfig config,
        IPublisher<RunnerDiedMessage> diedPublisher)
    {
        m_Model = model;
        m_Config = config;
        m_DiedPublisher = diedPublisher;
    }

    public void SwitchLane(int direction)
    {
        if (m_Model.IsStumbling || m_Model.IsDead) return;
        m_Model.CurrentLane = Mathf.Clamp(m_Model.CurrentLane + direction, -1, 1);
        m_Model.TargetX = m_Model.CurrentLane * m_Config.LaneWidth;
    }

    public void BeginJump()
    {
        if (!m_Model.IsGrounded || m_Model.IsSliding || m_Model.IsDead) return;
        m_Model.VerticalVelocity = m_Config.MinJumpForce;
        m_Model.IsGrounded = false;
        m_Model.IsJumping = true;
        m_Model.JumpHoldTime = 0f;
    }

    // Call every frame while jump button is held for variable height
    public void HoldJump(float deltaTime)
    {
        if (!m_Model.IsJumping) return;
        m_Model.JumpHoldTime += deltaTime;
        if (m_Model.JumpHoldTime < m_Config.MaxJumpHoldDuration)
        {
            m_Model.VerticalVelocity += m_Config.JumpHoldAcceleration * deltaTime;
            m_Model.VerticalVelocity = Mathf.Min(m_Model.VerticalVelocity, m_Config.MaxJumpForce);
        }
    }

    public void ReleaseJump()
    {
        m_Model.IsJumping = false;
    }

    public void BeginSlide()
    {
        if (m_Model.IsSliding || m_Model.IsDead) return;
        m_Model.IsSliding = true;
        m_Model.SlideTimer = m_Config.SlideDuration;
        // If airborne, slam down fast
        if (!m_Model.IsGrounded)
        {
            m_Model.VerticalVelocity = m_Config.SlideSlamVelocity;
        }
    }

    public void Tick(float deltaTime)
    {
        if (m_Model.IsDead) return;

        // Slide timer
        if (m_Model.IsSliding)
        {
            m_Model.SlideTimer -= deltaTime;
            if (m_Model.SlideTimer <= 0f)
            {
                m_Model.IsSliding = false;
            }
        }

        // Stumble timer
        if (m_Model.IsStumbling)
        {
            m_Model.StumbleTimer -= deltaTime;
            if (m_Model.StumbleTimer <= 0f)
            {
                m_Model.IsStumbling = false;
            }
        }

        // Gravity
        if (!m_Model.IsGrounded)
        {
            m_Model.VerticalVelocity += m_Config.Gravity * deltaTime;
        }
    }

    public void OnLanded()
    {
        m_Model.IsGrounded = true;
        m_Model.IsJumping = false;
        m_Model.VerticalVelocity = -1f; // small downward to stay grounded
    }

    public void Dispose() { }
}

// RunnerConfig — ScriptableObject for all tuning values
[CreateAssetMenu(menuName = "Runner/Movement Config")]
public sealed class RunnerConfig : ScriptableObject
{
    [Header("Lanes")]
    [SerializeField] private float m_LaneWidth = 2.5f;
    [SerializeField] private float m_LaneSwitchSpeed = 18f;
    [SerializeField] private float m_MagneticSnapDistance = 0.05f;

    [Header("Jump — Variable Height")]
    [SerializeField] private float m_MinJumpForce = 8f;
    [SerializeField] private float m_MaxJumpForce = 14f;
    [SerializeField] private float m_JumpHoldAcceleration = 30f;
    [SerializeField] private float m_MaxJumpHoldDuration = 0.25f;
    [SerializeField] private float m_Gravity = -40f;

    [Header("Slide")]
    [SerializeField] private float m_SlideDuration = 0.6f;
    [SerializeField] private float m_SlideColliderHeight = 0.5f;
    [SerializeField] private float m_SlideSlamVelocity = -25f;

    public float LaneWidth => m_LaneWidth;
    public float LaneSwitchSpeed => m_LaneSwitchSpeed;
    public float MagneticSnapDistance => m_MagneticSnapDistance;
    public float MinJumpForce => m_MinJumpForce;
    public float MaxJumpForce => m_MaxJumpForce;
    public float JumpHoldAcceleration => m_JumpHoldAcceleration;
    public float MaxJumpHoldDuration => m_MaxJumpHoldDuration;
    public float Gravity => m_Gravity;
    public float SlideDuration => m_SlideDuration;
    public float SlideColliderHeight => m_SlideColliderHeight;
    public float SlideSlamVelocity => m_SlideSlamVelocity;
}
```

### Animation State Sync (View Layer)

The view reads the model and drives the Animator. No logic here — just mapping state to parameters.

```csharp
public sealed class RunnerAnimationView : MonoBehaviour
{
    [SerializeField] private Animator m_Animator;

    private static readonly int k_IsGrounded = Animator.StringToHash("IsGrounded");
    private static readonly int k_IsSliding = Animator.StringToHash("IsSliding");
    private static readonly int k_IsStumbling = Animator.StringToHash("IsStumbling");
    private static readonly int k_IsDead = Animator.StringToHash("IsDead");
    private static readonly int k_VerticalVelocity = Animator.StringToHash("VerticalVelocity");

    private RunnerMovementModel m_Model;

    [Inject]
    public void Construct(RunnerMovementModel model)
    {
        m_Model = model;
    }

    private void LateUpdate()
    {
        m_Animator.SetBool(k_IsGrounded, m_Model.IsGrounded);
        m_Animator.SetBool(k_IsSliding, m_Model.IsSliding);
        m_Animator.SetBool(k_IsStumbling, m_Model.IsStumbling);
        m_Animator.SetBool(k_IsDead, m_Model.IsDead);
        m_Animator.SetFloat(k_VerticalVelocity, m_Model.VerticalVelocity);
    }
}
```

### Magnetic Lane Alignment

When the character is close enough to the target lane center, snap it precisely. This prevents visible oscillation from `MoveTowards` never quite reaching the target.

```csharp
// Inside the view's Update, after computing lateral movement:
float distanceToTarget = Mathf.Abs(currentX - m_Model.TargetX);
if (distanceToTarget < m_Config.MagneticSnapDistance)
{
    newX = m_Model.TargetX; // snap exactly
}
else
{
    newX = Mathf.MoveTowards(currentX, m_Model.TargetX, m_Config.LaneSwitchSpeed * Time.deltaTime);
}
```

---

## Obstacle Detection and Response

Use trigger colliders for obstacle detection. Physics-based collision causes jitter at high speeds and unpredictable knockback.

### Trigger-Based Hit Detection

```csharp
public sealed class ObstacleHitDetector : MonoBehaviour
{
    private RunnerMovementModel m_Model;
    private RunnerHitSystem m_HitSystem;

    [Inject]
    public void Construct(RunnerMovementModel model, RunnerHitSystem hitSystem)
    {
        m_Model = model;
        m_HitSystem = hitSystem;
    }

    private void OnTriggerEnter(Collider other)
    {
        if (m_Model.IsStumbling || m_Model.IsDead) return;

        if (other.CompareTag("Obstacle"))
        {
            m_HitSystem.HandleObstacleHit();
        }
        else if (other.CompareTag("NearMiss"))
        {
            m_HitSystem.HandleNearMiss();
        }
    }
}
```

### Hit Response System

```csharp
public sealed class RunnerHitSystem : IDisposable
{
    private readonly RunnerMovementModel m_Model;
    private readonly RunnerStatsModel m_Stats;
    private readonly IPublisher<RunnerHitMessage> m_HitPublisher;
    private readonly IPublisher<NearMissMessage> m_NearMissPublisher;
    private readonly RunnerConfig m_Config;

    private bool m_HasShield;

    [Inject]
    public RunnerHitSystem(
        RunnerMovementModel model,
        RunnerStatsModel stats,
        RunnerConfig config,
        IPublisher<RunnerHitMessage> hitPublisher,
        IPublisher<NearMissMessage> nearMissPublisher)
    {
        m_Model = model;
        m_Stats = stats;
        m_Config = config;
        m_HitPublisher = hitPublisher;
        m_NearMissPublisher = nearMissPublisher;
    }

    public void HandleObstacleHit()
    {
        // Shield absorbs one hit
        if (m_HasShield)
        {
            m_HasShield = false;
            m_HitPublisher.Publish(new RunnerHitMessage(HitResult.ShieldAbsorbed));
            return;
        }

        m_Stats.HitCount++;

        if (m_Stats.HitCount >= m_Config.MaxHitsBeforeDeath)
        {
            m_Model.IsDead = true;
            m_HitPublisher.Publish(new RunnerHitMessage(HitResult.Death));
            return;
        }

        // Stumble: brief invulnerability + speed penalty
        m_Model.IsStumbling = true;
        m_Model.StumbleTimer = m_Config.StumbleDuration;
        m_HitPublisher.Publish(new RunnerHitMessage(HitResult.Stumble));
    }

    public void HandleNearMiss()
    {
        m_Stats.NearMissCount++;
        int bonus = m_Config.NearMissBaseScore * m_Stats.NearMissCount;
        m_NearMissPublisher.Publish(new NearMissMessage(bonus));
    }

    public void GrantShield() => m_HasShield = true;
    public void Dispose() { }
}

public readonly struct RunnerHitMessage
{
    public readonly HitResult Result;
    public RunnerHitMessage(HitResult result) { Result = result; }
}

public readonly struct NearMissMessage
{
    public readonly int BonusScore;
    public NearMissMessage(int bonusScore) { BonusScore = bonusScore; }
}

public enum HitResult { Stumble, Death, ShieldAbsorbed }
```

### Near-Miss Detection Setup

Place a slightly larger trigger collider around each obstacle. Tag the outer collider `NearMiss` and the inner collider `Obstacle`. When the player enters the outer zone but not the inner zone, it counts as a near miss. Remove the near-miss trigger after the player passes to avoid double-counting.

---

## Game Loop State Machine

```csharp
public enum RunnerGameState { Menu, Countdown, Running, Stumble, GameOver, Results }

public sealed class RunnerGameLoopSystem : IDisposable
{
    private readonly RunnerMovementModel m_MovementModel;
    private readonly RunnerStatsModel m_StatsModel;
    private readonly IPublisher<GameStateChangedMessage> m_StatePublisher;
    private readonly ISubscriber<RunnerHitMessage> m_HitSubscriber;
    private readonly IObjectPool<GameObject> m_ObstaclePool;
    private readonly CancellationTokenSource m_Cts = new();
    private IDisposable m_HitSubscription;

    private RunnerGameState m_CurrentState = RunnerGameState.Menu;

    [Inject]
    public RunnerGameLoopSystem(
        RunnerMovementModel movementModel,
        RunnerStatsModel statsModel,
        IPublisher<GameStateChangedMessage> statePublisher,
        ISubscriber<RunnerHitMessage> hitSubscriber)
    {
        m_MovementModel = movementModel;
        m_StatsModel = statsModel;
        m_StatePublisher = statePublisher;
        m_HitSubscriber = hitSubscriber;
        m_HitSubscription = m_HitSubscriber.Subscribe(OnRunnerHit);
    }

    public void TransitionTo(RunnerGameState newState)
    {
        ExitState(m_CurrentState);
        m_CurrentState = newState;
        EnterState(newState);
        m_StatePublisher.Publish(new GameStateChangedMessage(newState));
    }

    private void EnterState(RunnerGameState state)
    {
        switch (state)
        {
            case RunnerGameState.Countdown:
                StartCountdownAsync(m_Cts.Token).Forget();
                break;
            case RunnerGameState.Running:
                Time.timeScale = 1f;
                break;
            case RunnerGameState.GameOver:
                m_MovementModel.IsDead = true;
                break;
            case RunnerGameState.Results:
                // Finalize score, check high score
                break;
        }
    }

    private void ExitState(RunnerGameState state)
    {
        switch (state)
        {
            case RunnerGameState.Running:
                // Pause speed accumulation
                break;
            case RunnerGameState.GameOver:
                // Clean up death effects
                break;
        }
    }

    private async UniTaskVoid StartCountdownAsync(CancellationToken token)
    {
        // 3-2-1 countdown, then transition to Running
        await UniTask.Delay(TimeSpan.FromSeconds(3), cancellationToken: token);
        TransitionTo(RunnerGameState.Running);
    }

    private void OnRunnerHit(RunnerHitMessage message)
    {
        if (message.Result == HitResult.Death)
        {
            TransitionTo(RunnerGameState.GameOver);
        }
    }

    // Pause: freeze time and disable input (view layer handles input gating)
    public void SetPaused(bool paused)
    {
        Time.timeScale = paused ? 0f : 1f;
    }

    // Restart without scene reload: reset all models and pools
    public void Restart()
    {
        m_MovementModel.CurrentLane = 0;
        m_MovementModel.TargetX = 0f;
        m_MovementModel.VerticalVelocity = 0f;
        m_MovementModel.IsGrounded = true;
        m_MovementModel.IsSliding = false;
        m_MovementModel.IsJumping = false;
        m_MovementModel.IsStumbling = false;
        m_MovementModel.IsDead = false;

        m_StatsModel.Reset();

        // Return all pooled objects (chunks, obstacles, collectibles)
        // The ChunkSpawner and ObstacleSpawner listen to GameStateChanged
        // and handle their own pool cleanup

        TransitionTo(RunnerGameState.Countdown);
    }

    public void Dispose()
    {
        m_Cts.Cancel();
        m_HitSubscription.Dispose();
    }
}

public readonly struct GameStateChangedMessage
{
    public readonly RunnerGameState NewState;
    public GameStateChangedMessage(RunnerGameState newState) { NewState = newState; }
}
```

---

## Difficulty Progression

### Speed Curve with Plateaus

Rather than a constant ramp, use a stepped curve that gives players breathing room at each plateau before the next speed increase.

```csharp
[CreateAssetMenu(menuName = "Runner/Difficulty Config")]
public sealed class DifficultyConfig : ScriptableObject
{
    [SerializeField] private AnimationCurve m_SpeedCurve;   // X = distance, Y = speed
    [SerializeField] private float m_MaxSpeed = 30f;

    [Header("Obstacle Density")]
    [SerializeField] private float m_BaseObstacleSpacing = 15f;
    [SerializeField] private float m_MinObstacleSpacing = 5f;
    [SerializeField] private AnimationCurve m_DensityCurve; // X = distance, Y = 0..1

    [Header("Obstacle Type Unlocks")]
    [SerializeField] private ObstacleUnlockEntry[] m_ObstacleUnlocks;

    public float GetSpeed(float distance)
    {
        return m_SpeedCurve.Evaluate(distance) * m_MaxSpeed;
    }

    public float GetObstacleSpacing(float distance)
    {
        float density = m_DensityCurve.Evaluate(distance);
        return Mathf.Lerp(m_BaseObstacleSpacing, m_MinObstacleSpacing, density);
    }

    public ObstacleType[] GetAvailableObstacles(float distance)
    {
        int count = 0;
        for (int entryIndex = 0; entryIndex < m_ObstacleUnlocks.Length; entryIndex++)
        {
            if (distance >= m_ObstacleUnlocks[entryIndex].UnlockDistance)
            {
                count++;
            }
        }
        var result = new ObstacleType[count];
        int resultIndex = 0;
        for (int entryIndex = 0; entryIndex < m_ObstacleUnlocks.Length; entryIndex++)
        {
            if (distance >= m_ObstacleUnlocks[entryIndex].UnlockDistance)
            {
                result[resultIndex++] = m_ObstacleUnlocks[entryIndex].Type;
            }
        }
        return result;
    }
}

[System.Serializable]
public struct ObstacleUnlockEntry
{
    public ObstacleType Type;
    public float UnlockDistance;
}
```

### Chunk Difficulty Rating

Assign each chunk prefab a difficulty rating. The chunk spawner selects chunks whose rating matches the current distance bracket, with weighted random selection favoring appropriate difficulty.

```csharp
[System.Serializable]
public struct ChunkEntry
{
    public GameObject Prefab;
    [Range(1, 10)] public int DifficultyRating;
    public float Weight;
}

// In ChunkSpawner, select chunks weighted by closeness to target difficulty:
// targetDifficulty = Mathf.Clamp((int)(distance / 100f) + 1, 1, 10);
// Weight chunks where |rating - target| <= 2, with higher weight for exact match.
```

---

## Power-Up System

### Power-Up Definitions

```csharp
public enum PowerUpType { Magnet, Shield, ScoreMultiplier, MegaJump }

[CreateAssetMenu(menuName = "Runner/Power-Up Definition")]
public sealed class PowerUpDefinition : ScriptableObject
{
    [SerializeField] private PowerUpType m_Type;
    [SerializeField] private float m_Duration = 5f;
    [SerializeField] private float m_EffectValue = 2f;  // multiplier or radius
    [SerializeField] private bool m_RefreshOnReCollect = true; // true = reset timer, false = stack

    public PowerUpType Type => m_Type;
    public float Duration => m_Duration;
    public float EffectValue => m_EffectValue;
    public bool RefreshOnReCollect => m_RefreshOnReCollect;
}
```

### Power-Up Tracking System

```csharp
public sealed class PowerUpSystem : ITickable, IDisposable
{
    // Pre-allocated array avoids allocation in Tick
    private readonly ActivePowerUp[] m_ActivePowerUps = new ActivePowerUp[8];
    private int m_ActiveCount;
    private readonly IPublisher<PowerUpChangedMessage> m_Publisher;

    [Inject]
    public PowerUpSystem(IPublisher<PowerUpChangedMessage> publisher)
    {
        m_Publisher = publisher;
    }

    public void Activate(PowerUpDefinition definition)
    {
        // Check if already active
        for (int powerUpIndex = 0; powerUpIndex < m_ActiveCount; powerUpIndex++)
        {
            if (m_ActivePowerUps[powerUpIndex].Type == definition.Type)
            {
                if (definition.RefreshOnReCollect)
                {
                    m_ActivePowerUps[powerUpIndex].RemainingTime = definition.Duration;
                }
                else
                {
                    m_ActivePowerUps[powerUpIndex].StackCount++;
                }
                m_Publisher.Publish(new PowerUpChangedMessage(definition.Type, true));
                return;
            }
        }

        if (m_ActiveCount >= m_ActivePowerUps.Length) return;

        m_ActivePowerUps[m_ActiveCount] = new ActivePowerUp
        {
            Type = definition.Type,
            RemainingTime = definition.Duration,
            EffectValue = definition.EffectValue,
            StackCount = 1
        };
        m_ActiveCount++;
        m_Publisher.Publish(new PowerUpChangedMessage(definition.Type, true));
    }

    public void Tick()
    {
        float deltaTime = Time.deltaTime;
        for (int powerUpIndex = m_ActiveCount - 1; powerUpIndex >= 0; powerUpIndex--)
        {
            m_ActivePowerUps[powerUpIndex].RemainingTime -= deltaTime;
            if (m_ActivePowerUps[powerUpIndex].RemainingTime <= 0f)
            {
                PowerUpType expired = m_ActivePowerUps[powerUpIndex].Type;
                // Swap-remove to avoid shifting
                m_ActivePowerUps[powerUpIndex] = m_ActivePowerUps[m_ActiveCount - 1];
                m_ActiveCount--;
                m_Publisher.Publish(new PowerUpChangedMessage(expired, false));
            }
        }
    }

    public bool IsActive(PowerUpType type)
    {
        for (int powerUpIndex = 0; powerUpIndex < m_ActiveCount; powerUpIndex++)
        {
            if (m_ActivePowerUps[powerUpIndex].Type == type) return true;
        }
        return false;
    }

    public float GetRemainingTime(PowerUpType type)
    {
        for (int powerUpIndex = 0; powerUpIndex < m_ActiveCount; powerUpIndex++)
        {
            if (m_ActivePowerUps[powerUpIndex].Type == type)
            {
                return m_ActivePowerUps[powerUpIndex].RemainingTime;
            }
        }
        return 0f;
    }

    public void Dispose() { }
}

public struct ActivePowerUp
{
    public PowerUpType Type;
    public float RemainingTime;
    public float EffectValue;
    public int StackCount;
}

public readonly struct PowerUpChangedMessage
{
    public readonly PowerUpType Type;
    public readonly bool IsActive;
    public PowerUpChangedMessage(PowerUpType type, bool isActive) { Type = type; IsActive = isActive; }
}
```

### Power-Up Spawn Frequency

Scale power-up spawn rate inversely with distance. Early on, power-ups appear frequently to teach the player. Later, they become rarer and more valuable.

```csharp
// In DifficultyConfig:
// [SerializeField] private AnimationCurve m_PowerUpFrequencyCurve; // X = distance, Y = chance per chunk
// Typical values: 0.4 at distance 0, dropping to 0.1 by distance 5000
```

---

## Common Pitfalls

### Chunk Seams

Visible gaps between spawned chunks happen when chunk prefabs have inconsistent lengths, or floating-point drift accumulates over long distances. Fix: always compute spawn position from a running `m_SpawnZ` counter rather than reading the previous chunk's transform. Ensure all chunk prefabs are authored at exactly the same length in the editor.

### Physics Jitter at High Speed

At high forward speeds, `CharacterController.Move` can skip past thin colliders. Use kinematic movement with manual overlap checks instead of relying on the physics engine. Set the character controller's skin width appropriately and call `Physics.SyncTransforms()` if you move objects manually.

### Animation Desync During Lane Switch Mid-Jump

When a player switches lanes while airborne, the lateral movement interpolation can fight with the jump animation's root motion. Fix: disable root motion on the animator and drive all position from code. The animation is cosmetic only — the system state is authoritative.

### Pool Exhaustion at Extreme Speed

When the player reaches maximum speed, chunks scroll past faster than they can be recycled. The spawn-ahead distance must scale with speed:

```csharp
float spawnAhead = m_ChunkLength * m_ActiveChunkCount * (m_CurrentSpeed / m_Config.BaseSpeed);
```

Also increase the pool's initial capacity to match the worst-case active count at max speed.

### Floating-Point Precision at Large Z

After running for several minutes, Z values exceed 10,000+. At these distances, float precision degrades and visual jitter appears. Two solutions:
1. **World shift:** Periodically teleport everything back toward the origin when Z exceeds a threshold (e.g., 1000). Update all active chunk positions, the player, and the camera in a single frame.
2. **Stationary player:** Keep the player at Z=0 and move the world backward. This avoids large coordinates entirely but requires all spawning logic to work in relative space.

### Input Eating on Simultaneous Swipes

Players often swipe diagonally (e.g., up-right for jump + lane switch). Treat horizontal and vertical components independently rather than choosing one or the other. If both components exceed their respective thresholds, fire both actions.
