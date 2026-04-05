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
Launch → Check offline progress → Show earnings popup
    → Main screen (tap to earn + automated income)
        → Buy upgrades (increase tap/auto income)
            → Prestige when progress slows
                → Restart with permanent bonuses
```

## Key Design Rules

- **Exponential growth** — costs and rewards both scale exponentially
- **Multiple income sources** — tap income, auto generators, prestige bonuses
- **Clear next goal** — always show what the player is working toward
- **Satisfying numbers** — big numbers going up is the core reward
- **Offline progress** — player must feel rewarded for coming back
- **No skill required** — progress is time + decisions, not reflexes

## Mobile-Specific

- **Battery friendly** — cap at 30fps, reduce Update frequency for idle generators
- **Background handling** — `OnApplicationPause` saves state immediately
- **Notification hooks** — "Your generators have earned 1M gold!" after offline period
- **Minimal UI updates** — update currency display every 0.1s, not every frame
