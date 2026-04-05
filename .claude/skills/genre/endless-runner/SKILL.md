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

- **Distance score:** increases with time × speed
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
