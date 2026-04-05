---
name: scriptable-objects
description: "ScriptableObject architecture patterns — event channels, variable references, runtime sets, factory pattern, data containers. The backbone of data-driven Unity architecture."
alwaysApply: true
---

# ScriptableObject Architecture

ScriptableObjects (SOs) are Unity's most powerful architectural tool. They are asset-based data containers that live outside scenes, enabling data-driven design, loose coupling, and designer-friendly workflows.

## Pattern 1: Data Container

The simplest and most common use — define game data as assets.

```csharp
[CreateAssetMenu(fileName = "NewWeapon", menuName = "Game/Weapon Definition")]
public sealed class WeaponDefinition : ScriptableObject
{
    [Header("Identity")]
    [SerializeField] private string m_DisplayName;
    [SerializeField] private Sprite m_Icon;
    [TextArea]
    [SerializeField] private string m_Description;

    [Header("Stats")]
    [SerializeField] private float m_Damage = 10f;
    [SerializeField] private float m_FireRate = 0.5f;
    [SerializeField] private int m_AmmoCapacity = 30;
    [SerializeField] private GameObject m_Prefab;

    public string DisplayName => m_DisplayName;
    public Sprite Icon => m_Icon;
    public float Damage => m_Damage;
    public float FireRate => m_FireRate;
    public int AmmoCapacity => m_AmmoCapacity;
    public GameObject Prefab => m_Prefab;
}
```

**Use for:** Items, abilities, enemy configs, level data, audio events, UI themes.

## Pattern 2: Event Channel

Zero-coupling communication between systems. No direct references needed.

```csharp
[CreateAssetMenu(fileName = "NewVoidEvent", menuName = "Events/Void Event")]
public sealed class VoidEventChannel : ScriptableObject
{
    private System.Action m_OnRaised;

    public void Raise()
    {
        m_OnRaised?.Invoke();
    }

    public void Subscribe(System.Action listener)
    {
        m_OnRaised += listener;
    }

    public void Unsubscribe(System.Action listener)
    {
        m_OnRaised -= listener;
    }
}

// Generic version for typed events
[CreateAssetMenu(fileName = "NewIntEvent", menuName = "Events/Int Event")]
public sealed class IntEventChannel : ScriptableObject
{
    private System.Action<int> m_OnRaised;

    public void Raise(int value) => m_OnRaised?.Invoke(value);
    public void Subscribe(System.Action<int> listener) => m_OnRaised += listener;
    public void Unsubscribe(System.Action<int> listener) => m_OnRaised -= listener;
}
```

**Usage:**
```csharp
// Publisher (e.g., ScoreSystem)
[SerializeField] private IntEventChannel m_OnScoreChanged;
m_OnScoreChanged.Raise(newScore);

// Subscriber (e.g., ScoreUI) — wire the SAME SO asset in inspector
[SerializeField] private IntEventChannel m_OnScoreChanged;
private void OnEnable() => m_OnScoreChanged.Subscribe(UpdateDisplay);
private void OnDisable() => m_OnScoreChanged.Unsubscribe(UpdateDisplay);
```

## Pattern 3: Variable Reference

Inspector-tweakable values that can be shared or overridden per instance.

```csharp
[System.Serializable]
public sealed class FloatReference
{
    [SerializeField] private bool m_UseConstant = true;
    [SerializeField] private float m_ConstantValue;
    [SerializeField] private FloatVariable m_Variable;

    public float Value => m_UseConstant ? m_ConstantValue : m_Variable.Value;
}

[CreateAssetMenu(fileName = "NewFloatVar", menuName = "Variables/Float")]
public sealed class FloatVariable : ScriptableObject
{
    [SerializeField] private float m_Value;
    public float Value { get => m_Value; set => m_Value = value; }
}
```

**Use for:** Player health, move speed, gravity — values that designers tweak and multiple systems read.

## Pattern 4: Runtime Set

Track all active instances of a type without FindObjectsOfType.

```csharp
[CreateAssetMenu(fileName = "NewRuntimeSet", menuName = "Sets/Transform Set")]
public sealed class TransformRuntimeSet : ScriptableObject
{
    private readonly List<Transform> m_Items = new();

    public IReadOnlyList<Transform> Items => m_Items;
    public int Count => m_Items.Count;

    public void Add(Transform item)
    {
        if (!m_Items.Contains(item))
        {
            m_Items.Add(item);
        }
    }

    public void Remove(Transform item)
    {
        m_Items.Remove(item);
    }
}

// Usage: enemies register themselves
public sealed class Enemy : MonoBehaviour
{
    [SerializeField] private TransformRuntimeSet m_EnemySet;

    private void OnEnable() => m_EnemySet.Add(transform);
    private void OnDisable() => m_EnemySet.Remove(transform);
}
```

## Pattern 5: Factory Configuration

```csharp
[CreateAssetMenu(fileName = "NewSpawnConfig", menuName = "Game/Spawn Config")]
public sealed class SpawnConfiguration : ScriptableObject
{
    [SerializeField] private GameObject m_Prefab;
    [SerializeField] private int m_PoolSize = 10;
    [SerializeField] private float m_SpawnRate = 2f;
    [SerializeField] private float m_SpawnRadius = 15f;

    public GameObject Prefab => m_Prefab;
    public int PoolSize => m_PoolSize;
    public float SpawnRate => m_SpawnRate;
    public float SpawnRadius => m_SpawnRadius;
}
```

## Anti-Patterns

1. **Mutable state not reset between plays** — SOs persist across play sessions in Editor. If you write to an SO at runtime, it keeps the value when you stop playing. Use `OnEnable` or `OnDisable` to reset, or use a separate runtime copy.

2. **SO as singleton** — Don't use `Resources.Load` to access SOs globally. Use `[SerializeField]` references or DI.

3. **Too much logic in SOs** — SOs are data and events. Keep complex logic in systems/services.

4. **Circular SO references** — SO A references SO B which references SO A. Can cause infinite loops in serialization.
