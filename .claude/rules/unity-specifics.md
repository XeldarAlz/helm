# Unity-Specific Rules

## Editor vs Runtime

```csharp
// Runtime code (Assets/Scripts/) — NEVER use UnityEditor unguarded
#if UNITY_EDITOR
using UnityEditor;
#endif

private void OnValidate()
{
    #if UNITY_EDITOR
    EditorUtility.SetDirty(this);
    #endif
}
```

- Code in `Editor/` folder: editor-only, excluded from builds automatically
- Code outside `Editor/`: must guard any `UnityEditor` usage with `#if UNITY_EDITOR`
- Forgetting the guard: compiles in Editor, **fails on build** with no warning until build time

## Platform Defines

```csharp
// GOOD — always provide fallback
#if UNITY_ANDROID
    string dataPath = Application.persistentDataPath;
#elif UNITY_IOS
    string dataPath = Application.persistentDataPath;
#else
    string dataPath = Application.dataPath;
#endif

// BAD — code silently excluded on other platforms
#if UNITY_ANDROID
    SetupMobileControls();
#endif
```

## The `?.` Operator Trap

```csharp
// DANGEROUS — bypasses Unity's destroyed-object detection
m_Target?.TakeDamage(10);  // Calls TakeDamage on destroyed objects!

// SAFE — Unity's == operator detects destroyed objects
if (m_Target != null)
{
    m_Target.TakeDamage(10);
}
```

Unity overrides `==` to return `true` when comparing destroyed objects to `null`. The `?.` operator uses C# reference equality, which does NOT detect destroyed objects. This is the #1 most subtle Unity bug.

## Lifecycle Order

```
Awake()       → called once when object is created (even if disabled)
OnEnable()    → called when object becomes active
Start()       → called once before first Update (only if enabled)
FixedUpdate() → physics tick (0.02s default)
Update()      → every frame
LateUpdate()  → every frame, after all Updates
OnDisable()   → called when object becomes inactive
OnDestroy()   → called when object is destroyed
```

- Don't depend on Awake order across objects — use `[DefaultExecutionOrder]` or explicit init
- `OnDisable` is called before `OnDestroy` — unsubscribe events in `OnDisable`
- `Start` is NOT called if the object is never enabled

## Threading

Unity API is main-thread only. Background threads cannot:
- Access `Transform`, `GameObject`, `Component`
- Call `Instantiate`, `Destroy`
- Access `Time`, `Input`, `Physics`

```csharp
// Return to main thread with UniTask:
await UniTask.SwitchToMainThread();

// Or with SynchronizationContext:
SynchronizationContext.Current.Post(_ => { /* Unity API here */ }, null);
```

## No Coroutines — Use UniTask

Do not use `StartCoroutine` / `IEnumerator` / `yield return`. Use UniTask for all async work.

Coroutine problems that UniTask solves:
- Coroutines stop silently when `gameObject.SetActive(false)` and don't resume
- Coroutines have no cancellation, error handling, or return values
- Coroutines allocate on the heap

```csharp
// BAD — coroutine
private IEnumerator WaitAndDo()
{
    yield return new WaitForSeconds(1f);
    DoSomething();
}

// GOOD — UniTask
private async UniTask WaitAndDoAsync(CancellationToken token)
{
    await UniTask.Delay(TimeSpan.FromSeconds(1), cancellationToken: token);
    DoSomething();
}
```

Always pass `CancellationToken`. In Views: `this.GetCancellationTokenOnDestroy()`. In Systems: own a `CancellationTokenSource` and cancel in `Dispose()`.

## DontDestroyOnLoad

Use sparingly. Prefer a bootstrapper scene pattern:
```
BootstrapScene (loads once, contains persistent services)
    → Additively loads GameScene, MenuScene, etc.
```

## Transform

- `transform.SetParent(parent, false)` — use `worldPositionStays: false` to preserve local transform
- `Application.isPlaying` — check in OnDisable/OnDestroy to avoid cleanup during editor domain reload

## Time

- `Time.deltaTime` in `Update` and `LateUpdate`
- `Time.fixedDeltaTime` in `FixedUpdate`
- Never use `Time.deltaTime` in `FixedUpdate` (it equals `fixedDeltaTime` there, but it's confusing)
- `Time.unscaledDeltaTime` for pause-independent logic (UI animations, etc.)

## Component Attributes

```csharp
[RequireComponent(typeof(Rigidbody))]        // Auto-adds Rigidbody, prevents removal
[DisallowMultipleComponent]                   // Prevents duplicate components
[DefaultExecutionOrder(-100)]                 // Runs before default scripts
[SelectionBase]                               // Click selects this object, not children
```

## Input

Use the new Input System package, not legacy `Input.GetKey`:
```csharp
// BAD — legacy
if (Input.GetKeyDown(KeyCode.Space)) Jump();

// GOOD — Input System
private PlayerControls m_Controls;
private void OnEnable() => m_Controls.Enable();
private void OnDisable() => m_Controls.Disable();
```

## .meta Files

- NEVER edit manually
- ALWAYS commit alongside their asset
- Missing .meta = Unity regenerates GUID = all references break
- Orphaned .meta = clutter and potential conflicts
