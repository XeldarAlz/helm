# C# Style — Unity Conventions

## Field Declarations

- `[SerializeField] private` for inspector-exposed fields — never public
- Private/protected fields use `_lowerCamelCase`: `_moveSpeed`, `_health`
- Public fields use `lowerCamelCase`: `moveSpeed`, `health`
- Properties (public and private) use `UpperCamelCase`: `MoveSpeed`, `Health`
- `static readonly` fields use `UpperCamelCase`: `JumpHash`, `DefaultColor`
- `const` fields use `UPPER_SNAKE_CASE`: `MAX_HEALTH`, `MAX_PLAYER_COUNT`
- `readonly` for fields set only in constructor or Awake

```csharp
[SerializeField] private float _moveSpeed = 5f;
[SerializeField] private Transform _spawnPoint;

private Rigidbody _rigidbody;
private static readonly int JumpHash = Animator.StringToHash("Jump");
private const int MAX_JUMP_COUNT = 3;
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
