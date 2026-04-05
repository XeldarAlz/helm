---
name: vcontainer
description: "VContainer dependency injection for Unity — LifetimeScope hierarchy, registration patterns, constructor injection for plain C#, [Inject] for MonoBehaviours. Lightweight alternative to Zenject."
globs: ["**/VContainer*", "**/*LifetimeScope*.cs", "**/*Installer*.cs", "**/Container*.cs"]
---

# VContainer — Dependency Injection for Unity

VContainer is a lightweight, fast DI framework for Unity by hadashiA. It provides constructor injection for plain C# classes, method injection for MonoBehaviours, hierarchical scoping, and lifecycle management without the complexity of Zenject.

## Why Dependency Injection in Unity

- **Decouple systems**: Components depend on interfaces, not concrete types
- **Testability**: Swap real implementations for mocks in tests
- **No singletons**: Avoid static state and its hidden coupling
- **Configurable composition**: Change wiring without changing code
- **Explicit dependencies**: Constructor parameters document what a class needs

## LifetimeScope Hierarchy

VContainer uses `LifetimeScope` MonoBehaviours as composition roots. They form a parent-child hierarchy for dependency resolution.

```
RootLifetimeScope (DontDestroyOnLoad)
  |- AudioService (Singleton)
  |- SaveSystem (Singleton)
  |- AnalyticsService (Singleton)
  +- ISettingsProvider (Singleton)
      |
      |- MainMenuLifetimeScope (MainMenu scene)
      |     |- MainMenuController
      |     +- LeaderboardService
      |
      +- GameLifetimeScope (Game scene)
            |- GameManager
            |- SpawnSystem
            +- ScoreSystem
```

### Root Scope (Project-Wide Services)

```csharp
using VContainer;
using VContainer.Unity;

public class RootLifetimeScope : LifetimeScope
{
    [SerializeField] private AudioSettings m_AudioSettings;

    protected override void Configure(IContainerBuilder builder)
    {
        // Singletons survive scene loads
        builder.Register<AudioService>(Lifetime.Singleton).As<IAudioService>();
        builder.Register<SaveSystem>(Lifetime.Singleton).As<ISaveSystem>();
        builder.Register<AnalyticsService>(Lifetime.Singleton).As<IAnalyticsService>();

        // ScriptableObject instance
        builder.RegisterInstance(m_AudioSettings);
    }
}
```

### Scene Scope (Scene-Specific)

```csharp
public class GameLifetimeScope : LifetimeScope
{
    [SerializeField] private LevelConfig m_LevelConfig;

    protected override void Configure(IContainerBuilder builder)
    {
        // Scene-specific registrations
        builder.Register<ScoreSystem>(Lifetime.Scoped);
        builder.Register<WaveSpawner>(Lifetime.Scoped);

        // Entry point with lifecycle
        builder.RegisterEntryPoint<GameFlowController>();

        // MonoBehaviour already in scene hierarchy
        builder.RegisterComponentInHierarchy<PlayerController>();
        builder.RegisterComponentInHierarchy<HUDManager>();

        // Config data
        builder.RegisterInstance(m_LevelConfig);
    }
}
```

### Parent-Child Auto-Resolution

Child scopes automatically resolve dependencies from their parent. A `GameLifetimeScope` can inject `IAudioService` registered in `RootLifetimeScope` without explicit wiring.

```csharp
// GameFlowController receives IAudioService from Root + ScoreSystem from Game scope
public class GameFlowController : IStartable, ITickable, IDisposable
{
    private readonly IAudioService m_Audio;
    private readonly ScoreSystem m_Score;

    public GameFlowController(IAudioService audio, ScoreSystem score)
    {
        m_Audio = audio;
        m_Score = score;
    }
}
```

## Registration Patterns

### Plain C# Classes — Constructor Injection (Preferred)

```csharp
// Registration
builder.Register<ScoreSystem>(Lifetime.Singleton);

// The class — dependencies are constructor parameters
public class ScoreSystem
{
    private readonly IAudioService m_Audio;
    private readonly ISaveSystem m_Save;

    public ScoreSystem(IAudioService audio, ISaveSystem save)
    {
        m_Audio = audio;
        m_Save = save;
    }

    public void AddScore(int points)
    {
        // Use injected services
        m_Audio.PlaySfx("score");
        m_Save.SetInt("score", points);
    }
}
```

### Interface Binding

```csharp
// Register concrete type, expose as interface
builder.Register<AudioService>(Lifetime.Singleton).As<IAudioService>();

// Multiple interfaces for same implementation
builder.Register<NetworkManager>(Lifetime.Singleton)
    .As<INetworkSender>()
    .As<INetworkReceiver>();

// Self + interface
builder.Register<GameManager>(Lifetime.Singleton)
    .AsSelf()
    .As<IGameStateProvider>();
```

### MonoBehaviour Registration

MonoBehaviours cannot use constructor injection. Use `[Inject]` method injection.

```csharp
// MonoBehaviour already placed in the scene
builder.RegisterComponentInHierarchy<PlayerController>();

// Create MonoBehaviour on a new GameObject
builder.RegisterComponentOnNewGameObject<HUDManager>(
    Lifetime.Scoped,
    "HUDManager"  // Optional GameObject name
);

// Register existing component reference from LifetimeScope's serialized fields
[SerializeField] private PlayerController m_Player;
// In Configure:
builder.RegisterComponent(m_Player);
```

```csharp
// MonoBehaviour with [Inject]
public class PlayerController : MonoBehaviour
{
    private IAudioService m_Audio;
    private IInputService m_Input;

    [Inject]
    public void Construct(IAudioService audio, IInputService input)
    {
        m_Audio = audio;
        m_Input = input;
    }

    private void Update()
    {
        if (m_Input.JumpPressed)
        {
            Jump();
            m_Audio.PlaySfx("jump");
        }
    }
}
```

### Entry Points — Lifecycle Without MonoBehaviour

Entry points implement lifecycle interfaces and run without needing a GameObject.

```csharp
builder.RegisterEntryPoint<GameFlowController>();
```

```csharp
public class GameFlowController : IStartable, ITickable, IFixedTickable, IDisposable
{
    private readonly ScoreSystem m_Score;

    public GameFlowController(ScoreSystem score) => m_Score = score;

    public void Start()
    {
        // Called once after injection, like MonoBehaviour.Start
        Debug.Log("Game started");
    }

    public void Tick()
    {
        // Called every frame, like MonoBehaviour.Update
    }

    public void FixedTick()
    {
        // Called every FixedUpdate
    }

    public void Dispose()
    {
        // Called when scope is destroyed, like MonoBehaviour.OnDestroy
        Debug.Log("Game ended");
    }
}
```

Available lifecycle interfaces:
- `IStartable` — `Start()` called once after construction
- `ITickable` — `Tick()` called every Update
- `IPostTickable` — `PostTick()` called every LateUpdate
- `IFixedTickable` — `FixedTick()` called every FixedUpdate
- `IDisposable` — `Dispose()` called when scope is destroyed
- `IAsyncStartable` — `async UniTask StartAsync(CancellationToken ct)`

### Factory Registration

For objects that need runtime creation with injected dependencies.

```csharp
// Registration
builder.Register<EnemyFactory>(Lifetime.Scoped);

// Factory class
public class EnemyFactory
{
    private readonly IObjectResolver m_Container;
    private readonly EnemySettings m_Settings;

    public EnemyFactory(IObjectResolver container, EnemySettings settings)
    {
        m_Container = container;
        m_Settings = settings;
    }

    public Enemy Create(EnemyType type)
    {
        GameObject prefab = m_Settings.GetPrefab(type);
        GameObject instance = Object.Instantiate(prefab);
        m_Container.InjectGameObject(instance); // Inject into all [Inject] methods
        return instance.GetComponent<Enemy>();
    }
}
```

### Func Factory (Lightweight)

```csharp
// Register a factory delegate
builder.RegisterFactory<Vector3, Bullet>(container =>
{
    var pool = container.Resolve<BulletPool>();
    return position => pool.Get(position);
}, Lifetime.Scoped);

// Inject the factory
public class Weapon
{
    private readonly Func<Vector3, Bullet> m_CreateBullet;

    public Weapon(Func<Vector3, Bullet> createBullet)
    {
        m_CreateBullet = createBullet;
    }

    public void Fire(Vector3 muzzlePos)
    {
        Bullet b = m_CreateBullet(muzzlePos);
    }
}
```

## Lifetime Options

| Lifetime | Behavior | Use For |
|----------|----------|---------|
| `Singleton` | One instance for the entire scope lifetime | Services, managers, shared state |
| `Transient` | New instance every time it is resolved | Stateless utilities, value objects |
| `Scoped` | One instance per scope (child scopes get their own) | Per-scene services, per-context state |

```csharp
builder.Register<AudioService>(Lifetime.Singleton);   // One forever
builder.Register<DamageCalculator>(Lifetime.Transient); // New each time
builder.Register<ScoreTracker>(Lifetime.Scoped);        // One per scene scope
```

## ScriptableObject and Asset Registration

```csharp
public class GameLifetimeScope : LifetimeScope
{
    [SerializeField] private GameConfig m_GameConfig;
    [SerializeField] private EnemyDatabase m_EnemyDatabase;

    protected override void Configure(IContainerBuilder builder)
    {
        // Register as instances (not created by container)
        builder.RegisterInstance(m_GameConfig);
        builder.RegisterInstance(m_EnemyDatabase).As<IEnemyDatabase>();
    }
}
```

## Child Scope Creation at Runtime

```csharp
public class RoomManager
{
    private readonly LifetimeScope m_ParentScope;

    public RoomManager(LifetimeScope parentScope) => m_ParentScope = parentScope;

    public LifetimeScope CreateRoomScope(RoomConfig config)
    {
        return m_ParentScope.CreateChild(builder =>
        {
            builder.RegisterInstance(config);
            builder.Register<RoomController>(Lifetime.Scoped);
            builder.RegisterEntryPoint<RoomLogic>();
        });
    }
}
```

## Common Mistakes

### Circular Dependencies

```csharp
// BAD — A depends on B, B depends on A. Container throws at resolution.
public class A { public A(B b) { } }
public class B { public B(A a) { } }

// FIX — break the cycle with an interface, event, or lazy resolution
public class A { public A(IEventBus bus) { } }
public class B { public B(IEventBus bus) { } }
```

### Registering MonoBehaviours as Transient

```csharp
// BAD — creates a new GameObject every resolve, probably not what you want
builder.RegisterComponentOnNewGameObject<Player>(Lifetime.Transient);

// GOOD — one per scope
builder.RegisterComponentOnNewGameObject<Player>(Lifetime.Scoped);

// GOOD — use existing scene object
builder.RegisterComponentInHierarchy<Player>();
```

### Forgetting to Register Dependencies

```csharp
// If ScoreSystem depends on IAudioService but IAudioService is not registered,
// VContainer throws VContainerException at build time.
// Always verify all dependencies are registered in the scope or a parent scope.
```

### Using [Inject] on Fields (Avoid)

```csharp
// BAD — field injection hides dependencies, untestable
public class BadComponent : MonoBehaviour
{
    [Inject] private IAudioService m_Audio;
}

// GOOD — method injection is explicit
public class GoodComponent : MonoBehaviour
{
    private IAudioService m_Audio;

    [Inject]
    public void Construct(IAudioService audio) => m_Audio = audio;
}
```

## Testing with VContainer

```csharp
[Test]
public void ScoreSystem_AddsScore()
{
    // No container needed — just use constructor
    var mockAudio = new MockAudioService();
    var mockSave = new MockSaveSystem();
    var score = new ScoreSystem(mockAudio, mockSave);

    score.AddScore(100);

    Assert.AreEqual(100, score.CurrentScore);
    Assert.IsTrue(mockAudio.PlayedSfx.Contains("score"));
}
```

## Project Structure Convention

```
Assets/
  Scripts/
    Installers/         (or Scopes/)
      RootLifetimeScope.cs
      GameLifetimeScope.cs
      MainMenuLifetimeScope.cs
    Services/
      IAudioService.cs
      AudioService.cs
    Game/
      GameFlowController.cs
      ScoreSystem.cs
```
