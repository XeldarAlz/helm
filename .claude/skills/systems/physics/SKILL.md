---
name: physics
description: "Unity physics — non-allocating queries, collision layers, FixedUpdate discipline, continuous collision detection, character controllers, joints."
globs: ["**/*Physics*.cs", "**/*Collider*.cs", "**/*Rigidbody*.cs", "**/*Trigger*.cs"]
---

# Physics System

## FixedUpdate Discipline

All physics code goes in `FixedUpdate`. All input reading goes in `Update`.

```csharp
private Vector2 m_MoveInput;

private void Update()
{
    m_MoveInput = new Vector2(Input.GetAxisRaw("Horizontal"), Input.GetAxisRaw("Vertical"));
}

private void FixedUpdate()
{
    m_Rigidbody.AddForce(m_MoveInput * m_Force);
}
```

## Non-Allocating Queries

```csharp
// Pre-allocate buffers
private static readonly RaycastHit[] s_HitBuffer = new RaycastHit[16];
private static readonly Collider[] s_OverlapBuffer = new Collider[32];

// Raycast
int hitCount = Physics.RaycastNonAlloc(origin, direction, s_HitBuffer, maxDistance, layerMask);
for (int i = 0; i < hitCount; i++)
{
    RaycastHit hit = s_HitBuffer[i];
    // Process hit
}

// Overlap sphere (area detection)
int overlapCount = Physics.OverlapSphereNonAlloc(center, radius, s_OverlapBuffer, layerMask);

// Sphere cast (fat raycast)
int castCount = Physics.SphereCastNonAlloc(origin, radius, direction, s_HitBuffer, maxDistance, layerMask);
```

## Layer Collision Matrix

```csharp
// Ignore collisions between layers programmatically
Physics.IgnoreLayerCollision(playerLayer, pickupLayer, true);

// Or configure in Edit > Project Settings > Physics > Layer Collision Matrix
```

Layer organization:
```
6: Player
7: Ground
8: Enemy
9: Projectile
10: Trigger (no physics collision, triggers only)
11: Interactable
```

## Collision Detection Modes

| Mode | Use When |
|------|----------|
| Discrete | Slow objects (default) |
| Continuous | Fast objects that might tunnel through thin colliders |
| Continuous Dynamic | Fast objects colliding with other fast objects |
| Continuous Speculative | Good balance of accuracy and performance |

## Collision vs Trigger Callbacks

```csharp
// Collision (both have colliders, at least one has Rigidbody, neither is trigger)
private void OnCollisionEnter(Collision collision) { }
private void OnCollisionStay(Collision collision) { }
private void OnCollisionExit(Collision collision) { }

// Trigger (at least one collider has isTrigger = true)
private void OnTriggerEnter(Collider other) { }
private void OnTriggerStay(Collider other) { }
private void OnTriggerExit(Collider other) { }
```

## Physics.SyncTransforms

After moving a transform directly, physics queries won't reflect the new position until the next physics step. Force sync:
```csharp
transform.position = newPosition;
Physics.SyncTransforms(); // Now raycasts see the new position
```

## Rigidbody Configuration

- **Interpolation:** `Interpolate` for player (smooths between physics steps), `None` for others
- **Constraints:** Freeze rotation for 2D-like behavior in 3D
- **Collision Detection:** Continuous for fast-moving objects

## 2D Physics Equivalents

| 3D | 2D |
|----|-----|
| `Rigidbody` | `Rigidbody2D` |
| `BoxCollider` | `BoxCollider2D` |
| `Physics.Raycast` | `Physics2D.Raycast` |
| `Physics.OverlapSphereNonAlloc` | `Physics2D.OverlapCircleNonAlloc` |
| `OnCollisionEnter(Collision)` | `OnCollisionEnter2D(Collision2D)` |
| `OnTriggerEnter(Collider)` | `OnTriggerEnter2D(Collider2D)` |

## Joints

| Joint | Use |
|-------|-----|
| Fixed | Weld objects together |
| Hinge | Doors, wheels |
| Spring | Bouncy connections |
| Configurable | Full control over all axes |
| Character | Character controller with physics |

## Advanced Query Patterns

### CapsuleCast for Character-Shaped Tests

CapsuleCast matches the shape of a character controller. Use it for movement prediction
and line-of-sight checks where a sphere is too wide or too narrow.

```csharp
private static readonly RaycastHit[] s_CapsuleHitBuffer = new RaycastHit[8];

// point1 = bottom sphere center, point2 = top sphere center
private int CapsuleSweep(Vector3 origin, Vector3 direction, float distance)
{
    Vector3 point1 = origin + Vector3.up * m_CapsuleRadius;
    Vector3 point2 = origin + Vector3.up * (m_CapsuleHeight - m_CapsuleRadius);
    return Physics.CapsuleCastNonAlloc(
        point1, point2, m_CapsuleRadius, direction,
        s_CapsuleHitBuffer, distance, m_ObstacleMask);
}
```

### QueryTriggerInteraction

Control whether queries hit triggers on a per-call basis:

```csharp
// Ignore triggers — only hit solid colliders
Physics.RaycastNonAlloc(origin, direction, s_HitBuffer, maxDistance,
    layerMask, QueryTriggerInteraction.Ignore);

// Only collide with triggers
Physics.OverlapSphereNonAlloc(center, radius, s_OverlapBuffer,
    triggerLayerMask, QueryTriggerInteraction.Collide);

// Use global Physics settings (default)
Physics.RaycastNonAlloc(origin, direction, s_HitBuffer, maxDistance,
    layerMask, QueryTriggerInteraction.UseGlobal);
```

### BoxCast for Wide Area Checks

BoxCast is ideal for wide melee attacks, door-frame clearance checks, and vehicle collision:

```csharp
private static readonly RaycastHit[] s_BoxHitBuffer = new RaycastHit[16];

// halfExtents defines the box size (half-width, half-height, half-depth)
int hitCount = Physics.BoxCastNonAlloc(
    center, halfExtents, direction, s_BoxHitBuffer,
    orientation, maxDistance, layerMask);
```

### Layer + Distance Combined Filtering

When a NonAlloc query returns multiple hits, sort by distance and filter by layer:

```csharp
int hitCount = Physics.RaycastNonAlloc(origin, direction, s_HitBuffer, maxDistance, layerMask);
// Sort the valid portion of the buffer by distance (ascending)
System.Array.Sort(s_HitBuffer, 0, hitCount, s_DistanceComparer);

// s_DistanceComparer is a cached IComparer<RaycastHit>
private sealed class HitDistanceComparer : IComparer<RaycastHit>
{
    public int Compare(RaycastHit a, RaycastHit b) => a.distance.CompareTo(b.distance);
}
private static readonly HitDistanceComparer s_DistanceComparer = new();
```

## Collision Response

### ContactPoint Extraction

Access individual contact points from a collision to determine impact location,
surface normal, and separation distance:

```csharp
private readonly ContactPoint[] m_ContactBuffer = new ContactPoint[8];

private void OnCollisionEnter(Collision collision)
{
    int contactCount = collision.GetContacts(m_ContactBuffer);
    for (int contactIndex = 0; contactIndex < contactCount; contactIndex++)
    {
        ContactPoint contact = m_ContactBuffer[contactIndex];
        Vector3 point = contact.point;
        Vector3 normal = contact.normal;
        float separation = contact.separation;
        // Spawn spark VFX at contact.point facing contact.normal
    }
}
```

### Impact Force Calculation

Use `relativeVelocity` to scale damage, sound volume, or VFX intensity:

```csharp
private void OnCollisionEnter(Collision collision)
{
    float impactSpeed = collision.relativeVelocity.magnitude;
    if (impactSpeed < m_MinImpactThreshold)
    {
        return;
    }

    float normalizedForce = Mathf.InverseLerp(m_MinImpactThreshold, m_MaxImpactThreshold, impactSpeed);
    // Use normalizedForce (0-1) to scale damage, audio volume, particle count
}
```

### Bounce Physics with PhysicMaterial

Configure PhysicMaterial on colliders to control bounce and friction:

```csharp
// Create PhysicMaterial via code (prefer ScriptableObject asset in practice)
var bouncyMaterial = new PhysicMaterial("Bouncy")
{
    bounciness = 0.8f,
    bounceCombine = PhysicMaterialCombine.Maximum,
    dynamicFriction = 0.2f,
    staticFriction = 0.2f,
    frictionCombine = PhysicMaterialCombine.Average
};
m_Collider.material = bouncyMaterial;
```

### Friction Control

| Combine Mode | Behavior |
|-------------|----------|
| Average | (a + b) / 2 — default, predictable |
| Minimum | min(a, b) — one icy surface makes everything slide |
| Maximum | max(a, b) — one sticky surface grips everything |
| Multiply | a * b — both must be high for strong friction |

Static friction resists initial movement. Dynamic friction resists ongoing movement.
Static should be >= dynamic for realistic behavior.

## Character Physics Pattern

### Ground Detection with SphereCast

SphereCast from the character's feet detects ground with tolerance for uneven surfaces:

```csharp
private static readonly RaycastHit[] s_GroundHitBuffer = new RaycastHit[4];

private bool m_IsGrounded;
private Vector3 m_GroundNormal;

private void CheckGround()
{
    Vector3 sphereOrigin = m_Transform.position + Vector3.up * m_GroundCheckRadius;
    int hitCount = Physics.SphereCastNonAlloc(
        sphereOrigin, m_GroundCheckRadius, Vector3.down,
        s_GroundHitBuffer, m_GroundCheckDistance, m_GroundMask,
        QueryTriggerInteraction.Ignore);

    m_IsGrounded = false;
    for (int hitIndex = 0; hitIndex < hitCount; hitIndex++)
    {
        float angle = Vector3.Angle(s_GroundHitBuffer[hitIndex].normal, Vector3.up);
        if (angle <= m_MaxSlopeAngle)
        {
            m_IsGrounded = true;
            m_GroundNormal = s_GroundHitBuffer[hitIndex].normal;
            return;
        }
    }
}
```

### Slope Angle Calculation and Sliding

Calculate the angle between the surface normal and world up. If above the max slope,
project gravity along the slope to create a sliding force:

```csharp
private void HandleSlope()
{
    float slopeAngle = Vector3.Angle(m_GroundNormal, Vector3.up);
    if (slopeAngle <= m_MaxSlopeAngle)
    {
        return;
    }

    // Project movement onto the slope surface
    Vector3 slopeDirection = Vector3.ProjectOnPlane(Vector3.down, m_GroundNormal).normalized;
    float slideForce = m_Gravity * Mathf.Sin(slopeAngle * Mathf.Deg2Rad);
    m_Rigidbody.AddForce(slopeDirection * slideForce, ForceMode.Acceleration);
}

// Project desired movement onto the slope so the character follows terrain
private Vector3 GetSlopeAdjustedDirection(Vector3 moveDirection)
{
    return Vector3.ProjectOnPlane(moveDirection, m_GroundNormal).normalized;
}
```

### Step Climbing with Raycast Offset

Check if an obstacle is short enough to step over by casting a ray from step height:

```csharp
private bool CanStepOver(Vector3 moveDirection)
{
    // First ray: is there a wall at foot level?
    bool wallAtFeet = Physics.Raycast(
        m_Transform.position + Vector3.up * 0.05f,
        moveDirection, m_StepCheckDistance, m_ObstacleMask);

    if (!wallAtFeet)
    {
        return false;
    }

    // Second ray: is the space clear at step height?
    bool clearAtStepHeight = !Physics.Raycast(
        m_Transform.position + Vector3.up * m_MaxStepHeight,
        moveDirection, m_StepCheckDistance, m_ObstacleMask);

    return clearAtStepHeight;
}
```

## Ragdoll System

### Ragdoll Activation

Disable the Animator and enable all Rigidbody components on the skeleton to switch
from animated to ragdoll state:

```csharp
public sealed class RagdollController : MonoBehaviour
{
    [SerializeField] private Animator m_Animator;

    private Rigidbody[] m_RagdollBodies;
    private Collider[] m_RagdollColliders;

    private void Awake()
    {
        m_RagdollBodies = GetComponentsInChildren<Rigidbody>();
        m_RagdollColliders = GetComponentsInChildren<Collider>();
        SetRagdollActive(false);
    }

    public void SetRagdollActive(bool active)
    {
        for (int bodyIndex = 0; bodyIndex < m_RagdollBodies.Length; bodyIndex++)
        {
            m_RagdollBodies[bodyIndex].isKinematic = !active;
        }
        for (int colliderIndex = 0; colliderIndex < m_RagdollColliders.Length; colliderIndex++)
        {
            m_RagdollColliders[colliderIndex].enabled = active;
        }
        m_Animator.enabled = !active;
    }

    // Apply a death impulse at the hit point
    public void ApplyDeathForce(Vector3 force, Vector3 hitPoint)
    {
        SetRagdollActive(true);
        // Find the closest ragdoll body to the hit point
        Rigidbody closest = null;
        float closestDist = float.MaxValue;
        for (int bodyIndex = 0; bodyIndex < m_RagdollBodies.Length; bodyIndex++)
        {
            float dist = (m_RagdollBodies[bodyIndex].position - hitPoint).sqrMagnitude;
            if (dist < closestDist)
            {
                closestDist = dist;
                closest = m_RagdollBodies[bodyIndex];
            }
        }
        if (closest != null)
        {
            closest.AddForce(force, ForceMode.Impulse);
        }
    }
}
```

### Partial Ragdoll

Enable ragdoll only on specific body parts (e.g. upper body hit reaction)
by marking bones with tags or a serialized list:

```csharp
[SerializeField] private Rigidbody[] m_UpperBodyBones;

public void ActivateUpperBodyRagdoll()
{
    for (int boneIndex = 0; boneIndex < m_UpperBodyBones.Length; boneIndex++)
    {
        m_UpperBodyBones[boneIndex].isKinematic = false;
    }
}
```

### Ragdoll to Get-Up Transition

Record the ragdoll pose, blend to a get-up animation clip:

```csharp
public async UniTask TransitionToGetUp(CancellationToken token)
{
    // Wait for ragdoll to settle
    await UniTask.Delay(TimeSpan.FromSeconds(1.5f), cancellationToken: token);

    // Snapshot the hip position to align the animation
    Vector3 hipPosition = m_RagdollBodies[0].position;
    m_Transform.position = new Vector3(hipPosition.x, m_Transform.position.y, hipPosition.z);

    SetRagdollActive(false);
    m_Animator.Play("GetUp");
}
```

## Physics Performance

### Physics.simulationMode

Unity 6 supports three simulation modes:
- `SimulationMode.FixedUpdate` — default, runs in sync with FixedUpdate
- `SimulationMode.Update` — physics ticks every frame (variable delta)
- `SimulationMode.Script` — manual `Physics.Simulate(dt)` for replay/determinism

Use `Script` mode for lockstep netcode or replay systems:
```csharp
Physics.simulationMode = SimulationMode.Script;
// Then call manually each tick:
Physics.Simulate(Time.fixedDeltaTime);
```

### Auto Sync Transforms

`Physics.autoSyncTransforms` copies Transform data to the physics engine every query.
Disable it and call `Physics.SyncTransforms()` explicitly when needed:
```csharp
Physics.autoSyncTransforms = false; // Set once in bootstrap
```
This avoids redundant syncs when many queries run per frame.

### Compound Colliders vs Mesh Colliders

Prefer compound primitive colliders (box + capsule + sphere) over MeshCollider:
- Primitive colliders: fast broadphase, cheap narrowphase
- MeshCollider convex: required for Rigidbody, max 255 triangles
- MeshCollider non-convex: static only, expensive, use only for terrain/environment

### Sleep Threshold Tuning

Objects at rest enter sleep state and skip simulation. Lower the threshold for
sensitive gameplay (stacking puzzles), raise it for large crowds of physics objects:
```csharp
// Global setting — lower = more accurate sleep, higher = better perf
Physics.sleepThreshold = 0.005f; // default is 0.005
// Per-body override
m_Rigidbody.sleepThreshold = 0.01f;
```

## 2D Physics Specifics

### Rigidbody2D Body Types

| Body Type | Behavior | Use Case |
|-----------|----------|----------|
| Dynamic | Full simulation, responds to forces | Player, enemies, projectiles |
| Kinematic | Moves via script, no forces | Moving platforms, elevators |
| Static | Never moves | Walls, ground, static environment |

Switching body type at runtime is valid but resets velocity:
```csharp
m_Rigidbody2D.bodyType = RigidbodyType2D.Kinematic;
```

### Physics2D Non-Allocating Queries

```csharp
private static readonly RaycastHit2D[] s_Hit2DBuffer = new RaycastHit2D[16];
private static readonly Collider2D[] s_Overlap2DBuffer = new Collider2D[32];

// OverlapCircle (equivalent to OverlapSphere in 3D)
int count = Physics2D.OverlapCircleNonAlloc(center, radius, s_Overlap2DBuffer, layerMask);

// BoxCast
int castCount = Physics2D.BoxCastNonAlloc(origin, size, angle, direction,
    s_Hit2DBuffer, distance, layerMask);

// OverlapBox for area checks
int boxCount = Physics2D.OverlapBoxNonAlloc(center, size, angle, s_Overlap2DBuffer, layerMask);
```

### CompositeCollider2D for Tilemap Collision

Attach `CompositeCollider2D` to the Tilemap's parent GameObject. Individual tile
colliders merge into optimized edge or polygon geometry:

```
Tilemap GameObject
  +-- TilemapCollider2D (check "Used By Composite")
  +-- CompositeCollider2D (Geometry Type: Polygons or Outlines)
  +-- Rigidbody2D (set to Static body type)
```

This reduces collision checks from hundreds of individual tile colliders to a few merged shapes.

### 2D Effectors

Effectors modify physics behavior in a region. Attach to a trigger collider:

| Effector | Behavior | Use Case |
|----------|----------|----------|
| PlatformEffector2D | One-way collision, optional side friction | Jump-through platforms |
| SurfaceEffector2D | Applies tangent speed to contacting bodies | Conveyor belts, moving walkways |
| AreaEffector2D | Applies force in a direction within the area | Wind zones, water currents |
| BuoyancyEffector2D | Simulates buoyancy based on density | Water volumes, lava pools |
| PointEffector2D | Attracts or repels from a point | Magnets, gravity wells, explosions |

```csharp
// One-way platform setup
[RequireComponent(typeof(PlatformEffector2D))]
[RequireComponent(typeof(BoxCollider2D))]
public sealed class OneWayPlatform : MonoBehaviour
{
    private void Awake()
    {
        var effector = GetComponent<PlatformEffector2D>();
        effector.surfaceArc = 170f; // Angle of the "solid" surface arc
        effector.useOneWay = true;

        var boxCollider = GetComponent<BoxCollider2D>();
        boxCollider.usedByEffector = true;
    }
}
```
