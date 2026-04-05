---
name: navmesh
description: "Unity navigation — NavMeshAgent configuration, NavMeshSurface, off-mesh links, dynamic obstacles, pathfinding patterns."
globs: ["**/*Nav*.cs", "**/*Pathfind*.cs", "**/*Agent*.cs"]
---

# NavMesh Navigation

## Setup

1. Add `NavMeshSurface` component to environment parent object
2. Click "Bake" to generate NavMesh
3. Add `NavMeshAgent` to moving characters

## NavMeshAgent Configuration

```csharp
[SerializeField] private NavMeshAgent m_Agent;

private void Awake()
{
    m_Agent = GetComponent<NavMeshAgent>();
    m_Agent.speed = 3.5f;
    m_Agent.acceleration = 8f;
    m_Agent.angularSpeed = 120f;
    m_Agent.stoppingDistance = 0.5f;
    m_Agent.autoBraking = true;
}

public void MoveTo(Vector3 destination)
{
    m_Agent.SetDestination(destination);
}
```

## Path Status Checking

```csharp
private void Update()
{
    if (m_Agent.pathPending) return; // Still calculating

    switch (m_Agent.pathStatus)
    {
        case NavMeshPathStatus.PathComplete:
            // Full path found
            break;
        case NavMeshPathStatus.PathPartial:
            // Can only get partway — obstacle or unreachable
            break;
        case NavMeshPathStatus.PathInvalid:
            // No path possible
            break;
    }

    // Check if arrived
    if (!m_Agent.pathPending && m_Agent.remainingDistance <= m_Agent.stoppingDistance)
    {
        // Arrived at destination
    }
}
```

## Patrol Pattern

```csharp
public sealed class PatrolBehavior : MonoBehaviour
{
    [SerializeField] private Transform[] m_Waypoints;
    [SerializeField] private float m_WaitTime = 2f;

    private NavMeshAgent m_Agent;
    private int m_CurrentWaypoint;
    private float m_WaitTimer;

    private void Update()
    {
        if (m_Agent.pathPending) return;

        if (m_Agent.remainingDistance <= m_Agent.stoppingDistance)
        {
            m_WaitTimer -= Time.deltaTime;
            if (m_WaitTimer <= 0f)
            {
                m_CurrentWaypoint = (m_CurrentWaypoint + 1) % m_Waypoints.Length;
                m_Agent.SetDestination(m_Waypoints[m_CurrentWaypoint].position);
                m_WaitTimer = m_WaitTime;
            }
        }
    }
}
```

## NavMeshObstacle

- **Carve:** cuts a hole in the NavMesh (expensive, use for static/rare movement)
- **Block:** agents path around without modifying NavMesh (cheaper, use for moving obstacles)

## Off-Mesh Links

For jumps, ladders, teleporters — connections between disconnected NavMesh areas.
- Auto-generated: set Jump Distance and Drop Height on NavMeshSurface
- Manual: `NavMeshLink` component between two points

## Runtime NavMesh Modification

```csharp
// Rebake at runtime (e.g., after terrain change)
m_NavMeshSurface.BuildNavMesh();

// Or update only:
m_NavMeshSurface.UpdateNavMesh(m_NavMeshSurface.navMeshData);
```

## Areas and Costs

- Define areas: Walkable, Water, Road (in Navigation settings)
- Set area cost: higher cost = agents avoid that area
- Override per-agent: `m_Agent.SetAreaCost(areaIndex, cost)`
- Use for: roads (low cost = preferred), mud (high cost = avoided)

## Advanced Path Queries

### NavMesh.Raycast for Line-of-Sight

`NavMesh.Raycast` tests whether a straight line between two points on the NavMesh is unobstructed. This is not a physics raycast — it tests NavMesh connectivity only.

```csharp
using UnityEngine;
using UnityEngine.AI;

public sealed class NavMeshLineOfSight
{
    // Returns true if there is a clear NavMesh path in a straight line
    public bool HasDirectPath(Vector3 from, Vector3 to)
    {
        NavMeshHit hit;
        bool isBlocked = NavMesh.Raycast(from, to, out hit, NavMesh.AllAreas);
        return !isBlocked;
    }

    // Returns the point where the NavMesh line is broken
    public Vector3 GetBlockedPoint(Vector3 from, Vector3 to)
    {
        NavMeshHit hit;
        NavMesh.Raycast(from, to, out hit, NavMesh.AllAreas);
        return hit.position;
    }
}
```

Use cases: checking if an enemy can charge in a straight line, validating shortcut paths, determining if a flee direction is open.

### NavMesh.SamplePosition for Nearest Valid Point

When a position might be off the NavMesh (click on a wall, spawning at an arbitrary point), snap it to the nearest valid location.

```csharp
public static bool TryGetNearestNavMeshPoint(Vector3 source, float maxDistance, out Vector3 result)
{
    NavMeshHit hit;
    if (NavMesh.SamplePosition(source, out hit, maxDistance, NavMesh.AllAreas))
    {
        result = hit.position;
        return true;
    }
    result = source;
    return false;
}
```

Always call `SamplePosition` before `SetDestination` when the target comes from player input or external data. A destination off the NavMesh causes `SetDestination` to fail silently.

### Manual Path Evaluation

Calculate a path without committing the agent to it. Useful for AI decision-making.

```csharp
public sealed class PathEvaluator
{
    private readonly NavMeshPath m_Path = new NavMeshPath();

    // Evaluate path length without moving the agent
    public float GetPathLength(Vector3 from, Vector3 to)
    {
        if (!NavMesh.CalculatePath(from, to, NavMesh.AllAreas, m_Path))
        {
            return float.MaxValue;
        }

        if (m_Path.status != NavMeshPathStatus.PathComplete)
        {
            return float.MaxValue;
        }

        float totalDistance = 0f;
        Vector3[] corners = m_Path.corners;
        for (int cornerIndex = 1; cornerIndex < corners.Length; cornerIndex++)
        {
            totalDistance += Vector3.Distance(corners[cornerIndex - 1], corners[cornerIndex]);
        }

        return totalDistance;
    }

    // Check if destination is reachable before committing
    public bool IsReachable(Vector3 from, Vector3 to)
    {
        NavMesh.CalculatePath(from, to, NavMesh.AllAreas, m_Path);
        return m_Path.status == NavMeshPathStatus.PathComplete;
    }
}
```

Reuse the `NavMeshPath` instance to avoid allocations. Calling `new NavMeshPath()` once and reusing it is the correct pattern.

## Formation Movement

### Formation Patterns

Formations define offset positions relative to a leader. Each member is assigned a slot index.

```csharp
using UnityEngine;

// Pure C# formation calculator — no MonoBehaviour, no Unity API beyond Vector3
public sealed class FormationCalculator
{
    public enum FormationType
    {
        Line,
        Triangle,
        Circle,
        Wedge
    }

    private const float k_DefaultSpacing = 2f;

    // Returns local offsets for each slot (relative to leader facing forward along Z)
    public Vector3[] CalculateSlotOffsets(FormationType type, int memberCount, float spacing = k_DefaultSpacing)
    {
        return type switch
        {
            FormationType.Line => CalculateLineOffsets(memberCount, spacing),
            FormationType.Triangle => CalculateTriangleOffsets(memberCount, spacing),
            FormationType.Circle => CalculateCircleOffsets(memberCount, spacing),
            FormationType.Wedge => CalculateWedgeOffsets(memberCount, spacing),
            _ => CalculateLineOffsets(memberCount, spacing)
        };
    }

    private Vector3[] CalculateLineOffsets(int count, float spacing)
    {
        var offsets = new Vector3[count];
        float startX = -(count - 1) * spacing * 0.5f;
        for (int slotIndex = 0; slotIndex < count; slotIndex++)
        {
            offsets[slotIndex] = new Vector3(startX + slotIndex * spacing, 0f, 0f);
        }
        return offsets;
    }

    private Vector3[] CalculateTriangleOffsets(int count, float spacing)
    {
        var offsets = new Vector3[count];
        int row = 0;
        int placed = 0;
        while (placed < count)
        {
            int columnsInRow = row + 1;
            float rowStartX = -(columnsInRow - 1) * spacing * 0.5f;
            for (int col = 0; col < columnsInRow && placed < count; col++)
            {
                offsets[placed] = new Vector3(rowStartX + col * spacing, 0f, -row * spacing);
                placed++;
            }
            row++;
        }
        return offsets;
    }

    private Vector3[] CalculateCircleOffsets(int count, float spacing)
    {
        var offsets = new Vector3[count];
        float radius = count * spacing / (2f * Mathf.PI);
        radius = Mathf.Max(radius, spacing);
        for (int slotIndex = 0; slotIndex < count; slotIndex++)
        {
            float angle = slotIndex * (2f * Mathf.PI / count);
            offsets[slotIndex] = new Vector3(Mathf.Sin(angle) * radius, 0f, Mathf.Cos(angle) * radius);
        }
        return offsets;
    }

    private Vector3[] CalculateWedgeOffsets(int count, float spacing)
    {
        var offsets = new Vector3[count];
        offsets[0] = Vector3.zero; // Leader at front
        for (int slotIndex = 1; slotIndex < count; slotIndex++)
        {
            int row = (slotIndex + 1) / 2;
            float side = (slotIndex % 2 == 1) ? -1f : 1f;
            offsets[slotIndex] = new Vector3(side * row * spacing, 0f, -row * spacing);
        }
        return offsets;
    }
}
```

### Leader-Follower with Offset Calculation

```csharp
// Convert local formation offset to world position based on leader transform
public static Vector3 GetWorldSlotPosition(Vector3 leaderPosition, Vector3 leaderForward, Vector3 localOffset)
{
    Quaternion rotation = Quaternion.LookRotation(leaderForward, Vector3.up);
    return leaderPosition + rotation * localOffset;
}
```

Each follower agent calls `SetDestination` to its world slot position. Update slot positions only when the leader moves beyond a threshold distance (e.g., 1 unit) to avoid constant repathing.

### Dynamic Formation Resize

When a member dies or joins, recalculate offsets for the new count and reassign slots. Prefer shifting surviving members to the closest available slot rather than reassigning all slots to minimize path changes.

## Dynamic Obstacles

### Carve vs Block Trade-Offs

| Feature | Carve (NavMeshObstacle.carving = true) | Block (carving = false) |
|---------|----------------------------------------|-------------------------|
| NavMesh modified | Yes — cuts a hole at runtime | No — agents steer around via avoidance |
| Performance cost | High — triggers local NavMesh rebuild | Low — avoidance only |
| Path accuracy | Perfect — paths go around the carved hole | Approximate — agents may clip through |
| Use case | Doors, placed buildings, barricades | Moving NPCs, rolling boulders |

### Carve Update Timing

On `NavMeshObstacle` with carving enabled:
- `Carve Only Stationary`: enable this for obstacles that move then stop (placed turrets, furniture)
- `Move Threshold`: minimum distance the obstacle must move before re-carving (default 0.1, increase for less frequent updates)
- `Time To Stationary`: seconds of no movement before the obstacle is considered stationary and carving triggers (default 0.5)

Keep the total number of carving obstacles low (under 20 actively moving). Each carve triggers a local NavMesh rebuild.

### Runtime Placement Validation

Before placing a dynamic obstacle, verify the position is on the NavMesh:

```csharp
public bool CanPlaceObstacle(Vector3 position, float checkRadius)
{
    NavMeshHit hit;
    return NavMesh.SamplePosition(position, out hit, checkRadius, NavMesh.AllAreas);
}
```

### Door and Gate Pattern

```csharp
public sealed class NavMeshDoor : MonoBehaviour
{
    [SerializeField] private NavMeshObstacle m_Obstacle;

    private void Awake()
    {
        m_Obstacle = GetComponent<NavMeshObstacle>();
        m_Obstacle.carving = true;
    }

    public void Open()
    {
        m_Obstacle.carving = false; // Remove the carved hole, agents can path through
    }

    public void Close()
    {
        m_Obstacle.carving = true; // Re-carve, agents path around
    }
}
```

Toggle `carving` rather than enabling/disabling the obstacle component. Disabling the component removes avoidance entirely.

## Repath Strategies

### Repath on Destination Unreachable

When `pathStatus` becomes `PathInvalid` or `PathPartial`, the agent needs a new destination. Do not retry the same unreachable destination every frame.

```csharp
private void HandleUnreachableDestination()
{
    if (m_Agent.pathStatus == NavMeshPathStatus.PathInvalid)
    {
        // Find nearest reachable point to the original target
        NavMeshHit hit;
        if (NavMesh.SamplePosition(m_TargetPosition, out hit, 10f, NavMesh.AllAreas))
        {
            m_Agent.SetDestination(hit.position);
        }
    }
}
```

### Periodic Repath for Moving Targets

When chasing a moving target, do not call `SetDestination` every frame. Use a timer or distance threshold.

```csharp
private const float k_RepathInterval = 0.3f; // Repath every 0.3 seconds
private float m_RepathTimer;

private void Update()
{
    m_RepathTimer -= Time.deltaTime;
    if (m_RepathTimer <= 0f)
    {
        m_RepathTimer = k_RepathInterval;
        m_Agent.SetDestination(m_Target.position);
    }
}
```

For 50+ agents chasing moving targets, stagger repath timers so they do not all repath on the same frame. Initialize `m_RepathTimer` to `Random.Range(0f, k_RepathInterval)`.

### Partial Path Acceptance

Sometimes getting close is good enough (fleeing enemies, area denial). Accept partial paths and move to the closest reachable point:

```csharp
if (m_Agent.pathStatus == NavMeshPathStatus.PathPartial)
{
    // Agent will move to the end of the partial path automatically
    // Decide: is partial good enough, or pick a different destination?
}
```

## Agent Pool Pattern

### Pooling NavMeshAgent Components

NavMeshAgent components should not be destroyed and recreated. Pool the entire GameObject.

```csharp
public sealed class NavAgentPoolHelper : MonoBehaviour
{
    [SerializeField] private NavMeshAgent m_Agent;

    public void OnGetFromPool(Vector3 spawnPosition)
    {
        // Disable agent before moving to prevent path recalculation during warp
        m_Agent.enabled = false;
        transform.position = spawnPosition;
        m_Agent.enabled = true;

        // Warp to ensure agent is properly placed on NavMesh
        m_Agent.Warp(spawnPosition);
    }

    public void OnReturnToPool()
    {
        m_Agent.ResetPath();
        m_Agent.enabled = false;
        gameObject.SetActive(false);
    }
}
```

### Enable vs Disable Navigation

- `m_Agent.enabled = false`: disables pathfinding and movement but keeps the component. Agent is removed from NavMesh simulation.
- `m_Agent.isStopped = true`: agent stays on NavMesh and blocks other agents but does not move. Use for paused/stunned enemies.
- `gameObject.SetActive(false)`: fully removes from all systems. Use for pooled objects.

### Warp Pattern for Teleportation

Never set `transform.position` directly on an object with an active NavMeshAgent. The agent will try to correct the position back to the NavMesh surface it was on.

```csharp
// Correct teleportation sequence
m_Agent.enabled = false;
transform.position = newPosition;
m_Agent.enabled = true;
m_Agent.Warp(newPosition);
```

### Agent Reset on Pool Return

Before returning an agent to the pool, always:
1. Call `m_Agent.ResetPath()` to clear the current path
2. Set `m_Agent.enabled = false` to remove from NavMesh simulation
3. Deactivate the GameObject

On retrieval, re-enable and warp before setting a new destination.

## Multi-Agent Coordination

### Avoidance Priority

`NavMeshAgent.avoidancePriority` ranges from 0 (highest priority, others avoid it) to 99 (lowest priority, avoids everyone).

```
Priority 0-10:  Bosses, VIP NPCs (never pushed aside)
Priority 20-40: Player companions (mostly hold formation)
Priority 50-70: Regular enemies (standard avoidance)
Priority 80-99: Minions, swarm units (easily pushed aside)
```

Higher-priority agents do not steer around lower-priority agents. Use this to prevent important NPCs from being pushed off their path by crowds.

### Separation Steering to Prevent Stacking

NavMesh avoidance alone does not prevent agents from overlapping at their destination. Add a separation pass:

```csharp
// Run after NavMeshAgent updates, in LateUpdate or a system tick
public static Vector3 CalculateSeparation(
    Vector3 agentPosition,
    Vector3[] neighborPositions,
    int neighborCount,
    float desiredSeparation)
{
    Vector3 separationForce = Vector3.zero;
    for (int neighborIndex = 0; neighborIndex < neighborCount; neighborIndex++)
    {
        Vector3 offset = agentPosition - neighborPositions[neighborIndex];
        float distance = offset.magnitude;
        if (distance > 0f && distance < desiredSeparation)
        {
            separationForce += offset.normalized * (desiredSeparation - distance);
        }
    }
    return separationForce;
}
```

Apply the separation force as a velocity offset or adjust the agent's destination by a small amount. Do not fight the NavMeshAgent's own steering — nudge, do not override.

### Crowd Management

When many agents share a destination (e.g., all enemies rushing a chokepoint):
- Stagger destination assignment: spread targets around the destination in a ring pattern
- Use formation slots so agents aim for unique positions instead of the exact same point
- Limit simultaneous active agents: keep a max of 30 to 50 actively pathing agents, queue the rest

### Performance Budgeting

| Agent Count | Recommendation |
|-------------|---------------|
| 1-20 | No special handling needed |
| 20-50 | Stagger repath timers, use avoidance priority |
| 50-100 | Reduce repath frequency, disable avoidance on distant agents |
| 100+ | Use simple steering for distant agents, only NavMeshAgent for nearby |

For agents far from the camera, consider disabling `NavMeshAgent` and using simple `Vector3.MoveTowards` until they enter a relevant range.

## Common Pitfalls

### Agent Stuck on NavMeshLink

When an agent traverses an off-mesh link, `m_Agent.isOnOffMeshLink` becomes true. The agent will not auto-complete the traversal unless you handle it:

```csharp
private void Update()
{
    if (m_Agent.isOnOffMeshLink)
    {
        // Complete the link traversal
        m_Agent.CompleteOffMeshLink();
    }
}
```

For animated traversals (jumping, climbing), use `m_Agent.autoTraverseOffMeshLink = false` and manually move the agent across the link with UniTask, then call `CompleteOffMeshLink()`.

### Bake Settings Mismatch

The NavMesh is baked with a specific agent radius and height. If the NavMeshAgent component's radius differs from the bake settings, the agent may:
- Clip through narrow passages the NavMesh allows
- Be unable to pass through openings that look wide enough
- Float above or sink below the NavMesh surface

Always match NavMeshAgent radius/height to the NavMeshSurface agent settings. For different agent sizes (small minion vs large boss), bake multiple NavMesh surfaces with different agent type settings.

### SetDestination Before Agent Is on NavMesh

`SetDestination` fails silently if the agent is not on the NavMesh. This happens when:
- The agent was just spawned at a position off the NavMesh
- The agent was teleported without using `Warp`
- The NavMesh was not yet baked

Always verify placement:

```csharp
private bool IsAgentOnNavMesh()
{
    NavMeshHit hit;
    return NavMesh.SamplePosition(m_Agent.transform.position, out hit, 0.5f, NavMesh.AllAreas)
        && m_Agent.isOnNavMesh;
}
```

### Destroy vs Disable for Pooling

- Calling `Destroy` on a NavMeshAgent component and re-adding it later is expensive and error-prone
- Always disable the agent (`m_Agent.enabled = false`) and deactivate the GameObject for pooling
- Re-enable and warp on retrieval
- Never call `SetDestination` on a disabled agent — it will throw or fail silently depending on Unity version

### Agent updatePosition and updateRotation

By default, NavMeshAgent controls both position and rotation. If you also apply forces via Rigidbody or animate root motion:
- Set `m_Agent.updatePosition = false` and sync manually via `m_Agent.nextPosition`
- Set `m_Agent.updateRotation = false` if you handle rotation through animation or custom code
- Failing to sync causes the agent's internal position to desync from the transform, leading to teleporting or jittering
