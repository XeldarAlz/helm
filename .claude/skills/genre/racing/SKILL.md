---
name: Racing
description: Racing game patterns — vehicle physics, track design, lap tracking, AI opponents, drift mechanics
globs: ["**/Vehicle*.cs", "**/Car*.cs", "**/Race*.cs", "**/Lap*.cs", "**/Track*.cs", "**/Drift*.cs", "**/Kart*.cs"]
---

# Racing Game Patterns

Comprehensive reference for building racing games in Unity. Covers vehicle physics, drift mechanics, track and waypoint systems, lap tracking, AI opponents, boost/power-ups, camera, and UI. Applies to arcade racers, kart racers, and simulation-lite driving games.

## Overview

The core loop of a racing game:

1. **Pre-Race** -- grid placement, countdown timer (3-2-1-GO)
2. **Racing** -- drive laps around a circuit (or point-to-point)
3. **Finish** -- cross the finish line, record final time
4. **Results** -- show standings, rewards, replay option

Race variations:
- **Circuit** -- multiple laps around a closed track
- **Point-to-Point** -- start and finish at different locations
- **Time Trial** -- solo run, beat your best time or a ghost
- **Elimination** -- last-place driver eliminated each lap

All game logic lives in plain C# Systems. MonoBehaviour Views handle rendering and physics callbacks. VContainer wires everything. MessagePipe carries race events (lap completed, position changed, boost activated).

---

## Vehicle Controller

### WheelCollider-Based (Semi-Realistic)

Unity's `WheelCollider` provides suspension, friction curves, and motor/brake torque. Best for games that want a physics-grounded feel.

```csharp
public sealed class VehicleModel
{
    public float CurrentSpeed { get; set; }
    public float SteerAngle { get; set; }
    public float ThrottleInput { get; set; }
    public float BrakeInput { get; set; }
    public bool IsDrifting { get; set; }
    public float DriftAmount { get; set; }
    public float BoostFuel { get; set; }
    public int CurrentLap { get; set; }
    public int RacePosition { get; set; }
}

[CreateAssetMenu(menuName = "Racing/Vehicle Definition")]
public sealed class VehicleDefinition : ScriptableObject
{
    [SerializeField] private string m_VehicleName;
    [SerializeField] private float m_MaxMotorTorque = 1500f;
    [SerializeField] private float m_MaxBrakeTorque = 3000f;
    [SerializeField] private float m_MaxSteerAngle = 30f;
    [SerializeField] private float m_TopSpeed = 50f;
    [SerializeField] private float m_DownforceCoefficient = 3f;
    [SerializeField] private Vector3 m_CenterOfMass = new(0f, -0.5f, 0.2f);
    [SerializeField] private AnimationCurve m_TorqueCurve;

    public string VehicleName => m_VehicleName;
    public float MaxMotorTorque => m_MaxMotorTorque;
    public float MaxBrakeTorque => m_MaxBrakeTorque;
    public float MaxSteerAngle => m_MaxSteerAngle;
    public float TopSpeed => m_TopSpeed;
    public float DownforceCoefficient => m_DownforceCoefficient;
    public Vector3 CenterOfMass => m_CenterOfMass;
    public AnimationCurve TorqueCurve => m_TorqueCurve;
}
```

### Vehicle Physics View

```csharp
public sealed class VehiclePhysicsView : MonoBehaviour
{
    [SerializeField] private WheelCollider m_FrontLeftWheel;
    [SerializeField] private WheelCollider m_FrontRightWheel;
    [SerializeField] private WheelCollider m_RearLeftWheel;
    [SerializeField] private WheelCollider m_RearRightWheel;
    [SerializeField] private Transform m_FrontLeftMesh;
    [SerializeField] private Transform m_FrontRightMesh;
    [SerializeField] private Transform m_RearLeftMesh;
    [SerializeField] private Transform m_RearRightMesh;

    private Rigidbody m_Rigidbody;
    private VehicleDefinition m_Definition;
    private VehicleModel m_Model;

    [Inject]
    public void Construct(VehicleDefinition definition, VehicleModel model)
    {
        m_Definition = definition;
        m_Model = model;
    }

    private void Awake()
    {
        m_Rigidbody = GetComponent<Rigidbody>();
        m_Rigidbody.centerOfMass = m_Definition.CenterOfMass;
    }

    private void FixedUpdate()
    {
        ApplyMotor();
        ApplySteering();
        ApplyBraking();
        ApplyDownforce();
        UpdateModel();
        SyncWheelMeshes();
    }

    private void ApplyMotor()
    {
        float speedRatio = m_Rigidbody.velocity.magnitude / m_Definition.TopSpeed;
        float torqueMultiplier = m_Definition.TorqueCurve.Evaluate(speedRatio);
        float torque = m_Model.ThrottleInput * m_Definition.MaxMotorTorque * torqueMultiplier;

        m_RearLeftWheel.motorTorque = torque;
        m_RearRightWheel.motorTorque = torque;
    }

    private void ApplySteering()
    {
        float steer = m_Model.SteerAngle * m_Definition.MaxSteerAngle;
        m_FrontLeftWheel.steerAngle = steer;
        m_FrontRightWheel.steerAngle = steer;
    }

    private void ApplyBraking()
    {
        float brakeTorque = m_Model.BrakeInput * m_Definition.MaxBrakeTorque;
        m_FrontLeftWheel.brakeTorque = brakeTorque;
        m_FrontRightWheel.brakeTorque = brakeTorque;
        m_RearLeftWheel.brakeTorque = brakeTorque;
        m_RearRightWheel.brakeTorque = brakeTorque;
    }

    private void ApplyDownforce()
    {
        float speed = m_Rigidbody.velocity.magnitude;
        m_Rigidbody.AddForce(-transform.up * (speed * m_Definition.DownforceCoefficient));
    }

    private void UpdateModel()
    {
        m_Model.CurrentSpeed = m_Rigidbody.velocity.magnitude;
    }

    private void SyncWheelMeshes()
    {
        SyncWheel(m_FrontLeftWheel, m_FrontLeftMesh);
        SyncWheel(m_FrontRightWheel, m_FrontRightMesh);
        SyncWheel(m_RearLeftWheel, m_RearLeftMesh);
        SyncWheel(m_RearRightWheel, m_RearRightMesh);
    }

    private void SyncWheel(WheelCollider collider, Transform mesh)
    {
        collider.GetWorldPose(out Vector3 position, out Quaternion rotation);
        mesh.position = position;
        mesh.rotation = rotation;
    }
}
```

### Arcade Alternative (Rigidbody with Custom Forces)

For kart-style games, skip WheelColliders entirely. Apply forces directly to a Rigidbody for full control over feel.

```csharp
public sealed class ArcadeVehicleView : MonoBehaviour
{
    [SerializeField] private float m_Acceleration = 30f;
    [SerializeField] private float m_MaxSpeed = 40f;
    [SerializeField] private float m_TurnSpeed = 100f;
    [SerializeField] private float m_GravityForce = 20f;
    [SerializeField] private float m_DragCoefficient = 2f;
    [SerializeField] private LayerMask m_GroundLayer;

    private Rigidbody m_Rigidbody;
    private VehicleModel m_Model;
    private bool m_IsGrounded;
    private readonly RaycastHit[] m_GroundHits = new RaycastHit[1];

    [Inject]
    public void Construct(VehicleModel model)
    {
        m_Model = model;
    }

    private void Awake()
    {
        m_Rigidbody = GetComponent<Rigidbody>();
    }

    private void FixedUpdate()
    {
        CheckGround();
        ApplyAcceleration();
        ApplyTurning();
        ApplyDrag();
        ApplyGravity();
        m_Model.CurrentSpeed = m_Rigidbody.velocity.magnitude;
    }

    private void CheckGround()
    {
        int hitCount = Physics.RaycastNonAlloc(
            transform.position, -transform.up, m_GroundHits, 1.5f, m_GroundLayer);
        m_IsGrounded = hitCount > 0;
    }

    private void ApplyAcceleration()
    {
        if (!m_IsGrounded) return;
        if (m_Rigidbody.velocity.magnitude < m_MaxSpeed)
        {
            m_Rigidbody.AddForce(
                transform.forward * (m_Model.ThrottleInput * m_Acceleration),
                ForceMode.Acceleration);
        }
    }

    private void ApplyTurning()
    {
        if (!m_IsGrounded) return;
        float speedFactor = m_Rigidbody.velocity.magnitude / m_MaxSpeed;
        float turnAmount = m_Model.SteerAngle * m_TurnSpeed * speedFactor * Time.fixedDeltaTime;
        Quaternion turnRotation = Quaternion.Euler(0f, turnAmount, 0f);
        m_Rigidbody.MoveRotation(m_Rigidbody.rotation * turnRotation);
    }

    private void ApplyDrag()
    {
        Vector3 velocity = m_Rigidbody.velocity;
        Vector3 lateralVelocity = Vector3.Project(velocity, transform.right);
        m_Rigidbody.AddForce(-lateralVelocity * m_DragCoefficient, ForceMode.Acceleration);
    }

    private void ApplyGravity()
    {
        if (!m_IsGrounded)
        {
            m_Rigidbody.AddForce(Vector3.down * m_GravityForce, ForceMode.Acceleration);
        }
    }
}
```

The arcade approach gives full control over turn response, lateral grip, and air behavior. Tune `m_DragCoefficient` to control how "slidey" the car feels.

---

## Drift Mechanics

### Drift Detection and State Machine

```csharp
public enum DriftState { Normal, Drifting, BoostReady }

public sealed class DriftSystem : IDisposable
{
    private readonly VehicleModel m_Model;
    private readonly IPublisher<DriftBoostMessage> m_BoostPublisher;

    private DriftState m_State;
    private float m_DriftTimer;
    private float m_DriftDirection;

    private const float k_DriftEntrySpeedThreshold = 15f;
    private const float k_DriftEntrySteerThreshold = 0.7f;
    private const float k_DriftBoostTime1 = 1.0f;
    private const float k_DriftBoostTime2 = 2.5f;
    private const float k_DriftBoostTime3 = 4.0f;

    [Inject]
    public DriftSystem(VehicleModel model, IPublisher<DriftBoostMessage> boostPublisher)
    {
        m_Model = model;
        m_BoostPublisher = boostPublisher;
    }

    public void UpdateDrift(bool driftButtonHeld, float steerInput, float deltaTime)
    {
        switch (m_State)
        {
            case DriftState.Normal:
                if (driftButtonHeld
                    && m_Model.CurrentSpeed > k_DriftEntrySpeedThreshold
                    && Mathf.Abs(steerInput) > k_DriftEntrySteerThreshold)
                {
                    m_State = DriftState.Drifting;
                    m_DriftDirection = Mathf.Sign(steerInput);
                    m_DriftTimer = 0f;
                    m_Model.IsDrifting = true;
                }
                break;

            case DriftState.Drifting:
                m_DriftTimer += deltaTime;
                m_Model.DriftAmount = m_DriftTimer;

                if (!driftButtonHeld)
                {
                    ReleaseDrift();
                }
                break;
        }
    }

    private void ReleaseDrift()
    {
        int boostLevel = 0;
        if (m_DriftTimer >= k_DriftBoostTime3) boostLevel = 3;
        else if (m_DriftTimer >= k_DriftBoostTime2) boostLevel = 2;
        else if (m_DriftTimer >= k_DriftBoostTime1) boostLevel = 1;

        if (boostLevel > 0)
        {
            m_BoostPublisher.Publish(new DriftBoostMessage(boostLevel));
        }

        m_State = DriftState.Normal;
        m_DriftTimer = 0f;
        m_Model.IsDrifting = false;
        m_Model.DriftAmount = 0f;
    }

    public void Dispose() { }
}

public readonly struct DriftBoostMessage
{
    public readonly int BoostLevel;

    public DriftBoostMessage(int boostLevel)
    {
        BoostLevel = boostLevel;
    }
}
```

### Drift Visuals

Tire smoke and skid marks sell the drift feel. Use particle systems for smoke and `TrailRenderer` for skid marks on the rear wheels.

```csharp
public sealed class DriftVisualsView : MonoBehaviour
{
    [SerializeField] private ParticleSystem m_LeftTireSmoke;
    [SerializeField] private ParticleSystem m_RightTireSmoke;
    [SerializeField] private TrailRenderer m_LeftSkidMark;
    [SerializeField] private TrailRenderer m_RightSkidMark;

    private VehicleModel m_Model;

    [Inject]
    public void Construct(VehicleModel model)
    {
        m_Model = model;
    }

    private void Update()
    {
        bool drifting = m_Model.IsDrifting;

        if (drifting && !m_LeftTireSmoke.isPlaying)
        {
            m_LeftTireSmoke.Play();
            m_RightTireSmoke.Play();
        }
        else if (!drifting && m_LeftTireSmoke.isPlaying)
        {
            m_LeftTireSmoke.Stop();
            m_RightTireSmoke.Stop();
        }

        m_LeftSkidMark.emitting = drifting;
        m_RightSkidMark.emitting = drifting;
    }
}
```

---

## Track and Waypoint System

### Spline-Based Track Definition

Define the track centerline as a spline (Unity Splines package or custom Catmull-Rom). Waypoints are sampled at regular intervals along the spline.

```csharp
[CreateAssetMenu(menuName = "Racing/Track Definition")]
public sealed class TrackDefinition : ScriptableObject
{
    [SerializeField] private Vector3[] m_WaypointPositions;
    [SerializeField] private float[] m_WaypointWidths;
    [SerializeField] private float m_TrackLength;
    [SerializeField] private int m_TotalLaps = 3;

    public IReadOnlyList<Vector3> WaypointPositions => m_WaypointPositions;
    public IReadOnlyList<float> WaypointWidths => m_WaypointWidths;
    public float TrackLength => m_TrackLength;
    public int TotalLaps => m_TotalLaps;
    public int WaypointCount => m_WaypointPositions.Length;
}
```

### Waypoint Tracker

Track each racer's progress along the waypoint sequence to determine race position and detect wrong-way driving.

```csharp
public sealed class WaypointTracker
{
    private readonly TrackDefinition m_Track;
    private int m_CurrentWaypointIndex;
    private float m_DistanceAlongTrack;

    public int CurrentWaypointIndex => m_CurrentWaypointIndex;
    public float DistanceAlongTrack => m_DistanceAlongTrack;
    public float TotalRaceDistance => m_DistanceAlongTrack;

    public WaypointTracker(TrackDefinition track)
    {
        m_Track = track;
    }

    public void UpdatePosition(Vector3 racerPosition)
    {
        int nextIndex = (m_CurrentWaypointIndex + 1) % m_Track.WaypointCount;
        Vector3 nextWaypoint = m_Track.WaypointPositions[nextIndex];
        float distanceToNext = Vector3.Distance(racerPosition, nextWaypoint);

        if (distanceToNext < m_Track.WaypointWidths[nextIndex])
        {
            m_CurrentWaypointIndex = nextIndex;
        }

        m_DistanceAlongTrack = m_CurrentWaypointIndex + GetFractionToNext(racerPosition);
    }

    private float GetFractionToNext(Vector3 position)
    {
        int nextIndex = (m_CurrentWaypointIndex + 1) % m_Track.WaypointCount;
        Vector3 current = m_Track.WaypointPositions[m_CurrentWaypointIndex];
        Vector3 next = m_Track.WaypointPositions[nextIndex];
        Vector3 segment = next - current;
        float segmentLength = segment.magnitude;
        if (segmentLength < 0.001f) return 0f;

        float projection = Vector3.Dot(position - current, segment / segmentLength);
        return Mathf.Clamp01(projection / segmentLength);
    }
}
```

### Track Bounds Detection

Use trigger colliders along track edges to detect when a racer goes off-track. Apply a speed penalty and optionally reset the vehicle to the last valid position.

```csharp
public readonly struct OffTrackMessage
{
    public readonly int RacerId;
    public readonly Vector3 ResetPosition;

    public OffTrackMessage(int racerId, Vector3 resetPosition)
    {
        RacerId = racerId;
        ResetPosition = resetPosition;
    }
}
```

### Shortcut Detection

Track the expected waypoint sequence. If a racer jumps from waypoint 3 to waypoint 7 without passing 4-5-6, flag it. Legitimate shortcuts should have their own waypoint chain to pass through.

---

## Lap Tracking

### Checkpoint System

Place trigger colliders at intervals around the track. A racer must pass through all checkpoints in order before the finish line counts as a valid lap.

```csharp
public sealed class LapModel
{
    public int CurrentLap { get; set; }
    public int TotalLaps { get; set; }
    public float CurrentLapTime { get; set; }
    public float BestLapTime { get; set; } = float.MaxValue;
    public float TotalRaceTime { get; set; }
    public int NextCheckpointIndex { get; set; }
    public int TotalCheckpoints { get; set; }
    public bool IsFinished { get; set; }
}

public readonly struct LapCompletedMessage
{
    public readonly int RacerId;
    public readonly int LapNumber;
    public readonly float LapTime;

    public LapCompletedMessage(int racerId, int lapNumber, float lapTime)
    {
        RacerId = racerId;
        LapNumber = lapNumber;
        LapTime = lapTime;
    }
}

public sealed class LapSystem : IDisposable
{
    private readonly LapModel m_Model;
    private readonly IPublisher<LapCompletedMessage> m_LapPublisher;

    [Inject]
    public LapSystem(LapModel model, IPublisher<LapCompletedMessage> lapPublisher)
    {
        m_Model = model;
        m_LapPublisher = lapPublisher;
    }

    public void OnCheckpointReached(int checkpointIndex)
    {
        if (checkpointIndex != m_Model.NextCheckpointIndex) return;
        m_Model.NextCheckpointIndex = (checkpointIndex + 1) % m_Model.TotalCheckpoints;
    }

    public void OnFinishLineCrossed(int racerId)
    {
        // Only count if all checkpoints were hit (next should be 0 again)
        if (m_Model.NextCheckpointIndex != 0) return;

        float lapTime = m_Model.CurrentLapTime;
        m_Model.CurrentLap++;

        if (lapTime < m_Model.BestLapTime)
        {
            m_Model.BestLapTime = lapTime;
        }

        m_LapPublisher.Publish(new LapCompletedMessage(racerId, m_Model.CurrentLap, lapTime));

        if (m_Model.CurrentLap >= m_Model.TotalLaps)
        {
            m_Model.IsFinished = true;
        }
        else
        {
            m_Model.CurrentLapTime = 0f;
        }
    }

    public void Tick(float deltaTime)
    {
        if (m_Model.IsFinished) return;
        m_Model.CurrentLapTime += deltaTime;
        m_Model.TotalRaceTime += deltaTime;
    }

    public void Dispose() { }
}
```

### Wrong-Way Detection

Compare the racer's forward direction against the direction toward the next waypoint. If the dot product is negative for more than a brief threshold, show a "WRONG WAY" warning.

```csharp
public sealed class WrongWayDetector
{
    private readonly TrackDefinition m_Track;
    private float m_WrongWayTimer;
    private const float k_WrongWayThreshold = 1.0f;

    public bool IsGoingWrongWay { get; private set; }

    public WrongWayDetector(TrackDefinition track)
    {
        m_Track = track;
    }

    public void Update(Vector3 position, Vector3 forward, int nextWaypointIndex, float deltaTime)
    {
        Vector3 toWaypoint = (m_Track.WaypointPositions[nextWaypointIndex] - position).normalized;
        float dot = Vector3.Dot(forward, toWaypoint);

        if (dot < -0.3f)
        {
            m_WrongWayTimer += deltaTime;
        }
        else
        {
            m_WrongWayTimer = 0f;
        }

        IsGoingWrongWay = m_WrongWayTimer > k_WrongWayThreshold;
    }
}
```

---

## AI Opponents

### Waypoint-Following AI

AI drivers steer toward the next waypoint with a look-ahead. Smooth the path by targeting a point ahead of the immediate next waypoint.

```csharp
public sealed class RacingAISystem : IDisposable
{
    private readonly TrackDefinition m_Track;
    private readonly VehicleModel m_Model;
    private readonly WaypointTracker m_Tracker;

    private int m_LookAheadCount = 3;

    [Inject]
    public RacingAISystem(TrackDefinition track, VehicleModel model, WaypointTracker tracker)
    {
        m_Track = track;
        m_Model = model;
        m_Tracker = tracker;
    }

    public void UpdateAI(Vector3 position, Vector3 forward)
    {
        int targetIndex = (m_Tracker.CurrentWaypointIndex + m_LookAheadCount)
            % m_Track.WaypointCount;
        Vector3 targetPos = m_Track.WaypointPositions[targetIndex];

        Vector3 toTarget = (targetPos - position).normalized;
        float steerDot = Vector3.Dot(Vector3.Cross(forward, toTarget), Vector3.up);

        m_Model.SteerAngle = Mathf.Clamp(steerDot * 2f, -1f, 1f);
        m_Model.ThrottleInput = 1f;

        // Brake before sharp turns
        float turnAngle = Vector3.Angle(forward, toTarget);
        if (turnAngle > 40f && m_Model.CurrentSpeed > 20f)
        {
            m_Model.ThrottleInput = 0f;
            m_Model.BrakeInput = 0.5f;
        }
        else
        {
            m_Model.BrakeInput = 0f;
        }
    }

    public void Dispose() { }
}
```

### Difficulty Levels

Scale AI difficulty with these knobs:
- **Speed cap** -- limit AI top speed percentage (easy: 85%, medium: 95%, hard: 100%)
- **Reaction time** -- delay between detecting a turn and starting to steer
- **Rubber banding** -- speed up AI when far behind, slow down when far ahead
- **Line accuracy** -- add noise to the AI's target waypoint position

```csharp
[CreateAssetMenu(menuName = "Racing/AI Difficulty")]
public sealed class AIDifficultyDefinition : ScriptableObject
{
    [SerializeField] private string m_DifficultyName;
    [SerializeField, Range(0.5f, 1f)] private float m_SpeedMultiplier = 0.9f;
    [SerializeField] private float m_ReactionDelay = 0.15f;
    [SerializeField] private float m_RubberBandStrength = 0.3f;
    [SerializeField] private float m_LineAccuracyNoise = 2f;

    public string DifficultyName => m_DifficultyName;
    public float SpeedMultiplier => m_SpeedMultiplier;
    public float ReactionDelay => m_ReactionDelay;
    public float RubberBandStrength => m_RubberBandStrength;
    public float LineAccuracyNoise => m_LineAccuracyNoise;
}
```

### Rubber Banding

Subtle rubber banding keeps races competitive. Scale the AI speed based on distance to the player. If too obvious, players feel cheated -- keep the multiplier range tight (0.9x to 1.1x).

```csharp
public float CalculateRubberBandMultiplier(float aiDistance, float playerDistance, float strength)
{
    float gap = playerDistance - aiDistance;
    float normalized = Mathf.Clamp(gap / 100f, -1f, 1f);
    return 1f + normalized * strength;
}
```

### Collision Avoidance

Raycast forward and to the sides from the AI vehicle. When an obstacle or another racer is detected, bias the steering away.

---

## Boost and Power-Up System

### Boost Pads

Place trigger colliders on the track surface. When a racer enters the trigger, apply a timed speed boost.

```csharp
public readonly struct BoostActivatedMessage
{
    public readonly int RacerId;
    public readonly float BoostDuration;
    public readonly float BoostMultiplier;

    public BoostActivatedMessage(int racerId, float boostDuration, float boostMultiplier)
    {
        RacerId = racerId;
        BoostDuration = boostDuration;
        BoostMultiplier = boostMultiplier;
    }
}

public sealed class BoostSystem : IDisposable
{
    private float m_BoostTimer;
    private float m_CurrentMultiplier = 1f;

    public float SpeedMultiplier => m_CurrentMultiplier;

    public void ActivateBoost(float duration, float multiplier)
    {
        m_BoostTimer = duration;
        m_CurrentMultiplier = multiplier;
    }

    public void Tick(float deltaTime)
    {
        if (m_BoostTimer > 0f)
        {
            m_BoostTimer -= deltaTime;
            if (m_BoostTimer <= 0f)
            {
                m_CurrentMultiplier = 1f;
            }
        }
    }

    public void Dispose() { }
}
```

### Item Box Pickups

For kart-style racers, item boxes give random items based on race position. Players in last place get stronger items (rubber-band balancing through items). Items can be offensive (shell, bomb), defensive (shield, banana peel), or utility (speed mushroom, shortcut key).

---

## Camera System

### Chase Camera with Speed-Based FOV

The camera follows behind the vehicle. At higher speeds, increase the FOV for a sense of speed.

```csharp
public sealed class RaceCameraView : MonoBehaviour
{
    [SerializeField] private Transform m_Target;
    [SerializeField] private Camera m_Camera;
    [SerializeField] private float m_FollowDistance = 8f;
    [SerializeField] private float m_FollowHeight = 3f;
    [SerializeField] private float m_SmoothSpeed = 10f;
    [SerializeField] private float m_BaseFOV = 60f;
    [SerializeField] private float m_MaxFOV = 80f;
    [SerializeField] private float m_FOVSpeedThreshold = 40f;

    private VehicleModel m_Model;

    [Inject]
    public void Construct(VehicleModel model)
    {
        m_Model = model;
    }

    private void LateUpdate()
    {
        if (m_Target == null) return;

        // Position
        Vector3 targetPosition = m_Target.position
            - m_Target.forward * m_FollowDistance
            + Vector3.up * m_FollowHeight;
        transform.position = Vector3.Lerp(
            transform.position, targetPosition, Time.deltaTime * m_SmoothSpeed);

        // Look at target
        transform.LookAt(m_Target.position + Vector3.up * 1f);

        // Speed-based FOV
        float speedRatio = Mathf.Clamp01(m_Model.CurrentSpeed / m_FOVSpeedThreshold);
        m_Camera.fieldOfView = Mathf.Lerp(m_BaseFOV, m_MaxFOV, speedRatio);
    }
}
```

### Rear-View Mirror

Render a secondary camera facing backward to a small RenderTexture displayed on a RawImage in the UI. Keep the render texture resolution low (256x128) to minimize GPU cost.

### Replay Camera

After the race, let the player rewatch from cinematic angles. Store vehicle transform data each FixedUpdate in a ring buffer. On replay, play back the transforms and switch between preset camera positions (trackside, helicopter, onboard).

---

## UI Patterns

### Speedometer

Display current speed as a numeric value and an analog gauge arc. Use `TextMeshProUGUI` for the number, a filled `Image` for the gauge arc.

```csharp
public sealed class SpeedometerView : MonoBehaviour
{
    [SerializeField] private TMPro.TextMeshProUGUI m_SpeedText;
    [SerializeField] private Image m_GaugeArc;
    [SerializeField] private float m_MaxDisplaySpeed = 200f;

    private VehicleModel m_Model;

    [Inject]
    public void Construct(VehicleModel model)
    {
        m_Model = model;
    }

    private void Update()
    {
        float displaySpeed = m_Model.CurrentSpeed * 3.6f; // m/s to km/h
        m_SpeedText.SetText("{0:0}", displaySpeed);
        m_GaugeArc.fillAmount = Mathf.Clamp01(displaySpeed / m_MaxDisplaySpeed);
    }
}
```

### Minimap

Render a top-down orthographic camera to a RenderTexture. Show racer icons as UI elements positioned by projecting world coordinates onto the minimap rect. Use a `RawImage` for the track outline and `Image` icons for each racer.

### Position Indicator

Show "1st", "2nd", "3rd" etc. with ordinal suffix. Update via MessagePipe when the RacePositionSystem recalculates standings.

### Lap Counter

Display "Lap 2/3" using `TextMeshProUGUI`. Subscribe to `LapCompletedMessage` to update.

### Countdown Overlay

Show a full-screen countdown (3, 2, 1, GO!) before the race starts. Use async UniTask for the sequence:

```csharp
public async UniTask PlayCountdown(TMPro.TextMeshProUGUI countdownText, CancellationToken token)
{
    string[] steps = { "3", "2", "1", "GO!" };
    for (int stepIndex = 0; stepIndex < steps.Length; stepIndex++)
    {
        countdownText.SetText(steps[stepIndex]);
        countdownText.gameObject.SetActive(true);
        await UniTask.Delay(TimeSpan.FromSeconds(1), cancellationToken: token);
    }
    countdownText.gameObject.SetActive(false);
}
```

---

## Race Position System

Calculate each racer's position based on laps completed and distance along the current lap.

```csharp
public sealed class RacePositionSystem : IDisposable
{
    private readonly List<RacerProgress> m_Racers = new();
    private readonly IPublisher<PositionChangedMessage> m_PositionPublisher;

    [Inject]
    public RacePositionSystem(IPublisher<PositionChangedMessage> positionPublisher)
    {
        m_PositionPublisher = positionPublisher;
    }

    public void RegisterRacer(int racerId, LapModel lapModel, WaypointTracker tracker)
    {
        m_Racers.Add(new RacerProgress(racerId, lapModel, tracker));
    }

    public void RecalculatePositions()
    {
        // Sort by laps descending, then by distance descending
        m_Racers.Sort((a, b) =>
        {
            int lapCompare = b.LapModel.CurrentLap.CompareTo(a.LapModel.CurrentLap);
            if (lapCompare != 0) return lapCompare;
            return b.Tracker.DistanceAlongTrack.CompareTo(a.Tracker.DistanceAlongTrack);
        });

        for (int racerIndex = 0; racerIndex < m_Racers.Count; racerIndex++)
        {
            int newPosition = racerIndex + 1;
            RacerProgress racer = m_Racers[racerIndex];
            if (newPosition != racer.LastPosition)
            {
                racer.LastPosition = newPosition;
                m_PositionPublisher.Publish(new PositionChangedMessage(racer.RacerId, newPosition));
            }
        }
    }

    public void Dispose() { }
}

public sealed class RacerProgress
{
    public int RacerId { get; }
    public LapModel LapModel { get; }
    public WaypointTracker Tracker { get; }
    public int LastPosition { get; set; }

    public RacerProgress(int racerId, LapModel lapModel, WaypointTracker tracker)
    {
        RacerId = racerId;
        LapModel = lapModel;
        Tracker = tracker;
    }
}

public readonly struct PositionChangedMessage
{
    public readonly int RacerId;
    public readonly int Position;

    public PositionChangedMessage(int racerId, int position)
    {
        RacerId = racerId;
        Position = position;
    }
}
```

---

## Common Pitfalls

### WheelCollider Jitter at Low Speed

WheelColliders can oscillate at very low velocities. Clamp the Rigidbody to sleep when speed is below a threshold (e.g., 0.5 m/s and no input). Alternatively, increase `WheelCollider.suspensionSpring.damper` and lower `spring` for stability at rest.

### AI Rubber Banding Too Obvious

If the AI visibly accelerates when behind or brakes when ahead, players notice and feel cheated. Apply rubber banding through subtle means: slightly wider racing lines (slower), minor steering hesitation, or reduced top speed rather than sudden speed changes.

### Checkpoint Skipping Exploits

Players may find shortcuts that skip checkpoints. Every valid path through the track must pass through all checkpoints. Place checkpoints at unavoidable narrow points (tunnels, bridges). For open tracks, use wider trigger volumes.

### Physics Instability at High Speed

At very high velocities, objects can tunnel through colliders. Set the Rigidbody's collision detection to `ContinuousDynamic` for the player vehicle. For AI vehicles, `ContinuousSpeculative` is cheaper. Also increase `Physics.defaultSolverIterations` if vehicles stack or clip.

### Center of Mass

Always lower the center of mass on vehicles. The default (geometric center) causes easy rollovers. Set it via `Rigidbody.centerOfMass` in Awake. A typical value is 0.3-0.5 meters below the geometric center, slightly forward of center.

### Input Deadzone

Analog stick input without a deadzone causes vehicles to drift slightly even when the player is not touching the stick. Apply a radial deadzone of 0.1-0.15 before feeding input to the vehicle model.

---

## Performance

### LOD for Distant Vehicles

Use Unity's LOD Group on vehicle meshes. Distant opponents can use simplified meshes with fewer polygons and no interior detail. At extreme distance (minimap only), disable the mesh renderer entirely and just track position.

### Tire Mark Pooling

Skid marks via `TrailRenderer` create geometry each frame while emitting. Cap the maximum number of trail points per renderer. When marks are old enough, disable the TrailRenderer and return it to a pool. Reuse pooled renderers for new skid marks rather than creating new GameObjects.

### Physics Update Rate

Vehicle physics should run at the default 50Hz FixedUpdate. Do not increase to 100Hz unless absolutely needed -- it doubles the physics cost for all colliders in the scene. Instead, use `Rigidbody.interpolation = RigidbodyInterpolation.Interpolate` for smooth visual movement between physics ticks.

### Particle System Budget

Tire smoke, exhaust, and boost effects can spawn many particles. Set `maxParticles` conservatively (50-100 per emitter). Disable particle collision unless the effect specifically needs it. Use world-space simulation only when particles must persist after the emitter moves (skid smoke); otherwise use local space to reduce transform updates.

### Audio Source Pooling

Engine sounds, tire screeches, and boost audio for many vehicles can exhaust AudioSource limits. Pool AudioSources per vehicle and reuse them. Use `AudioSource.priority` to ensure the player's vehicle always has audio, while distant AI vehicles may have their sources reclaimed.
