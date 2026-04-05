# Performance Rules

## The Golden Rule

**Zero heap allocations in Update, FixedUpdate, and LateUpdate.**

Every allocation triggers GC, which causes frame spikes. Profile with Unity Profiler's GC Alloc column.

## Cache Everything

```csharp
// BAD — FindObjectOfType every frame
private void Update()
{
    Camera.main.WorldToScreenPoint(transform.position); // Camera.main calls FindObjectOfType
    GetComponent<Rigidbody>().AddForce(Vector3.up);
}

// GOOD — cache in Awake
private Camera m_MainCamera;
private Rigidbody m_Rigidbody;

private void Awake()
{
    m_MainCamera = Camera.main;
    m_Rigidbody = GetComponent<Rigidbody>();
}
```

Cache these in Awake — NEVER call in Update:
- `GetComponent<T>()` / `TryGetComponent<T>()`
- `Camera.main` (does FindObjectOfType internally)
- `transform` / `gameObject` (minor but adds up in hot loops)
- `Animator.StringToHash()` / `Shader.PropertyToID()` → `static readonly int`

## Avoid Allocations

| Allocates | Use Instead |
|-----------|------------|
| `new List<T>()` in Update | Pre-allocate, reuse with `.Clear()` |
| `new WaitForSeconds(n)` | Cache as field: `WaitForSeconds m_Wait = new(0.5f)` |
| `string + string` | `StringBuilder` or `string.Format` |
| `foreach` on non-List | `for` loop with index |
| `LINQ` (`.Where`, `.Select`, `.Any`) | Manual loops |
| `FindObjectOfType` | Cached reference or SO runtime set |
| `tag == "tag"` | `CompareTag("tag")` |
| `SendMessage` / `BroadcastMessage` | Direct reference or events |
| `Physics.RaycastAll` | `Physics.RaycastNonAlloc` with pre-allocated array |

## Physics

- Use non-allocating variants: `OverlapSphereNonAlloc`, `RaycastNonAlloc`, `SphereCastNonAlloc`
- Pre-allocate result arrays: `private RaycastHit[] m_HitBuffer = new RaycastHit[16]`
- Physics queries in `FixedUpdate`, not `Update`

## Object Lifecycle

- Pool frequently instantiated objects — `ObjectPool<T>` or custom pool
- `SetActive(false)` to return to pool, not `Destroy`
- `DontDestroyOnLoad` sparingly — prefer bootstrapper scene

## Debug

- No `Debug.Log` in production — use `[Conditional("UNITY_EDITOR")]` wrapper
- Strip debug code with scripting defines, not runtime checks
