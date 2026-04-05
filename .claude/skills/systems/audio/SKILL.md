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

## Advanced Mixer Routing

### Send Effects (Reverb Bus, Delay Bus)

Create auxiliary mixer groups for shared effects. Route sound through Send effects rather than duplicating reverb on every source:

```
Master
├── Music
├── SFX
│   ├── Weapons
│   ├── Environment
│   └── UI
├── Voice
├── ReverbBus (Receive effect → SFX Send targets this)
└── DelayBus (Receive effect → specific groups send here)
```

On the SFX group, add a **Send** effect pointing to `ReverbBus`. Adjust send level per group — weapons get more reverb in indoor scenes, UI gets none.

### Ducking (Sidechain Compression)

Duck music volume when dialogue plays. On the Music group, add a **Duck Volume** effect:
- Threshold: -30 dB (triggers when Voice group exceeds this)
- Ratio: 3:1
- Attack: 50 ms
- Release: 500 ms

Set the Music group's Duck Volume to be sidechained from the Voice group. Music automatically lowers when characters speak.

### Snapshot Blending with Transition Time

```csharp
// Blend between snapshots for scene transitions
public void EnterCave()
{
    m_CaveSnapshot.TransitionTo(1.5f);
}

public void ExitCave()
{
    m_OutdoorSnapshot.TransitionTo(2.0f);
}

// Weighted blend of multiple snapshots
public void BlendSnapshots(AudioMixerSnapshot[] snapshots, float[] weights, float transitionTime)
{
    m_Mixer.TransitionToSnapshots(snapshots, weights, transitionTime);
}
```

Snapshots store the state of all mixer parameters. `TransitionToSnapshots` with weighted arrays lets you crossfade between multiple acoustic environments simultaneously.

## Spatial Audio Deep Dive

### 3D Sound Cone

Configure AudioSource's 3D settings for directional sound:
- **Spread:** angle of the sound cone. 0 = point source (laser), 180 = wide (explosion), 360 = omnidirectional.
- **Inside Cone Angle:** full volume region.
- **Outside Cone Angle:** attenuated region.
- **Outside Volume:** volume multiplier outside the cone (0 = silent).

Use directional cones for speakers, TVs, and directional ambient sources like waterfalls.

### Custom Rolloff Curves

Logarithmic rolloff is physically accurate but often sounds wrong in games. Use Custom rolloff with an AnimationCurve:

```csharp
public void ConfigureSpatialAudio(AudioSource source, float minDist, float maxDist)
{
    source.spatialBlend = 1f;
    source.rolloffMode = AudioRolloffMode.Custom;
    source.minDistance = minDist;
    source.maxDistance = maxDist;

    // Custom curve: sharp falloff near max distance
    AnimationCurve rolloff = new AnimationCurve(
        new Keyframe(0f, 1f),
        new Keyframe(0.5f, 0.6f),
        new Keyframe(0.85f, 0.15f),
        new Keyframe(1f, 0f)
    );
    source.SetCustomCurve(AudioSourceCurveType.CustomRolloff, rolloff);
}
```

### Doppler Effect Tuning

`AudioSource.dopplerLevel` controls pitch shift from relative velocity. Set to 0 for most games (sounds unnatural). Use 0.5-1.0 only for racing games or fast-moving projectiles where doppler adds immersion.

### Audio Listener Positioning

- **Third-person camera:** listener on the camera. Sound perspective matches what the player sees.
- **Top-down camera:** listener at the player character, not the camera. Otherwise everything sounds distant.
- **Split-screen:** one listener per player is not supported. Place listener at the midpoint or use the primary player's position.

## Audio Scheduling and Beat Sync

### PlayScheduled with DSP Time

`AudioSource.PlayScheduled` uses `AudioSettings.dspTime` for sample-accurate timing. Regular `Play()` has up to one audio buffer of latency.

```csharp
// Schedule a sound to play exactly 2 seconds from now
double scheduledTime = AudioSettings.dspTime + 2.0;
m_AudioSource.PlayScheduled(scheduledTime);
```

### Beat-Accurate Music Transitions

```csharp
public sealed class BeatSyncManager
{
    private readonly double m_BeatsPerMinute;
    private readonly double m_SecondsPerBeat;
    private double m_NextBeatTime;

    public BeatSyncManager(double bpm)
    {
        m_BeatsPerMinute = bpm;
        m_SecondsPerBeat = 60.0 / bpm;
        m_NextBeatTime = AudioSettings.dspTime + m_SecondsPerBeat;
    }

    // Returns the DSP time of the next beat boundary
    public double GetNextBeatTime()
    {
        double currentDsp = AudioSettings.dspTime;
        while (m_NextBeatTime <= currentDsp)
        {
            m_NextBeatTime += m_SecondsPerBeat;
        }
        return m_NextBeatTime;
    }

    // Schedule a clip to start on the next beat
    public void PlayOnNextBeat(AudioSource source, AudioClip clip)
    {
        source.clip = clip;
        source.PlayScheduled(GetNextBeatTime());
    }
}
```

### Gapless Loop with Double-Buffer

```csharp
// Two sources alternate to eliminate gaps between loops
public void ScheduleGaplessLoop(AudioSource sourceA, AudioSource sourceB, AudioClip clip)
{
    double startTime = AudioSettings.dspTime + 0.1;
    double clipDuration = (double)clip.samples / clip.frequency;

    sourceA.clip = clip;
    sourceB.clip = clip;

    sourceA.PlayScheduled(startTime);
    sourceB.PlayScheduled(startTime + clipDuration);

    // Continue scheduling in a tick loop, alternating sources
}
```

Schedule the next loop iteration before the current one ends. The audio thread handles the transition with zero gap.

## Procedural SFX

### Pitch and Volume Randomization

```csharp
public void PlayRandomized(AudioSource source, AudioClip clip, float volumeBase = 1f)
{
    source.pitch = Random.Range(0.9f, 1.1f);
    float volumeVariation = Random.Range(0.85f, 1f);
    source.PlayOneShot(clip, volumeBase * volumeVariation);
    source.pitch = 1f; // Reset for next play if source is shared
}
```

Small pitch and volume variation prevents the "machine gun" effect when the same clip repeats rapidly. Always reset pitch after PlayOneShot if the source is pooled.

### Layered Impact Sounds

Combine a heavy base layer with a lighter sweetener for rich impacts:

```csharp
public void PlayImpact(AudioSource source, AudioClip baseClip, AudioClip sweetenerClip, float force)
{
    float normalizedForce = Mathf.Clamp01(force / 100f);

    // Base: always plays, volume scales with force
    source.PlayOneShot(baseClip, normalizedForce);

    // Sweetener: higher pitch, lower volume, adds texture
    if (normalizedForce > 0.3f)
    {
        source.pitch = Random.Range(1.1f, 1.3f);
        source.PlayOneShot(sweetenerClip, normalizedForce * 0.4f);
        source.pitch = 1f;
    }
}
```

### Footstep System Driven by Animation Events

Pair with animation events for frame-accurate footsteps. The animation event sends a foot index (0 = left, 1 = right). The audio system selects a clip based on the surface material under that foot using a short raycast cached per-frame.

## Audio Pool Manager

### Complete AudioSourcePool Implementation

```csharp
public sealed class AudioSourcePool : MonoBehaviour
{
    [SerializeField] private int m_PoolSize = 24;
    [SerializeField] private AudioMixerGroup m_DefaultGroup;

    private AudioSource[] m_Sources;
    private float[] m_Priority;
    private int m_ActiveCount;

    private void Awake()
    {
        m_Sources = new AudioSource[m_PoolSize];
        m_Priority = new float[m_PoolSize];

        for (int sourceIndex = 0; sourceIndex < m_PoolSize; sourceIndex++)
        {
            GameObject obj = new GameObject($"PooledAudio_{sourceIndex}");
            obj.transform.SetParent(transform);
            AudioSource source = obj.AddComponent<AudioSource>();
            source.outputAudioMixerGroup = m_DefaultGroup;
            source.playOnAwake = false;
            m_Sources[sourceIndex] = source;
            m_Priority[sourceIndex] = 0f;
        }
    }

    public AudioSource Play(AudioClip clip, Vector3 position, float volume, float priority)
    {
        int slotIndex = FindAvailableSlot(priority);
        if (slotIndex < 0)
        {
            return null; // All slots occupied by higher-priority sounds
        }

        AudioSource source = m_Sources[slotIndex];
        source.Stop();
        source.transform.position = position;
        source.spatialBlend = 1f;
        source.priority = (int)(128 - priority * 128);
        source.PlayOneShot(clip, volume);
        m_Priority[slotIndex] = priority;

        return source;
    }

    private int FindAvailableSlot(float requestedPriority)
    {
        int lowestPriorityIndex = -1;
        float lowestPriority = float.MaxValue;

        for (int slotIndex = 0; slotIndex < m_PoolSize; slotIndex++)
        {
            // Prefer slots that are not playing
            if (!m_Sources[slotIndex].isPlaying)
            {
                return slotIndex;
            }

            // Track lowest priority for potential steal
            if (m_Priority[slotIndex] < lowestPriority)
            {
                lowestPriority = m_Priority[slotIndex];
                lowestPriorityIndex = slotIndex;
            }
        }

        // Steal from lowest priority if request is higher
        if (lowestPriorityIndex >= 0 && requestedPriority > lowestPriority)
        {
            return lowestPriorityIndex;
        }

        return -1;
    }
}
```

### Priority System

Assign priority values by sound category:
- UI clicks: 1.0 (always play)
- Player weapons: 0.9
- Player footsteps: 0.7
- Enemy weapons: 0.6
- Enemy footsteps: 0.3
- Ambient details: 0.2

Higher priority sounds steal pool slots from lower priority ones when the pool is full.

### Distance-Based Culling

Before playing a sound, check distance from the AudioListener. Skip sounds beyond audible range to save pool slots:

```csharp
public bool ShouldPlay(Vector3 soundPosition, float maxAudibleDistance)
{
    AudioListener listener = FindAudioListener();
    if (listener == null)
    {
        return false;
    }

    float sqrDistance = (soundPosition - listener.transform.position).sqrMagnitude;
    return sqrDistance <= maxAudibleDistance * maxAudibleDistance;
}
```

Use squared distance to avoid the `sqrt` in `Vector3.Distance`.

### Maximum Concurrent Sounds Per Category

Limit how many sounds of the same type play simultaneously. More than 3-4 gunshots at once turns into noise:

```csharp
private int[] m_CategoryCounts;
private int[] m_CategoryLimits;

public bool CanPlayCategory(int categoryIndex)
{
    return m_CategoryCounts[categoryIndex] < m_CategoryLimits[categoryIndex];
}
```

Track active counts per category. When a pooled source finishes playing, decrement the count.

## Performance and Platform

### Compression Format Selection

| Format | CPU Cost | Size | Quality | Use Case |
|--------|----------|------|---------|----------|
| PCM | None | Large | Perfect | Short UI clips, critical SFX |
| ADPCM | Low | ~3.5x smaller than PCM | Good | Short SFX (< 1 sec), gunshots |
| Vorbis | Medium | ~10x smaller than PCM | Good-Excellent | Music, long ambience, voice |

Vorbis quality slider: 40-60% for music (barely audible difference), 70% for voice, 100% only if quality loss is noticeable.

### Load Type Selection

- **Decompress On Load:** uses more memory, zero CPU at play time. Best for short, frequent SFX.
- **Compressed In Memory:** smaller memory footprint, decompresses on play. Best for longer clips that play less often.
- **Streaming:** minimal memory, streams from disk. Best for music and ambient loops. Adds slight latency on first play.

### Mobile Audio Budget

Mobile devices typically support 16-24 simultaneous voices (AudioSources playing at once). Exceeding this causes:
- Sounds silently dropped by the audio engine
- Priority-based culling (lower priority sources cut first)
- Increased CPU usage from voice management

Set `AudioSettings.GetConfiguration().numRealVoices` to match platform capability. Use `numVirtualVoices` for sounds that are tracked but not rendered (they resume when a real voice frees up).

### Preload vs On-Demand Loading

Mark frequently used clips with **Preload Audio Data** enabled in the import settings. For large sound banks (hundreds of clips), load on demand via Addressables:

```csharp
// Load clip from Addressables when needed
public async UniTask<AudioClip> LoadClipAsync(string address, CancellationToken token)
{
    AsyncOperationHandle<AudioClip> handle = Addressables.LoadAssetAsync<AudioClip>(address);
    AudioClip clip = await handle.ToUniTask(cancellationToken: token);
    return clip;
}
```

Release handles when clips are no longer needed to free memory. Never load audio synchronously on mobile — it causes frame hitches.
