---
name: topdown
description: "Top-down mobile game architecture — virtual joystick/tap-to-move/twin-stick touch movement, room transitions, fog of war, spawner patterns, wave systems, minimap."
globs: ["**/TopDown*.cs", "**/Room*.cs", "**/Wave*.cs", "**/Spawn*.cs"]
---

# Top-Down Game Patterns

## Movement Types

### Virtual Joystick (Mobile Touch)
```csharp
public sealed class VirtualJoystickController : MonoBehaviour
{
    [SerializeField] private float m_MoveSpeed = 6f;
    [SerializeField] private float m_JoystickDeadZone = 0.1f;

    private Rigidbody2D m_Rb;
    private Vector2 m_MoveInput;

    private void Awake()
    {
        m_Rb = GetComponent<Rigidbody2D>();
    }

    // Called by virtual joystick UI component
    public void SetMoveInput(Vector2 input)
    {
        m_MoveInput = input.magnitude > m_JoystickDeadZone ? input : Vector2.zero;

        // Auto-aim in move direction
        if (m_MoveInput.sqrMagnitude > 0.01f)
        {
            float angle = Mathf.Atan2(m_MoveInput.y, m_MoveInput.x) * Mathf.Rad2Deg - 90f;
            transform.rotation = Quaternion.Euler(0f, 0f, angle);
        }
    }

    private void FixedUpdate()
    {
        m_Rb.linearVelocity = m_MoveInput.normalized * m_MoveSpeed;
    }
}
```

### Twin-Stick Touch (Two Joysticks)
```csharp
// Left joystick: movement
// Right joystick: aim direction (auto-fire when aiming)
// Common in mobile shooters (Archero, Brawl Stars)
```

### Tap-to-Move (NavMeshAgent)
```csharp
private NavMeshAgent m_Agent;
private Camera m_Camera;

private void Update()
{
    if (UnityEngine.InputSystem.Touchscreen.current == null) return;

    UnityEngine.InputSystem.Controls.TouchControl touch =
        UnityEngine.InputSystem.Touchscreen.current.primaryTouch;

    if (touch.press.wasPressedThisFrame)
    {
        Vector2 touchPos = touch.position.ReadValue();
        Ray ray = m_Camera.ScreenPointToRay(touchPos);
        if (Physics.Raycast(ray, out RaycastHit hit, 100f, m_GroundLayer))
        {
            m_Agent.SetDestination(hit.point);
        }
    }
}
```

## Camera Setup

- Orthographic camera, fixed Y height
- Cinemachine with Framing Transposer (damping for smooth follow)
- Confiner 2D for room bounds (PolygonCollider2D)

## Room Transitions

```csharp
public sealed class RoomTransition : MonoBehaviour
{
    [SerializeField] private Transform m_SpawnPoint;
    [SerializeField] private CinemachineConfiner2D m_NextRoomConfiner;

    private void OnTriggerEnter2D(Collider2D other)
    {
        if (other.CompareTag("Player"))
        {
            other.transform.position = m_SpawnPoint.position;
            // Switch camera confiner to new room bounds
            CinemachineVirtualCamera vcam = FindFirstObjectByType<CinemachineVirtualCamera>();
            CinemachineConfiner2D confiner = vcam.GetComponent<CinemachineConfiner2D>();
            confiner.m_BoundingShape2D = m_NextRoomConfiner.m_BoundingShape2D;
        }
    }
}
```

## Wave System

```csharp
[System.Serializable]
public sealed class EnemyWave
{
    public List<SpawnEntry> Entries;
    public float DelayBeforeWave = 2f;
}

[System.Serializable]
public sealed class SpawnEntry
{
    public GameObject Prefab;
    public int Count;
    public float SpawnDelay = 0.5f;
}

public sealed class WaveManager : MonoBehaviour
{
    [SerializeField] private List<EnemyWave> m_Waves;
    [SerializeField] private Transform[] m_SpawnPoints;

    private int m_CurrentWave;
    private int m_EnemiesAlive;

    public event System.Action<int> OnWaveStarted;
    public event System.Action OnAllWavesComplete;

    public void StartNextWave()
    {
        if (m_CurrentWave >= m_Waves.Count)
        {
            OnAllWavesComplete?.Invoke();
            return;
        }

        StartCoroutine(SpawnWave(m_Waves[m_CurrentWave]));
        OnWaveStarted?.Invoke(m_CurrentWave);
        m_CurrentWave++;
    }

    private IEnumerator SpawnWave(EnemyWave wave)
    {
        yield return new WaitForSeconds(wave.DelayBeforeWave);

        for (int i = 0; i < wave.Entries.Count; i++)
        {
            SpawnEntry entry = wave.Entries[i];
            for (int j = 0; j < entry.Count; j++)
            {
                Transform spawnPoint = m_SpawnPoints[Random.Range(0, m_SpawnPoints.Length)];
                Instantiate(entry.Prefab, spawnPoint.position, Quaternion.identity);
                m_EnemiesAlive++;
                yield return new WaitForSeconds(entry.SpawnDelay);
            }
        }
    }

    public void OnEnemyDied()
    {
        m_EnemiesAlive--;
        if (m_EnemiesAlive <= 0)
        {
            StartNextWave();
        }
    }
}
```

## Minimap

1. Create a secondary camera (orthographic, top-down, high Y)
2. Set it to render to a RenderTexture
3. Display RenderTexture on a UI RawImage
4. Camera follows player position (X/Z only)
5. Use layers to control what the minimap camera sees

## Projectile Patterns

- **Single:** straight line from muzzle
- **Spread:** 3-5 projectiles in a fan arc
- **Burst:** N projectiles with delay between each
- **Homing:** Lerp direction toward target each frame
- **Circular:** spawn ring of projectiles expanding outward

## Fog of War (Simple)

1. Full-screen quad with black texture
2. Reveal circle around player (shader: distance from player position -> alpha)
3. Persistent reveal: write to reveal texture, never erase
4. Performance: use low-res render texture, blur the edges

---

## Enemy AI State Machine

### State Interface and Base Implementation

All enemy AI uses an explicit state machine with `IState` transitions. Each state owns its enter/exit/tick logic. The `EnemyAISystem` is a plain C# class registered in VContainer — MonoBehaviours only forward Unity callbacks.

```csharp
public interface IEnemyState
{
    void Enter(EnemyAIContext context);
    void Tick(EnemyAIContext context, float deltaTime);
    void Exit(EnemyAIContext context);
}

// Shared context passed to all states — avoids coupling states to MonoBehaviour
public sealed class EnemyAIContext
{
    public EnemyModel Model { get; }
    public Transform Transform { get; }
    public Transform PlayerTransform { get; set; }
    public NavMeshAgent Agent { get; }
    public LayerMask SightMask { get; }
    public float SightRange { get; }
    public float AttackRange { get; }
    public float FleeHealthThreshold { get; }

    public EnemyAIContext(EnemyModel model, Transform transform,
        NavMeshAgent agent, LayerMask sightMask,
        float sightRange, float attackRange, float fleeHealthThreshold)
    {
        Model = model;
        Transform = transform;
        Agent = agent;
        SightMask = sightMask;
        SightRange = sightRange;
        AttackRange = attackRange;
        FleeHealthThreshold = fleeHealthThreshold;
    }
}
```

### Patrol State

```csharp
public sealed class PatrolState : IEnemyState
{
    private readonly Vector3[] m_Waypoints;
    private int m_CurrentWaypointIndex;
    private static readonly float k_WaypointReachThreshold = 0.5f;

    public PatrolState(Vector3[] waypoints)
    {
        m_Waypoints = waypoints;
    }

    public void Enter(EnemyAIContext context)
    {
        context.Agent.isStopped = false;
        context.Agent.speed = context.Model.PatrolSpeed;
        context.Agent.SetDestination(m_Waypoints[m_CurrentWaypointIndex]);
    }

    public void Tick(EnemyAIContext context, float deltaTime)
    {
        // Advance to next waypoint when close enough
        float distSq = (context.Transform.position - m_Waypoints[m_CurrentWaypointIndex]).sqrMagnitude;
        if (distSq < k_WaypointReachThreshold * k_WaypointReachThreshold)
        {
            m_CurrentWaypointIndex = (m_CurrentWaypointIndex + 1) % m_Waypoints.Length;
            context.Agent.SetDestination(m_Waypoints[m_CurrentWaypointIndex]);
        }
    }

    public void Exit(EnemyAIContext context) { }
}
```

### Chase State

```csharp
public sealed class ChaseState : IEnemyState
{
    private static readonly float k_RepathInterval = 0.3f;
    private float m_RepathTimer;

    public void Enter(EnemyAIContext context)
    {
        context.Agent.isStopped = false;
        context.Agent.speed = context.Model.ChaseSpeed;
        m_RepathTimer = 0f;
    }

    public void Tick(EnemyAIContext context, float deltaTime)
    {
        m_RepathTimer -= deltaTime;

        if (m_RepathTimer <= 0f && context.PlayerTransform != null)
        {
            context.Agent.SetDestination(context.PlayerTransform.position);
            m_RepathTimer = k_RepathInterval;
        }
    }

    public void Exit(EnemyAIContext context)
    {
        context.Agent.isStopped = true;
    }
}
```

### Attack State

```csharp
public sealed class AttackState : IEnemyState
{
    private float m_AttackCooldownTimer;
    private readonly float m_AttackCooldown;
    private readonly IPublisher<EnemyAttackMessage> m_AttackPublisher;

    public AttackState(float attackCooldown, IPublisher<EnemyAttackMessage> attackPublisher)
    {
        m_AttackCooldown = attackCooldown;
        m_AttackPublisher = attackPublisher;
    }

    public void Enter(EnemyAIContext context)
    {
        context.Agent.isStopped = true;
        m_AttackCooldownTimer = 0f;
    }

    public void Tick(EnemyAIContext context, float deltaTime)
    {
        // Face the player
        if (context.PlayerTransform != null)
        {
            Vector3 direction = context.PlayerTransform.position - context.Transform.position;
            if (direction.sqrMagnitude > 0.01f)
            {
                float angle = Mathf.Atan2(direction.y, direction.x) * Mathf.Rad2Deg;
                context.Transform.rotation = Quaternion.Euler(0f, 0f, angle);
            }
        }

        m_AttackCooldownTimer -= deltaTime;
        if (m_AttackCooldownTimer <= 0f)
        {
            m_AttackPublisher.Publish(new EnemyAttackMessage(
                context.Model.AttackDamage, context.Transform.position));
            m_AttackCooldownTimer = m_AttackCooldown;
        }
    }

    public void Exit(EnemyAIContext context) { }
}

public readonly struct EnemyAttackMessage
{
    public readonly int Damage;
    public readonly Vector3 Position;

    public EnemyAttackMessage(int damage, Vector3 position)
    {
        Damage = damage;
        Position = position;
    }
}
```

### Flee State

```csharp
public sealed class FleeState : IEnemyState
{
    private static readonly float k_FleeDistance = 8f;

    public void Enter(EnemyAIContext context)
    {
        context.Agent.isStopped = false;
        context.Agent.speed = context.Model.ChaseSpeed * 1.2f;
    }

    public void Tick(EnemyAIContext context, float deltaTime)
    {
        if (context.PlayerTransform == null) return;

        // Run directly away from player
        Vector3 fleeDirection = context.Transform.position - context.PlayerTransform.position;
        fleeDirection.Normalize();

        Vector3 fleeTarget = context.Transform.position + fleeDirection * k_FleeDistance;
        context.Agent.SetDestination(fleeTarget);
    }

    public void Exit(EnemyAIContext context)
    {
        context.Agent.isStopped = true;
    }
}
```

### Sight Detection and Alert Propagation

Enemies check line of sight using `Physics2D.Raycast`. When one enemy spots the player, it publishes an alert message so nearby enemies transition to Chase without individually detecting the player.

```csharp
public sealed class SightDetector
{
    private readonly float m_SightRange;
    private readonly float m_SightAngle;
    private readonly LayerMask m_ObstacleMask;

    public SightDetector(float sightRange, float sightAngle, LayerMask obstacleMask)
    {
        m_SightRange = sightRange;
        m_SightAngle = sightAngle;
        m_ObstacleMask = obstacleMask;
    }

    public bool CanSeeTarget(Vector3 origin, Vector3 forward, Vector3 targetPosition)
    {
        Vector3 toTarget = targetPosition - origin;
        float distSq = toTarget.sqrMagnitude;

        if (distSq > m_SightRange * m_SightRange) return false;

        // Check angle
        float angle = Vector3.Angle(forward, toTarget);
        if (angle > m_SightAngle * 0.5f) return false;

        // Raycast for obstacles between enemy and target
        float dist = Mathf.Sqrt(distSq);
        RaycastHit2D hit = Physics2D.Raycast(origin, toTarget.normalized, dist, m_ObstacleMask);

        // If raycast hit nothing (no obstacle), target is visible
        return hit.collider == null;
    }
}

// Alert message — one enemy spots player, nearby enemies react
public readonly struct EnemyAlertMessage
{
    public readonly Vector3 PlayerPosition;
    public readonly float AlertRadius;

    public EnemyAlertMessage(Vector3 playerPosition, float alertRadius)
    {
        PlayerPosition = playerPosition;
        AlertRadius = alertRadius;
    }
}
```

### State Machine Runner with Transitions

```csharp
public sealed class EnemyStateMachine
{
    private IEnemyState m_CurrentState;
    private readonly EnemyAIContext m_Context;
    private readonly PatrolState m_PatrolState;
    private readonly ChaseState m_ChaseState;
    private readonly AttackState m_AttackState;
    private readonly FleeState m_FleeState;
    private readonly SightDetector m_SightDetector;

    public EnemyStateMachine(EnemyAIContext context, PatrolState patrolState,
        ChaseState chaseState, AttackState attackState, FleeState fleeState,
        SightDetector sightDetector)
    {
        m_Context = context;
        m_PatrolState = patrolState;
        m_ChaseState = chaseState;
        m_AttackState = attackState;
        m_FleeState = fleeState;
        m_SightDetector = sightDetector;
    }

    public void Start()
    {
        TransitionTo(m_PatrolState);
    }

    public void Tick(float deltaTime)
    {
        m_CurrentState.Tick(m_Context, deltaTime);
        EvaluateTransitions();
    }

    private void EvaluateTransitions()
    {
        bool canSeePlayer = m_Context.PlayerTransform != null
            && m_SightDetector.CanSeeTarget(
                m_Context.Transform.position,
                m_Context.Transform.right,
                m_Context.PlayerTransform.position);

        float healthPercent = (float)m_Context.Model.Health.Value / m_Context.Model.MaxHealth;
        float distToPlayerSq = m_Context.PlayerTransform != null
            ? (m_Context.PlayerTransform.position - m_Context.Transform.position).sqrMagnitude
            : float.MaxValue;

        float attackRangeSq = m_Context.AttackRange * m_Context.AttackRange;

        // Flee takes priority
        if (healthPercent <= m_Context.FleeHealthThreshold && canSeePlayer)
        {
            if (m_CurrentState != m_FleeState) TransitionTo(m_FleeState);
            return;
        }

        // Attack if in range and can see
        if (canSeePlayer && distToPlayerSq <= attackRangeSq)
        {
            if (m_CurrentState != m_AttackState) TransitionTo(m_AttackState);
            return;
        }

        // Chase if can see but not in attack range
        if (canSeePlayer)
        {
            if (m_CurrentState != m_ChaseState) TransitionTo(m_ChaseState);
            return;
        }

        // Default to patrol
        if (m_CurrentState != m_PatrolState) TransitionTo(m_PatrolState);
    }

    private void TransitionTo(IEnemyState newState)
    {
        m_CurrentState?.Exit(m_Context);
        m_CurrentState = newState;
        m_CurrentState.Enter(m_Context);
    }
}
```

---

## Pathfinding and NavMesh Integration

### NavMeshAgent Setup for Top-Down 2D

For 2D top-down games using NavMesh, constrain the agent to the XY plane. Use the NavMeshPlus package for 2D NavMesh baking, or use 3D NavMesh with the camera looking down the Y axis.

```csharp
public sealed class TopDownNavSetup : MonoBehaviour
{
    [SerializeField] private NavMeshAgent m_Agent;

    private void Awake()
    {
        // Lock Y rotation so the agent does not tilt on slopes
        m_Agent.updateRotation = false;
        m_Agent.updateUpAxis = false;
    }
}
```

### Dynamic Obstacle Handling

Place `NavMeshObstacle` components on destructible objects and doors. Use `carve = true` for objects that block paths permanently and `carve = false` for objects that only push agents aside.

When a door opens, disable the `NavMeshObstacle` component so the NavMesh re-carves and agents can path through. Do not destroy and recreate obstacles — toggling the component is cheaper.

### Stopping Distance and Attack Range Coordination

Set `NavMeshAgent.stoppingDistance` slightly less than the attack range so the enemy stops before overshooting into melee:

```csharp
// In ChaseState.Enter:
context.Agent.stoppingDistance = context.AttackRange * 0.8f;
```

If the stopping distance equals the attack range, the agent may oscillate between "arrived" and "not arrived" when the player stands right at the boundary. The 0.8 multiplier provides a buffer.

### Repath on Target Move Threshold

Do not repath every frame. Cache the last known target position and only call `SetDestination` when the target moves beyond a threshold:

```csharp
private Vector3 m_LastTargetPosition;
private static readonly float k_RepathThresholdSq = 1f; // 1 unit squared

private void RepathIfNeeded(EnemyAIContext context)
{
    if (context.PlayerTransform == null) return;

    Vector3 currentTarget = context.PlayerTransform.position;
    float distSq = (currentTarget - m_LastTargetPosition).sqrMagnitude;

    if (distSq > k_RepathThresholdSq)
    {
        context.Agent.SetDestination(currentTarget);
        m_LastTargetPosition = currentTarget;
    }
}
```

---

## Shooting System Architecture

### Projectile Model and Messages

```csharp
public readonly struct ProjectileFiredMessage
{
    public readonly Vector3 Origin;
    public readonly Vector3 Direction;
    public readonly float Speed;
    public readonly int Damage;
    public readonly int OwnerLayer;

    public ProjectileFiredMessage(Vector3 origin, Vector3 direction,
        float speed, int damage, int ownerLayer)
    {
        Origin = origin;
        Direction = direction;
        Speed = speed;
        Damage = damage;
        OwnerLayer = ownerLayer;
    }
}

public readonly struct DamageDealtMessage
{
    public readonly int Amount;
    public readonly Vector3 HitPosition;
    public readonly Vector3 HitNormal;

    public DamageDealtMessage(int amount, Vector3 hitPosition, Vector3 hitNormal)
    {
        Amount = amount;
        HitPosition = hitPosition;
        HitNormal = hitNormal;
    }
}
```

### Shooting System with Object Pooling

```csharp
public sealed class ShootingSystem : IDisposable
{
    private readonly ObjectPool<ProjectileView> m_ProjectilePool;
    private readonly IPublisher<ProjectileFiredMessage> m_FirePublisher;
    private readonly Camera m_Camera;

    [Inject]
    public ShootingSystem(
        ObjectPool<ProjectileView> projectilePool,
        IPublisher<ProjectileFiredMessage> firePublisher,
        Camera camera)
    {
        m_ProjectilePool = projectilePool;
        m_FirePublisher = firePublisher;
        m_Camera = camera;
    }

    // Calculate aim direction from player position toward mouse world position
    public Vector3 GetAimDirection(Vector3 shooterPosition, Vector2 screenPosition)
    {
        Vector3 worldPos = m_Camera.ScreenToWorldPoint(
            new Vector3(screenPosition.x, screenPosition.y, m_Camera.nearClipPlane));
        worldPos.z = 0f;

        Vector3 direction = (worldPos - shooterPosition).normalized;
        return direction;
    }

    public void FireProjectile(Vector3 origin, Vector3 direction, float speed,
        int damage, int ownerLayer)
    {
        ProjectileView projectile = m_ProjectilePool.Get();
        projectile.transform.position = origin;
        projectile.Initialize(direction, speed, damage, ownerLayer);

        m_FirePublisher.Publish(new ProjectileFiredMessage(
            origin, direction, speed, damage, ownerLayer));
    }

    // Spread shot: fires multiple projectiles in a fan
    public void FireSpread(Vector3 origin, Vector3 centerDirection, float speed,
        int damage, int ownerLayer, int projectileCount, float spreadAngle)
    {
        float angleStep = spreadAngle / (projectileCount - 1);
        float startAngle = -spreadAngle * 0.5f;

        for (int projectileIndex = 0; projectileIndex < projectileCount; projectileIndex++)
        {
            float currentAngle = startAngle + angleStep * projectileIndex;
            Vector3 rotatedDir = Quaternion.Euler(0f, 0f, currentAngle) * centerDirection;
            FireProjectile(origin, rotatedDir, speed, damage, ownerLayer);
        }
    }

    public void Dispose() { }
}
```

### Projectile View (Pooled)

```csharp
public sealed class ProjectileView : MonoBehaviour
{
    [SerializeField] private float m_Lifetime = 3f;
    [SerializeField] private ParticleSystem m_HitParticles;
    [SerializeField] private TrailRenderer m_Trail;

    private Vector3 m_Direction;
    private float m_Speed;
    private int m_Damage;
    private float m_AliveTimer;
    private ObjectPool<ProjectileView> m_Pool;
    private IPublisher<DamageDealtMessage> m_DamagePublisher;

    [Inject]
    public void Construct(IPublisher<DamageDealtMessage> damagePublisher)
    {
        m_DamagePublisher = damagePublisher;
    }

    public void SetPool(ObjectPool<ProjectileView> pool)
    {
        m_Pool = pool;
    }

    public void Initialize(Vector3 direction, float speed, int damage, int ownerLayer)
    {
        m_Direction = direction;
        m_Speed = speed;
        m_Damage = damage;
        m_AliveTimer = 0f;
        gameObject.layer = ownerLayer;

        if (m_Trail != null) m_Trail.Clear();
    }

    private void Update()
    {
        m_AliveTimer += Time.deltaTime;
        if (m_AliveTimer >= m_Lifetime)
        {
            ReturnToPool();
            return;
        }

        transform.position += m_Direction * (m_Speed * Time.deltaTime);
    }

    private void OnTriggerEnter2D(Collider2D other)
    {
        m_DamagePublisher.Publish(new DamageDealtMessage(
            m_Damage, transform.position, -m_Direction));

        if (m_HitParticles != null)
        {
            m_HitParticles.transform.SetParent(null);
            m_HitParticles.Play();
        }

        ReturnToPool();
    }

    private void ReturnToPool()
    {
        if (m_Pool != null)
        {
            m_Pool.Release(this);
        }
        else
        {
            gameObject.SetActive(false);
        }
    }
}
```

### Muzzle Flash and Shell Casings

Attach a `ParticleSystem` at the muzzle point. Play on fire. Use short-lived particles (0.05s lifetime) with a bright additive material. Shell casings use a separate particle system with physics-enabled particles that bounce off the ground.

```csharp
public sealed class WeaponEffectsView : MonoBehaviour
{
    [SerializeField] private ParticleSystem m_MuzzleFlash;
    [SerializeField] private ParticleSystem m_ShellCasing;
    [SerializeField] private Light2D m_MuzzleLight;
    [SerializeField] private float m_MuzzleLightDuration = 0.05f;

    public void PlayFireEffects()
    {
        m_MuzzleFlash.Play();
        m_ShellCasing.Emit(1);

        if (m_MuzzleLight != null)
        {
            FlashMuzzleLightAsync(this.GetCancellationTokenOnDestroy()).Forget();
        }
    }

    private async UniTaskVoid FlashMuzzleLightAsync(CancellationToken token)
    {
        m_MuzzleLight.enabled = true;
        await UniTask.Delay(
            TimeSpan.FromSeconds(m_MuzzleLightDuration),
            cancellationToken: token);
        m_MuzzleLight.enabled = false;
    }
}
```

---

## Input Normalization

### 8-Directional vs Analog

For keyboard input, raw values snap to -1, 0, or 1 on each axis, producing 8 directions. Diagonal movement is faster by a factor of ~1.41 unless normalized. Always normalize the combined input vector:

```csharp
private Vector2 GetNormalizedInput(Vector2 rawInput)
{
    // Clamp magnitude to 1 so diagonal is not faster than cardinal
    if (rawInput.sqrMagnitude > 1f)
    {
        return rawInput.normalized;
    }
    return rawInput;
}
```

### Camera-Relative Movement for Top-Down

When the camera can rotate (e.g., isometric view), transform input to be relative to the camera orientation:

```csharp
private Vector2 CameraRelativeInput(Vector2 input, Transform cameraTransform)
{
    // For a top-down 2D game with a non-rotated camera, this is identity.
    // For rotated or isometric cameras, project the camera's right/up onto the XY plane.
    Vector3 camRight = cameraTransform.right;
    Vector3 camUp = cameraTransform.up;

    // Flatten to 2D
    Vector2 right2D = new Vector2(camRight.x, camRight.y).normalized;
    Vector2 up2D = new Vector2(camUp.x, camUp.y).normalized;

    return right2D * input.x + up2D * input.y;
}
```

### Cursor Aim vs Stick Aim Switching

Detect active input device and switch aim mode. Mouse aim uses screen-to-world conversion. Gamepad aim uses right stick direction directly.

```csharp
public sealed class AimSystem
{
    private readonly Camera m_Camera;
    private bool m_UsingGamepad;
    private Vector2 m_AimDirection;

    public AimSystem(Camera camera)
    {
        m_Camera = camera;
    }

    // Call each frame with current input device state
    public Vector2 GetAimDirection(Vector3 playerPosition,
        Vector2 mouseScreenPos, Vector2 rightStickInput)
    {
        // Prefer gamepad if stick has input
        if (rightStickInput.sqrMagnitude > 0.1f)
        {
            m_UsingGamepad = true;
            m_AimDirection = rightStickInput.normalized;
        }
        else if (!m_UsingGamepad || rightStickInput.sqrMagnitude < 0.01f)
        {
            m_UsingGamepad = false;
            Vector3 worldPos = m_Camera.ScreenToWorldPoint(
                new Vector3(mouseScreenPos.x, mouseScreenPos.y, m_Camera.nearClipPlane));
            worldPos.z = 0f;
            m_AimDirection = ((Vector2)(worldPos - playerPosition)).normalized;
        }

        return m_AimDirection;
    }

    public bool IsUsingGamepad => m_UsingGamepad;
}
```

---

## Performance at Scale (50+ Enemies)

### Spatial Partitioning with Grid Buckets

When many enemies need to detect the player or each other, a flat loop over all enemies is O(n^2). Use a grid-based spatial hash to only check neighbors in adjacent cells.

```csharp
public sealed class SpatialGrid<T> where T : class
{
    private readonly Dictionary<int, List<T>> m_Buckets = new();
    private readonly float m_CellSize;
    private readonly List<T> m_EmptyList = new();

    // Pre-allocated list for reuse in queries
    private readonly List<T> m_QueryResult = new(64);

    public SpatialGrid(float cellSize)
    {
        m_CellSize = cellSize;
    }

    public void Clear()
    {
        foreach (KeyValuePair<int, List<T>> pair in m_Buckets)
        {
            pair.Value.Clear();
        }
    }

    public void Insert(T item, Vector2 position)
    {
        int key = GetKey(position);
        if (!m_Buckets.TryGetValue(key, out List<T> bucket))
        {
            bucket = new List<T>(16);
            m_Buckets[key] = bucket;
        }
        bucket.Add(item);
    }

    // Returns items in the cell containing position and all 8 neighbors
    public List<T> Query(Vector2 position)
    {
        m_QueryResult.Clear();

        int cellX = Mathf.FloorToInt(position.x / m_CellSize);
        int cellY = Mathf.FloorToInt(position.y / m_CellSize);

        for (int offsetX = -1; offsetX <= 1; offsetX++)
        {
            for (int offsetY = -1; offsetY <= 1; offsetY++)
            {
                int key = HashCell(cellX + offsetX, cellY + offsetY);
                if (m_Buckets.TryGetValue(key, out List<T> bucket))
                {
                    for (int itemIndex = 0; itemIndex < bucket.Count; itemIndex++)
                    {
                        m_QueryResult.Add(bucket[itemIndex]);
                    }
                }
            }
        }

        return m_QueryResult;
    }

    private int GetKey(Vector2 position)
    {
        int cellX = Mathf.FloorToInt(position.x / m_CellSize);
        int cellY = Mathf.FloorToInt(position.y / m_CellSize);
        return HashCell(cellX, cellY);
    }

    private static int HashCell(int x, int y)
    {
        // Simple spatial hash — works well for moderate grid sizes
        return x * 73856093 ^ y * 19349663;
    }
}
```

### Staggered AI Updates

Not every enemy needs to run its full AI tick every frame. Spread updates across frames using a modular frame counter:

```csharp
public sealed class StaggeredUpdateSystem
{
    private readonly int m_StaggerFrames;

    public StaggeredUpdateSystem(int staggerFrames)
    {
        m_StaggerFrames = staggerFrames;
    }

    // Each enemy gets an index. Only tick when (frameCount + index) % stagger == 0
    public bool ShouldTick(int enemyIndex, int frameCount)
    {
        return (frameCount + enemyIndex) % m_StaggerFrames == 0;
    }
}
```

With 60 enemies and `m_StaggerFrames = 4`, only 15 enemies run AI each frame. Multiply `deltaTime` by `m_StaggerFrames` in the AI tick to compensate for the reduced update rate.

### LOD-Based Behavior

Enemies far from the camera or off-screen can use simplified behavior:

| Distance | Behavior |
|----------|----------|
| On-screen, near | Full AI, full animation, full VFX |
| On-screen, far | Full AI, reduced animation (skip blend trees), no VFX |
| Off-screen | Simplified AI (patrol only), disable renderer and animator |
| Very far | Freeze entirely, resume when player approaches |

Check visibility with `Renderer.isVisible` or a manual distance check against the camera viewport.

### Object Pooling for Enemies

Pool enemies by type. When an enemy dies, play the death animation, then return to pool via `SetActive(false)`. Reset all state in an `OnReturnToPool` method before releasing back.

```csharp
public sealed class EnemyPoolManager : IDisposable
{
    private readonly Dictionary<EnemyType, ObjectPool<EnemyView>> m_Pools = new();

    public void RegisterPool(EnemyType type, EnemyView prefab, int initialSize)
    {
        ObjectPool<EnemyView> pool = new ObjectPool<EnemyView>(
            createFunc: () =>
            {
                EnemyView instance = Object.Instantiate(prefab);
                instance.gameObject.SetActive(false);
                return instance;
            },
            actionOnGet: enemy => enemy.gameObject.SetActive(true),
            actionOnRelease: enemy =>
            {
                enemy.ResetState();
                enemy.gameObject.SetActive(false);
            },
            actionOnDestroy: enemy => Object.Destroy(enemy.gameObject),
            defaultCapacity: initialSize,
            maxSize: initialSize * 4
        );

        m_Pools[type] = pool;
    }

    public EnemyView Spawn(EnemyType type, Vector3 position)
    {
        EnemyView enemy = m_Pools[type].Get();
        enemy.transform.position = position;
        return enemy;
    }

    public void Despawn(EnemyType type, EnemyView enemy)
    {
        m_Pools[type].Release(enemy);
    }

    public void Dispose()
    {
        foreach (KeyValuePair<EnemyType, ObjectPool<EnemyView>> pair in m_Pools)
        {
            pair.Value.Dispose();
        }
        m_Pools.Clear();
    }
}
```

---

## Room Design and Encounter Pacing

### Room Types

Structure dungeon or level layouts around distinct room categories:

| Room Type | Purpose | Player Experience |
|-----------|---------|-------------------|
| Combat | Enemy encounters | Tension, skill test |
| Puzzle | Environmental puzzles, switches | Mental break from combat |
| Treasure | Loot, upgrades, shops | Reward, progression |
| Rest | Save point, healing fountain | Relief, safe zone |
| Boss | Major encounter | Climax of current area |

### Spawn Point Placement

Place spawn points along room edges or behind cover, never directly on top of the player entrance. Use a minimum distance from the entry door so the player has time to react.

```csharp
[CreateAssetMenu(menuName = "Game/Room Definition")]
public sealed class RoomDefinition : ScriptableObject
{
    [SerializeField] private RoomType m_RoomType;
    [SerializeField] private int m_MinEnemies;
    [SerializeField] private int m_MaxEnemies;
    [SerializeField] private float m_MinSpawnDistFromEntry = 4f;
    [SerializeField] private EnemyType[] m_AllowedEnemyTypes;
    [SerializeField] private int m_DifficultyTier;
}
```

### Difficulty Scaling Per Room Depth

Scale enemy count and stats based on how deep the player is in the dungeon:

```csharp
public sealed class DifficultyScaler
{
    private readonly float m_HealthScalePerDepth;
    private readonly float m_DamageScalePerDepth;
    private readonly float m_CountScalePerDepth;

    public DifficultyScaler(float healthScale, float damageScale, float countScale)
    {
        m_HealthScalePerDepth = healthScale;
        m_DamageScalePerDepth = damageScale;
        m_CountScalePerDepth = countScale;
    }

    public int ScaleEnemyCount(int baseCount, int roomDepth)
    {
        return Mathf.RoundToInt(baseCount * (1f + m_CountScalePerDepth * roomDepth));
    }

    public int ScaleHealth(int baseHealth, int roomDepth)
    {
        return Mathf.RoundToInt(baseHealth * (1f + m_HealthScalePerDepth * roomDepth));
    }

    public int ScaleDamage(int baseDamage, int roomDepth)
    {
        return Mathf.RoundToInt(baseDamage * (1f + m_DamageScalePerDepth * roomDepth));
    }
}
```

### Lock-on-Enter, Unlock-on-Clear

When the player enters a combat room, lock all doors and start spawning. Unlock when all enemies are dead.

```csharp
public sealed class RoomEncounterSystem : IDisposable
{
    private readonly IPublisher<RoomLockedMessage> m_LockPublisher;
    private readonly IPublisher<RoomClearedMessage> m_ClearPublisher;
    private int m_EnemiesRemaining;
    private bool m_IsActive;

    [Inject]
    public RoomEncounterSystem(
        IPublisher<RoomLockedMessage> lockPublisher,
        IPublisher<RoomClearedMessage> clearPublisher,
        ISubscriber<EnemyDiedMessage> enemyDiedSubscriber)
    {
        m_LockPublisher = lockPublisher;
        m_ClearPublisher = clearPublisher;
        enemyDiedSubscriber.Subscribe(OnEnemyDied);
    }

    public void StartEncounter(int enemyCount)
    {
        m_EnemiesRemaining = enemyCount;
        m_IsActive = true;
        m_LockPublisher.Publish(new RoomLockedMessage());
    }

    private void OnEnemyDied(EnemyDiedMessage message)
    {
        if (!m_IsActive) return;

        m_EnemiesRemaining--;
        if (m_EnemiesRemaining <= 0)
        {
            m_IsActive = false;
            m_ClearPublisher.Publish(new RoomClearedMessage());
        }
    }

    public void Dispose() { }
}

public readonly struct RoomLockedMessage { }
public readonly struct RoomClearedMessage { }
public readonly struct EnemyDiedMessage
{
    public readonly Vector3 Position;
    public EnemyDiedMessage(Vector3 position) { Position = position; }
}
```

---

## Common Pitfalls

### NavMesh Agent Jitter on Narrow Corridors

When a corridor is barely wider than the agent radius, the agent oscillates between left and right walls. Fix by setting `NavMeshAgent.radius` slightly smaller than half the corridor width, and increase `NavMeshAgent.obstacleAvoidanceType` to `HighQualityObstacleAvoidance`. For very narrow passages, temporarily disable avoidance:

```csharp
// Disable avoidance in tight corridors
context.Agent.obstacleAvoidanceType = ObstacleAvoidanceType.NoObstacleAvoidance;
// Re-enable when back in open space
context.Agent.obstacleAvoidanceType = ObstacleAvoidanceType.HighQualityObstacleAvoidance;
```

### Z-Sorting Issues with Top-Down Sprites

In a top-down 2D game, sprites with the same sorting layer overlap incorrectly. Set the camera's `TransparencySort.mode` to **Custom Axis** with `(0, 1, 0)` so sprites lower on screen render in front:

```csharp
// In a bootstrap or camera setup script
Camera.main.transparencySortMode = TransparencySortMode.CustomAxis;
Camera.main.transparencySortAxis = new Vector3(0f, 1f, 0f);
```

Alternatively, set each `SpriteRenderer.sortingOrder` dynamically based on Y position. The camera approach is cheaper and automatic.

### Camera Confiner Edge Snapping

When the room is smaller than the camera viewport, `CinemachineConfiner2D` forces the camera to a corner, creating jarring snaps. Fix by ensuring confiner bounds are always at least as large as the camera viewport, or use `Damping` on the confiner extension to smooth the transition.

For rooms smaller than the viewport, center the camera on the room center and disable confining for that room.

### Projectiles Passing Through Thin Colliders

Fast projectiles (high speed, small collider) pass through walls in a single frame. Solutions:

1. **Continuous Collision Detection**: Set `Rigidbody2D.collisionDetectionMode` to `Continuous` on the projectile. Costs more but catches tunneling.
2. **Raycast ahead**: Before moving, raycast from current position to next position. If a hit is detected, place the projectile at the hit point and trigger the collision.
3. **Thicker colliders**: Make wall colliders wider than the maximum distance a projectile moves per frame.

The raycast approach is most reliable for very fast projectiles:

```csharp
private void UpdateProjectile(float deltaTime)
{
    float moveDistance = m_Speed * deltaTime;
    RaycastHit2D hit = Physics2D.Raycast(
        transform.position, m_Direction, moveDistance, m_CollisionMask);

    if (hit.collider != null)
    {
        transform.position = hit.point;
        OnHit(hit);
    }
    else
    {
        transform.position += (Vector3)(m_Direction * moveDistance);
    }
}
```

### Aim Direction Jitter at Low Framerates

When converting mouse position to world aim direction, small mouse movements at low FPS cause the aim direction to flicker. Smooth the aim direction with interpolation:

```csharp
private Vector2 m_SmoothedAim;
private static readonly float k_AimSmoothing = 15f;

private Vector2 SmoothAim(Vector2 rawAim, float deltaTime)
{
    m_SmoothedAim = Vector2.Lerp(m_SmoothedAim, rawAim, k_AimSmoothing * deltaTime);
    return m_SmoothedAim.normalized;
}
```

Only apply smoothing to mouse aim, not gamepad stick aim, since stick input is already analog and smoothing adds perceptible input lag.
