---
name: Tower Defense
description: Tower defense game patterns — placement grids, enemy pathing, wave spawning, tower upgrades, economy
globs: ["**/Tower*.cs", "**/Wave*.cs", "**/Turret*.cs", "**/TD*.cs", "**/Defense*.cs", "**/Path*.cs"]
---

# Tower Defense Patterns

## Overview

The tower defense core loop: players place towers on a grid to defend against waves of enemies traveling along predefined paths. Enemies spawn in waves of increasing difficulty. Towers automatically acquire targets and fire projectiles. Killing enemies earns currency used to buy new towers or upgrade existing ones. The game ends if too many enemies reach the exit.

Key systems: placement grid, tower targeting and firing, enemy pathing, wave spawning, economy, projectiles, and upgrades. Each system is a plain C# class registered in VContainer. MonoBehaviour Views are thin wrappers that read Models and render visuals.

---

## Placement Grid System

### Grid Model

The grid is a pure C# model that tracks which cells are buildable and which are occupied. No Unity API in the model.

```csharp
public enum CellState { Empty, Occupied, Blocked, Path }

public sealed class GridModel
{
    private readonly CellState[,] m_Cells;
    private readonly int m_Width;
    private readonly int m_Height;
    private readonly float m_CellSize;
    private readonly Vector3 m_Origin;

    public int Width => m_Width;
    public int Height => m_Height;
    public float CellSize => m_CellSize;

    public GridModel(int width, int height, float cellSize, Vector3 origin)
    {
        m_Width = width;
        m_Height = height;
        m_CellSize = cellSize;
        m_Origin = origin;
        m_Cells = new CellState[width, height];
    }

    public bool IsValidCell(int x, int y)
    {
        return x >= 0 && x < m_Width && y >= 0 && y < m_Height;
    }

    public CellState GetCellState(int x, int y)
    {
        return m_Cells[x, y];
    }

    public void SetCellState(int x, int y, CellState state)
    {
        m_Cells[x, y] = state;
    }

    public bool CanPlaceTower(int x, int y)
    {
        return IsValidCell(x, y) && m_Cells[x, y] == CellState.Empty;
    }

    public Vector3 CellToWorld(int x, int y)
    {
        return m_Origin + new Vector3(
            x * m_CellSize + m_CellSize * 0.5f,
            0f,
            y * m_CellSize + m_CellSize * 0.5f
        );
    }

    public (int x, int y) WorldToCell(Vector3 worldPos)
    {
        Vector3 local = worldPos - m_Origin;
        int x = Mathf.FloorToInt(local.x / m_CellSize);
        int y = Mathf.FloorToInt(local.z / m_CellSize);
        return (x, y);
    }
}
```

### Placement System

The PlacementSystem handles validation, preview ghost positioning, and committing a tower to the grid. It publishes a message when placement succeeds so the economy system can deduct currency.

```csharp
public readonly struct TowerPlacedMessage
{
    public readonly int GridX;
    public readonly int GridY;
    public readonly TowerDefinition Definition;

    public TowerPlacedMessage(int gridX, int gridY, TowerDefinition definition)
    {
        GridX = gridX;
        GridY = gridY;
        Definition = definition;
    }
}

public sealed class PlacementSystem : IDisposable
{
    private readonly GridModel m_Grid;
    private readonly EconomyModel m_Economy;
    private readonly IPublisher<TowerPlacedMessage> m_PlacedPublisher;

    [Inject]
    public PlacementSystem(
        GridModel grid,
        EconomyModel economy,
        IPublisher<TowerPlacedMessage> placedPublisher)
    {
        m_Grid = grid;
        m_Economy = economy;
        m_PlacedPublisher = placedPublisher;
    }

    public bool TryPlace(int x, int y, TowerDefinition definition)
    {
        if (!m_Grid.CanPlaceTower(x, y))
        {
            return false;
        }

        if (m_Economy.Currency.Value < definition.Cost)
        {
            return false;
        }

        m_Grid.SetCellState(x, y, CellState.Occupied);
        m_Economy.Currency.Value -= definition.Cost;
        m_PlacedPublisher.Publish(new TowerPlacedMessage(x, y, definition));
        return true;
    }

    public bool CanAffordAndPlace(int x, int y, TowerDefinition definition)
    {
        return m_Grid.CanPlaceTower(x, y) && m_Economy.Currency.Value >= definition.Cost;
    }

    public void Dispose() { }
}
```

### Ghost Preview View

The View shows a translucent preview of the tower under the cursor. It snaps to the grid and tints green or red based on placement validity.

```csharp
public sealed class PlacementGhostView : MonoBehaviour
{
    [SerializeField] private MeshRenderer m_GhostRenderer;
    [SerializeField] private Material m_ValidMaterial;
    [SerializeField] private Material m_InvalidMaterial;

    private GridModel m_Grid;
    private PlacementSystem m_Placement;
    private TowerDefinition m_SelectedDefinition;

    [Inject]
    public void Construct(GridModel grid, PlacementSystem placement)
    {
        m_Grid = grid;
        m_Placement = placement;
    }

    private void Update()
    {
        if (m_SelectedDefinition == null)
        {
            m_GhostRenderer.enabled = false;
            return;
        }

        Ray ray = Camera.main.ScreenPointToRay(Input.mousePosition);
        if (!Physics.Raycast(ray, out RaycastHit hit, 100f))
        {
            m_GhostRenderer.enabled = false;
            return;
        }

        (int x, int y) = m_Grid.WorldToCell(hit.point);
        if (!m_Grid.IsValidCell(x, y))
        {
            m_GhostRenderer.enabled = false;
            return;
        }

        m_GhostRenderer.enabled = true;
        transform.position = m_Grid.CellToWorld(x, y);

        bool valid = m_Placement.CanAffordAndPlace(x, y, m_SelectedDefinition);
        m_GhostRenderer.sharedMaterial = valid ? m_ValidMaterial : m_InvalidMaterial;
    }

    public void SetSelectedTower(TowerDefinition definition)
    {
        m_SelectedDefinition = definition;
    }

    public void ClearSelection()
    {
        m_SelectedDefinition = null;
    }
}
```

---

## Tower Architecture

### Tower Definition (ScriptableObject)

All tower data lives in a ScriptableObject. Runtime mutable state (cooldown timer, current target) goes in the TowerModel.

```csharp
public enum TargetingMode { Nearest, Strongest, First, Last }

[CreateAssetMenu(menuName = "TD/Tower Definition")]
public sealed class TowerDefinition : ScriptableObject
{
    [SerializeField] private string m_TowerId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private int m_Cost;
    [SerializeField] private float m_Range;
    [SerializeField] private float m_Damage;
    [SerializeField] private float m_FireRate;
    [SerializeField] private TargetingMode m_DefaultTargeting;
    [SerializeField] private GameObject m_Prefab;
    [SerializeField] private ProjectileDefinition m_Projectile;
    [SerializeField] private TowerDefinition[] m_UpgradePath;
    [SerializeField] private float m_SellRefundPercent = 0.7f;

    public string TowerId => m_TowerId;
    public string DisplayName => m_DisplayName;
    public int Cost => m_Cost;
    public float Range => m_Range;
    public float Damage => m_Damage;
    public float FireRate => m_FireRate;
    public float FireInterval => 1f / m_FireRate;
    public TargetingMode DefaultTargeting => m_DefaultTargeting;
    public GameObject Prefab => m_Prefab;
    public ProjectileDefinition Projectile => m_Projectile;
    public IReadOnlyList<TowerDefinition> UpgradePath => m_UpgradePath;
    public int SellValue => Mathf.RoundToInt(m_Cost * m_SellRefundPercent);
}
```

### Tower Model

```csharp
public sealed class TowerModel
{
    public TowerDefinition Definition { get; set; }
    public int GridX { get; }
    public int GridY { get; }
    public Vector3 WorldPosition { get; }
    public TargetingMode Targeting { get; set; }
    public float CooldownRemaining { get; set; }
    public EnemyModel CurrentTarget { get; set; }
    public int UpgradeLevel { get; set; }

    public TowerModel(TowerDefinition definition, int gridX, int gridY, Vector3 worldPosition)
    {
        Definition = definition;
        GridX = gridX;
        GridY = gridY;
        WorldPosition = worldPosition;
        Targeting = definition.DefaultTargeting;
    }

    public bool IsReady => CooldownRemaining <= 0f;
}
```

### Tower System — Target Acquisition and Firing

The TowerSystem iterates all towers each tick, acquires targets from the enemy registry, and publishes fire messages. No MonoBehaviour, no Unity lifecycle.

```csharp
public readonly struct TowerFiredMessage
{
    public readonly TowerModel Tower;
    public readonly EnemyModel Target;

    public TowerFiredMessage(TowerModel tower, EnemyModel target)
    {
        Tower = tower;
        Target = target;
    }
}

public sealed class TowerSystem : ITickable, IDisposable
{
    private readonly List<TowerModel> m_Towers = new();
    private readonly EnemyRegistry m_EnemyRegistry;
    private readonly IPublisher<TowerFiredMessage> m_FirePublisher;

    [Inject]
    public TowerSystem(
        EnemyRegistry enemyRegistry,
        IPublisher<TowerFiredMessage> firePublisher)
    {
        m_EnemyRegistry = enemyRegistry;
        m_FirePublisher = firePublisher;
    }

    public void RegisterTower(TowerModel tower)
    {
        m_Towers.Add(tower);
    }

    public void UnregisterTower(TowerModel tower)
    {
        m_Towers.Remove(tower);
    }

    public void Tick()
    {
        float dt = Time.deltaTime;

        for (int towerIndex = 0; towerIndex < m_Towers.Count; towerIndex++)
        {
            TowerModel tower = m_Towers[towerIndex];
            tower.CooldownRemaining -= dt;

            if (!tower.IsReady)
            {
                continue;
            }

            EnemyModel target = AcquireTarget(tower);
            if (target == null)
            {
                continue;
            }

            tower.CurrentTarget = target;
            tower.CooldownRemaining = tower.Definition.FireInterval;
            m_FirePublisher.Publish(new TowerFiredMessage(tower, target));
        }
    }

    private EnemyModel AcquireTarget(TowerModel tower)
    {
        float rangeSqr = tower.Definition.Range * tower.Definition.Range;
        EnemyModel best = null;
        float bestScore = float.MaxValue;

        ReadOnlySpan<EnemyModel> enemies = m_EnemyRegistry.ActiveEnemies;
        for (int enemyIndex = 0; enemyIndex < enemies.Length; enemyIndex++)
        {
            EnemyModel enemy = enemies[enemyIndex];
            if (enemy.IsDead)
            {
                continue;
            }

            float distSqr = (enemy.Position - tower.WorldPosition).sqrMagnitude;
            if (distSqr > rangeSqr)
            {
                continue;
            }

            float score = tower.Targeting switch
            {
                TargetingMode.Nearest => distSqr,
                TargetingMode.Strongest => -enemy.Health,
                TargetingMode.First => -enemy.DistanceTraveled,
                TargetingMode.Last => enemy.DistanceTraveled,
                _ => distSqr
            };

            if (score < bestScore)
            {
                bestScore = score;
                best = enemy;
            }
        }

        return best;
    }

    public void Dispose() { }
}
```

**Key design points:**
- `ITickable` is a VContainer interface that calls `Tick()` every frame without needing MonoBehaviour.
- Target acquisition uses squared distance to avoid `sqrt` on every comparison.
- The switch expression maps targeting mode to a score for unified comparison.
- `EnemyRegistry` exposes a `ReadOnlySpan<EnemyModel>` to avoid allocation when iterating.

---

## Enemy Pathing

### Waypoint Path Definition

Paths are defined as a sequence of world-space waypoints. Enemies follow these points in order.

```csharp
[CreateAssetMenu(menuName = "TD/Path Definition")]
public sealed class PathDefinition : ScriptableObject
{
    [SerializeField] private Vector3[] m_Waypoints;

    public IReadOnlyList<Vector3> Waypoints => m_Waypoints;
    public int WaypointCount => m_Waypoints.Length;

    public float TotalLength
    {
        get
        {
            float total = 0f;
            for (int waypointIndex = 1; waypointIndex < m_Waypoints.Length; waypointIndex++)
            {
                total += Vector3.Distance(m_Waypoints[waypointIndex - 1], m_Waypoints[waypointIndex]);
            }
            return total;
        }
    }
}
```

### Enemy Model

```csharp
public sealed class EnemyModel
{
    public EnemyDefinition Definition { get; }
    public float Health { get; set; }
    public float MaxHealth { get; }
    public float Speed { get; set; }
    public float SpeedMultiplier { get; set; } = 1f;
    public Vector3 Position { get; set; }
    public int CurrentWaypointIndex { get; set; }
    public float DistanceTraveled { get; set; }
    public bool IsDead => Health <= 0f;
    public bool IsFlying { get; }
    public bool ReachedEnd { get; set; }

    public EnemyModel(EnemyDefinition definition)
    {
        Definition = definition;
        Health = definition.MaxHealth;
        MaxHealth = definition.MaxHealth;
        Speed = definition.MoveSpeed;
        IsFlying = definition.IsFlying;
    }
}
```

### Enemy Definition (ScriptableObject)

```csharp
[CreateAssetMenu(menuName = "TD/Enemy Definition")]
public sealed class EnemyDefinition : ScriptableObject
{
    [SerializeField] private string m_EnemyId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private float m_MaxHealth;
    [SerializeField] private float m_MoveSpeed;
    [SerializeField] private int m_CurrencyReward;
    [SerializeField] private int m_Damage;
    [SerializeField] private bool m_IsFlying;
    [SerializeField] private GameObject m_Prefab;

    public string EnemyId => m_EnemyId;
    public float MaxHealth => m_MaxHealth;
    public float MoveSpeed => m_MoveSpeed;
    public int CurrencyReward => m_CurrencyReward;
    public int Damage => m_Damage;
    public bool IsFlying => m_IsFlying;
    public GameObject Prefab => m_Prefab;
}
```

### Pathing System

The pathing system moves all active enemies along their assigned paths each tick. When an enemy reaches the final waypoint, it publishes a message so the lives system can respond.

```csharp
public readonly struct EnemyReachedEndMessage
{
    public readonly EnemyModel Enemy;
    public EnemyReachedEndMessage(EnemyModel enemy) { Enemy = enemy; }
}

public sealed class PathingSystem : ITickable, IDisposable
{
    private readonly EnemyRegistry m_EnemyRegistry;
    private readonly IPublisher<EnemyReachedEndMessage> m_ReachedEndPublisher;

    [Inject]
    public PathingSystem(
        EnemyRegistry enemyRegistry,
        IPublisher<EnemyReachedEndMessage> reachedEndPublisher)
    {
        m_EnemyRegistry = enemyRegistry;
        m_ReachedEndPublisher = reachedEndPublisher;
    }

    public void Tick()
    {
        float dt = Time.deltaTime;
        ReadOnlySpan<EnemyModel> enemies = m_EnemyRegistry.ActiveEnemies;

        for (int enemyIndex = 0; enemyIndex < enemies.Length; enemyIndex++)
        {
            EnemyModel enemy = enemies[enemyIndex];
            if (enemy.IsDead || enemy.ReachedEnd)
            {
                continue;
            }

            MoveAlongPath(enemy, dt);
        }
    }

    private void MoveAlongPath(EnemyModel enemy, float dt)
    {
        PathDefinition path = m_EnemyRegistry.GetPathForEnemy(enemy);
        float speed = enemy.Speed * enemy.SpeedMultiplier * dt;

        while (speed > 0f && enemy.CurrentWaypointIndex < path.WaypointCount)
        {
            Vector3 target = path.Waypoints[enemy.CurrentWaypointIndex];
            Vector3 toTarget = target - enemy.Position;
            float distance = toTarget.magnitude;

            if (distance <= speed)
            {
                enemy.Position = target;
                enemy.DistanceTraveled += distance;
                speed -= distance;
                enemy.CurrentWaypointIndex++;
            }
            else
            {
                Vector3 direction = toTarget / distance;
                enemy.Position += direction * speed;
                enemy.DistanceTraveled += speed;
                speed = 0f;
            }
        }

        if (enemy.CurrentWaypointIndex >= path.WaypointCount)
        {
            enemy.ReachedEnd = true;
            m_ReachedEndPublisher.Publish(new EnemyReachedEndMessage(enemy));
        }
    }

    public void Dispose() { }
}
```

**Flying enemies** use the same path but with an elevated Y offset. The pathing system adds a vertical offset from `EnemyDefinition.FlyHeight` so flying enemies pass over obstacles. Ground-targeting towers skip flying enemies, and anti-air towers skip ground enemies -- handled in `TowerSystem.AcquireTarget` with a flag check.

---

## Wave Spawning System

### Wave Definition

```csharp
[System.Serializable]
public sealed class WaveGroup
{
    [SerializeField] private EnemyDefinition m_EnemyType;
    [SerializeField] private int m_Count;
    [SerializeField] private float m_SpawnInterval;
    [SerializeField] private float m_DelayBeforeGroup;

    public EnemyDefinition EnemyType => m_EnemyType;
    public int Count => m_Count;
    public float SpawnInterval => m_SpawnInterval;
    public float DelayBeforeGroup => m_DelayBeforeGroup;
}

[CreateAssetMenu(menuName = "TD/Wave Definition")]
public sealed class WaveDefinition : ScriptableObject
{
    [SerializeField] private WaveGroup[] m_Groups;
    [SerializeField] private float m_DelayAfterWave;
    [SerializeField] private int m_BonusCurrency;

    public IReadOnlyList<WaveGroup> Groups => m_Groups;
    public float DelayAfterWave => m_DelayAfterWave;
    public int BonusCurrency => m_BonusCurrency;
}
```

### Wave Model

```csharp
public sealed class WaveModel
{
    public ReactiveProperty<int> CurrentWaveIndex { get; } = new(0);
    public ReactiveProperty<int> EnemiesRemainingInWave { get; } = new(0);
    public ReactiveProperty<bool> IsSpawning { get; } = new(false);
    public int TotalWaves { get; set; }
    public bool AllWavesComplete => CurrentWaveIndex.Value >= TotalWaves;
}
```

### Wave Spawner System

Uses UniTask for async spawn timing. No coroutines.

```csharp
public readonly struct WaveStartedMessage
{
    public readonly int WaveIndex;
    public WaveStartedMessage(int waveIndex) { WaveIndex = waveIndex; }
}

public readonly struct WaveCompletedMessage
{
    public readonly int WaveIndex;
    public WaveCompletedMessage(int waveIndex) { WaveIndex = waveIndex; }
}

public sealed class WaveSpawnerSystem : IDisposable
{
    private readonly WaveModel m_WaveModel;
    private readonly WaveDefinition[] m_Waves;
    private readonly EnemyRegistry m_EnemyRegistry;
    private readonly IPublisher<WaveStartedMessage> m_WaveStartedPublisher;
    private readonly IPublisher<WaveCompletedMessage> m_WaveCompletedPublisher;
    private readonly CancellationTokenSource m_Cts = new();

    [Inject]
    public WaveSpawnerSystem(
        WaveModel waveModel,
        WaveDefinition[] waves,
        EnemyRegistry enemyRegistry,
        IPublisher<WaveStartedMessage> waveStartedPublisher,
        IPublisher<WaveCompletedMessage> waveCompletedPublisher)
    {
        m_WaveModel = waveModel;
        m_Waves = waves;
        m_EnemyRegistry = enemyRegistry;
        m_WaveStartedPublisher = waveStartedPublisher;
        m_WaveCompletedPublisher = waveCompletedPublisher;
        m_WaveModel.TotalWaves = waves.Length;
    }

    public async UniTaskVoid StartNextWave()
    {
        int waveIndex = m_WaveModel.CurrentWaveIndex.Value;
        if (waveIndex >= m_Waves.Length)
        {
            return;
        }

        WaveDefinition wave = m_Waves[waveIndex];
        m_WaveModel.IsSpawning.Value = true;
        m_WaveStartedPublisher.Publish(new WaveStartedMessage(waveIndex));

        int totalEnemies = 0;
        for (int groupIndex = 0; groupIndex < wave.Groups.Count; groupIndex++)
        {
            totalEnemies += wave.Groups[groupIndex].Count;
        }
        m_WaveModel.EnemiesRemainingInWave.Value = totalEnemies;

        for (int groupIndex = 0; groupIndex < wave.Groups.Count; groupIndex++)
        {
            WaveGroup group = wave.Groups[groupIndex];

            if (group.DelayBeforeGroup > 0f)
            {
                await UniTask.Delay(
                    TimeSpan.FromSeconds(group.DelayBeforeGroup),
                    cancellationToken: m_Cts.Token);
            }

            for (int enemyIndex = 0; enemyIndex < group.Count; enemyIndex++)
            {
                m_EnemyRegistry.SpawnEnemy(group.EnemyType);

                if (enemyIndex < group.Count - 1)
                {
                    await UniTask.Delay(
                        TimeSpan.FromSeconds(group.SpawnInterval),
                        cancellationToken: m_Cts.Token);
                }
            }
        }

        m_WaveModel.IsSpawning.Value = false;
        m_WaveModel.CurrentWaveIndex.Value = waveIndex + 1;
        m_WaveCompletedPublisher.Publish(new WaveCompletedMessage(waveIndex));
    }

    public void Dispose() => m_Cts.Cancel();
}
```

**Auto-start vs manual start:** For auto-start, subscribe to `WaveCompletedMessage` in a controller system that waits `DelayAfterWave` seconds then calls `StartNextWave()`. For manual start, expose a "Send Next Wave" button in the UI that calls the same method.

---

## Economy and Upgrade System

### Economy Model

```csharp
public sealed class EconomyModel
{
    public ReactiveProperty<int> Currency { get; } = new(0);
    public ReactiveProperty<int> Lives { get; } = new(20);
}
```

### Economy System

```csharp
public readonly struct EnemyKilledMessage
{
    public readonly EnemyModel Enemy;
    public EnemyKilledMessage(EnemyModel enemy) { Enemy = enemy; }
}

public sealed class EconomySystem : IDisposable
{
    private readonly EconomyModel m_Model;
    private readonly IDisposable m_KillSubscription;
    private readonly IDisposable m_ReachedEndSubscription;

    [Inject]
    public EconomySystem(
        EconomyModel model,
        ISubscriber<EnemyKilledMessage> killSubscriber,
        ISubscriber<EnemyReachedEndMessage> reachedEndSubscriber)
    {
        m_Model = model;
        m_KillSubscription = killSubscriber.Subscribe(OnEnemyKilled);
        m_ReachedEndSubscription = reachedEndSubscriber.Subscribe(OnEnemyReachedEnd);
    }

    private void OnEnemyKilled(EnemyKilledMessage message)
    {
        m_Model.Currency.Value += message.Enemy.Definition.CurrencyReward;
    }

    private void OnEnemyReachedEnd(EnemyReachedEndMessage message)
    {
        m_Model.Lives.Value -= message.Enemy.Definition.Damage;
    }

    public void Dispose()
    {
        m_KillSubscription.Dispose();
        m_ReachedEndSubscription.Dispose();
    }
}
```

### Upgrade System

Towers upgrade by swapping their `TowerDefinition` to the next entry in the upgrade path. The upgrade costs the difference between the new tower cost and the current one. Stats change immediately.

```csharp
public sealed class UpgradeSystem : IDisposable
{
    private readonly EconomyModel m_Economy;
    private readonly IPublisher<TowerUpgradedMessage> m_UpgradedPublisher;

    [Inject]
    public UpgradeSystem(
        EconomyModel economy,
        IPublisher<TowerUpgradedMessage> upgradedPublisher)
    {
        m_Economy = economy;
        m_UpgradedPublisher = upgradedPublisher;
    }

    public bool TryUpgrade(TowerModel tower)
    {
        IReadOnlyList<TowerDefinition> path = tower.Definition.UpgradePath;
        if (tower.UpgradeLevel >= path.Count)
        {
            return false;
        }

        TowerDefinition nextLevel = path[tower.UpgradeLevel];
        int upgradeCost = nextLevel.Cost;

        if (m_Economy.Currency.Value < upgradeCost)
        {
            return false;
        }

        m_Economy.Currency.Value -= upgradeCost;
        tower.Definition = nextLevel;
        tower.UpgradeLevel++;
        m_UpgradedPublisher.Publish(new TowerUpgradedMessage(tower));
        return true;
    }

    public int SellTower(TowerModel tower)
    {
        int refund = tower.Definition.SellValue;
        m_Economy.Currency.Value += refund;
        return refund;
    }

    public void Dispose() { }
}

public readonly struct TowerUpgradedMessage
{
    public readonly TowerModel Tower;
    public TowerUpgradedMessage(TowerModel tower) { Tower = tower; }
}
```

---

## Projectile System

Projectiles are pooled. Never call `Instantiate` or `Destroy` at runtime. Use `ObjectPool<T>`.

### Projectile Definition

```csharp
public enum ProjectileType { Homing, Straight, AoE }

[CreateAssetMenu(menuName = "TD/Projectile Definition")]
public sealed class ProjectileDefinition : ScriptableObject
{
    [SerializeField] private ProjectileType m_Type;
    [SerializeField] private float m_Speed;
    [SerializeField] private float m_SplashRadius;
    [SerializeField] private GameObject m_Prefab;
    [SerializeField] private float m_SlowPercent;
    [SerializeField] private float m_SlowDuration;

    public ProjectileType Type => m_Type;
    public float Speed => m_Speed;
    public float SplashRadius => m_SplashRadius;
    public GameObject Prefab => m_Prefab;
    public float SlowPercent => m_SlowPercent;
    public float SlowDuration => m_SlowDuration;
}
```

### Projectile Model

```csharp
public sealed class ProjectileModel
{
    public ProjectileDefinition Definition { get; set; }
    public Vector3 Position { get; set; }
    public Vector3 Direction { get; set; }
    public EnemyModel Target { get; set; }
    public float Damage { get; set; }
    public bool IsActive { get; set; }
}
```

### Projectile System

```csharp
public sealed class ProjectileSystem : ITickable, IDisposable
{
    private readonly List<ProjectileModel> m_ActiveProjectiles = new();
    private readonly EnemyRegistry m_EnemyRegistry;
    private readonly IPublisher<EnemyKilledMessage> m_KilledPublisher;
    private static readonly float k_HitDistanceSqr = 0.25f;

    [Inject]
    public ProjectileSystem(
        EnemyRegistry enemyRegistry,
        IPublisher<EnemyKilledMessage> killedPublisher)
    {
        m_EnemyRegistry = enemyRegistry;
        m_KilledPublisher = killedPublisher;
    }

    public void SpawnProjectile(ProjectileDefinition definition, Vector3 origin, EnemyModel target, float damage)
    {
        var projectile = new ProjectileModel
        {
            Definition = definition,
            Position = origin,
            Target = target,
            Damage = damage,
            Direction = (target.Position - origin).normalized,
            IsActive = true
        };
        m_ActiveProjectiles.Add(projectile);
    }

    public void Tick()
    {
        float dt = Time.deltaTime;

        for (int projectileIndex = m_ActiveProjectiles.Count - 1; projectileIndex >= 0; projectileIndex--)
        {
            ProjectileModel projectile = m_ActiveProjectiles[projectileIndex];
            if (!projectile.IsActive)
            {
                m_ActiveProjectiles.RemoveAt(projectileIndex);
                continue;
            }

            MoveProjectile(projectile, dt);
            CheckHit(projectile);
        }
    }

    private void MoveProjectile(ProjectileModel projectile, float dt)
    {
        float speed = projectile.Definition.Speed * dt;

        if (projectile.Definition.Type == ProjectileType.Homing && projectile.Target != null && !projectile.Target.IsDead)
        {
            projectile.Direction = (projectile.Target.Position - projectile.Position).normalized;
        }

        projectile.Position += projectile.Direction * speed;
    }

    private void CheckHit(ProjectileModel projectile)
    {
        if (projectile.Target == null || projectile.Target.IsDead)
        {
            // Target died mid-flight: straight projectiles continue, homing deactivate
            if (projectile.Definition.Type == ProjectileType.Homing)
            {
                projectile.IsActive = false;
            }
            return;
        }

        float distSqr = (projectile.Position - projectile.Target.Position).sqrMagnitude;
        if (distSqr > k_HitDistanceSqr)
        {
            return;
        }

        if (projectile.Definition.Type == ProjectileType.AoE)
        {
            ApplySplashDamage(projectile);
        }
        else
        {
            ApplyDamage(projectile.Target, projectile);
        }

        projectile.IsActive = false;
    }

    private void ApplySplashDamage(ProjectileModel projectile)
    {
        float splashRadiusSqr = projectile.Definition.SplashRadius * projectile.Definition.SplashRadius;
        ReadOnlySpan<EnemyModel> enemies = m_EnemyRegistry.ActiveEnemies;

        for (int enemyIndex = 0; enemyIndex < enemies.Length; enemyIndex++)
        {
            EnemyModel enemy = enemies[enemyIndex];
            if (enemy.IsDead)
            {
                continue;
            }

            float distSqr = (enemy.Position - projectile.Position).sqrMagnitude;
            if (distSqr <= splashRadiusSqr)
            {
                ApplyDamage(enemy, projectile);
            }
        }
    }

    private void ApplyDamage(EnemyModel enemy, ProjectileModel projectile)
    {
        enemy.Health -= projectile.Damage;

        if (projectile.Definition.SlowPercent > 0f)
        {
            enemy.SpeedMultiplier = 1f - projectile.Definition.SlowPercent;
            // SlowDuration handling would be managed by a StatusEffectSystem
        }

        if (enemy.IsDead)
        {
            m_KilledPublisher.Publish(new EnemyKilledMessage(enemy));
        }
    }

    public void Dispose() { }
}
```

---

## UI Patterns

### Tower Selection

Use a radial menu or bottom toolbar showing available towers. Each button displays the tower icon, name, and cost. Gray out towers the player cannot afford.

```csharp
public sealed class TowerButtonView : MonoBehaviour
{
    [SerializeField] private Image m_Icon;
    [SerializeField] private TMP_Text m_CostLabel;
    [SerializeField] private Button m_Button;
    [SerializeField] private CanvasGroup m_CanvasGroup;

    private TowerDefinition m_Definition;
    private EconomyModel m_Economy;

    [Inject]
    public void Construct(EconomyModel economy)
    {
        m_Economy = economy;
    }

    public void SetTower(TowerDefinition definition)
    {
        m_Definition = definition;
        m_CostLabel.text = definition.Cost.ToString();
    }

    private void Update()
    {
        bool canAfford = m_Economy.Currency.Value >= m_Definition.Cost;
        m_CanvasGroup.alpha = canAfford ? 1f : 0.4f;
        m_Button.interactable = canAfford;
    }
}
```

### Range Indicator

When selecting a tower (either for placement or when clicking an existing tower), draw a circle on the ground showing its range. Use a projector or a flat mesh scaled to `range * 2` diameter.

### Enemy Health Bars

Use a world-space canvas with a slider per enemy. Pool the health bar UI elements along with the enemy GameObjects. Update the slider value from `EnemyModel.Health / EnemyModel.MaxHealth` in the View.

### Wave Counter

Display current wave and total waves: "Wave 3 / 15". Subscribe to `WaveModel.CurrentWaveIndex` with `ReactiveProperty.Subscribe` in the View.

---

## Common Pitfalls

**Target switching mid-shot.** If a tower re-acquires a new target every tick, projectiles already in flight toward the old target look wrong. Solution: lock the tower's target until the projectile hits or the target dies/exits range. Only then re-acquire.

**Projectile hitting dead enemy.** Multiple towers can fire at the same enemy. By the time the second projectile arrives, the enemy is already dead. Solution: in `CheckHit`, if the target is dead, either pass through (straight projectiles) or deactivate (homing). Never deal damage to a dead enemy.

**Wave spawn before previous cleared.** If auto-start triggers the next wave while enemies from the last wave are still alive, difficulty spikes unexpectedly. Solution: track `EnemiesRemainingInWave` and only start the next wave when it hits zero (or add a deliberate overlap for harder modes).

**Placement on occupied cell.** Race condition when the player clicks rapidly. Solution: `PlacementSystem.TryPlace` atomically checks and sets the cell state in a single call. The grid model is the single source of truth.

**Selling and re-placing exploit.** If sell refund is 100%, players can freely rearrange towers. Solution: always use a sell penalty (70% is standard). Track total investment per tower including upgrades.

---

## Performance

### Spatial Partitioning for Target Acquisition

When tower count and enemy count are both high (50+ towers, 200+ enemies), brute-force `O(towers * enemies)` target acquisition becomes expensive. Use a spatial hash grid:

```csharp
public sealed class SpatialHash
{
    private readonly Dictionary<int, List<EnemyModel>> m_Cells = new();
    private readonly float m_CellSize;

    public SpatialHash(float cellSize)
    {
        m_CellSize = cellSize;
    }

    public int GetKey(Vector3 position)
    {
        int x = Mathf.FloorToInt(position.x / m_CellSize);
        int z = Mathf.FloorToInt(position.z / m_CellSize);
        return x * 73856093 ^ z * 19349663;
    }

    public void Clear()
    {
        foreach (KeyValuePair<int, List<EnemyModel>> kvp in m_Cells)
        {
            kvp.Value.Clear();
        }
    }

    public void Insert(EnemyModel enemy)
    {
        int key = GetKey(enemy.Position);
        if (!m_Cells.TryGetValue(key, out List<EnemyModel> list))
        {
            list = new List<EnemyModel>(16);
            m_Cells[key] = list;
        }
        list.Add(enemy);
    }
}
```

Rebuild the spatial hash once per frame, then query only cells within tower range.

### Staggered Tower Updates

Not every tower needs to check for targets every frame. Distribute towers across frames:

```csharp
// In TowerSystem.Tick():
int towerCountThisFrame = Mathf.CeilToInt(m_Towers.Count / 3f);
int startIndex = (m_FrameCounter % 3) * towerCountThisFrame;
int endIndex = Mathf.Min(startIndex + towerCountThisFrame, m_Towers.Count);
m_FrameCounter++;
```

This spreads the cost over 3 frames. Towers still fire at the correct rate because cooldown timers tick every frame.

### Enemy LOD

Enemies far from the camera can use simpler meshes and skip particle effects. Use Unity LOD groups or manual distance checks. Enemies beyond the camera frustum can skip visual updates entirely while their model still moves along the path.

### Pool Everything

Pre-warm pools for enemies, projectiles, health bars, and VFX. Size pools to the maximum expected count per wave. Never call `Instantiate` or `Destroy` during gameplay.
