---
name: platformer-2d
description: "2D platformer architecture — tight controls (coyote time, input buffer, variable jump), level design patterns, collectibles, checkpoints, hazards, boss patterns."
globs: ["**/Platform*.cs", "**/Player*.cs", "**/Level*.cs"]
---

# 2D Platformer Patterns

## Controller Feel — The Numbers That Matter

```csharp
public sealed class PlatformerController : MonoBehaviour
{
    [Header("Movement")]
    [SerializeField] private float m_MoveSpeed = 8f;
    [SerializeField] private float m_Acceleration = 50f;
    [SerializeField] private float m_Deceleration = 60f;
    [SerializeField] private float m_AirControlMultiplier = 0.65f;

    [Header("Jump")]
    [SerializeField] private float m_JumpForce = 16f;
    [SerializeField] private float m_FallMultiplier = 2.5f;
    [SerializeField] private float m_LowJumpMultiplier = 2f;
    [SerializeField] private float m_CoyoteTime = 0.1f;
    [SerializeField] private float m_JumpBufferTime = 0.15f;
    [SerializeField] private float m_ApexHangMultiplier = 0.5f;
    [SerializeField] private float m_ApexThreshold = 1.5f;

    [Header("Ground Check")]
    [SerializeField] private Transform m_GroundCheck;
    [SerializeField] private float m_GroundCheckRadius = 0.15f;
    [SerializeField] private LayerMask m_GroundLayer;

    private Rigidbody2D m_Rb;
    private float m_CoyoteTimer;
    private float m_JumpBufferTimer;
    private bool m_IsGrounded;
    private bool m_JumpHeld;

    private void Awake()
    {
        m_Rb = GetComponent<Rigidbody2D>();
    }

    private void Update()
    {
        // Ground check
        m_IsGrounded = Physics2D.OverlapCircle(m_GroundCheck.position, m_GroundCheckRadius, m_GroundLayer);

        // Coyote time
        if (m_IsGrounded) m_CoyoteTimer = m_CoyoteTime;
        else m_CoyoteTimer -= Time.deltaTime;

        // Jump buffer
        if (Input.GetButtonDown("Jump"))
        {
            m_JumpBufferTimer = m_JumpBufferTime;
            m_JumpHeld = true;
        }
        if (Input.GetButtonUp("Jump")) m_JumpHeld = false;
        m_JumpBufferTimer -= Time.deltaTime;

        // Trigger jump
        if (m_JumpBufferTimer > 0f && m_CoyoteTimer > 0f)
        {
            m_Rb.linearVelocity = new Vector2(m_Rb.linearVelocity.x, m_JumpForce);
            m_JumpBufferTimer = 0f;
            m_CoyoteTimer = 0f;
        }
    }

    private void FixedUpdate()
    {
        // Horizontal movement with acceleration
        float targetSpeed = Input.GetAxisRaw("Horizontal") * m_MoveSpeed;
        float accel = m_IsGrounded ? m_Acceleration : m_Acceleration * m_AirControlMultiplier;
        float decel = m_IsGrounded ? m_Deceleration : m_Deceleration * m_AirControlMultiplier;
        float rate = Mathf.Abs(targetSpeed) > 0.01f ? accel : decel;

        float newSpeedX = Mathf.MoveTowards(m_Rb.linearVelocity.x, targetSpeed, rate * Time.fixedDeltaTime);
        m_Rb.linearVelocity = new Vector2(newSpeedX, m_Rb.linearVelocity.y);

        // Variable jump height + apex hang
        float yVel = m_Rb.linearVelocity.y;
        if (yVel < 0f)
        {
            // Falling — faster fall
            m_Rb.linearVelocity += Vector2.up * (Physics2D.gravity.y * (m_FallMultiplier - 1f) * Time.fixedDeltaTime);
        }
        else if (yVel > 0f && !m_JumpHeld)
        {
            // Released jump early — cut height
            m_Rb.linearVelocity += Vector2.up * (Physics2D.gravity.y * (m_LowJumpMultiplier - 1f) * Time.fixedDeltaTime);
        }

        // Apex hang — slow gravity near jump apex for more control
        if (Mathf.Abs(yVel) < m_ApexThreshold)
        {
            m_Rb.linearVelocity += Vector2.up * (Physics2D.gravity.y * (m_ApexHangMultiplier - 1f) * Time.fixedDeltaTime);
        }
    }
}
```

## Wall Slide / Wall Jump

```csharp
// In Update:
bool isTouchingWall = Physics2D.Raycast(transform.position, facingDirection, 0.5f, m_GroundLayer);
bool isWallSliding = isTouchingWall && !m_IsGrounded && m_Rb.linearVelocity.y < 0f;

if (isWallSliding)
{
    // Apply wall slide friction (cap fall speed)
    m_Rb.linearVelocity = new Vector2(m_Rb.linearVelocity.x,
        Mathf.Max(m_Rb.linearVelocity.y, -m_WallSlideSpeed));
}

// Wall jump: jump away from wall
if (m_JumpBufferTimer > 0f && isWallSliding)
{
    m_Rb.linearVelocity = new Vector2(-facingDirection.x * m_WallJumpForce.x, m_WallJumpForce.y);
    m_JumpBufferTimer = 0f;
}
```

## Dash

```csharp
private bool m_CanDash = true;
private float m_DashCooldown = 0.5f;

private IEnumerator Dash(Vector2 direction)
{
    m_CanDash = false;
    m_Rb.gravityScale = 0f;
    m_Rb.linearVelocity = direction.normalized * m_DashSpeed;

    // I-frames during dash
    Physics2D.IgnoreLayerCollision(playerLayer, enemyLayer, true);

    yield return m_DashDuration; // cached WaitForSeconds

    m_Rb.gravityScale = m_DefaultGravity;
    Physics2D.IgnoreLayerCollision(playerLayer, enemyLayer, false);

    yield return new WaitForSeconds(m_DashCooldown);
    m_CanDash = true;
}
```

## One-Way Platforms

Use `PlatformEffector2D` with `surfaceArc = 180` and `useOneWay = true`.

Drop through: temporarily disable collider or set the effector's `rotationalOffset`.

## Level Design Patterns

- **Tilemap-based:** Rule Tiles for auto-tiling, Tile Palette for painting
- **Chunk loading:** divide large levels into chunks, load/unload based on player position
- **Parallax scrolling:** multiple background layers at different speeds

## Checkpoint System

```csharp
public sealed class Checkpoint : MonoBehaviour
{
    [SerializeField] private VoidEventChannel m_OnCheckpointReached;

    private static Vector3 s_LastCheckpointPosition;

    private void OnTriggerEnter2D(Collider2D other)
    {
        if (other.CompareTag("Player"))
        {
            s_LastCheckpointPosition = transform.position;
            m_OnCheckpointReached.Raise();
        }
    }

    public static Vector3 GetRespawnPosition() => s_LastCheckpointPosition;
}
```

## Boss Patterns

Phase-based state machine:
1. **Phase 1** (100-66% HP): basic attack pattern, vulnerable after combo
2. **Phase 2** (66-33% HP): faster attacks, new moves, environment hazards
3. **Phase 3** (33-0% HP): enraged, all attacks, short vulnerable windows

---

## Ground Detection Deep Dive

### OverlapCircle Setup

The ground check transform should be a child of the player positioned at the very bottom of the collider. Keep the radius small (0.1 to 0.2 units) to prevent false positives on nearby walls.

```csharp
public sealed class GroundDetector
{
    private readonly Transform m_CheckPoint;
    private readonly float m_Radius;
    private readonly LayerMask m_GroundMask;
    private readonly Collider2D[] m_ResultBuffer = new Collider2D[4];

    public bool IsGrounded { get; private set; }
    public bool WasGroundedLastFrame { get; private set; }
    public Vector2 GroundNormal { get; private set; }

    public GroundDetector(Transform checkPoint, float radius, LayerMask groundMask)
    {
        m_CheckPoint = checkPoint;
        m_Radius = radius;
        m_GroundMask = groundMask;
    }

    public void Tick()
    {
        WasGroundedLastFrame = IsGrounded;

        int hitCount = Physics2D.OverlapCircleNonAlloc(
            m_CheckPoint.position,
            m_Radius,
            m_ResultBuffer,
            m_GroundMask
        );

        IsGrounded = hitCount > 0;

        if (IsGrounded)
        {
            // Raycast downward to get the ground normal for slope detection
            RaycastHit2D normalHit = Physics2D.Raycast(
                m_CheckPoint.position,
                Vector2.down,
                m_Radius + 0.1f,
                m_GroundMask
            );
            GroundNormal = normalHit.collider != null ? normalHit.normal : Vector2.up;
        }
    }

    // True on the exact frame the player lands
    public bool JustLanded => IsGrounded && !WasGroundedLastFrame;

    // True on the exact frame the player leaves the ground
    public bool JustLeftGround => !IsGrounded && WasGroundedLastFrame;
}
```

### Raycast vs OverlapBox vs OverlapCircle

| Method | Best For | Caveat |
|--------|----------|--------|
| `OverlapCircle` | Round-bottomed characters, general use | Can clip walls if radius too large |
| `OverlapBox` | Flat-bottomed characters, precise ledge detection | Corners can catch on edges |
| Dual raycast (left+right foot) | Ledge hanging detection, slope angle | Gaps between rays can miss thin platforms |

For one-way platforms, the OverlapCircle can return a hit even when the player is below the platform. Filter by checking that the contact point is above the check position:

```csharp
// Filter false positives from one-way platforms
if (IsGrounded && m_ResultBuffer[0] != null)
{
    Bounds platformBounds = m_ResultBuffer[0].bounds;
    float playerBottom = m_CheckPoint.position.y - m_Radius;

    // Player is below platform top — ignore this as false ground
    if (playerBottom < platformBounds.max.y - 0.05f
        && m_Rb.linearVelocity.y > 0.1f)
    {
        IsGrounded = false;
    }
}
```

For moving platforms, store the platform's `Collider2D` reference so the movement system can parent the player or apply platform velocity.

---

## Coyote Time and Input Buffering — Complete Implementation

Coyote time and input buffering work together to make controls feel responsive. The key is handling edge cases around wall jumps and double jumps so buffers do not stack or fire unexpectedly.

```csharp
public sealed class JumpBufferSystem
{
    private readonly float m_CoyoteDuration;
    private readonly float m_JumpBufferDuration;
    private readonly float m_WallCoyoteDuration;

    private float m_CoyoteTimer;
    private float m_JumpBufferTimer;
    private float m_WallCoyoteTimer;
    private int m_AirJumpsRemaining;
    private int m_MaxAirJumps;
    private bool m_WasWallSliding;

    public JumpBufferSystem(float coyoteDuration, float jumpBufferDuration,
        float wallCoyoteDuration, int maxAirJumps)
    {
        m_CoyoteDuration = coyoteDuration;
        m_JumpBufferDuration = jumpBufferDuration;
        m_WallCoyoteDuration = wallCoyoteDuration;
        m_MaxAirJumps = maxAirJumps;
    }

    public void Tick(float deltaTime, bool isGrounded, bool isWallSliding, bool jumpPressed)
    {
        // Ground coyote timer
        if (isGrounded)
        {
            m_CoyoteTimer = m_CoyoteDuration;
            m_AirJumpsRemaining = m_MaxAirJumps;
        }
        else
        {
            m_CoyoteTimer -= deltaTime;
        }

        // Wall coyote timer — separate from ground coyote
        if (isWallSliding)
        {
            m_WallCoyoteTimer = m_WallCoyoteDuration;
            m_WasWallSliding = true;
        }
        else
        {
            m_WallCoyoteTimer -= deltaTime;
        }

        // Jump input buffer
        if (jumpPressed)
        {
            m_JumpBufferTimer = m_JumpBufferDuration;
        }
        else
        {
            m_JumpBufferTimer -= deltaTime;
        }
    }

    // Returns the type of jump to execute, or None
    public JumpType ConsumeJump()
    {
        if (m_JumpBufferTimer <= 0f)
        {
            return JumpType.None;
        }

        // Priority: ground jump > wall jump > air jump
        if (m_CoyoteTimer > 0f)
        {
            m_JumpBufferTimer = 0f;
            m_CoyoteTimer = 0f;
            // Prevent wall coyote from firing right after a ground jump
            m_WallCoyoteTimer = 0f;
            return JumpType.Ground;
        }

        if (m_WallCoyoteTimer > 0f)
        {
            m_JumpBufferTimer = 0f;
            m_WallCoyoteTimer = 0f;
            m_WasWallSliding = false;
            return JumpType.Wall;
        }

        if (m_AirJumpsRemaining > 0)
        {
            m_JumpBufferTimer = 0f;
            m_AirJumpsRemaining--;
            return JumpType.Air;
        }

        return JumpType.None;
    }

    public void ResetOnLand()
    {
        m_AirJumpsRemaining = m_MaxAirJumps;
        m_WasWallSliding = false;
    }
}

public enum JumpType
{
    None,
    Ground,
    Wall,
    Air
}
```

The wall coyote timer is kept separate from the ground coyote timer. Without this, a player leaving a wall slide could accidentally consume ground coyote time (or vice versa), producing wrong jump arcs. The priority order (ground > wall > air) prevents double-firing when timers overlap.

---

## Animation State Machine Integration

Cache all animator parameter hashes as `static readonly int` with the `k_` prefix. Never pass raw strings to `Animator.SetBool` or `Animator.SetTrigger` in gameplay code.

```csharp
public sealed class PlayerAnimationView : MonoBehaviour
{
    private static readonly int k_SpeedHash = Animator.StringToHash("Speed");
    private static readonly int k_VerticalVelocityHash = Animator.StringToHash("VerticalVelocity");
    private static readonly int k_IsGroundedHash = Animator.StringToHash("IsGrounded");
    private static readonly int k_IsWallSlidingHash = Animator.StringToHash("IsWallSliding");
    private static readonly int k_IsDashingHash = Animator.StringToHash("IsDashing");
    private static readonly int k_JumpTriggerHash = Animator.StringToHash("Jump");
    private static readonly int k_LandTriggerHash = Animator.StringToHash("Land");
    private static readonly int k_HurtTriggerHash = Animator.StringToHash("Hurt");
    private static readonly int k_DeathTriggerHash = Animator.StringToHash("Death");

    [SerializeField] private Animator m_Animator;
    [SerializeField] private SpriteRenderer m_SpriteRenderer;

    private PlayerModel m_Model;
    private readonly CompositeDisposable m_Disposables = new();

    [Inject]
    public void Construct(PlayerModel model)
    {
        m_Model = model;
    }

    private void Start()
    {
        // Observe model state changes reactively — no polling in Update
        m_Model.MoveSpeed
            .Subscribe(speed => m_Animator.SetFloat(k_SpeedHash, Mathf.Abs(speed)))
            .AddTo(m_Disposables);

        m_Model.VerticalVelocity
            .Subscribe(vy => m_Animator.SetFloat(k_VerticalVelocityHash, vy))
            .AddTo(m_Disposables);

        m_Model.IsGrounded
            .Subscribe(grounded =>
            {
                m_Animator.SetBool(k_IsGroundedHash, grounded);
                // Trigger land animation on the transition frame
                if (grounded)
                {
                    m_Animator.SetTrigger(k_LandTriggerHash);
                }
            })
            .AddTo(m_Disposables);

        m_Model.IsWallSliding
            .Subscribe(sliding => m_Animator.SetBool(k_IsWallSlidingHash, sliding))
            .AddTo(m_Disposables);

        m_Model.IsDashing
            .Subscribe(dashing => m_Animator.SetBool(k_IsDashingHash, dashing))
            .AddTo(m_Disposables);

        m_Model.FacingDirection
            .Subscribe(dir => m_SpriteRenderer.flipX = dir < 0)
            .AddTo(m_Disposables);
    }

    // Called by Systems via MessagePipe when jump starts
    public void PlayJumpAnimation()
    {
        m_Animator.SetTrigger(k_JumpTriggerHash);
    }

    public void PlayHurtAnimation()
    {
        m_Animator.SetTrigger(k_HurtTriggerHash);
    }

    public void PlayDeathAnimation()
    {
        m_Animator.SetTrigger(k_DeathTriggerHash);
    }

    private void OnDestroy()
    {
        m_Disposables.Dispose();
    }
}
```

### Animator Controller Layout

Set up the Animator Controller with these states and transitions:

| From | To | Condition |
|------|----|-----------|
| Idle | Run | Speed > 0.01 |
| Run | Idle | Speed < 0.01 |
| Any State | Jump | Jump trigger |
| Jump | Fall | VerticalVelocity < -0.1 |
| Fall | Idle | IsGrounded = true, Land trigger |
| Any State | WallSlide | IsWallSliding = true |
| WallSlide | Fall | IsWallSliding = false |
| Any State | Dash | IsDashing = true |
| Dash | Fall | IsDashing = false |
| Any State | Hurt | Hurt trigger |
| Any State | Death | Death trigger |

Set transition duration to 0 for snappy response. Disable "Has Exit Time" on all gameplay transitions so states change immediately when conditions are met.

---

## Game Feel: Juice and Feedback

### Screen Shake via Cinemachine Impulse

Attach a `CinemachineImpulseSource` to the player. Fire impulses on landing, taking damage, or dealing a killing blow.

```csharp
public sealed class GameFeelView : MonoBehaviour
{
    [Header("Screen Shake")]
    [SerializeField] private CinemachineImpulseSource m_LandImpulse;
    [SerializeField] private CinemachineImpulseSource m_HitImpulse;

    [Header("Squash and Stretch")]
    [SerializeField] private Transform m_SpriteTransform;
    [SerializeField] private float m_SquashScaleY = 0.7f;
    [SerializeField] private float m_StretchScaleY = 1.3f;
    [SerializeField] private float m_SquashStretchDuration = 0.1f;

    [Header("Particles")]
    [SerializeField] private ParticleSystem m_LandDustParticles;
    [SerializeField] private ParticleSystem m_DashTrailParticles;
    [SerializeField] private ParticleSystem m_JumpDustParticles;

    private Vector3 m_DefaultScale;

    private void Awake()
    {
        m_DefaultScale = m_SpriteTransform.localScale;
    }

    // Called when player lands on ground
    public void OnLand(float fallSpeed)
    {
        // Scale shake intensity by how fast the player was falling
        float intensity = Mathf.InverseLerp(2f, 20f, Mathf.Abs(fallSpeed));

        if (intensity > 0.1f)
        {
            m_LandImpulse.GenerateImpulse(intensity);
        }

        m_LandDustParticles.Play();
        SquashAsync(this.GetCancellationTokenOnDestroy()).Forget();
    }

    // Called when player starts a jump
    public void OnJump()
    {
        m_JumpDustParticles.Play();
        StretchAsync(this.GetCancellationTokenOnDestroy()).Forget();
    }

    // Called when player takes damage
    public void OnHit()
    {
        m_HitImpulse.GenerateImpulse();
    }

    public void OnDashStart()
    {
        m_DashTrailParticles.Play();
    }

    public void OnDashEnd()
    {
        m_DashTrailParticles.Stop();
    }

    private async UniTaskVoid SquashAsync(CancellationToken token)
    {
        // Squash: compress vertically, expand horizontally
        Vector3 squashScale = new Vector3(
            m_DefaultScale.x * (1f / m_SquashScaleY),
            m_DefaultScale.y * m_SquashScaleY,
            m_DefaultScale.z
        );

        m_SpriteTransform.localScale = squashScale;
        float elapsed = 0f;

        while (elapsed < m_SquashStretchDuration)
        {
            elapsed += Time.deltaTime;
            float t = elapsed / m_SquashStretchDuration;
            m_SpriteTransform.localScale = Vector3.Lerp(squashScale, m_DefaultScale, t);
            await UniTask.Yield(token);
        }

        m_SpriteTransform.localScale = m_DefaultScale;
    }

    private async UniTaskVoid StretchAsync(CancellationToken token)
    {
        // Stretch: expand vertically, compress horizontally
        Vector3 stretchScale = new Vector3(
            m_DefaultScale.x * (1f / m_StretchScaleY),
            m_DefaultScale.y * m_StretchScaleY,
            m_DefaultScale.z
        );

        m_SpriteTransform.localScale = stretchScale;
        float elapsed = 0f;

        while (elapsed < m_SquashStretchDuration)
        {
            elapsed += Time.deltaTime;
            float t = elapsed / m_SquashStretchDuration;
            m_SpriteTransform.localScale = Vector3.Lerp(stretchScale, m_DefaultScale, t);
            await UniTask.Yield(token);
        }

        m_SpriteTransform.localScale = m_DefaultScale;
    }
}
```

### Haptic Feedback (Gamepad)

For gamepad rumble on landing or taking damage:

```csharp
private static void PulseHaptics(float lowFreq, float highFreq, float durationSeconds)
{
    Gamepad pad = Gamepad.current;
    if (pad == null) return;

    pad.SetMotorSpeeds(lowFreq, highFreq);

    // Stop rumble after duration — fire and forget
    StopHapticsAfterDelay(pad, durationSeconds, default).Forget();
}

private static async UniTaskVoid StopHapticsAfterDelay(
    Gamepad pad, float delay, CancellationToken token)
{
    await UniTask.Delay(TimeSpan.FromSeconds(delay), cancellationToken: token);
    pad.SetMotorSpeeds(0f, 0f);
}
```

---

## Collectibles System

### Pickup Model and Messages

```csharp
public readonly struct CoinCollectedMessage
{
    public readonly int Value;
    public readonly Vector3 Position;

    public CoinCollectedMessage(int value, Vector3 position)
    {
        Value = value;
        Position = position;
    }
}

public sealed class CurrencyModel
{
    public ReactiveProperty<int> Coins { get; } = new(0);
}
```

### Collectible System (Logic)

```csharp
public sealed class CollectibleSystem : IDisposable
{
    private readonly CurrencyModel m_CurrencyModel;
    private readonly IPublisher<CoinCollectedMessage> m_CoinPublisher;
    private readonly IDisposable m_Subscription;

    [Inject]
    public CollectibleSystem(
        CurrencyModel currencyModel,
        IPublisher<CoinCollectedMessage> coinPublisher,
        ISubscriber<CoinCollectedMessage> coinSubscriber)
    {
        m_CurrencyModel = currencyModel;
        m_CoinPublisher = coinPublisher;
        m_Subscription = coinSubscriber.Subscribe(OnCoinCollected);
    }

    private void OnCoinCollected(CoinCollectedMessage message)
    {
        m_CurrencyModel.Coins.Value += message.Value;
    }

    public void Dispose()
    {
        m_Subscription.Dispose();
    }
}
```

### Magnetize Pattern (Items Fly Toward Player)

Pooled collectibles that drift toward the player once within a magnet radius. Movement uses `MoveTowards` with an acceleration curve so items start slow and speed up as they approach.

```csharp
public sealed class MagnetCollectibleView : MonoBehaviour
{
    [SerializeField] private float m_MagnetRadius = 3f;
    [SerializeField] private float m_MagnetMaxSpeed = 15f;
    [SerializeField] private float m_MagnetAcceleration = 30f;
    [SerializeField] private int m_CoinValue = 1;

    private Transform m_PlayerTransform;
    private float m_CurrentSpeed;
    private bool m_IsMagnetized;
    private IPublisher<CoinCollectedMessage> m_CoinPublisher;

    [Inject]
    public void Construct(PlayerModel playerModel, IPublisher<CoinCollectedMessage> coinPublisher)
    {
        m_CoinPublisher = coinPublisher;
    }

    public void SetPlayerTransform(Transform player)
    {
        m_PlayerTransform = player;
    }

    private void Update()
    {
        if (m_PlayerTransform == null) return;

        float distSq = (m_PlayerTransform.position - transform.position).sqrMagnitude;
        float magnetRadiusSq = m_MagnetRadius * m_MagnetRadius;

        if (distSq < magnetRadiusSq)
        {
            m_IsMagnetized = true;
        }

        if (!m_IsMagnetized) return;

        m_CurrentSpeed = Mathf.MoveTowards(m_CurrentSpeed, m_MagnetMaxSpeed,
            m_MagnetAcceleration * Time.deltaTime);

        transform.position = Vector3.MoveTowards(
            transform.position,
            m_PlayerTransform.position,
            m_CurrentSpeed * Time.deltaTime
        );

        // Pickup threshold
        if (distSq < 0.1f)
        {
            m_CoinPublisher.Publish(new CoinCollectedMessage(m_CoinValue, transform.position));
            m_CurrentSpeed = 0f;
            m_IsMagnetized = false;
            gameObject.SetActive(false); // Return to pool
        }
    }
}
```

---

## Checkpoint and Respawn

### Checkpoint Model

```csharp
public sealed class CheckpointModel
{
    public Vector3 RespawnPosition { get; set; }
    public int CheckpointIndex { get; set; } = -1;
    public ReactiveProperty<bool> IsRespawning { get; } = new(false);

    // Store state that should reset on respawn
    public int CoinsAtCheckpoint { get; set; }
    public float TimeAtCheckpoint { get; set; }
}
```

### Checkpoint System

```csharp
public sealed class CheckpointSystem : IDisposable
{
    private readonly CheckpointModel m_CheckpointModel;
    private readonly PlayerModel m_PlayerModel;
    private readonly CurrencyModel m_CurrencyModel;
    private readonly IPublisher<RespawnStartedMessage> m_RespawnPublisher;

    [Inject]
    public CheckpointSystem(
        CheckpointModel checkpointModel,
        PlayerModel playerModel,
        CurrencyModel currencyModel,
        IPublisher<RespawnStartedMessage> respawnPublisher)
    {
        m_CheckpointModel = checkpointModel;
        m_PlayerModel = playerModel;
        m_CurrencyModel = currencyModel;
        m_RespawnPublisher = respawnPublisher;
    }

    public void ActivateCheckpoint(Vector3 position, int index)
    {
        // Only activate if this checkpoint is newer
        if (index <= m_CheckpointModel.CheckpointIndex) return;

        m_CheckpointModel.RespawnPosition = position;
        m_CheckpointModel.CheckpointIndex = index;
        m_CheckpointModel.CoinsAtCheckpoint = m_CurrencyModel.Coins.Value;
    }

    public void TriggerRespawn()
    {
        m_CheckpointModel.IsRespawning.Value = true;

        // Reset to checkpoint state
        m_CurrencyModel.Coins.Value = m_CheckpointModel.CoinsAtCheckpoint;
        m_PlayerModel.Health.Value = m_PlayerModel.MaxHealth;
        m_PlayerModel.Position.Value = m_CheckpointModel.RespawnPosition;

        m_RespawnPublisher.Publish(new RespawnStartedMessage());
    }

    public void Dispose() { }
}

public readonly struct RespawnStartedMessage { }
```

### Respawn Sequence View (Fade Out, Reposition, Fade In)

```csharp
public sealed class RespawnView : MonoBehaviour
{
    [SerializeField] private CanvasGroup m_FadeOverlay;
    [SerializeField] private float m_FadeDuration = 0.4f;

    private CheckpointModel m_CheckpointModel;
    private Transform m_PlayerTransform;
    private readonly CompositeDisposable m_Disposables = new();

    [Inject]
    public void Construct(CheckpointModel checkpointModel,
        ISubscriber<RespawnStartedMessage> respawnSubscriber)
    {
        m_CheckpointModel = checkpointModel;
        respawnSubscriber.Subscribe(_ => RespawnSequenceAsync(
            this.GetCancellationTokenOnDestroy()).Forget()
        ).AddTo(m_Disposables);
    }

    public void SetPlayerTransform(Transform player)
    {
        m_PlayerTransform = player;
    }

    private async UniTaskVoid RespawnSequenceAsync(CancellationToken token)
    {
        // Fade to black
        await FadeAsync(0f, 1f, m_FadeDuration, token);

        // Reposition player while screen is black
        if (m_PlayerTransform != null)
        {
            m_PlayerTransform.position = m_CheckpointModel.RespawnPosition;
        }

        // Brief pause at black
        await UniTask.Delay(TimeSpan.FromSeconds(0.2f), cancellationToken: token);

        // Fade back in
        await FadeAsync(1f, 0f, m_FadeDuration, token);

        m_CheckpointModel.IsRespawning.Value = false;
    }

    private async UniTask FadeAsync(float from, float to, float duration, CancellationToken token)
    {
        float elapsed = 0f;
        m_FadeOverlay.alpha = from;

        while (elapsed < duration)
        {
            elapsed += Time.deltaTime;
            m_FadeOverlay.alpha = Mathf.Lerp(from, to, elapsed / duration);
            await UniTask.Yield(token);
        }

        m_FadeOverlay.alpha = to;
    }

    private void OnDestroy()
    {
        m_Disposables.Dispose();
    }
}
```

---

## Common Pitfalls

### Ghost Platforms (Player Falls Through Solid Ground)

This happens when the ground check origin is inside or above the collider instead of at the bottom edge. Verify in Scene view with a Gizmo:

```csharp
private void OnDrawGizmosSelected()
{
    if (m_GroundCheck == null) return;
    Gizmos.color = m_IsGrounded ? Color.green : Color.red;
    Gizmos.DrawWireSphere(m_GroundCheck.position, m_GroundCheckRadius);
}
```

Also occurs with fast-moving characters. Set the Rigidbody2D collision detection to **Continuous** and increase `Physics2D.defaultContactOffset` if needed.

### Coyote Time Abuse with Wall Jumps

If coyote time and wall coyote time share the same timer, a player can leave a wall, wait for coyote time, and execute a ground-style jump instead of a wall jump. Always use separate timers for ground coyote and wall coyote as shown in the `JumpBufferSystem` above.

### Physics Jitter from Mixing Transform and Rigidbody

Never set `transform.position` directly on a Rigidbody2D-driven character during gameplay. This teleports the physics body and causes one frame of desync. Use `Rigidbody2D.MovePosition` in FixedUpdate for smooth repositioning, or set `Rigidbody2D.position` directly.

The one exception is respawn — teleporting via `transform.position` is fine when the player is inactive or the screen is faded to black.

### One-Way Platform Fall-Through Timing

When the player presses down to drop through a one-way platform, disable the platform collider for a fixed duration (0.2-0.3 seconds). Common mistakes:
- Duration too short: player collider re-enables before they clear the platform, snapping them back on top.
- Duration too long: player can fall through multiple platforms unintentionally.
- Disabling the player's collider instead of the platform's: breaks all other collision for that duration.

Use `Physics2D.IgnoreCollision` between the specific player collider and platform collider instead of disabling the entire collider:

```csharp
public sealed class OneWayPlatformDropper
{
    public async UniTask DropThroughAsync(
        Collider2D playerCollider,
        Collider2D platformCollider,
        CancellationToken token)
    {
        Physics2D.IgnoreCollision(playerCollider, platformCollider, true);
        await UniTask.Delay(TimeSpan.FromSeconds(0.25f), cancellationToken: token);
        Physics2D.IgnoreCollision(playerCollider, platformCollider, false);
    }
}
```

### Slope Sliding on Idle

If the character slides down slopes when standing still, set `Rigidbody2D.sharedMaterial` to a Physics Material 2D with friction = 0.4 when grounded and friction = 0 when airborne. Alternatively, zero out velocity when grounded with no input and the slope angle is below the walkable threshold.

### Double Jump Consuming Coyote Time

If the player walks off a ledge and double-jumps, coyote time should not count as the first jump. Track whether the player actually pressed jump or just fell. If they fell (coyote expired without a jump press), the first mid-air jump press should be the "free" coyote jump, not a double jump consumption.
