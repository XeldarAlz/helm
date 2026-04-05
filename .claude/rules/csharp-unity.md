# C# Style — Unity Conventions

## Field Declarations

- `[SerializeField] private` for inspector-exposed fields — never public
- `private` fields use `m_` prefix: `m_MoveSpeed`, `m_Health`
- `static` fields use `s_` prefix: `s_Instance`
- `const` and `static readonly` use `k_` prefix: `k_MaxHealth`, `k_JumpHash`
- `readonly` for fields set only in constructor or Awake

```csharp
[SerializeField] private float m_MoveSpeed = 5f;
[SerializeField] private Transform m_SpawnPoint;

private Rigidbody m_Rigidbody;
private static readonly int k_JumpHash = Animator.StringToHash("Jump");
```

## Types and Naming

- Use `var` when the type is obvious from the right-hand side. Use explicit types when it isn't
- One type per file — file name MUST match the primary class/struct name (Unity requirement for MonoBehaviour)
- `sealed` by default — only unseal when inheritance is explicitly designed
- Explicit access modifiers on everything — no implicit `private`

## Structure Ordering

```csharp
public sealed class PlayerController : MonoBehaviour
{
    // 1. Serialized fields
    // 2. Private fields / cached references
    // 3. Properties
    // 4. Unity lifecycle: Awake, OnEnable, Start, FixedUpdate, Update, LateUpdate, OnDisable, OnDestroy
    // 5. Public methods
    // 6. Private methods
}
```

## Control Flow

- Braces always, even for single-line `if`/`for`/`while`
- `for` over `foreach` in hot paths (Update, FixedUpdate)
- No abbreviated loop variables — `for (int enemyIndex = 0; ...)` not `for (int i = 0; ...)`
- No magic strings — use `nameof()`, `Animator.StringToHash()`, `Shader.PropertyToID()`

## Other

- No LINQ in gameplay code
- `StringBuilder` for string building
- `CompareTag("tag")` not `tag == "tag"`
