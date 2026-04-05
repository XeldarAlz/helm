# Architecture Rules

## Model-View-System (MVS) Pattern

All features follow a strict three-layer separation:

```
Model  — Pure C# classes. State + data only. No Unity API, no MonoBehaviour.
View   — MonoBehaviour. Reads Model, renders visuals, forwards input. No logic.
System — Plain C# class (registered in VContainer). Owns and mutates Models. Contains all logic.
```

```csharp
// --- Model (pure C#, serializable, no Unity dependencies) ---
public sealed class PlayerModel
{
    public ReactiveProperty<int> Health { get; } = new(100);
    public ReactiveProperty<Vector3> Position { get; } = new(Vector3.zero);
    public bool IsDead => Health.Value <= 0;
}

// --- System (plain C#, injected via VContainer, owns the Model) ---
public sealed class PlayerSystem : IDisposable
{
    private readonly PlayerModel m_Model;
    private readonly IPublisher<PlayerDiedMessage> m_DiedPublisher;

    [Inject]
    public PlayerSystem(PlayerModel model, IPublisher<PlayerDiedMessage> diedPublisher)
    {
        m_Model = model;
        m_DiedPublisher = diedPublisher;
    }

    public void TakeDamage(int amount)
    {
        m_Model.Health.Value = Mathf.Max(0, m_Model.Health.Value - amount);
        if (m_Model.IsDead)
        {
            m_DiedPublisher.Publish(new PlayerDiedMessage());
        }
    }

    public void Dispose() { }
}

// --- View (MonoBehaviour, observes Model, no logic) ---
public sealed class PlayerView : MonoBehaviour
{
    [SerializeField] private Slider m_HealthBar;

    private PlayerModel m_Model;
    private readonly CompositeDisposable m_Disposables = new();

    [Inject]
    public void Construct(PlayerModel model)
    {
        m_Model = model;
    }

    private void Start()
    {
        m_Model.Health
            .Subscribe(hp => m_HealthBar.value = hp / 100f)
            .AddTo(m_Disposables);
    }

    private void OnDestroy() => m_Disposables.Dispose();
}
```

**Rules:**
- Models NEVER reference Views or Systems
- Systems NEVER reference Views
- Views observe Models via `ReactiveProperty<T>.Subscribe()` — no polling in Update
- Views call Systems for actions (injected via VContainer)
- One System can own multiple Models; one View binds to one primary Model

## VContainer for Dependency Injection

VContainer is the **only** way to wire dependencies. No singletons, no static access, no `FindObjectOfType`.

```csharp
public sealed class GameLifetimeScope : LifetimeScope
{
    protected override void Configure(IContainerBuilder builder)
    {
        // Models — singleton per scope
        builder.Register<PlayerModel>(Lifetime.Singleton);
        builder.Register<InventoryModel>(Lifetime.Singleton);

        // Systems — singleton per scope
        builder.Register<PlayerSystem>(Lifetime.Singleton).AsImplementedInterfaces().AsSelf();
        builder.Register<InventorySystem>(Lifetime.Singleton).AsImplementedInterfaces().AsSelf();

        // Services — interface-based for swappability
        builder.Register<SaveService>(Lifetime.Singleton).As<ISaveService>();

        // Views — use EntryPoint for non-MonoBehaviour tick
        builder.RegisterEntryPoint<GameLoopSystem>();

        // MonoBehaviour views — find in scene or prefab
        builder.RegisterComponentInHierarchy<PlayerView>();

        // MessagePipe
        var messagePipeOptions = builder.RegisterMessagePipe();
        builder.RegisterMessageBroker<PlayerDiedMessage>(messagePipeOptions);
        builder.RegisterMessageBroker<ScoreChangedMessage>(messagePipeOptions);
        builder.RegisterMessageBroker<ItemPickedUpMessage>(messagePipeOptions);
    }
}
```

**Scope hierarchy:**
```
RootLifetimeScope          — app-wide services (audio, save, settings)
  └─ SceneLifetimeScope    — per-scene systems and models
       └─ Child scopes     — per-feature (e.g., UI popup, spawned entity)
```

- Use `Lifetime.Singleton` for shared state within a scope
- Use `Lifetime.Transient` for stateless services or factories
- Use child scopes for dynamically spawned entities that need injection
- NEVER use `[Inject]` on fields — use constructor injection for Systems, method injection (`[Inject] public void Construct(...)`) for MonoBehaviours

## MessagePipe for Communication

MessagePipe is the **only** messaging system. No SO event channels, no static EventBus, no C# events for cross-system communication.

```csharp
// --- Define messages as readonly structs ---
public readonly struct PlayerDiedMessage { }

public readonly struct DamageDealtMessage
{
    public readonly int Amount;
    public readonly Vector3 Position;

    public DamageDealtMessage(int amount, Vector3 position)
    {
        Amount = amount;
        Position = position;
    }
}

// --- Publishing (System → System or System → View) ---
public sealed class CombatSystem : IDisposable
{
    private readonly IPublisher<DamageDealtMessage> m_DamagePublisher;

    [Inject]
    public CombatSystem(IPublisher<DamageDealtMessage> damagePublisher)
    {
        m_DamagePublisher = damagePublisher;
    }

    public void DealDamage(int amount, Vector3 position)
    {
        m_DamagePublisher.Publish(new DamageDealtMessage(amount, position));
    }

    public void Dispose() { }
}

// --- Subscribing (in System or View) ---
public sealed class DamagePopupSystem : IDisposable
{
    private readonly IDisposable m_Subscription;

    [Inject]
    public DamagePopupSystem(ISubscriber<DamageDealtMessage> damageSubscriber)
    {
        m_Subscription = damageSubscriber.Subscribe(OnDamageDealt);
    }

    private void OnDamageDealt(DamageDealtMessage message)
    {
        // spawn popup at message.Position showing message.Amount
    }

    public void Dispose() => m_Subscription.Dispose();
}
```

**Rules:**
- Messages are `readonly struct` — zero allocation
- Register all message brokers in the LifetimeScope
- Always dispose subscriptions (Systems via `IDisposable`, Views via `CompositeDisposable`)
- Use `IAsyncSubscriber<T>` with UniTask when the handler needs async work
- Use `IBufferedPublisher<T>` / `IBufferedSubscriber<T>` when late subscribers need the last value

## UniTask for Async

UniTask replaces coroutines entirely. No `StartCoroutine`, no `IEnumerator`, no `yield return`.

```csharp
public sealed class WaveSpawnerSystem : IDisposable
{
    private readonly CancellationTokenSource m_Cts = new();

    public async UniTaskVoid StartSpawning()
    {
        for (int waveIndex = 0; waveIndex < 10; waveIndex++)
        {
            await SpawnWave(waveIndex, m_Cts.Token);
            await UniTask.Delay(TimeSpan.FromSeconds(5), cancellationToken: m_Cts.Token);
        }
    }

    private async UniTask SpawnWave(int waveIndex, CancellationToken token)
    {
        int enemyCount = waveIndex * 3;
        for (int enemyIndex = 0; enemyIndex < enemyCount; enemyIndex++)
        {
            // spawn logic
            await UniTask.Delay(TimeSpan.FromSeconds(0.2f), cancellationToken: token);
        }
    }

    public void Dispose() => m_Cts.Cancel();
}
```

**Rules:**
- Always pass `CancellationToken` — typically from `this.GetCancellationTokenOnDestroy()` in Views, or a `CancellationTokenSource` in Systems
- Use `UniTask` for awaitable operations, `UniTaskVoid` for fire-and-forget
- Use `UniTask.Delay` instead of `new WaitForSeconds`
- Use `UniTask.WhenAll` for parallel async operations
- Use `UniTask.SwitchToMainThread()` when returning from background thread
- No `async void` — always `async UniTask` or `async UniTaskVoid`

## Composition Over Inheritance

MonoBehaviour is a component, not a base class. Don't build deep inheritance trees.

Max MonoBehaviour inheritance depth: 2 (base + one subclass). Beyond that, compose.

Views should be thin — the logic lives in Systems, data lives in Models.

## ScriptableObjects for Static Data

Items, abilities, enemy configs, level data — all should be ScriptableObjects:

```csharp
[CreateAssetMenu(menuName = "Game/Weapon Definition")]
public sealed class WeaponDefinition : ScriptableObject
{
    [SerializeField] private string m_DisplayName;
    [SerializeField] private float m_Damage;
    [SerializeField] private float m_FireRate;
    [SerializeField] private GameObject m_Prefab;
}
```

ScriptableObjects hold **static/config data**. Runtime mutable state goes in Models.

## Dependency Direction

```
Views → Systems → Models
  ↓        ↓
MessagePipe (decoupled communication)
```

- Views depend on Systems and Models (via VContainer injection)
- Systems depend on Models and other Systems (via VContainer injection)
- Models depend on nothing
- Cross-system communication goes through MessagePipe, never direct references
- Assembly definitions enforce direction at compile time

## No God Objects

```csharp
// BAD
class GameManager : MonoBehaviour
{
    // Handles: score, lives, spawning, UI, audio, saving, input, pause...
}

// GOOD — separate Systems registered in VContainer
// PlayerSystem — health, movement
// ScoreSystem — scoring, combos
// SpawnSystem — enemy waves
// Each is a plain C# class with injected dependencies
```

## Scene Independence

Each scene should be loadable independently via LifetimeScope hierarchy:
1. `RootLifetimeScope` lives in a bootstrap scene (app-wide services, DontDestroyOnLoad)
2. Each game scene has its own `SceneLifetimeScope` that inherits from Root
3. No hidden dependencies on "the scene before this one"
4. Scene loading/unloading is async via UniTask:

```csharp
await SceneManager.LoadSceneAsync("GameScene", LoadSceneMode.Additive).ToUniTask();
```

## No Singletons

VContainer replaces all singleton patterns. Register as `Lifetime.Singleton` in the appropriate scope instead.

- Need app-wide? Register in `RootLifetimeScope`
- Need per-scene? Register in `SceneLifetimeScope`
- Need per-feature? Create a child scope
