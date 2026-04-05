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
