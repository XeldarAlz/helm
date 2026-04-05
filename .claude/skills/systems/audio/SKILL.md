---
name: audio
description: "Unity audio system — AudioMixer groups, snapshots, spatial audio, audio source pooling, compression per platform."
globs: ["**/*.mixer", "**/*Audio*.cs", "**/*Sound*.cs", "**/*Music*.cs"]
---

# Audio System

## AudioMixer Setup

```
Master (exposed: "MasterVolume")
├── Music (exposed: "MusicVolume")
├── SFX (exposed: "SFXVolume")
│   ├── Weapons
│   ├── Environment
│   └── UI
└── Voice (exposed: "VoiceVolume")
```

### Volume Control via Exposed Parameters
```csharp
[SerializeField] private AudioMixer m_Mixer;

public void SetMasterVolume(float normalizedValue)
{
    // Convert 0-1 slider to decibels (-80 to 0)
    float dB = normalizedValue > 0.001f
        ? Mathf.Log10(normalizedValue) * 20f
        : -80f;
    m_Mixer.SetFloat("MasterVolume", dB);
}
```

### Snapshots
```csharp
// Transition between snapshots for ambient changes
m_UnderwaterSnapshot.TransitionTo(0.5f);  // Muffle audio underwater
m_DefaultSnapshot.TransitionTo(1.0f);      // Return to normal
```

## Playing Sounds

```csharp
// One-shot SFX (fire and forget, doesn't interrupt)
m_AudioSource.PlayOneShot(m_ExplosionClip, 0.8f);

// Music (interruptible, one at a time per source)
m_MusicSource.clip = m_BattleMusic;
m_MusicSource.Play();
```

## Audio Source Pooling

```csharp
public sealed class SFXPool : MonoBehaviour
{
    [SerializeField] private int m_PoolSize = 16;
    [SerializeField] private AudioMixerGroup m_SFXGroup;

    private AudioSource[] m_Sources;
    private int m_NextIndex;

    private void Awake()
    {
        m_Sources = new AudioSource[m_PoolSize];
        for (int i = 0; i < m_PoolSize; i++)
        {
            GameObject obj = new GameObject($"SFX_{i}");
            obj.transform.SetParent(transform);
            AudioSource source = obj.AddComponent<AudioSource>();
            source.outputAudioMixerGroup = m_SFXGroup;
            source.playOnAwake = false;
            m_Sources[i] = source;
        }
    }

    public void PlayAt(AudioClip clip, Vector3 position, float volume = 1f)
    {
        AudioSource source = m_Sources[m_NextIndex];
        m_NextIndex = (m_NextIndex + 1) % m_PoolSize;

        source.transform.position = position;
        source.spatialBlend = 1f; // 3D
        source.PlayOneShot(clip, volume);
    }
}
```

## Spatial Audio

- `spatialBlend`: 0 = 2D (UI, music), 1 = 3D (world SFX)
- `minDistance`: full volume radius
- `maxDistance`: silence radius
- `rolloffMode`: Logarithmic (realistic) or Custom (AnimationCurve)
- `spread`: 0 = point source, 360 = omnidirectional

## Compression Per Platform

| Type | Format | Load Type | Use |
|------|--------|-----------|-----|
| Music | Vorbis (quality 40-60%) | Streaming | Background music |
| SFX (short) | ADPCM | Decompress On Load | Gunshots, jumps |
| SFX (long) | Vorbis (quality 70%) | Compressed In Memory | Ambient loops |
| UI | PCM (uncompressed) | Decompress On Load | Button clicks |

## Music System Pattern

```csharp
public sealed class MusicManager : MonoBehaviour
{
    [SerializeField] private AudioSource m_SourceA;
    [SerializeField] private AudioSource m_SourceB;
    [SerializeField] private float m_CrossfadeDuration = 2f;

    private AudioSource m_ActiveSource;

    public void CrossfadeTo(AudioClip newClip)
    {
        AudioSource incoming = m_ActiveSource == m_SourceA ? m_SourceB : m_SourceA;
        incoming.clip = newClip;
        incoming.volume = 0f;
        incoming.Play();

        StartCoroutine(Crossfade(m_ActiveSource, incoming));
        m_ActiveSource = incoming;
    }

    private IEnumerator Crossfade(AudioSource outgoing, AudioSource incoming)
    {
        float elapsed = 0f;
        while (elapsed < m_CrossfadeDuration)
        {
            elapsed += Time.unscaledDeltaTime;
            float t = elapsed / m_CrossfadeDuration;
            outgoing.volume = 1f - t;
            incoming.volume = t;
            yield return null;
        }
        outgoing.Stop();
    }
}
```

## Key Rules
- One `AudioListener` per scene (usually on the camera)
- Pool AudioSources for one-shot SFX — don't create/destroy
- Use `Time.unscaledDeltaTime` for audio during pause
