---
name: idle-clicker
description: "Idle/clicker game architecture — big number math, offline progress, prestige/rebirth, upgrade trees, automation, currency systems, time-based rewards."
globs: ["**/Idle*.cs", "**/Clicker*.cs", "**/Currency*.cs", "**/Upgrade*.cs", "**/Prestige*.cs"]
---

# Idle / Clicker Game Patterns

## Big Number System

Standard float/double loses precision at large values. Use a custom big number or double with formatted display.

```csharp
public readonly struct BigNumber
{
    private readonly double m_Value;
    private readonly int m_Exponent;

    public BigNumber(double value, int exponent = 0)
    {
        m_Value = value;
        m_Exponent = exponent;
        // Normalize would be called here
    }

    public static BigNumber operator +(BigNumber a, BigNumber b)
    {
        if (a.m_Exponent == b.m_Exponent)
        {
            return new BigNumber(a.m_Value + b.m_Value, a.m_Exponent);
        }
        int diff = a.m_Exponent - b.m_Exponent;
        if (diff > 15) return a; // b is negligible
        if (diff < -15) return b;
        if (diff > 0)
        {
            return new BigNumber(a.m_Value + b.m_Value / System.Math.Pow(10, diff), a.m_Exponent);
        }
        return new BigNumber(b.m_Value + a.m_Value / System.Math.Pow(10, -diff), b.m_Exponent);
    }

    public string ToFormattedString()
    {
        // 1.23K, 4.56M, 7.89B, 1.23T, etc.
        string[] suffixes = { "", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No", "Dc" };
        int tier = m_Exponent / 3;
        if (tier < suffixes.Length)
        {
            double displayValue = m_Value * System.Math.Pow(10, m_Exponent % 3);
            return $"{displayValue:F2}{suffixes[tier]}";
        }
        return $"{m_Value:F2}e{m_Exponent}";
    }
}
```

## Currency System

```csharp
[CreateAssetMenu(menuName = "Idle/Currency Definition")]
public sealed class CurrencyDefinition : ScriptableObject
{
    [SerializeField] private string m_CurrencyId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private Sprite m_Icon;

    public string CurrencyId => m_CurrencyId;
    public string DisplayName => m_DisplayName;
    public Sprite Icon => m_Icon;
}

public sealed class CurrencyManager : MonoBehaviour
{
    private readonly Dictionary<string, double> m_Currencies = new();

    public event System.Action<string, double> OnCurrencyChanged;

    public double GetAmount(string currencyId)
    {
        m_Currencies.TryGetValue(currencyId, out double amount);
        return amount;
    }

    public bool CanAfford(string currencyId, double cost)
    {
        return GetAmount(currencyId) >= cost;
    }

    public void Add(string currencyId, double amount)
    {
        if (!m_Currencies.ContainsKey(currencyId))
        {
            m_Currencies[currencyId] = 0;
        }
        m_Currencies[currencyId] += amount;
        OnCurrencyChanged?.Invoke(currencyId, m_Currencies[currencyId]);
    }

    public bool Spend(string currencyId, double cost)
    {
        if (!CanAfford(currencyId, cost)) return false;
        m_Currencies[currencyId] -= cost;
        OnCurrencyChanged?.Invoke(currencyId, m_Currencies[currencyId]);
        return true;
    }
}
```

## Upgrade System

```csharp
[CreateAssetMenu(menuName = "Idle/Upgrade Definition")]
public sealed class UpgradeDefinition : ScriptableObject
{
    [SerializeField] private string m_UpgradeId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private string m_CurrencyId;
    [SerializeField] private double m_BaseCost = 10;
    [SerializeField] private float m_CostMultiplier = 1.15f;
    [SerializeField] private int m_MaxLevel = -1; // -1 = unlimited
    [SerializeField] private double m_BaseEffect = 1;
    [SerializeField] private float m_EffectMultiplier = 1.0f;

    public string UpgradeId => m_UpgradeId;
    public string DisplayName => m_DisplayName;

    public double GetCost(int currentLevel)
    {
        return m_BaseCost * System.Math.Pow(m_CostMultiplier, currentLevel);
    }

    public double GetEffect(int currentLevel)
    {
        return m_BaseEffect + (m_EffectMultiplier * currentLevel);
    }

    public bool IsMaxed(int currentLevel)
    {
        return m_MaxLevel >= 0 && currentLevel >= m_MaxLevel;
    }
}
```

## Offline Progress

```csharp
public sealed class OfflineProgressCalculator : MonoBehaviour
{
    [SerializeField] private float m_OfflineEfficiency = 0.5f; // 50% of online rate
    [SerializeField] private float m_MaxOfflineHours = 8f;

    private const string k_LastPlayedKey = "LastPlayedTime";

    public double CalculateOfflineEarnings(double perSecondRate)
    {
        string lastPlayed = PlayerPrefs.GetString(k_LastPlayedKey, "");
        if (string.IsNullOrEmpty(lastPlayed)) return 0;

        long binary = long.Parse(lastPlayed);
        System.DateTime lastTime = System.DateTime.FromBinary(binary);
        System.TimeSpan elapsed = System.DateTime.UtcNow - lastTime;

        double seconds = System.Math.Min(elapsed.TotalSeconds, m_MaxOfflineHours * 3600);
        return perSecondRate * seconds * m_OfflineEfficiency;
    }

    public void SaveExitTime()
    {
        PlayerPrefs.SetString(k_LastPlayedKey, System.DateTime.UtcNow.ToBinary().ToString());
        PlayerPrefs.Save();
    }

    private void OnApplicationPause(bool paused)
    {
        if (paused) SaveExitTime();
    }

    private void OnApplicationQuit()
    {
        SaveExitTime();
    }
}
```

## Prestige / Rebirth System

```csharp
public sealed class PrestigeSystem : MonoBehaviour
{
    [SerializeField] private double m_PrestigeThreshold = 1000000;
    [SerializeField] private string m_PrimaryCurrencyId = "gold";
    [SerializeField] private string m_PrestigeCurrencyId = "gems";

    [SerializeField] private CurrencyManager m_CurrencyManager;

    public int PrestigeCount { get; private set; }

    public double CalculatePrestigeReward()
    {
        double total = m_CurrencyManager.GetAmount(m_PrimaryCurrencyId);
        if (total < m_PrestigeThreshold) return 0;
        return System.Math.Floor(System.Math.Sqrt(total / m_PrestigeThreshold));
    }

    public bool CanPrestige()
    {
        return CalculatePrestigeReward() > 0;
    }

    public void DoPrestige()
    {
        double reward = CalculatePrestigeReward();
        if (reward <= 0) return;

        // Award prestige currency
        m_CurrencyManager.Add(m_PrestigeCurrencyId, reward);
        PrestigeCount++;

        // Reset primary progress
        ResetProgress();
    }

    private void ResetProgress()
    {
        // Reset primary currency, upgrades, and generators
        // Keep prestige currency and prestige upgrades
    }
}
```

## Game Loop

```
Launch -> Check offline progress -> Show earnings popup
    -> Main screen (tap to earn + automated income)
        -> Buy upgrades (increase tap/auto income)
            -> Prestige when progress slows
                -> Restart with permanent bonuses
```

## Key Design Rules

- **Exponential growth** -- costs and rewards both scale exponentially
- **Multiple income sources** -- tap income, auto generators, prestige bonuses
- **Clear next goal** -- always show what the player is working toward
- **Satisfying numbers** -- big numbers going up is the core reward
- **Offline progress** -- player must feel rewarded for coming back
- **No skill required** -- progress is time + decisions, not reflexes

## Mobile-Specific

- **Battery friendly** -- cap at 30fps, reduce Update frequency for idle generators
- **Background handling** -- `OnApplicationPause` saves state immediately
- **Notification hooks** -- "Your generators have earned 1M gold!" after offline period
- **Minimal UI updates** -- update currency display every 0.1s, not every frame

---

## Tap Detection and Visual Feedback

### Tap Zone with Input Buffering

Use the new Input System for tap detection. Buffer rapid taps so none are lost during frame spikes.

```csharp
public sealed class TapInputView : MonoBehaviour
{
    [SerializeField] private int m_MaxBufferedTaps = 8;

    private TapSystem m_TapSystem;
    private int m_BufferedTapCount;

    [Inject]
    public void Construct(TapSystem tapSystem)
    {
        m_TapSystem = tapSystem;
    }

    // Called by Input System event or pointer handler
    public void OnTap()
    {
        if (m_BufferedTapCount < m_MaxBufferedTaps)
        {
            m_BufferedTapCount++;
        }
    }

    private void Update()
    {
        // Drain tap buffer each frame — system processes all buffered taps
        while (m_BufferedTapCount > 0)
        {
            m_TapSystem.ProcessTap();
            m_BufferedTapCount--;
        }
    }
}
```

### Scale Pop Animation

Manual lerp approach for zero-allocation tap feedback. No external tweening dependency required.

```csharp
public sealed class TapPopView : MonoBehaviour
{
    [SerializeField] private RectTransform m_TapTarget;
    [SerializeField] private float m_PopScale = 1.15f;
    [SerializeField] private float m_PopSpeed = 12f;

    private Vector3 m_BaseScale;
    private float m_PopTimer;

    private void Awake()
    {
        m_BaseScale = m_TapTarget.localScale;
    }

    public void TriggerPop()
    {
        m_PopTimer = 1f;
    }

    private void Update()
    {
        if (m_PopTimer > 0f)
        {
            m_PopTimer -= Time.deltaTime * m_PopSpeed;
            float scale = Mathf.Lerp(1f, m_PopScale, m_PopTimer);
            m_TapTarget.localScale = m_BaseScale * scale;
        }
    }
}
```

### Floating Damage Number Pooling

Pool floating text objects. Each shows the tap value, floats upward, fades, then returns to pool.

```csharp
public sealed class FloatingNumberView : MonoBehaviour
{
    [SerializeField] private TMPro.TextMeshProUGUI m_Text;
    [SerializeField] private CanvasGroup m_CanvasGroup;
    [SerializeField] private float m_FloatSpeed = 80f;
    [SerializeField] private float m_Lifetime = 0.8f;

    private RectTransform m_RectTransform;
    private float m_Timer;
    private System.Action<FloatingNumberView> m_ReturnToPool;

    private void Awake()
    {
        m_RectTransform = GetComponent<RectTransform>();
    }

    public void Show(string text, Vector2 position, System.Action<FloatingNumberView> returnCallback)
    {
        m_Text.text = text;
        m_RectTransform.anchoredPosition = position;
        m_CanvasGroup.alpha = 1f;
        m_Timer = m_Lifetime;
        m_ReturnToPool = returnCallback;
        gameObject.SetActive(true);
    }

    private void Update()
    {
        m_Timer -= Time.deltaTime;
        m_RectTransform.anchoredPosition += Vector2.up * (m_FloatSpeed * Time.deltaTime);
        m_CanvasGroup.alpha = Mathf.Clamp01(m_Timer / m_Lifetime);

        if (m_Timer <= 0f)
        {
            gameObject.SetActive(false);
            m_ReturnToPool?.Invoke(this);
        }
    }
}
```

### Multi-Tap Combo Detection

Track rapid consecutive taps. After a threshold, activate a combo multiplier that decays if tapping slows.

```csharp
public sealed class TapComboSystem : IDisposable
{
    private readonly TapComboConfig m_Config;
    private readonly IPublisher<ComboChangedMessage> m_ComboPublisher;

    private int m_ComboCount;
    private float m_ComboTimer;

    [Inject]
    public TapComboSystem(TapComboConfig config, IPublisher<ComboChangedMessage> comboPublisher)
    {
        m_Config = config;
        m_ComboPublisher = comboPublisher;
    }

    public int ComboMultiplier => m_ComboCount >= m_Config.ComboThreshold
        ? 1 + (m_ComboCount / m_Config.ComboThreshold)
        : 1;

    public void RegisterTap()
    {
        m_ComboCount++;
        m_ComboTimer = m_Config.ComboWindow;
        m_ComboPublisher.Publish(new ComboChangedMessage(m_ComboCount, ComboMultiplier));
    }

    public void Tick(float deltaTime)
    {
        if (m_ComboCount <= 0) return;
        m_ComboTimer -= deltaTime;
        if (m_ComboTimer <= 0f)
        {
            m_ComboCount = 0;
            m_ComboPublisher.Publish(new ComboChangedMessage(0, 1));
        }
    }

    public void Dispose() { }
}

public readonly struct ComboChangedMessage
{
    public readonly int ComboCount;
    public readonly int Multiplier;
    public ComboChangedMessage(int comboCount, int multiplier)
    {
        ComboCount = comboCount;
        Multiplier = multiplier;
    }
}
```

---

## Generator System Architecture

### Generator Model

```csharp
public sealed class GeneratorModel
{
    public string GeneratorId;
    public int Level;
    public double BaseOutput;          // output per second at level 1
    public double OutputMultiplier;    // from upgrades, prestige, managers
    public bool IsAutomated;           // true = earns without tapping
    public bool IsUnlocked;

    public double CurrentOutput => BaseOutput * Level * OutputMultiplier;
}
```

### Generator System with Batch Updates

Process all generators in a single pass per frame. Never iterate per-generator in separate Update calls.

```csharp
public sealed class GeneratorSystem : ITickable, IDisposable
{
    private readonly GeneratorModel[] m_Generators;
    private readonly int m_GeneratorCount;
    private readonly CurrencySystem m_CurrencySystem;
    private readonly IPublisher<IncomeTickMessage> m_IncomePublisher;

    private double m_CachedIncomePerSecond;
    private bool m_IncomeDirty = true;

    [Inject]
    public GeneratorSystem(
        GeneratorModel[] generators,
        CurrencySystem currencySystem,
        IPublisher<IncomeTickMessage> incomePublisher)
    {
        m_Generators = generators;
        m_GeneratorCount = generators.Length;
        m_CurrencySystem = currencySystem;
        m_IncomePublisher = incomePublisher;
    }

    public double IncomePerSecond
    {
        get
        {
            if (m_IncomeDirty)
            {
                RecalculateIncome();
            }
            return m_CachedIncomePerSecond;
        }
    }

    private void RecalculateIncome()
    {
        m_CachedIncomePerSecond = 0;
        for (int generatorIndex = 0; generatorIndex < m_GeneratorCount; generatorIndex++)
        {
            GeneratorModel generator = m_Generators[generatorIndex];
            if (generator.IsUnlocked && generator.IsAutomated)
            {
                m_CachedIncomePerSecond += generator.CurrentOutput;
            }
        }
        m_IncomeDirty = false;
    }

    public void MarkDirty() => m_IncomeDirty = true;

    // Called once per frame by VContainer's ITickable
    public void Tick()
    {
        double earned = IncomePerSecond * Time.deltaTime;
        if (earned > 0)
        {
            m_CurrencySystem.Add("gold", earned);
            m_IncomePublisher.Publish(new IncomeTickMessage(earned));
        }
    }

    public bool TryUpgrade(int generatorIndex, CurrencySystem currency)
    {
        GeneratorModel generator = m_Generators[generatorIndex];
        double cost = GetUpgradeCost(generator);
        if (!currency.CanAfford("gold", cost)) return false;

        currency.Spend("gold", cost);
        generator.Level++;
        m_IncomeDirty = true;

        if (!generator.IsUnlocked)
        {
            generator.IsUnlocked = true;
        }

        return true;
    }

    public double GetUpgradeCost(GeneratorModel generator)
    {
        // Standard idle cost formula: baseCost * multiplier^level
        return generator.BaseOutput * 10.0 * System.Math.Pow(1.15, generator.Level);
    }

    public void Dispose() { }
}

public readonly struct IncomeTickMessage
{
    public readonly double Amount;
    public IncomeTickMessage(double amount) { Amount = amount; }
}
```

### Generator Unlock Progression

Generators unlock sequentially based on total earnings thresholds. The UI shows the next locked generator and its unlock cost as a goal for the player.

```csharp
[CreateAssetMenu(menuName = "Idle/Generator Definition")]
public sealed class GeneratorDefinition : ScriptableObject
{
    [SerializeField] private string m_GeneratorId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private double m_BaseOutput = 1;
    [SerializeField] private double m_UnlockCost = 100;
    [SerializeField] private Sprite m_Icon;

    public string GeneratorId => m_GeneratorId;
    public string DisplayName => m_DisplayName;
    public double BaseOutput => m_BaseOutput;
    public double UnlockCost => m_UnlockCost;
    public Sprite Icon => m_Icon;
}
```

### Manager / Automation Upgrade

Once a player buys a "manager" for a generator, that generator produces income automatically without tapping. This is a boolean flag on the GeneratorModel toggled by a one-time purchase.

---

## Prestige Tree and Permanent Upgrades

### Prestige Currency Calculation

The prestige reward formula should feel generous enough to motivate a reset but not so generous that one prestige trivializes the next run.

```csharp
public sealed class PrestigeCalculator
{
    private const double k_BaseThreshold = 1e6;
    private const double k_ScalingExponent = 0.5;

    public double CalculatePrestigeCurrency(double totalLifetimeEarnings)
    {
        if (totalLifetimeEarnings < k_BaseThreshold) return 0;
        return System.Math.Floor(
            System.Math.Pow(totalLifetimeEarnings / k_BaseThreshold, k_ScalingExponent)
        );
    }
}
```

### Permanent Multiplier System

Prestige upgrades persist across resets and multiply all income sources.

```csharp
public sealed class PrestigeUpgradeModel
{
    public string UpgradeId;
    public int Level;
    public int MaxLevel;
    public double CostPerLevel;
    public double MultiplierPerLevel;  // e.g., 0.05 = +5% per level

    public double TotalMultiplier => 1.0 + (MultiplierPerLevel * Level);

    public double GetCost()
    {
        return CostPerLevel * System.Math.Pow(1.5, Level);
    }
}

public sealed class PrestigeUpgradeSystem : IDisposable
{
    private readonly PrestigeUpgradeModel[] m_Upgrades;
    private readonly CurrencySystem m_CurrencySystem;
    private readonly GeneratorSystem m_GeneratorSystem;
    private readonly string m_PrestigeCurrencyId;

    [Inject]
    public PrestigeUpgradeSystem(
        PrestigeUpgradeModel[] upgrades,
        CurrencySystem currencySystem,
        GeneratorSystem generatorSystem)
    {
        m_Upgrades = upgrades;
        m_CurrencySystem = currencySystem;
        m_GeneratorSystem = generatorSystem;
        m_PrestigeCurrencyId = "prestige_points";
    }

    public bool TryPurchase(int upgradeIndex)
    {
        PrestigeUpgradeModel upgrade = m_Upgrades[upgradeIndex];
        if (upgrade.Level >= upgrade.MaxLevel) return false;

        double cost = upgrade.GetCost();
        if (!m_CurrencySystem.CanAfford(m_PrestigeCurrencyId, cost)) return false;

        m_CurrencySystem.Spend(m_PrestigeCurrencyId, cost);
        upgrade.Level++;
        m_GeneratorSystem.MarkDirty();
        return true;
    }

    public double GetTotalPrestigeMultiplier()
    {
        double multiplier = 1.0;
        for (int upgradeIndex = 0; upgradeIndex < m_Upgrades.Length; upgradeIndex++)
        {
            multiplier *= m_Upgrades[upgradeIndex].TotalMultiplier;
        }
        return multiplier;
    }

    public void Dispose() { }
}
```

### Prestige Skill Tree

For branching prestige trees, model each node with prerequisites:

```csharp
[System.Serializable]
public struct PrestigeNodeDefinition
{
    public string NodeId;
    public string DisplayName;
    public string Description;
    public string[] PrerequisiteNodeIds;  // must all be purchased first
    public int MaxLevel;
    public double BaseCost;
    public float CostScaling;
    public PrestigeNodeEffect Effect;
}

public enum PrestigeNodeEffect
{
    GlobalIncomeMultiplier,
    TapDamageMultiplier,
    GeneratorCostReduction,
    OfflineEfficiencyBoost,
    StartingCurrencyBonus
}
```

The UI renders the tree as a graph. Nodes light up when their prerequisites are met. Each run feels meaningfully faster because the cumulative prestige multiplier compounds.

---

## Event and Milestone System

### Milestone Definitions

```csharp
[CreateAssetMenu(menuName = "Idle/Milestone Definition")]
public sealed class MilestoneDefinition : ScriptableObject
{
    [SerializeField] private string m_MilestoneId;
    [SerializeField] private string m_DisplayName;
    [SerializeField] private string m_Description;
    [SerializeField] private MilestoneCondition m_Condition;
    [SerializeField] private double m_TargetValue;
    [SerializeField] private MilestoneReward m_Reward;

    public string MilestoneId => m_MilestoneId;
    public string DisplayName => m_DisplayName;
    public string Description => m_Description;
    public MilestoneCondition Condition => m_Condition;
    public double TargetValue => m_TargetValue;
    public MilestoneReward Reward => m_Reward;
}

public enum MilestoneCondition
{
    TotalEarnings,
    TotalTaps,
    GeneratorLevel,
    PrestigeCount,
    TotalPlayTime
}

[System.Serializable]
public struct MilestoneReward
{
    public string CurrencyId;
    public double Amount;
}
```

### Milestone Tracking System

```csharp
public sealed class MilestoneSystem : ITickable, IDisposable
{
    private readonly MilestoneDefinition[] m_Definitions;
    private readonly bool[] m_Completed;
    private readonly IPublisher<MilestoneUnlockedMessage> m_Publisher;
    private readonly PlayerStatsModel m_Stats;

    // Check milestones at reduced frequency — not every frame
    private float m_CheckTimer;
    private const float k_CheckInterval = 0.5f;

    [Inject]
    public MilestoneSystem(
        MilestoneDefinition[] definitions,
        PlayerStatsModel stats,
        IPublisher<MilestoneUnlockedMessage> publisher)
    {
        m_Definitions = definitions;
        m_Completed = new bool[definitions.Length];
        m_Stats = stats;
        m_Publisher = publisher;
    }

    public void Tick()
    {
        m_CheckTimer -= Time.deltaTime;
        if (m_CheckTimer > 0f) return;
        m_CheckTimer = k_CheckInterval;

        for (int milestoneIndex = 0; milestoneIndex < m_Definitions.Length; milestoneIndex++)
        {
            if (m_Completed[milestoneIndex]) continue;

            MilestoneDefinition definition = m_Definitions[milestoneIndex];
            double currentValue = GetCurrentValue(definition.Condition);

            if (currentValue >= definition.TargetValue)
            {
                m_Completed[milestoneIndex] = true;
                m_Publisher.Publish(new MilestoneUnlockedMessage(definition));
            }
        }
    }

    private double GetCurrentValue(MilestoneCondition condition)
    {
        return condition switch
        {
            MilestoneCondition.TotalEarnings => m_Stats.TotalEarnings,
            MilestoneCondition.TotalTaps => m_Stats.TotalTaps,
            MilestoneCondition.PrestigeCount => m_Stats.PrestigeCount,
            MilestoneCondition.TotalPlayTime => m_Stats.TotalPlayTimeSeconds,
            _ => 0
        };
    }

    public float GetProgress(int milestoneIndex)
    {
        if (m_Completed[milestoneIndex]) return 1f;
        MilestoneDefinition definition = m_Definitions[milestoneIndex];
        double current = GetCurrentValue(definition.Condition);
        return (float)(current / definition.TargetValue);
    }

    public void Dispose() { }
}

public readonly struct MilestoneUnlockedMessage
{
    public readonly MilestoneDefinition Definition;
    public MilestoneUnlockedMessage(MilestoneDefinition definition) { Definition = definition; }
}
```

### Notification Queue with Priority

Milestones, achievements, and events all push to a shared notification queue. The UI pops them one at a time with a delay between each.

```csharp
public sealed class NotificationSystem : IDisposable
{
    private readonly List<NotificationEntry> m_Queue = new(16);
    private readonly IPublisher<ShowNotificationMessage> m_ShowPublisher;

    private float m_CooldownTimer;
    private const float k_DisplayInterval = 1.5f;

    [Inject]
    public NotificationSystem(IPublisher<ShowNotificationMessage> showPublisher)
    {
        m_ShowPublisher = showPublisher;
    }

    public void Enqueue(string title, string body, int priority)
    {
        m_Queue.Add(new NotificationEntry(title, body, priority));
        // Sort by priority descending — higher priority shows first
        m_Queue.Sort((a, b) => b.Priority.CompareTo(a.Priority));
    }

    public void Tick(float deltaTime)
    {
        if (m_Queue.Count == 0) return;
        m_CooldownTimer -= deltaTime;
        if (m_CooldownTimer > 0f) return;

        NotificationEntry entry = m_Queue[0];
        m_Queue.RemoveAt(0);
        m_ShowPublisher.Publish(new ShowNotificationMessage(entry.Title, entry.Body));
        m_CooldownTimer = k_DisplayInterval;
    }

    public void Dispose() { }
}

public readonly struct NotificationEntry
{
    public readonly string Title;
    public readonly string Body;
    public readonly int Priority;

    public NotificationEntry(string title, string body, int priority)
    {
        Title = title;
        Body = body;
        Priority = priority;
    }
}

public readonly struct ShowNotificationMessage
{
    public readonly string Title;
    public readonly string Body;
    public ShowNotificationMessage(string title, string body) { Title = title; Body = body; }
}
```

### Daily and Weekly Challenges

Model challenges as time-limited milestones with a reset timer. Store the challenge seed and expiry timestamp. On load, check if the current challenge has expired and generate a new one.

---

## Save Integration

### Serializing Game State

Use JSON serialization for all mutable state. Keep the save model flat and separate from runtime models.

```csharp
[System.Serializable]
public sealed class IdleSaveData
{
    public double PrimaryCurrency;
    public double PrestigeCurrency;
    public int PrestigeCount;
    public GeneratorSaveData[] Generators;
    public int[] PrestigeUpgradeLevels;
    public bool[] CompletedMilestones;
    public long LastSaveTimestamp; // UTC ticks
    public int SaveVersion;
}

[System.Serializable]
public struct GeneratorSaveData
{
    public string GeneratorId;
    public int Level;
    public bool IsAutomated;
}
```

### Offline Earnings on Load

```csharp
public sealed class OfflineEarningsSystem
{
    private const double k_MaxOfflineSeconds = 28800; // 8 hours
    private const double k_OfflineEfficiency = 0.5;

    public double CalculateOfflineEarnings(IdleSaveData saveData, double incomePerSecond)
    {
        long nowTicks = System.DateTime.UtcNow.Ticks;
        double elapsedSeconds = (nowTicks - saveData.LastSaveTimestamp) / (double)System.TimeSpan.TicksPerSecond;

        double cappedSeconds = System.Math.Min(elapsedSeconds, k_MaxOfflineSeconds);
        if (cappedSeconds < 1.0) return 0;

        return incomePerSecond * cappedSeconds * k_OfflineEfficiency;
    }
}
```

### Anti-Cheat: Server Time Validation

Never trust the device clock for offline earnings. If you have a server, fetch the timestamp from there. For offline-only games, compare against the last known save timestamp and reject backward jumps:

```csharp
public bool IsTimestampValid(long previousTimestamp, long currentTimestamp)
{
    // Reject if current time is before the last save (clock was set back)
    if (currentTimestamp < previousTimestamp) return false;

    // Reject if gap is unreasonably large (e.g., > 30 days)
    double gapSeconds = (currentTimestamp - previousTimestamp) / (double)System.TimeSpan.TicksPerSecond;
    if (gapSeconds > 30 * 24 * 3600) return false;

    return true;
}
```

### Save Migration for Balance Changes

When game balance changes between versions, existing saves need migration. Use a version number in the save data and apply sequential migrations.

```csharp
public sealed class SaveMigrator
{
    public IdleSaveData Migrate(IdleSaveData data)
    {
        if (data.SaveVersion < 2)
        {
            MigrateV1ToV2(data);
        }
        if (data.SaveVersion < 3)
        {
            MigrateV2ToV3(data);
        }
        return data;
    }

    private void MigrateV1ToV2(IdleSaveData data)
    {
        // Example: generator base output was rebalanced
        // Compensate existing players by adjusting their currency
        data.PrimaryCurrency *= 1.5;
        data.SaveVersion = 2;
    }

    private void MigrateV2ToV3(IdleSaveData data)
    {
        // Example: new prestige upgrades added, expand array
        if (data.PrestigeUpgradeLevels.Length < 10)
        {
            int[] expanded = new int[10];
            System.Array.Copy(data.PrestigeUpgradeLevels, expanded, data.PrestigeUpgradeLevels.Length);
            data.PrestigeUpgradeLevels = expanded;
        }
        data.SaveVersion = 3;
    }
}
```

---

## Common Pitfalls

### BigNumber Precision Loss

When using `double` as a BigNumber mantissa, precision degrades once the value exceeds approximately 2^53 (about 9e15). At that point, adding small numbers (like a single tap) has no effect because the double cannot represent the difference. Always normalize your BigNumber so the mantissa stays between 1.0 and 10.0, and track the exponent separately.

### Offline Earnings Exploit

Players can set their device clock forward to claim massive offline earnings. Mitigations:
1. Compare against the last save timestamp and reject backward jumps.
2. Cap offline duration (8 hours is standard).
3. If networking is available, validate against server time on each load.
4. Store a running hash of earnings to detect save file tampering.

### Generator Rebalancing Breaking Saves

Changing a generator's base output or cost scaling after launch will break the economy for existing players. Their generators were purchased at old prices but now produce at new rates. Always pair balance changes with save migration that compensates players fairly. Never change the cost formula without adjusting the levels already purchased.

### UI Update Frequency

With 20+ generators each producing income, updating every TextMeshPro element every frame wastes CPU. Instead:
- Update the total currency display at a fixed interval (0.1 seconds).
- Update individual generator displays only when that generator's state changes (level up, automation toggle) or when scrolled into view.
- Use a dirty flag pattern: mark the UI dirty when currency changes, then batch-refresh once per interval.

```csharp
// In the currency display view:
private float m_RefreshTimer;
private const float k_RefreshInterval = 0.1f;
private double m_LastDisplayedAmount;

private void Update()
{
    m_RefreshTimer -= Time.deltaTime;
    if (m_RefreshTimer > 0f) return;
    m_RefreshTimer = k_RefreshInterval;

    double currentAmount = m_CurrencySystem.GetAmount("gold");
    if (System.Math.Abs(currentAmount - m_LastDisplayedAmount) < 0.01) return;

    m_LastDisplayedAmount = currentAmount;
    m_CurrencyText.text = FormatNumber(currentAmount);
}
```

### Tap Rate Limiting

Players using auto-clicker tools can fire hundreds of taps per second. Cap the effective tap rate (e.g., 20 taps per second) to prevent economy exploits. Process buffered taps but discard those beyond the rate limit.
