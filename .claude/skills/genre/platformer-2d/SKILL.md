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
