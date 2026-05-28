// ==WindhawkMod==
// @id              d3d-cursor-effectzszs
// @name            D3D Cursor Effectzszs
// @description     GPU-rendered cursor overlay: physics trail, squishy head, click ripples, particle bursts, satellite orbitals, rainbow mode. Direct3D 11 + DirectComposition.
// @version         0.1.3
// @author          Nairod
// @include         explorer.exe
// @architecture    x86-64
// @compilerOptions -ld3d11 -ldxgi -ldcomp -ld3dcompiler -ldwmapi -luser32 -lgdi32 -lavrt -lhid
// ==/WindhawkMod==

// ==WindhawkModReadme==
/*
# D3D Cursor Master Overlay Effects — Ultimate Edition

A hyper-performance, hardware-accelerated global mouse and multi-touch overlay engine powered by **Direct3D 11** and **DirectComposition**. Injected cleanly into `explorer.exe`, it establishes an isolated, desktop-wide overlay layer to provide premium visual tracking effects without altering target application windows or degrading interface interactivity.

## Advanced Feature Modules & Capabilities

* **Multi-Layer Physics Ribbon Trail:** Generates a fluid ribbon trail lagging behind your cursor across up to 4 concurrent custom layers. Each layer features completely distinct configurable widths, transparency decay curves, and blur/feathering factor slopes mapped from head to tail.
* **10-Point Multi-Touch Spatial Tracking:** Captures global digitizer frames asynchronously to generate up to 10 independent touch trail entities concurrently, running identical physics calculations and blending behaviors alongside your main hardware pointer.
* **Dynamic Squishy SDF Cursor Head:** Renders an analytical Signed Distance Field (SDF) ellipse at the primary pointer tip that dynamically elongates, thins, and matches orientation along your current velocity vector to create an organic, responsive physics stretch effect.
* **Button-Aware Click Ripples:** Spawns expanding concentric ring primitives at the exact screen coordinates of a mouse click event. Supports custom, distinct coloring profiles mapped directly to Left, Right, and Middle mouse button triggers.
* **Kinematic Particle Burst Explosions:** Ejects a customizable collection of soft particle billboards on click events. The engine implements a dedicated kinematics pass tracking adjustable air resistance (drag coefficients) and continuous downward gravity vectors.
* **Mirrored Dual-Ring Satellite Orbitals:** Drives multiple geometric nodes revolving gracefully around the pointer coordinates. Includes optional background path rings and a synchronized secondary dual-ring reverse orbital configuration spinning in counter-rotation.
* **Full HSL Rainbow Synthesis Engine:** Overrides static layout profiles with a high-speed real-time HSL color-space hue cycling routine that steps across trail vertices sequentially to produce seamless chromatic wave motions.
* **Pure-GPU Bitmap Billboard OSD Suite:** Displays hyper-performant on-screen information nodes without triggering traditional high-cost GDI/Direct2D typography overhead. Features:
  * *Real-Time FPS Counter:* Measures and cycles rendering refresh speeds via adaptive interval counters.
  * *Live Modifier/Keystroke Logger:* Displays current keyboard arrays and complex combinators (e.g., Ctrl + Shift + Key).
  * *Mouse Action Overlay:* Echoes immediate hardware clicks adjacent to the active cursor stream.
  * *Flexible Layout Anchor Configuration:* Offers automatic multi-quadrant alignments or inline horizontal tracking centering.
* **Dynamic Interactive System Cursor Scaling:** Intercepts mouse clicks to scale up and bounce the actual Windows system cursor shape using optimized `SetSystemCursor` caching routines to guarantee clear visual accentuation without causing GDI handle leak crashes.

## Underlying Mathematical Models & Architecture

### 1. Spring-Mass-Damper Chain Kinematics
Instead of basic linear interpolation or time-lag arrays, trail segments simulate real physical links via an explicit integration scheme of the traditional spring-mass system:
$$F = -k \cdot x - c \cdot v$$
Where $k$ acts as the directional spring elasticity tension parameter, and $c$ drives the velocity dampening coefficient. This prevents tail segments from snapping unnaturally during high-speed directional changes or locking up during high-refresh frame pacing.

### 2. Catmull-Rom Spline Curve Smoothing
Discrete physical node coordinates are smoothly upsampled on-the-fly into dense geometric lines using multi-point Catmull-Rom interpolation loops. The interpolation frequency dynamically compresses via an automated speed evaluation routine to reduce GPU vertex processing overhead during rapid pointer trajectories.

### 3. Normal Vector Stabilization Pass
To eliminate twisting anomalies and self-overlapping mesh artifacts when the pointer cuts sharp corners, a safety tracking routine performs step-by-step dot product operations along the ribbon path:
$$\vec{n}_i \cdot \vec{n}_{i-1} < 0$$
If the dot product evaluates beneath zero, the local coordinate space flips normal vectors instantaneously, ensuring an unbroken, flat rendering strip topology.

### 4. Windowing & Compositor Layering
The mod establishes a transparent, full-screen overlay window crossing all available monitor bounds (`WS_EX_LAYERED | WS_EX_NOREDIRECTIONBITMAP | WS_EX_TRANSPARENT`). Instead of legacy GDI painting, it leverages DirectComposition to target the window surface directly. This feeds a `DXGI_SWAP_EFFECT_FLIP_DISCARD` swapchain, completely skipping window composition penalties and DWM delays.

### 5. Input Tracking Interception
* **Mouse Input:** Monitored via a low-level mouse hook (`WH_MOUSE_LL`) isolated on its own background thread to safeguard UI responsiveness.
* **Touch Input:** Utilizes Windows Raw Input API (`WM_INPUT`) targeting the HID Digitizer page (`0x0D`, Usage `0x04`). It manually unpacks raw link collections to track touch IDs, state flags, and scaling parameters.

### 6. GPU Optimization & Pipeline Economy
* **Single-Pass Ribbon Emission:** Spline ribbon vertices are emitted into a dynamic vertex buffer using degenerate triangles. This bridges independent layers and touch paths into a singular, highly efficient vertex stream.
* **Instanced SDF Rendering:** Rather than drawing intricate meshes for dots, satellites, heads, and rings, the engine draws generic quad billboards. A pixel shader evaluates local coordinates against analytical geometric equations (SDFs) to generate perfect, anti-aliased circles, ring lines, and custom 3x5 font matrix glyph pixels directly on the GPU.

### 7. Zero-Overhead Smart Power Throttling
The render loop utilizes a non-polling win32 notification structure driven by `MsgWaitForMultipleObjectsEx`. The moment the cursor stops moving and all kinematic particles, OSD animations, or click ripples have fully completed their lifespan fade cycles, the engine draws exactly 2 final clean clear frames to flush the flip-discard swapchain buffers, then drops CPU/GPU usage to exactly 0% until the next hardware input packet wakes the system thread.
*/
// ==/WindhawkModReadme==

// ==WindhawkModSettings==
/*
- headSpring: 50
  $name: Head Spring Strength
  $description: "Spring force pulling trail head toward cursor. Higher = snappier. Range 1-500."
- headFriction: 30
  $name: Head Friction/Damping
  $description: "Damping on trail head. Higher = smoother. Range 0-99."
- spring: 50
  $name: Trail Spring Strength
  $description: "Spring force between trail segments. Higher = tighter. Range 1-500."
- friction: 30
  $name: Trail Friction/Damping
  $description: "Damping on trail segments. Higher = more viscous. Range 0-99."
- trailLength: 100
  $name: Trail Length
  $description: "Physics nodes in trail chain. Range 5-100."
- positionHistorySkip: 0
  $name: Position History Skip
  $description: "Skip N cursor updates between trail target refreshes. Range 0-10."
- cursorSize: 40
  $name: Base Cursor Size (px)
  $description: "Master width reference for trail layers. Range 5-200."
- minTrailWidth: 2
  $name: Minimum Trail Width (px)
  $description: "Floor width for trail tail. Range 1-20."
- velocityWidthMultiplier: 5
  $name: Velocity Width Boost (x10)
  $description: "Trail widens during fast movement. /10 internally. Range 0-50."
- velocityAlphaMultiplier: 1
  $name: Velocity Opacity Boost (x10)
  $description: "Trail brightens during fast movement. /10 internally. Range 0-50."
- interpolationSteps: 1
  $name: Curve Smoothness
  $description: "Catmull-Rom samples per segment. Range 1-10."
- fadeMode: 3
  $name: Fade Curve
  $description: "0=Linear, 1=Ease-Out, 2=Exponential, 3=Sigmoid"
- enableTrail: true
  $name: Enable Trail
  $description: "Toggle the entire multi-layer trail effect on or off."
- enableGradient: true
  $name: Enable Color Gradient
  $description: "Layers fade from Start Color to End Color along trail."
- rainbowMode: false
  $name: Rainbow Mode
  $description: "Overrides layer colors with a cycling HSL rainbow."
- rainbowSpeed: 2
  $name: Rainbow Speed
  $description: "Hue degrees per frame when Rainbow Mode is on. Range 1-20."
- adaptiveQuality: true
  $name: Adaptive Quality
  $description: "Reduces curve smoothness during fast movement for performance."
- layer1:
  - enabled: true
    $name: Enable Layer 1
  - startColor: FFFFFFFF
    $name: Start Color (AARRGGBB hex)
  - endColor: 00FFFFFF
    $name: End Color (AARRGGBB hex)
  - widthFactor: 150
    $name: Width Factor (%)
  - alphaFactor: 100
    $name: Base Opacity (%)
  - startBlur: 39
    $name: Start Blur (%)
    $description: "Blur/feathering at the head of the trail. 0=sharp, 100=fully soft blur."
  - endBlur: 50
    $name: End Blur (%)
    $description: "Blur/feathering at the tail of the trail. 0=sharp, 100=fully soft blur."
  $name: Layer 1 (Outer Glow)
- layer2:
  - enabled: true
    $name: Enable Layer 2
  - startColor: FF000000
    $name: Start Color (AARRGGBB hex)
  - endColor: "00000000"
    $name: End Color (AARRGGBB hex)
  - widthFactor: 90
    $name: Width Factor (%)
  - alphaFactor: 100
    $name: Base Opacity (%)
  - startBlur: 10
    $name: Start Blur (%)
    $description: "Blur/feathering at the head of the trail. 0=sharp, 100=fully soft blur."
  - endBlur: 10
    $name: End Blur (%)
    $description: "Blur/feathering at the tail of the trail. 0=sharp, 100=fully soft blur."
  $name: Layer 2 (Mid Layer)
- layer3:
  - enabled: true
    $name: Enable Layer 3
  - startColor: FFFFFFFF
    $name: Start Color (AARRGGBB hex)
  - endColor: 00FFFFFF
    $name: End Color (AARRGGBB hex)
  - widthFactor: 50
    $name: Width Factor (%)
  - alphaFactor: 100
    $name: Base Opacity (%)
  - startBlur: 10
    $name: Start Blur (%)
    $description: "Blur/feathering at the head of the trail. 0=sharp, 100=fully soft blur."
  - endBlur: 10
    $name: End Blur (%)
    $description: "Blur/feathering at the tail of the trail. 0=sharp, 100=fully soft blur."
  $name: Layer 3 (Core)
- layer4:
  - enabled: true
    $name: Enable Layer 4
  - startColor: FF000000
    $name: Start Color (AARRGGBB hex)
  - endColor: FF000000
    $name: End Color (AARRGGBB hex)
  - widthFactor: 15
    $name: Width Factor (%)
  - alphaFactor: 100
    $name: Base Opacity (%)
  - startBlur: 10
    $name: Start Blur (%)
    $description: "Blur/feathering at the head of the trail. 0=sharp, 100=fully soft blur."
  - endBlur: 10
    $name: End Blur (%)
    $description: "Blur/feathering at the tail of the trail. 0=sharp, 100=fully soft blur."
  $name: Layer 4 (Inner Core)
- cursorHead:
  - enabled: false
    $name: Enable Squishy Cursor Head
  - filled: false
    $name: Filled (vs Outline)
  - color: FFFFFFFF
    $name: Color (AARRGGBB hex)
  - size: 18
    $name: Base Diameter (px)
  - outlineWidth: 2
    $name: Outline Width (px)
  - squishIntensity: 3
    $name: Squish Intensity
    $description: "How much the head deforms with velocity. 0=circle. Range 0-10."
  - squishSmoothing: 50
    $name: Squish Smoothing
    $description: "Interpolation speed. Higher=snappier. Range 10-100."
  - hideSystemCursor: false
    $name: Hide System Cursor
    $description: "If enabled, replaces the default system cursor with a transparent one, allowing the 'Squishy Cursor Head' to act as the primary pointer."
  $name: Squishy Cursor Head
- rippleEffect:
  - enabled: false
    $name: Enable Click Ripples
  - maxDiameter: 100
    $name: Max Diameter (px)
  - startWidth: 8
    $name: Start Stroke Width (px)
  - duration: 600
    $name: Duration (ms)
  - leftClickColor: FFFFFFFF
    $name: Left-Click Color (AARRGGBB hex)
  - rightClickColor: FFFF7777
    $name: Right-Click Color (AARRGGBB hex)
  - middleClickColor: FFFFFF66
    $name: Middle-Click Color (AARRGGBB hex)
  - enableClickScaling: true
    $name: Enable System Cursor Click Scaling
    $description: "If enabled, the system cursor will briefly enlarge on every click to provide visual feedback. This works best when 'Hide System Cursor' is disabled."
  - clickScaleFactor: 150
    $name: Cursor Click Scale Factor (%)
    $description: "The factor by which the system cursor will enlarge on click (e.g., 150% = 1.5x size)."
  - clickScaleDuration: 100
    $name: Cursor Click Scale Duration (ms)
    $description: "The duration in milliseconds for the cursor to scale up and back down on click."
  $name: Click Ripple Effect

- particleBurst:
  - enabled: false
    $name: Enable Click Particle Burst
  - count: 16
    $name: Particle Count
    $description: "Particles spawned per click. Range 4-64."
  - speed: 300
    $name: Burst Speed
    $description: "Initial outward velocity. Range 50-1000."
  - lifetime: 500
    $name: Lifetime (ms)
  - size: 3
    $name: Particle Size (px)
  - friction: 90
    $name: Particle Friction
    $description: "Air drag. Higher = particles slow faster. Range 50-99."
  - gravity: 200
    $name: Gravity
    $description: "Downward pull. 0 = no gravity. Range 0-1000."
  $name: Click Particle Burst
- satelliteEffect:
  - enabled: false
    $name: Enable Satellite Orbitals
  - count: 4
    $name: Satellite Count
    $description: "Number of orbiting circles. Range 1-12."
  - orbitDiameter: 50
    $name: Orbit Diameter (px)
  - satelliteSize: 6
    $name: Satellite Size (px)
  - filled: false
    $name: Filled (vs Outline)
  - outlineWidth: 2
    $name: Outline Width (px)
  - color: FFFFFFFF
    $name: Color (AARRGGBB hex)
  - speed: 2
    $name: Rotation Speed
    $description: "Degrees per frame. Negative = reverse. Range -20 to 20."
  - enableDualRing: false
    $name: Enable Mirrored Dual Ring
  - dualSpeed: 1
    $name: Dual Ring Speed
    $description: "Speed for the mirrored ring. Range -20 to 20."
  - showOrbitRing: false
    $name: Show Orbit Ring
  - ringWidth: 1
    $name: Ring Width (px)
  - ringColor: 44FFFFFF
    $name: Ring Color (AARRGGBB hex)
  $name: Satellite Orbitals
- keystrokeOverlay:
  - enabled: true
    $name: Display Keystrokes
    $description: "If enabled, shows currently pressed keys on screen."
  $name: Keystroke Overlay

- mouseClickOverlay:
  - enabled: true
    $name: Display Mouse Clicks
    $description: "If enabled, shows currently clicked mouse buttons on screen next to the cursor."
  - duration: 600
    $name: Display Duration (ms)
    $description: "The duration in milliseconds for the click text to remain visible and fade out."
  $name: Mouse Click Overlay

- fpsCounter:
  - enabled: true
    $name: Show FPS Counter
    $description: "Displays a real-time Frames Per Second (FPS) counter on screen."
  - alignBottom: true
    $name: Align to Bottom
    $description: "If enabled, the FPS counter is aligned to the bottom. Otherwise, it's at the top."
  - alignRight: false
    $name: Align to Right
    $description: "If enabled, the FPS counter is aligned to the right. Otherwise, it's on the left."
  - refreshRate: 500
    $name: Display Refresh Rate (ms)
    $description: "The time interval in milliseconds at which the displayed FPS value updates."
  $name: FPS Counter

- layout:
  - centerToCursorX: false
    $name: Center Horizontally to Cursor
    $description: "If enabled, the FPS counter and Keystroke overlay will be horizontally centered relative to the cursor's X position."
  $name: Layout

- bypassSystemCursor: false
  $name: Bypass System Cursor (GPU Rendered)
  $description: "If enabled, extracts and draws the active system cursor shape on the GPU, allowing smooth animations and custom rotation."
- bypassHideSystemCursor: true
  $name: Hide System Cursor (Bypass Mode)
  $description: "If enabled along with 'Bypass System Cursor', hides the standard Windows system cursor globally."
- rotateCursorWithMovement: true
  $name: Rotate Cursor with Movement
  $description: "If enabled, rotates the cursor pointing direction to match the direction of cursor movement."
- cursorRotationSmoothing: 5
  $name: Cursor Rotation Direction Smoothing
  $description: "Number of frames to average/estimate cursor direction vector. Higher = smoother rotation lag. Range 1-30."

- pyramidalCursor:
  - enabled: false
    $name: Enable 3D Pyramid Cursor
    $description: "Replaces the cursor with a rotating 3D pyramid with dots at the base corners."
  - baseSize: 30
    $name: Base Radius (px)
    $description: "Radius of the triangular base. Range 5-100."
  - height: 60
    $name: Pyramid Height (px)
    $description: "Length from apex to base center. Range 10-200."
  - spinSpeed: 90
    $name: Spin Speed (deg/sec)
    $description: "Rotation speed around the central axis. Range 0-720."
  - concaveDepth: 8
    $name: Concave Depth (px)
    $description: "How much the base is pushed inward. Range 0-40."
  - color: FF4488FF
    $name: Face Color (AARRGGBB hex)
  - dotSize: 6
    $name: Base Dot Size (px)
    $description: "Radius of dots at base corners. Range 2-20."
  - dotColor: FFFFFF00
    $name: Base Dot Color (AARRGGBB hex)

- enableDiagnosticLog: false
  $name: Enable Diagnostic Logging
  $description: "Verbose debug output. Disable for normal use."
*/
// ==/WindhawkModSettings==

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

#ifndef OCR_NORMAL
#define OCR_NORMAL 32512
#endif

#ifndef OCR_IBEAM
#define OCR_IBEAM       32513
#define OCR_WAIT        32514
#define OCR_CROSS       32515
#define OCR_UP          32516
#define OCR_SIZENWSE    32642
#define OCR_SIZENESW    32643
#define OCR_SIZEWE      32644
#define OCR_SIZENS      32645
#define OCR_SIZEALL     32646
#define OCR_NO          32648
#define OCR_HAND        32649
#endif

#include <set>
#include <map>
#include <windhawk_utils.h>
#include <windows.h>
typedef USHORT USAGE;
typedef USHORT *PUSAGE;
#include <hidpi.h>
#include <d3d11.h>
#include <d3dcompiler.h>
#include <dcomp.h>
#include <dxgi1_3.h>
#include <dwmapi.h>
#include <wrl/client.h>
#include <atomic>
#include <thread>
#include <mutex>
// Dynamic array vectors omitted to comply with heap allocations rules
#include <vector>
#include <chrono>
#include <cmath>
#include <algorithm>
#include <cstdint>
#include <cstring>
#include <avrt.h>

using Microsoft::WRL::ComPtr;

// =============================================================================
// RUNTIME SETTINGS
// =============================================================================

struct LayerSettings {
    bool     enabled;
    float    widthFactor;
    float    alphaFactor;
    uint32_t startARGB;
    uint32_t endARGB;
    float    startBlur;
    float    endBlur;
};

struct RippleSettings {
    bool     enabled;
    float    maxDiameter;
    float    startWidth;
    int      durationMs;
    uint32_t leftARGB;
    uint32_t rightARGB;
    uint32_t middleARGB;
    bool     enableClickScaling;
    int      clickScaleFactor;
    int      clickScaleDuration;
};

struct CursorHeadSettings {
    bool     enabled;
    bool     filled;
    uint32_t colorARGB;
    float    size;
    float    outlineWidth;
    float    squishIntensity;
    float    squishSmoothing;
    bool     hideSystemCursor;
};

struct ParticleBurstSettings {
    bool  enabled;
    int   count;
    float speed;
    int   lifetimeMs;
    float size;
    float friction;
    float gravity;
};

struct SatelliteSettings {
    bool     enabled;
    int      count;
    float    orbitDiameter;
    float    satelliteSize;
    bool     filled;
    float    outlineWidth;
    uint32_t colorARGB;
    float    speed;
    bool     enableDualRing;
    float    dualSpeed;
    bool     showOrbitRing;
    float    ringWidth;
    uint32_t ringColorARGB;
};

struct PyramidalCursorSettings {
    bool     enabled;
    float    baseRadius;
    float    height;
    float    spinSpeed;
    float    concaveDepth;
    uint32_t colorARGB;
    float    dotSize;
    uint32_t dotColorARGB;
};

struct FpsCounterSettings {
    bool enabled;
    bool alignBottom;
    bool alignRight;
    int refreshRate;
};

struct KeystrokeSettings {
    bool enabled;
};

struct MouseClickOverlaySettings {
    bool enabled;
    int duration;
};

struct LayoutSettings {
    bool centerToCursorX;
};

struct Settings {
    float headSpring, headFriction, bodySpring, bodyFriction;
    int   trailLength, positionSkip;
    float cursorSize, minTrailWidth, velWidthMul, velAlphaMul;
    int   interpSteps, fadeMode;
    bool  enableGradient, rainbowMode, adaptiveQuality, enableTrail;
    int   rainbowSpeed;
    LayerSettings       layers[4];
    RippleSettings      ripple;
    CursorHeadSettings  cursorHead;
    ParticleBurstSettings particleBurst;
    SatelliteSettings   satellite;
    FpsCounterSettings  fpsCounter;
    KeystrokeSettings   keystroke;
    MouseClickOverlaySettings mouseClick;
    LayoutSettings      layout;
    bool bypassSystemCursor, bypassHideSystemCursor, rotateCursorWithMovement;
    int  cursorRotationSmoothing;
    PyramidalCursorSettings pyramidalCursor;
    bool diagnosticLog;
};

static Settings g_settings = {};

// =============================================================================
// CAPACITY LIMITS
// =============================================================================

constexpr int kMaxTrailPoints      = 100;
constexpr int kMaxInterpSteps      = 10;
constexpr int kMaxLayers           = 4;
constexpr int kMaxRipples          = 256;
constexpr int kMaxParticles        = 1024;
constexpr int kMaxSatellites       = 24; // count * 2 for dual ring
constexpr int kMaxSamplesPerLayer  = (kMaxTrailPoints - 1) * kMaxInterpSteps + 1;
constexpr int kMaxVerticesPerLayer = kMaxSamplesPerLayer * 2 + 128;
constexpr int kMaxTotalVertices    = kMaxVerticesPerLayer * kMaxLayers * 11; // 1 cursor + 10 touch trails
constexpr int kMaxCircleInst       = kMaxRipples + kMaxSatellites + 2 + 1500; // ripples + satellites + (head, orbit ring) + text glyph pixels
constexpr float kReferenceFrameTime = 1.0f / 120.0f;

#define TIMER_ID_DISPLAY_CHANGE 1
#define OVERLAY_WINDOW_CLASS    (L"WHCursorOverlayD3D_d3d-cursor-effectzszs")

// =============================================================================
// DATA STRUCTURES
// =============================================================================

struct TrailPoint {
    float x, y, vx, vy, speed;
};

struct Sample {
    float x, y, nx, ny, speed, progress;
};

struct TrailVertex {
    float x, y, r, g, b, a;
    float u, v; // u = blur intensity, v = distance from cross-section center (-1 to 1)
};

struct GpuCursorVertex {
    float x, y;
    float u, v;
};

// Shared instance format for ripples, satellites, orbit rings, cursor head
struct CircleInstance {
    float cx, cy;
    float radX, radY;
    float angle;
    float thickness; // <0 = filled, >0 = ring outline
    float r, g, b, a;
};

// Particle instance (simple soft dot)
struct ParticleVertex {
    float x, y;     // center
    float size;
    float r, g, b, a;
};

struct Ripple {
    POINT pos;
    std::chrono::steady_clock::time_point startTime;
    uint32_t argb;
};

struct Particle {
    float x, y, vx, vy;
    std::chrono::steady_clock::time_point startTime;
    uint32_t argb;
};

struct SatelliteState {
    float angle;
    float mirrorAngle;
};

struct SquishyState {
    float posX, posY;
    float prevX, prevY;
    float currentScale, currentAngle;
    float targetScale, targetAngle;
};

struct TouchTrailState {
    DWORD pointerId = 0;
    bool active = false;
    float fadeAlpha = 0.0f;
    std::vector<TrailPoint> trail;
    std::vector<Sample> samples;
    POINT lastPos = {0, 0};
};

struct FrameCB {
    float invHalfResX, invHalfResY;
    float _pad0, _pad1;
};

// =============================================================================
// GLOBAL STATE
// =============================================================================

namespace {

std::atomic<bool>  g_running{false};
std::atomic<bool>  g_keystrokeEnabled{true};
std::atomic<bool>  g_unloading{false};
std::thread        g_renderThread;
std::atomic<bool>  g_settingsChanged{false};
std::thread        g_mouseHookThread;
std::atomic<DWORD> g_mouseHookThreadId{0};
HHOOK              g_mouseHook = nullptr;

std::thread        g_keyboardHookThread;
std::atomic<DWORD> g_keyboardHookThreadId{0};
HHOOK              g_keyboardHook = nullptr;

HWND g_overlayWnd = nullptr;
int  g_vScreenX = 0, g_vScreenY = 0;
int  g_vScreenW = 0, g_vScreenH = 0;

// Trail state
std::vector<TrailPoint> g_trail;
std::vector<Sample>     g_samples;
POINT                   g_lastUsedMousePos = {0, 0};
long long               g_frameCounter = 0;

// Frame timing
std::chrono::steady_clock::time_point g_lastFrameTime;
float g_deltaTime = kReferenceFrameTime;

// Diagnostic
int g_diagFrameCount = 0;
std::chrono::steady_clock::time_point g_diagLastLogTime;

// Ripple + particle state
std::mutex          g_effectsMutex;
std::vector<Ripple> g_ripples;
std::vector<Particle> g_particles;

namespace KeystrokeDisplay {
    std::set<DWORD> pressedKeys;
    std::wstring displayString;
    std::mutex keyMutex;
    std::atomic<bool> keystrokeChanged{false};
}

namespace MouseClickDisplay {
    std::wstring displayString;
    std::chrono::steady_clock::time_point startTime;
    std::mutex clickMutex;
    std::atomic<bool> clickChanged{false};
}

namespace FpsTracker {
    std::atomic<int> frameCount = 0;
    std::atomic<int> lastFps = 0;
    std::chrono::steady_clock::time_point lastFpsTime;
}

// Touch trails state
TouchTrailState g_touchTrails[10] = {};

// Rainbow hue state
float g_rainbowHue = 0.0f;

// Squishy cursor head state
SquishyState g_squishy = {};

// Satellite state
std::vector<SatelliteState> g_satellites;

// Adaptive quality
float g_maxRecentSpeed = 0.0f;
    
HICON g_hOriginalCursor = nullptr;

struct GpuCursorState {
    ComPtr<ID3D11Texture2D>          texture;
    ComPtr<ID3D11ShaderResourceView> srv;
    HCURSOR                          hLastCursor = nullptr;
    int                              width = 0;
    int                              height = 0;
    int                              hotspotX = 0;
    int                              hotspotY = 0;
    
    // Rotation tracking
    float                            currentAngle = -135.0f * (float)M_PI / 180.0f;
    
    // Animation Tracking parameters
    float                            currentScale = 1.0f;
    bool                             isAnimating = false;
    std::chrono::steady_clock::time_point animStartTime;
    
    bool                             shouldRotate = false;
    bool                             visible = false;
} g_gpuCursor;

struct PyramidalState {
    float spinAngle = 0.0f;
    float dirX = 0.0f, dirY = -1.0f;
    float cornerX[3], cornerY[3];
    float concX, concY;
    float faceBrightness[3];
    float baseBrightness;
    bool valid = false;
} g_pyramidState;

struct RawMouseHistory {
    float x = 0.0f, y = 0.0f;
    float vx = 0.0f, vy = 0.0f;
    float angle = -135.0f * (float)M_PI / 180.0f;
    bool isMoving = false;
} g_rawMouse;

std::map<UINT, HCURSOR> g_originalCursors;
std::map<UINT, HCURSOR> g_sharedCursors;

namespace CursorScaling {
    std::mutex mutex;
    HCURSOR hCursorToScale = nullptr;
    const std::vector<UINT> kCursorTypesToOverride = {
        OCR_NORMAL, OCR_IBEAM, OCR_WAIT, OCR_CROSS, OCR_UP, OCR_SIZENWSE,
        OCR_SIZENESW, OCR_SIZEWE, OCR_SIZENS, OCR_SIZEALL, OCR_NO, OCR_HAND
    };
    std::chrono::steady_clock::time_point scaleStartTime;
    bool isScalingActive = false;
    int targetScaleFactor = 150;
    // Cache the last applied integer scaled size so we don't reinstall all
    // 12 system cursors every frame when the rounded size hasn't changed.
    int lastAppliedWidth = -1;
    int lastAppliedHeight = -1;
}

// D3D resources
ComPtr<ID3D11Device>            g_device;
ComPtr<ID3D11DeviceContext>     g_context;
ComPtr<IDXGIDevice1>            g_dxgiDevice;
ComPtr<IDXGIFactory2>           g_dxgiFactory;

ComPtr<ID3D11VertexShader>      g_trailVS;
ComPtr<ID3D11PixelShader>       g_trailPS;
ComPtr<ID3D11InputLayout>       g_trailIL;
ComPtr<ID3D11VertexShader>      g_circleVS;
ComPtr<ID3D11PixelShader>       g_circlePS;
ComPtr<ID3D11InputLayout>       g_circleIL;
ComPtr<ID3D11VertexShader>      g_particleVS;
ComPtr<ID3D11PixelShader>       g_particlePS;
ComPtr<ID3D11InputLayout>       g_particleIL;
ComPtr<ID3D11BlendState>        g_blendPremul;
ComPtr<ID3D11RasterizerState>   g_rasterState;
ComPtr<ID3D11Buffer>            g_frameCB;
ComPtr<ID3D11Buffer>            g_trailVB;
ComPtr<ID3D11Buffer>            g_circleQuadVB;
ComPtr<ID3D11Buffer>            g_circleInstVB;
ComPtr<ID3D11Buffer>            g_particleQuadVB;
ComPtr<ID3D11Buffer>            g_particleInstVB;

ComPtr<ID3D11VertexShader>      g_gpuCursorVS;
ComPtr<ID3D11PixelShader>       g_gpuCursorPS;
ComPtr<ID3D11InputLayout>       g_gpuCursorIL;
ComPtr<ID3D11Buffer>            g_gpuCursorVB;
ComPtr<ID3D11SamplerState>      g_cursorSampler;

ComPtr<ID3D11VertexShader>      g_pyramidVS;
ComPtr<ID3D11PixelShader>       g_pyramidPS;
ComPtr<ID3D11InputLayout>       g_pyramidIL;
ComPtr<ID3D11Buffer>            g_pyramidVB;
ComPtr<ID3D11Buffer>            g_pyramidDotVB;

ComPtr<IDXGISwapChain1>         g_swapChain;
ComPtr<ID3D11RenderTargetView>  g_rtv;
ComPtr<IDCompositionDevice>     g_compositionDevice;
ComPtr<IDCompositionTarget>     g_compositionTarget;
ComPtr<IDCompositionVisual>     g_compositionVisual;

} // anonymous namespace

// =============================================================================
// FORWARD DECLARATIONS
// =============================================================================

bool InitDirectX();
void UninitDirectX();
bool CreatePipelineObjects();
void ReleasePipelineObjects();
bool CreateOverlayResources();
void ReleaseOverlayResources();
void HandleDeviceLost();
void HandleDisplayChange();
void InitTrail(POINT pt);
void UpdateTrailPhysics(POINT pt);
void UpdateSquishyCursor(POINT pt);
void UpdateTouchTrailPhysics(int slot);
void UpdateSatellites();
void UpdateGpuCursorTexture();
void UpdateGpuCursorAnimation();
void DrawGpuCursor();
void Update3DPyramidCursor();
void Draw3DPyramidCursor();
void BuildSamplesGeneric(const std::vector<TrailPoint>& trail, std::vector<Sample>& samples);
UINT BuildLayerVerticesGeneric(TrailVertex* dst, const LayerSettings& layer, const std::vector<Sample>& samples, float alphaScalar, UINT maxVertices);
UINT BuildCircleInstances(CircleInstance* dst);
UINT BuildParticleInstances(ParticleVertex* dst);
void ProcessRawTouch(RAWINPUT* raw);

// =============================================================================
// SHADER SOURCE
// =============================================================================

static const char* kHLSL = R"HLSL(

cbuffer Frame : register(b0)
{
    float2 invHalfRes;
    float2 _pad;
};

// ----- Trail (triangle-strip ribbon) -----
struct TrailVSIn  { float2 pos : POSITION; float4 col : COLOR; float2 tex : TEXCOORD; };
struct TrailVSOut { float4 pos : SV_Position; float4 col : COLOR; float2 tex : TEXCOORD; };

TrailVSOut TrailVS(TrailVSIn i)
{
    TrailVSOut o;
    o.pos = float4(i.pos.x * invHalfRes.x - 1.0,
                   1.0 - i.pos.y * invHalfRes.y,
                   0.0, 1.0);
    o.col = i.col;
    o.tex = i.tex;
    return o;
}
float4 TrailPS(TrailVSOut i) : SV_Target 
{ 
    float blur = clamp(i.tex.x, 0.001, 1.0);
    float v = abs(i.tex.y);
    float alpha = 1.0 - smoothstep(1.0 - blur, 1.0, v);
    return i.col * alpha; 
}

// ----- Circles (instanced SDF ring / filled circle) -----
// Used for ripples, satellites, orbit rings, and squishy cursor head
struct CircleVSIn
{
    float2 corner    : POSITION;
    float2 center    : I_CENTER;
    float2 radius    : I_RADIUS;
    float  angle     : I_ANGLE;
    float  thickness : I_THICK;
    float4 color     : I_COLOR;
};
struct CircleVSOut
{
    float4 pos   : SV_Position;
    float2 local : TEXCOORD0;
    float2 rad   : TEXCOORD1;
    float  thick : TEXCOORD2;
    float4 color : TEXCOORD3;
};

CircleVSOut CircleVS(CircleVSIn i)
{
    CircleVSOut o;
    float maxRad = max(i.radius.x, i.radius.y);
    float halfSize = maxRad + max(i.thickness, 0.0) * 0.5 + 2.0;
    float2 localPos = i.corner * halfSize;
    float2 world = i.center + localPos;
    o.pos   = float4(world.x * invHalfRes.x - 1.0,
                     1.0 - world.y * invHalfRes.y, 0.0, 1.0);

    float cosA = cos(i.angle);
    float sinA = sin(i.angle);
    o.local.x = localPos.x * cosA + localPos.y * sinA;
    o.local.y = -localPos.x * sinA + localPos.y * cosA;

    o.rad   = i.radius;
    o.thick = i.thickness;
    o.color = i.color;
    return o;
}

float4 CirclePS(CircleVSOut i) : SV_Target
{
    float lenL = length(i.local);
    float dNorm = length(i.local / i.rad);
    float dist = (lenL > 0.0 && dNorm > 0.0) ? (lenL * (dNorm - 1.0) / dNorm) : -1.0;

    float alpha;
    if (i.thick < 0.0) {
        // Filled mode
        alpha = 1.0 - smoothstep(-1.0, 1.0, dist);
    } else {
        // Ring mode
        float ringDist = abs(dist);
        float halfT    = i.thick * 0.5;
        alpha = 1.0 - smoothstep(halfT - 1.0, halfT + 1.0, ringDist);
    }
    return i.color * alpha;
}

// ----- Particles (instanced soft dots) -----
struct ParticleVSIn
{
    float2 corner  : POSITION;
    float2 center  : I_CENTER;
    float  psize   : I_SIZE;
    float4 color   : I_COLOR;
};
struct ParticleVSOut
{
    float4 pos   : SV_Position;
    float2 local : TEXCOORD0;
    float  psize : TEXCOORD1;
    float4 color : TEXCOORD2;
};

ParticleVSOut ParticleVS(ParticleVSIn i)
{
    ParticleVSOut o;
    float halfSize = i.psize + 1.0;
    float2 world = i.center + i.corner * halfSize;
    o.pos   = float4(world.x * invHalfRes.x - 1.0,
                     1.0 - world.y * invHalfRes.y, 0.0, 1.0);
    o.local = i.corner * halfSize;
    o.psize = i.psize;
    o.color = i.color;
    return o;
}

float4 ParticlePS(ParticleVSOut i) : SV_Target
{
    float d = length(i.local);
    float alpha = 1.0 - smoothstep(i.psize * 0.5, i.psize, d);
    return i.color * alpha;
}

// ----- GPU Cursor Shader -----
struct GpuCursorVSIn
{
    float2 pos : POSITION;
    float2 tex : TEXCOORD;
};
struct GpuCursorVSOut
{
    float4 pos : SV_Position;
    float2 tex : TEXCOORD;
};

GpuCursorVSOut GpuCursorVS(GpuCursorVSIn i)
{
    GpuCursorVSOut o;
    o.pos = float4(i.pos.x * invHalfRes.x - 1.0,
                   1.0 - i.pos.y * invHalfRes.y,
                   0.0, 1.0);
    o.tex = i.tex;
    return o;
}

Texture2D    CursorTexture : register(t0);
SamplerState LinearSampler : register(s0);

float4 GpuCursorPS(GpuCursorVSOut i) : SV_Target
{
    return CursorTexture.Sample(LinearSampler, i.tex);
}

// ----- 3D Pyramid Cursor -----
struct PyramidVSOut
{
    float4 pos : SV_Position;
    float4 col : COLOR;
};

PyramidVSOut PyramidVS(float2 pos : POSITION, float4 col : COLOR)
{
    PyramidVSOut o;
    o.pos = float4(pos.x * invHalfRes.x - 1.0,
                    1.0 - pos.y * invHalfRes.y,
                    0.0, 1.0);
    o.col = col;
    return o;
}

float4 PyramidPS(PyramidVSOut i) : SV_Target
{
    return i.col;
}

)HLSL";

// =============================================================================
// HELPERS
// =============================================================================

HMODULE GetCurrentModuleHandle() {
    HMODULE h = nullptr;
    GetModuleHandleEx(
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
            GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        reinterpret_cast<LPCWSTR>(&GetCurrentModuleHandle), &h);
    return h;
}

void RefreshVirtualScreenMetrics() {
    g_vScreenX = GetSystemMetrics(SM_XVIRTUALSCREEN);
    g_vScreenY = GetSystemMetrics(SM_YVIRTUALSCREEN);
    g_vScreenW = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    g_vScreenH = GetSystemMetrics(SM_CYVIRTUALSCREEN);
    if (g_vScreenW <= 0) g_vScreenW = 1;
    if (g_vScreenH <= 0) g_vScreenH = 1;
}

uint32_t ParseARGB(PCWSTR hex) {
    if (!hex || !hex[0]) return 0xFFFFFFFF;
    if (hex[0] == L'#') hex++; // Handle optional '#' prefix
    size_t len = wcslen(hex);
    uint32_t val = 0;
    for (size_t i = 0; i < len && i < 8; ++i) {
        val <<= 4;
        wchar_t c = hex[i];
        if (c >= L'0' && c <= L'9')      val |= (c - L'0');
        else if (c >= L'A' && c <= L'F') val |= (c - L'A' + 10);
        else if (c >= L'a' && c <= L'f') val |= (c - L'a' + 10);
    }
    if (len == 6) {
        val |= 0xFF000000; // Auto-fill alpha for standard RRGGBB inputs
    }
    return val;
}

inline void UnpackPremul(uint32_t argb,
                        float& r, float& g, float& b, float& a) {
    a = float((argb >> 24) & 0xFF) / 255.0f;
    r = float((argb >> 16) & 0xFF) / 255.0f * a;
    g = float((argb >>  8) & 0xFF) / 255.0f * a;
    b = float((argb      ) & 0xFF) / 255.0f * a;
}

inline void LerpARGBPremul(uint32_t c0, uint32_t c1, float t,
                           float& r, float& g, float& b, float& a) {
    t = std::clamp(t, 0.0f, 1.0f);
    auto u8 = [](uint32_t c, int s) { return float((c >> s) & 0xFF) / 255.0f; };
    float a0 = u8(c0, 24), a1 = u8(c1, 24);
    a = a0 + (a1 - a0) * t;
    r = (u8(c0, 16) + (u8(c1, 16) - u8(c0, 16)) * t) * a;
    g = (u8(c0,  8) + (u8(c1,  8) - u8(c0,  8)) * t) * a;
    b = (u8(c0,  0) + (u8(c1,  0) - u8(c0,  0)) * t) * a;
}

inline float ApplyFadeCurve(float progress, int mode) {
    switch (mode) {
        case 1: return 1.0f - progress * progress;
        case 2: return std::exp(-progress * 3.0f);
        case 3: return 1.0f / (1.0f + std::exp(8.0f * (progress - 0.5f)));
        default: return 1.0f - progress;
    }
}

inline void CatmullRom(float p0x, float p0y, float p1x, float p1y,
                       float p2x, float p2y, float p3x, float p3y,
                       float t, float& outX, float& outY) {
    float t2 = t * t, t3 = t2 * t;
    outX = 0.5f * ((2*p1x) + (-p0x+p2x)*t + (2*p0x-5*p1x+4*p2x-p3x)*t2 + (-p0x+3*p1x-3*p2x+p3x)*t3);
    outY = 0.5f * ((2*p1y) + (-p0y+p2y)*t + (2*p0y-5*p1y+4*p2y-p3y)*t2 + (-p0y+3*p1y-3*p2y+p3y)*t3);
}

// HSL to premultiplied ARGB (full alpha)
uint32_t HSLToARGB(float hue, float sat, float lit) {
    hue = fmodf(hue, 360.0f);
    if (hue < 0) hue += 360.0f;
    float c = (1.0f - fabsf(2.0f * lit - 1.0f)) * sat;
    float x = c * (1.0f - fabsf(fmodf(hue / 60.0f, 2.0f) - 1.0f));
    float m = lit - c / 2.0f;
    float r1, g1, b1;
    if (hue < 60)       { r1 = c; g1 = x; b1 = 0; }
    else if (hue < 120) { r1 = x; g1 = c; b1 = 0; }
    else if (hue < 180) { r1 = 0; g1 = c; b1 = x; }
    else if (hue < 240) { r1 = 0; g1 = x; b1 = c; }
    else if (hue < 300) { r1 = x; g1 = 0; b1 = c; }
    else                { r1 = c; g1 = 0; b1 = x; }
    uint8_t rr = uint8_t((r1 + m) * 255.0f);
    uint8_t gg = uint8_t((g1 + m) * 255.0f);
    uint8_t bb = uint8_t((b1 + m) * 255.0f);
    return (0xFFu << 24) | (rr << 16) | (gg << 8) | bb;
}

HCURSOR CreateTaggedCursor(int tagIndex) {
    int w = GetSystemMetrics(SM_CXCURSOR);
    int h = GetSystemMetrics(SM_CYCURSOR);
    if (w <= 0) w = 32;
    if (h <= 0) h = 32;
    
    int widthInBytes = ((w + 15) / 16) * 2;
    int maskSize = widthInBytes * h;
    
    std::vector<uint8_t> andMask(maskSize, 0xFF);
    std::vector<uint8_t> xorMask(maskSize, 0x00);
    
    // Pass nullptr for hInstance to prevent cursor handle invalidation when DLL is unloaded
    return CreateCursor(nullptr, 29, tagIndex, w, h, andMask.data(), xorMask.data());
}

void CacheOriginalCursors() {
    for (UINT id : CursorScaling::kCursorTypesToOverride) {
        HCURSOR hCur = (HCURSOR)LoadImage(nullptr, MAKEINTRESOURCE(id), IMAGE_CURSOR, 0, 0, LR_SHARED);
        if (!hCur) {
            hCur = LoadCursor(nullptr, MAKEINTRESOURCE(id));
        }
        if (hCur) {
            g_sharedCursors[id] = hCur;
            if (g_originalCursors.find(id) == g_originalCursors.end()) {
                g_originalCursors[id] = CopyCursor(hCur);
            }
        }
    }
}

void ApplyBypassCursors() {
    for (size_t i = 0; i < CursorScaling::kCursorTypesToOverride.size(); ++i) {
        UINT id = CursorScaling::kCursorTypesToOverride[i];
        HCURSOR hInv = CreateTaggedCursor(static_cast<int>(i));
        if (hInv) {
            if (!SetSystemCursor(hInv, id)) {
                DestroyCursor(hInv);
            }
        }
    }
}

void RestoreSystemCursors() {
    for (const auto& pair : g_originalCursors) {
        if (pair.second) {
            HCURSOR hCopy = CopyCursor(pair.second);
            if (hCopy) {
                if (!SetSystemCursor(hCopy, pair.first)) {
                    DestroyCursor(hCopy);
                }
            }
        }
    }
    SystemParametersInfo(SPI_SETCURSORS, 0, nullptr, 0);
    Sleep(50); // Allow OS threads to process cursor change messages
}

// =============================================================================
// LOAD SETTINGS
// =============================================================================

void LoadSettings() {
    std::lock_guard<std::mutex> lk(g_effectsMutex);

    g_settings.headSpring    = static_cast<float>(std::clamp(Wh_GetIntSetting(L"headSpring"), 1, 500));
    g_settings.headFriction  = static_cast<float>(std::clamp(Wh_GetIntSetting(L"headFriction"), 0, 99));
    g_settings.bodySpring    = static_cast<float>(std::clamp(Wh_GetIntSetting(L"spring"), 1, 500));
    g_settings.bodyFriction  = static_cast<float>(std::clamp(Wh_GetIntSetting(L"friction"), 0, 99));
    g_settings.trailLength   = std::clamp(Wh_GetIntSetting(L"trailLength"), 5, 100);
    g_settings.positionSkip  = std::clamp(Wh_GetIntSetting(L"positionHistorySkip"), 0, 10);

    g_settings.cursorSize    = static_cast<float>(std::clamp(Wh_GetIntSetting(L"cursorSize"), 5, 200));
    g_settings.minTrailWidth = static_cast<float>(std::clamp(Wh_GetIntSetting(L"minTrailWidth"), 1, 20));
    g_settings.velWidthMul   = static_cast<float>(std::clamp(Wh_GetIntSetting(L"velocityWidthMultiplier"), 0, 50)) / 10.0f;
    g_settings.velAlphaMul   = static_cast<float>(std::clamp(Wh_GetIntSetting(L"velocityAlphaMultiplier"), 0, 50)) / 10.0f;
    g_settings.interpSteps   = std::clamp(Wh_GetIntSetting(L"interpolationSteps"), 1, 10);
    g_settings.fadeMode      = std::clamp(Wh_GetIntSetting(L"fadeMode"), 0, 3);
    g_settings.enableGradient = Wh_GetIntSetting(L"enableGradient");
    g_settings.enableTrail    = Wh_GetIntSetting(L"enableTrail");
    g_settings.rainbowMode   = Wh_GetIntSetting(L"rainbowMode");
    g_settings.rainbowSpeed  = std::clamp(Wh_GetIntSetting(L"rainbowSpeed"), 1, 20);
    g_settings.adaptiveQuality = Wh_GetIntSetting(L"adaptiveQuality");

    auto loadLayer = [](const wchar_t* prefix, LayerSettings& layer) {
        wchar_t key[128];
        swprintf_s(key, L"%s.enabled", prefix);
        layer.enabled = Wh_GetIntSetting(key) != 0;
        PCWSTR str;
        swprintf_s(key, L"%s.startColor", prefix);
        str = Wh_GetStringSetting(key); layer.startARGB = ParseARGB(str); Wh_FreeStringSetting(str);
        swprintf_s(key, L"%s.endColor", prefix);
        str = Wh_GetStringSetting(key); layer.endARGB = ParseARGB(str); Wh_FreeStringSetting(str);
        swprintf_s(key, L"%s.widthFactor", prefix);
        layer.widthFactor = static_cast<float>(std::clamp(Wh_GetIntSetting(key), 1, 500)) / 100.0f;
        swprintf_s(key, L"%s.alphaFactor", prefix);
        layer.alphaFactor = static_cast<float>(std::clamp(Wh_GetIntSetting(key), 0, 100)) / 100.0f;
        swprintf_s(key, L"%s.startBlur", prefix);
        layer.startBlur = static_cast<float>(std::clamp(Wh_GetIntSetting(key), 0, 100)) / 100.0f;
        swprintf_s(key, L"%s.endBlur", prefix);
        layer.endBlur = static_cast<float>(std::clamp(Wh_GetIntSetting(key), 0, 100)) / 100.0f;
    };
    loadLayer(L"layer1", g_settings.layers[0]);
    loadLayer(L"layer2", g_settings.layers[1]);
    loadLayer(L"layer3", g_settings.layers[2]);
    loadLayer(L"layer4", g_settings.layers[3]);

    // Cursor head
    g_settings.cursorHead.enabled        = Wh_GetIntSetting(L"cursorHead.enabled") != 0;
    g_settings.cursorHead.filled         = Wh_GetIntSetting(L"cursorHead.filled") != 0;
    { PCWSTR s = Wh_GetStringSetting(L"cursorHead.color");
      g_settings.cursorHead.colorARGB = ParseARGB(s); Wh_FreeStringSetting(s); }
    g_settings.cursorHead.size           = static_cast<float>(std::clamp(Wh_GetIntSetting(L"cursorHead.size"), 5, 100));
    g_settings.cursorHead.outlineWidth   = static_cast<float>(std::clamp(Wh_GetIntSetting(L"cursorHead.outlineWidth"), 1, 20));
    g_settings.cursorHead.squishIntensity = static_cast<float>(std::clamp(Wh_GetIntSetting(L"cursorHead.squishIntensity"), 0, 10));
    g_settings.cursorHead.squishSmoothing = static_cast<float>(std::clamp(Wh_GetIntSetting(L"cursorHead.squishSmoothing"), 10, 100));
    g_settings.cursorHead.hideSystemCursor = Wh_GetIntSetting(L"cursorHead.hideSystemCursor") != 0;

    // Ripples
    g_settings.ripple.enabled      = Wh_GetIntSetting(L"rippleEffect.enabled") != 0;
    g_settings.ripple.maxDiameter  = static_cast<float>(std::clamp(Wh_GetIntSetting(L"rippleEffect.maxDiameter"), 10, 500));
    g_settings.ripple.startWidth   = static_cast<float>(std::clamp(Wh_GetIntSetting(L"rippleEffect.startWidth"), 1, 100));
    g_settings.ripple.durationMs   = std::clamp(Wh_GetIntSetting(L"rippleEffect.duration"), 100, 5000);
    auto loadColor = [](const wchar_t* k) -> uint32_t {
        PCWSTR s = Wh_GetStringSetting(k); uint32_t c = ParseARGB(s); Wh_FreeStringSetting(s); return c; };
    g_settings.ripple.leftARGB   = loadColor(L"rippleEffect.leftClickColor");
    g_settings.ripple.rightARGB  = loadColor(L"rippleEffect.rightClickColor");
    g_settings.ripple.middleARGB = loadColor(L"rippleEffect.middleClickColor");
    g_settings.ripple.enableClickScaling = Wh_GetIntSetting(L"rippleEffect.enableClickScaling") != 0;
    g_settings.ripple.clickScaleFactor   = std::clamp(Wh_GetIntSetting(L"rippleEffect.clickScaleFactor"), 1, 500);
    g_settings.ripple.clickScaleDuration = std::clamp(Wh_GetIntSetting(L"rippleEffect.clickScaleDuration"), 50, 500);

    // Particle burst
    g_settings.particleBurst.enabled    = Wh_GetIntSetting(L"particleBurst.enabled") != 0;
    g_settings.particleBurst.count      = std::clamp(Wh_GetIntSetting(L"particleBurst.count"), 4, 64);
    g_settings.particleBurst.speed      = static_cast<float>(std::clamp(Wh_GetIntSetting(L"particleBurst.speed"), 50, 1000));
    g_settings.particleBurst.lifetimeMs = std::clamp(Wh_GetIntSetting(L"particleBurst.lifetime"), 100, 3000);
    g_settings.particleBurst.size       = static_cast<float>(std::clamp(Wh_GetIntSetting(L"particleBurst.size"), 1, 20));
    g_settings.particleBurst.friction   = static_cast<float>(std::clamp(Wh_GetIntSetting(L"particleBurst.friction"), 50, 99));
    g_settings.particleBurst.gravity    = static_cast<float>(std::clamp(Wh_GetIntSetting(L"particleBurst.gravity"), 0, 1000));

    // Satellites
    g_settings.satellite.enabled        = Wh_GetIntSetting(L"satelliteEffect.enabled") != 0;
    g_settings.satellite.count          = std::clamp(Wh_GetIntSetting(L"satelliteEffect.count"), 1, 12);
    g_settings.satellite.orbitDiameter  = static_cast<float>(std::clamp(Wh_GetIntSetting(L"satelliteEffect.orbitDiameter"), 10, 500));
    g_settings.satellite.satelliteSize  = static_cast<float>(std::clamp(Wh_GetIntSetting(L"satelliteEffect.satelliteSize"), 2, 50));
    g_settings.satellite.filled         = Wh_GetIntSetting(L"satelliteEffect.filled") != 0;
    g_settings.satellite.outlineWidth   = static_cast<float>(std::clamp(Wh_GetIntSetting(L"satelliteEffect.outlineWidth"), 1, 10));
    g_settings.satellite.colorARGB      = loadColor(L"satelliteEffect.color");
    g_settings.satellite.speed          = static_cast<float>(Wh_GetIntSetting(L"satelliteEffect.speed"));
    g_settings.satellite.enableDualRing = Wh_GetIntSetting(L"satelliteEffect.enableDualRing") != 0;
    g_settings.satellite.dualSpeed      = static_cast<float>(Wh_GetIntSetting(L"satelliteEffect.dualSpeed"));
    g_settings.satellite.showOrbitRing  = Wh_GetIntSetting(L"satelliteEffect.showOrbitRing") != 0;
    g_settings.satellite.ringWidth      = static_cast<float>(std::clamp(Wh_GetIntSetting(L"satelliteEffect.ringWidth"), 1, 10));
    g_settings.satellite.ringColorARGB  = loadColor(L"satelliteEffect.ringColor");

    // Ported GDI+ Features
    g_settings.fpsCounter.enabled     = Wh_GetIntSetting(L"fpsCounter.enabled") != 0;
    g_settings.fpsCounter.alignBottom = Wh_GetIntSetting(L"fpsCounter.alignBottom") != 0;
    g_settings.fpsCounter.alignRight  = Wh_GetIntSetting(L"fpsCounter.alignRight") != 0;
    g_settings.fpsCounter.refreshRate = Wh_GetIntSetting(L"fpsCounter.refreshRate");
    g_settings.keystroke.enabled      = Wh_GetIntSetting(L"keystrokeOverlay.enabled") != 0;
    g_settings.mouseClick.enabled     = Wh_GetIntSetting(L"mouseClickOverlay.enabled") != 0;
    g_settings.mouseClick.duration    = std::clamp(Wh_GetIntSetting(L"mouseClickOverlay.duration"), 100, 5000);
    g_settings.layout.centerToCursorX = Wh_GetIntSetting(L"layout.centerToCursorX") != 0;

    // Pyramidal cursor
    g_settings.pyramidalCursor.enabled      = Wh_GetIntSetting(L"pyramidalCursor.enabled") != 0;
    g_settings.pyramidalCursor.baseRadius   = static_cast<float>(std::clamp(Wh_GetIntSetting(L"pyramidalCursor.baseSize"), 5, 100));
    g_settings.pyramidalCursor.height       = static_cast<float>(std::clamp(Wh_GetIntSetting(L"pyramidalCursor.height"), 10, 200));
    g_settings.pyramidalCursor.spinSpeed    = static_cast<float>(std::clamp(Wh_GetIntSetting(L"pyramidalCursor.spinSpeed"), 0, 720));
    g_settings.pyramidalCursor.concaveDepth = static_cast<float>(std::clamp(Wh_GetIntSetting(L"pyramidalCursor.concaveDepth"), 0, 40));
    { PCWSTR s = Wh_GetStringSetting(L"pyramidalCursor.color");
      g_settings.pyramidalCursor.colorARGB = ParseARGB(s); Wh_FreeStringSetting(s); }
    g_settings.pyramidalCursor.dotSize      = static_cast<float>(std::clamp(Wh_GetIntSetting(L"pyramidalCursor.dotSize"), 2, 20));
    { PCWSTR s = Wh_GetStringSetting(L"pyramidalCursor.dotColor");
      g_settings.pyramidalCursor.dotColorARGB = ParseARGB(s); Wh_FreeStringSetting(s); }

    g_settings.diagnosticLog = Wh_GetIntSetting(L"enableDiagnosticLog") != 0;

    bool wasBypass = g_settings.bypassSystemCursor;
    bool wasHideBypass = g_settings.bypassHideSystemCursor;
    g_settings.bypassSystemCursor = Wh_GetIntSetting(L"bypassSystemCursor") != 0;
    g_settings.bypassHideSystemCursor = Wh_GetIntSetting(L"bypassHideSystemCursor") != 0;
    g_settings.rotateCursorWithMovement = Wh_GetIntSetting(L"rotateCursorWithMovement") != 0;
    g_settings.cursorRotationSmoothing = std::clamp(Wh_GetIntSetting(L"cursorRotationSmoothing"), 1, 30);

    bool bypassStateChanged = (wasBypass != g_settings.bypassSystemCursor);
    bool hideStateChanged = (wasHideBypass != g_settings.bypassHideSystemCursor);

    if (bypassStateChanged || hideStateChanged) {
        if (g_settings.bypassSystemCursor) {
            CacheOriginalCursors();
            if (g_settings.bypassHideSystemCursor) {
                ApplyBypassCursors();
            } else {
                RestoreSystemCursors();
            }
        } else {
            RestoreSystemCursors();
        }
    }

    g_keystrokeEnabled.store(g_settings.keystroke.enabled);

    // Resize trail if length changed
    if ((int)g_trail.size() != g_settings.trailLength && !g_trail.empty()) {
        POINT pt; GetCursorPos(&pt); InitTrail(pt);
    }

    // Resize satellites if count changed
    if ((int)g_satellites.size() != g_settings.satellite.count) {
        g_satellites.resize(g_settings.satellite.count);
        float inc = 2.0f * (float)M_PI / g_settings.satellite.count;
        for (int i = 0; i < g_settings.satellite.count; ++i) {
            g_satellites[i].angle = i * inc;
            g_satellites[i].mirrorAngle = i * inc;
        }
    }

    // Clear touch arrays if layout rules modify length parameters
    for (int i = 0; i < 10; ++i) {
        if ((int)g_touchTrails[i].trail.size() != g_settings.trailLength && !g_touchTrails[i].trail.empty()) {
            g_touchTrails[i].trail.assign(g_settings.trailLength, TrailPoint{ float(g_touchTrails[i].lastPos.x), float(g_touchTrails[i].lastPos.y), 0, 0, 0 });
        }
    }

    Wh_Log(L"[SETTINGS] Loaded v0.1.3: enableTrail=%s trail=%d interp=%d rainbow=%s adaptive=%s head=%s satellites=%s particles=%s",
           g_settings.enableTrail ? L"ON" : L"OFF",
           g_settings.trailLength, g_settings.interpSteps,
           g_settings.rainbowMode ? L"ON" : L"OFF",
           g_settings.adaptiveQuality ? L"ON" : L"OFF",
           g_settings.cursorHead.enabled ? L"ON" : L"OFF",
           g_settings.satellite.enabled ? L"ON" : L"OFF",
           g_settings.particleBurst.enabled ? L"ON" : L"OFF");
}

// =============================================================================
// D3D INITIALIZATION
// =============================================================================

bool InitDirectX() {
    HRESULT hr;
    UINT flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_SINGLETHREADED;
    D3D_FEATURE_LEVEL fls[] = { D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_11_0,
                                D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_10_0 };
    D3D_FEATURE_LEVEL achievedFL;
    hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, flags,
                           fls, ARRAYSIZE(fls), D3D11_SDK_VERSION,
                           &g_device, &achievedFL, &g_context);
    if (FAILED(hr)) {
        hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_WARP, nullptr, flags,
                               fls, ARRAYSIZE(fls), D3D11_SDK_VERSION,
                               &g_device, &achievedFL, &g_context);
        if (FAILED(hr)) { Wh_Log(L"[INIT] Device creation failed: 0x%08X", hr); return false; }
    }
    hr = g_device.As(&g_dxgiDevice);
    if (FAILED(hr)) { Wh_Log(L"[INIT] IDXGIDevice1 query failed: 0x%08X", hr); return false; }
    g_dxgiDevice->SetMaximumFrameLatency(1);
    if (FAILED(CreateDXGIFactory2(0, IID_PPV_ARGS(&g_dxgiFactory)))) return false;
    return true;
}

void UninitDirectX() {
    g_gpuCursor.texture.Reset();
    g_gpuCursor.srv.Reset();
    if (g_context) g_context->ClearState();
    g_dxgiFactory.Reset(); g_dxgiDevice.Reset(); g_context.Reset(); g_device.Reset();
}

// =============================================================================
// PIPELINE OBJECTS
// =============================================================================

bool CreatePipelineObjects() {
    auto compile = [&](const char* entry, const char* target, ComPtr<ID3DBlob>& blob) -> bool {
        ComPtr<ID3DBlob> err;
        constexpr UINT kCompileFlags = D3DCOMPILE_OPTIMIZATION_LEVEL3
                                     | D3DCOMPILE_PARTIAL_PRECISION;
        HRESULT r = D3DCompile(kHLSL, strlen(kHLSL), nullptr, nullptr, nullptr,
                               entry, target, kCompileFlags, 0, &blob, &err);
        if (FAILED(r)) {
            if (err) Wh_Log(L"[SHADER] %S: %S", entry, (const char*)err->GetBufferPointer());
            return false;
        }
        return true;
    };

    ComPtr<ID3DBlob> tvsB, tpsB, cvsB, cpsB, pvsB, ppsB, gvsB, gpsB;
    if (!compile("TrailVS", "vs_4_0", tvsB) || !compile("TrailPS", "ps_4_0", tpsB) ||
        !compile("CircleVS", "vs_4_0", cvsB) || !compile("CirclePS", "ps_4_0", cpsB) ||
        !compile("ParticleVS", "vs_4_0", pvsB) || !compile("ParticlePS", "ps_4_0", ppsB) ||
        !compile("GpuCursorVS", "vs_4_0", gvsB) || !compile("GpuCursorPS", "ps_4_0", gpsB))
        return false;

    g_device->CreateVertexShader(tvsB->GetBufferPointer(), tvsB->GetBufferSize(), nullptr, &g_trailVS);
    g_device->CreatePixelShader(tpsB->GetBufferPointer(), tpsB->GetBufferSize(), nullptr, &g_trailPS);
    g_device->CreateVertexShader(cvsB->GetBufferPointer(), cvsB->GetBufferSize(), nullptr, &g_circleVS);
    g_device->CreatePixelShader(cpsB->GetBufferPointer(), cpsB->GetBufferSize(), nullptr, &g_circlePS);
    g_device->CreateVertexShader(pvsB->GetBufferPointer(), pvsB->GetBufferSize(), nullptr, &g_particleVS);
    g_device->CreatePixelShader(ppsB->GetBufferPointer(), ppsB->GetBufferSize(), nullptr, &g_particlePS);
    g_device->CreateVertexShader(gvsB->GetBufferPointer(), gvsB->GetBufferSize(), nullptr, &g_gpuCursorVS);
    g_device->CreatePixelShader(gpsB->GetBufferPointer(), gpsB->GetBufferSize(), nullptr, &g_gpuCursorPS);

    // Trail input layout
    D3D11_INPUT_ELEMENT_DESC trailL[] = {
        { "POSITION", 0, DXGI_FORMAT_R32G32_FLOAT,       0, 0, D3D11_INPUT_PER_VERTEX_DATA, 0 },
        { "COLOR",    0, DXGI_FORMAT_R32G32B32A32_FLOAT, 0, 8, D3D11_INPUT_PER_VERTEX_DATA, 0 },
        { "TEXCOORD", 0, DXGI_FORMAT_R32G32_FLOAT,       0, 24, D3D11_INPUT_PER_VERTEX_DATA, 0 },
    };
    g_device->CreateInputLayout(trailL, 3, tvsB->GetBufferPointer(), tvsB->GetBufferSize(), &g_trailIL);

    // Circle input layout (shared for ripples, satellites, cursor head)
    D3D11_INPUT_ELEMENT_DESC circleL[] = {
        { "POSITION", 0, DXGI_FORMAT_R32G32_FLOAT,       0, 0,  D3D11_INPUT_PER_VERTEX_DATA,   0 },
        { "I_CENTER", 0, DXGI_FORMAT_R32G32_FLOAT,       1, 0,  D3D11_INPUT_PER_INSTANCE_DATA, 1 },
        { "I_RADIUS", 0, DXGI_FORMAT_R32G32_FLOAT,       1, 8,  D3D11_INPUT_PER_INSTANCE_DATA, 1 },
        { "I_ANGLE",  0, DXGI_FORMAT_R32_FLOAT,          1, 16, D3D11_INPUT_PER_INSTANCE_DATA, 1 },
        { "I_THICK",  0, DXGI_FORMAT_R32_FLOAT,          1, 20, D3D11_INPUT_PER_INSTANCE_DATA, 1 },
        { "I_COLOR",  0, DXGI_FORMAT_R32G32B32A32_FLOAT, 1, 24, D3D11_INPUT_PER_INSTANCE_DATA, 1 },
    };
    g_device->CreateInputLayout(circleL, 6, cvsB->GetBufferPointer(), cvsB->GetBufferSize(), &g_circleIL);

    // Particle input layout
    D3D11_INPUT_ELEMENT_DESC particleL[] = {
        { "POSITION", 0, DXGI_FORMAT_R32G32_FLOAT,       0, 0,  D3D11_INPUT_PER_VERTEX_DATA,   0 },
        { "I_CENTER", 0, DXGI_FORMAT_R32G32_FLOAT,       1, 0,  D3D11_INPUT_PER_INSTANCE_DATA, 1 },
        { "I_SIZE",   0, DXGI_FORMAT_R32_FLOAT,          1, 8,  D3D11_INPUT_PER_INSTANCE_DATA, 1 },
        { "I_COLOR",  0, DXGI_FORMAT_R32G32B32A32_FLOAT, 1, 12, D3D11_INPUT_PER_INSTANCE_DATA, 1 },
    };
    g_device->CreateInputLayout(particleL, 4, pvsB->GetBufferPointer(), pvsB->GetBufferSize(), &g_particleIL);

    // GPU Cursor input layout
    D3D11_INPUT_ELEMENT_DESC cursorL[] = {
        { "POSITION", 0, DXGI_FORMAT_R32G32_FLOAT, 0, 0, D3D11_INPUT_PER_VERTEX_DATA, 0 },
        { "TEXCOORD", 0, DXGI_FORMAT_R32G32_FLOAT, 0, 8, D3D11_INPUT_PER_VERTEX_DATA, 0 },
    };
    g_device->CreateInputLayout(cursorL, 2, gvsB->GetBufferPointer(), gvsB->GetBufferSize(), &g_gpuCursorIL);

    // Blend state
    D3D11_BLEND_DESC bd = {};
    bd.RenderTarget[0].BlendEnable = TRUE;
    bd.RenderTarget[0].SrcBlend = D3D11_BLEND_ONE;
    bd.RenderTarget[0].DestBlend = D3D11_BLEND_INV_SRC_ALPHA;
    bd.RenderTarget[0].BlendOp = D3D11_BLEND_OP_ADD;
    bd.RenderTarget[0].SrcBlendAlpha = D3D11_BLEND_ONE;
    bd.RenderTarget[0].DestBlendAlpha = D3D11_BLEND_INV_SRC_ALPHA;
    bd.RenderTarget[0].BlendOpAlpha = D3D11_BLEND_OP_ADD;
    bd.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL;
    g_device->CreateBlendState(&bd, &g_blendPremul);

    // Rasterizer
    D3D11_RASTERIZER_DESC rd = {};
    rd.FillMode = D3D11_FILL_SOLID; rd.CullMode = D3D11_CULL_NONE; rd.DepthClipEnable = FALSE;
    g_device->CreateRasterizerState(&rd, &g_rasterState);

    // Constant buffer — DYNAMIC+Map/Unmap avoids the driver sync stall that
    // UpdateSubresource causes on a DEFAULT buffer when called every frame.
    D3D11_BUFFER_DESC cbd = {}; cbd.ByteWidth = sizeof(FrameCB);
    cbd.Usage = D3D11_USAGE_DYNAMIC; cbd.BindFlags = D3D11_BIND_CONSTANT_BUFFER;
    cbd.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
    g_device->CreateBuffer(&cbd, nullptr, &g_frameCB);

    // Dynamic trail VB
    D3D11_BUFFER_DESC tvbd = {}; tvbd.ByteWidth = sizeof(TrailVertex) * kMaxTotalVertices;
    tvbd.Usage = D3D11_USAGE_DYNAMIC; tvbd.BindFlags = D3D11_BIND_VERTEX_BUFFER;
    tvbd.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
    g_device->CreateBuffer(&tvbd, nullptr, &g_trailVB);

    // Static quad VB for circles and particles
    float quad[8] = { -1,-1, +1,-1, -1,+1, +1,+1 };
    D3D11_BUFFER_DESC qbd = {}; qbd.ByteWidth = sizeof(quad);
    qbd.Usage = D3D11_USAGE_IMMUTABLE; qbd.BindFlags = D3D11_BIND_VERTEX_BUFFER;
    D3D11_SUBRESOURCE_DATA qsd = {}; qsd.pSysMem = quad;
    g_device->CreateBuffer(&qbd, &qsd, &g_circleQuadVB);
    g_device->CreateBuffer(&qbd, &qsd, &g_particleQuadVB);

    // Dynamic instance VBs
    D3D11_BUFFER_DESC civbd = {}; civbd.ByteWidth = sizeof(CircleInstance) * kMaxCircleInst;
    civbd.Usage = D3D11_USAGE_DYNAMIC; civbd.BindFlags = D3D11_BIND_VERTEX_BUFFER;
    civbd.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
    g_device->CreateBuffer(&civbd, nullptr, &g_circleInstVB);

    D3D11_BUFFER_DESC pivbd = {}; pivbd.ByteWidth = sizeof(ParticleVertex) * kMaxParticles;
    pivbd.Usage = D3D11_USAGE_DYNAMIC; pivbd.BindFlags = D3D11_BIND_VERTEX_BUFFER;
    pivbd.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
    g_device->CreateBuffer(&pivbd, nullptr, &g_particleInstVB);

    // Create sampler state
    D3D11_SAMPLER_DESC sd = {};
    sd.Filter = D3D11_FILTER_MIN_MAG_MIP_LINEAR;
    sd.AddressU = D3D11_TEXTURE_ADDRESS_CLAMP;
    sd.AddressV = D3D11_TEXTURE_ADDRESS_CLAMP;
    sd.AddressW = D3D11_TEXTURE_ADDRESS_CLAMP;
    g_device->CreateSamplerState(&sd, &g_cursorSampler);

    // Dynamic GPU cursor VB
    D3D11_BUFFER_DESC cvbd = {};
    cvbd.ByteWidth = sizeof(GpuCursorVertex) * 4;
    cvbd.Usage = D3D11_USAGE_DYNAMIC;
    cvbd.BindFlags = D3D11_BIND_VERTEX_BUFFER;
    cvbd.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
    g_device->CreateBuffer(&cvbd, nullptr, &g_gpuCursorVB);

    // Pyramid cursor resources
    ComPtr<ID3DBlob> pyraVSb, pyraPSb;
    if (compile("PyramidVS", "vs_4_0", pyraVSb) && compile("PyramidPS", "ps_4_0", pyraPSb)) {
        g_device->CreateVertexShader(pyraVSb->GetBufferPointer(), pyraVSb->GetBufferSize(), nullptr, &g_pyramidVS);
        g_device->CreatePixelShader(pyraPSb->GetBufferPointer(), pyraPSb->GetBufferSize(), nullptr, &g_pyramidPS);
        D3D11_INPUT_ELEMENT_DESC pyramidL[] = {
            { "POSITION", 0, DXGI_FORMAT_R32G32_FLOAT,       0, 0,  D3D11_INPUT_PER_VERTEX_DATA, 0 },
            { "COLOR",    0, DXGI_FORMAT_R32G32B32A32_FLOAT, 0, 8,  D3D11_INPUT_PER_VERTEX_DATA, 0 },
        };
        g_device->CreateInputLayout(pyramidL, 2, pyraVSb->GetBufferPointer(), pyraVSb->GetBufferSize(), &g_pyramidIL);
    }
    {
        D3D11_BUFFER_DESC pvbd = {};
        pvbd.ByteWidth = sizeof(float) * 2 * 18 + sizeof(float) * 4 * 18; // PyramidVert * 18
        pvbd.Usage = D3D11_USAGE_DYNAMIC;
        pvbd.BindFlags = D3D11_BIND_VERTEX_BUFFER;
        pvbd.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
        g_device->CreateBuffer(&pvbd, nullptr, &g_pyramidVB);
    }
    {
        D3D11_BUFFER_DESC pdbd = {};
        pdbd.ByteWidth = sizeof(CircleInstance) * 3; // 3 dots
        pdbd.Usage = D3D11_USAGE_DYNAMIC;
        pdbd.BindFlags = D3D11_BIND_VERTEX_BUFFER;
        pdbd.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
        g_device->CreateBuffer(&pdbd, nullptr, &g_pyramidDotVB);
    }

    Wh_Log(L"[INIT] Pipeline created with circle + particle + GPU cursor + pyramid shaders.");
    return true;
}

void ReleasePipelineObjects() {
    g_pyramidDotVB.Reset(); g_pyramidVB.Reset();
    g_pyramidIL.Reset(); g_pyramidPS.Reset(); g_pyramidVS.Reset();
    g_cursorSampler.Reset(); g_gpuCursorVB.Reset();
    g_gpuCursorIL.Reset(); g_gpuCursorPS.Reset(); g_gpuCursorVS.Reset();
    g_particleInstVB.Reset(); g_particleQuadVB.Reset();
    g_circleInstVB.Reset(); g_circleQuadVB.Reset(); g_trailVB.Reset();
    g_frameCB.Reset(); g_rasterState.Reset(); g_blendPremul.Reset();
    g_particleIL.Reset(); g_particlePS.Reset(); g_particleVS.Reset();
    g_circleIL.Reset(); g_circlePS.Reset(); g_circleVS.Reset();
    g_trailIL.Reset(); g_trailPS.Reset(); g_trailVS.Reset();
}

// =============================================================================
// OVERLAY RESOURCES
// =============================================================================

bool CreateOverlayResources() {
    HRESULT hr;
    DXGI_SWAP_CHAIN_DESC1 scd = {};
    scd.Width = UINT(g_vScreenW); scd.Height = UINT(g_vScreenH);
    scd.Format = DXGI_FORMAT_B8G8R8A8_UNORM; scd.SampleDesc.Count = 1;
    scd.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT; scd.BufferCount = 2;
    scd.Scaling = DXGI_SCALING_STRETCH; scd.SwapEffect = DXGI_SWAP_EFFECT_FLIP_DISCARD;
    scd.AlphaMode = DXGI_ALPHA_MODE_PREMULTIPLIED;

    hr = g_dxgiFactory->CreateSwapChainForComposition(g_dxgiDevice.Get(), &scd, nullptr, &g_swapChain);
    if (FAILED(hr)) { Wh_Log(L"[INIT] Swapchain failed: 0x%08X", hr); return false; }

    ComPtr<ID3D11Texture2D> backBuf;
    hr = g_swapChain->GetBuffer(0, IID_PPV_ARGS(&backBuf));
    if (FAILED(hr)) { Wh_Log(L"[INIT] GetBuffer failed: 0x%08X", hr); return false; }
    hr = g_device->CreateRenderTargetView(backBuf.Get(), nullptr, &g_rtv);
    if (FAILED(hr)) { Wh_Log(L"[INIT] CreateRTV failed: 0x%08X", hr); return false; }

    hr = DCompositionCreateDevice(g_dxgiDevice.Get(), IID_PPV_ARGS(&g_compositionDevice));
    if (FAILED(hr)) { Wh_Log(L"[INIT] DComp device failed: 0x%08X", hr); return false; }
    hr = g_compositionDevice->CreateTargetForHwnd(g_overlayWnd, TRUE, &g_compositionTarget);
    if (FAILED(hr)) { Wh_Log(L"[INIT] DComp target failed: 0x%08X", hr); return false; }
    hr = g_compositionDevice->CreateVisual(&g_compositionVisual);
    if (FAILED(hr)) { Wh_Log(L"[INIT] DComp visual failed: 0x%08X", hr); return false; }
    g_compositionVisual->SetContent(g_swapChain.Get());
    g_compositionTarget->SetRoot(g_compositionVisual.Get());
    g_compositionDevice->Commit();
    return true;
}

void ReleaseOverlayResources() {
    if (g_context) g_context->OMSetRenderTargets(0, nullptr, nullptr);
    g_gpuCursor.texture.Reset();
    g_gpuCursor.srv.Reset();
    if (g_compositionVisual) {
        g_compositionVisual->SetContent(nullptr);
    }
    if (g_compositionTarget) {
        g_compositionTarget->SetRoot(nullptr);
    }
    if (g_compositionDevice) {
        g_compositionDevice->Commit();
        Sleep(10); // Allow DWM thread to receive commit signal
    }
    g_compositionVisual.Reset(); g_compositionTarget.Reset(); g_compositionDevice.Reset();
    g_rtv.Reset(); g_swapChain.Reset();
}

void HandleDeviceLost() {
    Wh_Log(L"[RECOVERY] Device lost, rebuilding...");
    ReleaseOverlayResources(); ReleasePipelineObjects(); UninitDirectX();
    if (!g_overlayWnd || g_unloading) return;
    if (InitDirectX() && CreatePipelineObjects() && CreateOverlayResources())
        Wh_Log(L"[RECOVERY] Rebuild succeeded.");
    else
        Wh_Log(L"[RECOVERY] Rebuild FAILED.");
}

void HandleDisplayChange() {
    if (!g_overlayWnd || g_unloading) return;
    RefreshVirtualScreenMetrics();
    SetWindowPos(g_overlayWnd, nullptr, g_vScreenX, g_vScreenY, g_vScreenW, g_vScreenH,
                 SWP_NOZORDER | SWP_NOACTIVATE);
    ReleaseOverlayResources();
    CreateOverlayResources();
}

// =============================================================================
// TRAIL PHYSICS (delta-time corrected)
// =============================================================================

void SetSystemCursorVisibility(bool visible) {
    if (!visible) {
        if (g_hOriginalCursor) return;
        HMODULE hModule = GetCurrentModuleHandle();
        BYTE andMask[] = { 0xFF };
        BYTE xorMask[] = { 0x00 };
        HICON hInvisibleIcon = CreateIcon(hModule, 1, 1, 1, 1, andMask, xorMask);
        if (hInvisibleIcon) {
            HICON hCursorCopy = CopyIcon(hInvisibleIcon);
            if (hCursorCopy) {
                g_hOriginalCursor = (HICON)1;
                if (!SetSystemCursor(hCursorCopy, OCR_NORMAL)) {
                    Wh_Log(L"Failed to set system cursor to invisible. Error: %lu", GetLastError());
                    // SetSystemCursor only takes ownership of the handle on success;
                    // on failure we must destroy the copy ourselves to avoid a GDI leak.
                    DestroyIcon(hCursorCopy);
                    g_hOriginalCursor = nullptr;
                }
            }
            DestroyIcon(hInvisibleIcon);
        }
    } else {
        if (g_hOriginalCursor) {
            SystemParametersInfo(SPI_SETCURSORS, 0, nullptr, 0);
            g_hOriginalCursor = nullptr;
        }
    }
}

void ScaleAndSetCursor(int scaleFactor) {
    std::lock_guard<std::mutex> lk(CursorScaling::mutex);
    CursorScaling::scaleStartTime = std::chrono::steady_clock::now();
    CursorScaling::targetScaleFactor = scaleFactor;
    if (CursorScaling::isScalingActive) return;
    CursorScaling::isScalingActive = true;
    CursorScaling::lastAppliedWidth = -1;
    CursorScaling::lastAppliedHeight = -1;

    if (CursorScaling::hCursorToScale) {
        DestroyCursor(CursorScaling::hCursorToScale);
        CursorScaling::hCursorToScale = nullptr;
    }

    CURSORINFO ci = { sizeof(CURSORINFO) };
    if (GetCursorInfo(&ci) && ci.hCursor != nullptr) {
        CursorScaling::hCursorToScale = CopyCursor(ci.hCursor);
    }
}

void UpdateAndApplyCursorScaling() {
    std::lock_guard<std::mutex> lk(CursorScaling::mutex);
    if (!CursorScaling::isScalingActive) return;
    CURSORINFO ci = { sizeof(CURSORINFO) };
    if (!GetCursorInfo(&ci)) {
        CursorScaling::isScalingActive = false;
        return;
    }
    auto now = std::chrono::steady_clock::now();
    auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - CursorScaling::scaleStartTime).count();
    float progress = (float)elapsed / (float)g_settings.ripple.clickScaleDuration;
    if (progress >= 1.0f) {
        CursorScaling::isScalingActive = false;
        CursorScaling::lastAppliedWidth = -1;
        CursorScaling::lastAppliedHeight = -1;
        SystemParametersInfo(SPI_SETCURSORS, 0, nullptr, 0);
        if (CursorScaling::hCursorToScale) {
            DestroyCursor(CursorScaling::hCursorToScale);
            CursorScaling::hCursorToScale = nullptr;
        }
        if (g_settings.cursorHead.hideSystemCursor) {
            HMODULE hModule = GetCurrentModuleHandle();
            BYTE andMask[] = { 0xFF }; BYTE xorMask[] = { 0x00 };
            HICON hInvisibleIcon = CreateIcon(hModule, 1, 1, 1, 1, andMask, xorMask);
            if (hInvisibleIcon) {
                if (!SetSystemCursor(hInvisibleIcon, OCR_NORMAL)) {
                    DestroyIcon(hInvisibleIcon);
                }
            }
        }
        return;
    }
    float pingPong = 1.0f - 2.0f * std::abs(progress - 0.5f);
    float currentScale = 1.0f + (CursorScaling::targetScaleFactor * 0.01f - 1.0f) * pingPong;
    if (!CursorScaling::hCursorToScale) {
        if (ci.hCursor != nullptr) CursorScaling::hCursorToScale = CopyCursor(ci.hCursor);
        else { CursorScaling::isScalingActive = false; return; }
    }
    int baseWidth = GetSystemMetrics(SM_CXCURSOR);
    int baseHeight = GetSystemMetrics(SM_CYCURSOR);
    int newWidth = static_cast<int>(std::round(baseWidth * currentScale));
    int newHeight = static_cast<int>(std::round(baseHeight * currentScale));
    if (newWidth == CursorScaling::lastAppliedWidth &&
        newHeight == CursorScaling::lastAppliedHeight) {
        return;
    }
    CursorScaling::lastAppliedWidth = newWidth;
    CursorScaling::lastAppliedHeight = newHeight;
    HICON hNewCursor = (HICON)CopyImage(CursorScaling::hCursorToScale, IMAGE_CURSOR, newWidth, newHeight, LR_COPYRETURNORG);
    if (hNewCursor) {
        for (UINT cursorId : CursorScaling::kCursorTypesToOverride) {
            if (cursorId == OCR_NORMAL && g_settings.cursorHead.hideSystemCursor) continue;
            HICON hCopy = CopyIcon(hNewCursor);
            if (hCopy && !SetSystemCursor(hCopy, cursorId)) {
                DestroyIcon(hCopy);
            }
        }
        DestroyIcon(hNewCursor);
    }
}
void InitTrail(POINT pt) {
    g_trail.assign(g_settings.trailLength, TrailPoint{ float(pt.x), float(pt.y), 0, 0, 0 });
    g_lastUsedMousePos = pt;
    g_frameCounter = 0;
    g_squishy = { float(pt.x), float(pt.y), float(pt.x), float(pt.y), 0, 0, 0, 0 };
}

static void ApplyTrailPhysicsSegment(TrailPoint& head, float targetX, float targetY,
                                      float headSpring, float headFric, float dt_scale) {
    head.vx += (targetX - head.x) * headSpring * dt_scale;
    head.vy += (targetY - head.y) * headSpring * dt_scale;
    head.vx *= headFric; head.vy *= headFric;
    head.x += head.vx * dt_scale; head.y += head.vy * dt_scale;
    head.speed = std::sqrt(head.vx * head.vx + head.vy * head.vy);
}

static void ApplyTrailPhysicsChain(std::vector<TrailPoint>& trail,
                                    float bodySpring, float bodyFric, float dt_scale) {
    for (size_t i = 1; i < trail.size(); ++i) {
        TrailPoint& cur = trail[i];
        const TrailPoint& prev = trail[i - 1];
        if (i > 1) {
            const TrailPoint& pp = trail[i - 2];
            cur.vx += (pp.x - cur.x) * bodySpring * 0.3f * dt_scale;
            cur.vy += (pp.y - cur.y) * bodySpring * 0.3f * dt_scale;
        }
        cur.vx += (prev.x - cur.x) * bodySpring * dt_scale;
        cur.vy += (prev.y - cur.y) * bodySpring * dt_scale;
        cur.vx *= bodyFric; cur.vy *= bodyFric;
        cur.x += cur.vx * dt_scale; cur.y += cur.vy * dt_scale;
        cur.speed = std::sqrt(cur.vx * cur.vx + cur.vy * cur.vy);
    }
}

void UpdateTrailPhysics(POINT currentMousePos) {
    if (g_trail.empty()) return;

    int cycle = g_settings.positionSkip + 1;
    if (g_frameCounter % cycle == 0) g_lastUsedMousePos = currentMousePos;
    ++g_frameCounter;

    float dt_scale = std::clamp(g_deltaTime / kReferenceFrameTime, 0.1f, 5.0f);
    float headSpring  = g_settings.headSpring / 1000.0f;
    float headFricRaw = 1.0f - (g_settings.headFriction / 100.0f);
    float bodySpring  = g_settings.bodySpring / 1000.0f;
    float bodyFricRaw = 1.0f - (g_settings.bodyFriction / 100.0f);
    float headFric = powf(headFricRaw, dt_scale);
    float bodyFric = powf(bodyFricRaw, dt_scale);

    ApplyTrailPhysicsSegment(g_trail[0],
        float(g_lastUsedMousePos.x), float(g_lastUsedMousePos.y),
        headSpring, headFric, dt_scale);
    ApplyTrailPhysicsChain(g_trail, bodySpring, bodyFric, dt_scale);

    // Track max recent speed for adaptive quality
    g_maxRecentSpeed *= 0.95f;
    for (const auto& p : g_trail) g_maxRecentSpeed = std::max(g_maxRecentSpeed, p.speed);
}

void UpdateTouchTrailPhysics(int slot) {
    auto& tt = g_touchTrails[slot];
    if (tt.trail.empty()) return;

    float dt_scale = std::clamp(g_deltaTime / kReferenceFrameTime, 0.1f, 5.0f);
    float headSpring  = g_settings.headSpring / 1000.0f;
    float headFricRaw = 1.0f - (g_settings.headFriction / 100.0f);
    float bodySpring  = g_settings.bodySpring / 1000.0f;
    float bodyFricRaw = 1.0f - (g_settings.bodyFriction / 100.0f);
    float headFric = powf(headFricRaw, dt_scale);
    float bodyFric = powf(bodyFricRaw, dt_scale);

    ApplyTrailPhysicsSegment(tt.trail[0],
        float(tt.lastPos.x), float(tt.lastPos.y),
        headSpring, headFric, dt_scale);
    ApplyTrailPhysicsChain(tt.trail, bodySpring, bodyFric, dt_scale);

    if (!tt.active) {
        tt.fadeAlpha -= dt_scale * 0.04f;
        if (tt.fadeAlpha < 0.0f) tt.fadeAlpha = 0.0f;
    }
}

// =============================================================================
// SQUISHY CURSOR HEAD
// =============================================================================

void UpdateSquishyCursor(POINT pt) {
    if (!g_settings.cursorHead.enabled) return;
    float smoothing = g_settings.cursorHead.squishSmoothing / 100.0f;
    float dt_scale = std::clamp(g_deltaTime / kReferenceFrameTime, 0.1f, 5.0f);
    float adaptiveSmoothing = 1.0f - powf(1.0f - smoothing, dt_scale);

    g_squishy.posX += (pt.x - g_squishy.posX) * adaptiveSmoothing;
    g_squishy.posY += (pt.y - g_squishy.posY) * adaptiveSmoothing;

    float dx = g_squishy.posX - g_squishy.prevX;
    float dy = g_squishy.posY - g_squishy.prevY;
    float velocity = std::sqrt(dx * dx + dy * dy);
    g_squishy.prevX = g_squishy.posX;
    g_squishy.prevY = g_squishy.posY;

    float intensity = g_settings.cursorHead.squishIntensity / 100.0f;
    float amplified = std::min(velocity * 8.0f, 200.0f);
    g_squishy.targetScale = (amplified / 15.0f) * intensity;
    g_squishy.currentScale += (g_squishy.targetScale - g_squishy.currentScale) * adaptiveSmoothing;

    if (velocity > 0.5f) g_squishy.targetAngle = std::atan2(dy, dx);
    float angleDiff = g_squishy.targetAngle - g_squishy.currentAngle;
    while (angleDiff > (float)M_PI) angleDiff -= 2.0f * (float)M_PI;
    while (angleDiff < -(float)M_PI) angleDiff += 2.0f * (float)M_PI;
    g_squishy.currentAngle += angleDiff * adaptiveSmoothing;
}

// =============================================================================
// SATELLITE ORBITALS
// =============================================================================

void UpdateSatellites() {
    if (!g_settings.satellite.enabled) return;
    float dt_scale = std::clamp(g_deltaTime / kReferenceFrameTime, 0.1f, 5.0f);
    float baseSpeed = g_settings.satellite.speed * (float)M_PI / 180.0f * dt_scale;
    float dualSpd   = -g_settings.satellite.dualSpeed * (float)M_PI / 180.0f * dt_scale;
    for (auto& s : g_satellites) {
        s.angle = fmodf(s.angle + baseSpeed, 2.0f * (float)M_PI);
        if (g_settings.satellite.enableDualRing)
            s.mirrorAngle = fmodf(s.mirrorAngle + dualSpd, 2.0f * (float)M_PI);
    }
}

void UpdateGpuCursorTexture() {
    CURSORINFO ci = { sizeof(CURSORINFO) };
    if (!GetCursorInfo(&ci)) return;
    
    bool isVisible = (ci.flags & CURSOR_SHOWING) != 0 && ci.hCursor != nullptr;
    
    if (isVisible == g_gpuCursor.visible && 
        ci.hCursor == g_gpuCursor.hLastCursor && 
        g_gpuCursor.srv != nullptr) {
        return;
    }
    
    g_gpuCursor.visible = isVisible;
    g_gpuCursor.hLastCursor = ci.hCursor;
    
    if (!isVisible) {
        return;
    }
    
    ICONINFOEXW ii = { sizeof(ICONINFOEXW) };
    if (!GetIconInfoExW(ci.hCursor, &ii)) return;
    
    HCURSOR hExtractCursor = ci.hCursor;
    bool isSystemNormal = false;
    
    if (ii.xHotspot == 29 && ii.yHotspot >= 0 && ii.yHotspot < (int)CursorScaling::kCursorTypesToOverride.size()) {
        UINT activeCursorType = CursorScaling::kCursorTypesToOverride[ii.yHotspot];
        auto it = g_originalCursors.find(activeCursorType);
        if (it != g_originalCursors.end()) {
            hExtractCursor = it->second;
            isSystemNormal = (activeCursorType == OCR_NORMAL);
        }
    } else {
        for (const auto& pair : g_sharedCursors) {
            if (ci.hCursor == pair.second) {
                isSystemNormal = (pair.first == OCR_NORMAL);
                break;
            }
        }
    }
    
    if (ii.hbmColor) DeleteObject(ii.hbmColor);
    if (ii.hbmMask) DeleteObject(ii.hbmMask);
    
    ii = { sizeof(ICONINFOEXW) };
    if (!GetIconInfoExW(hExtractCursor, &ii)) return;
    
    int w = 0, h = 0;
    BITMAP bmColor = {};
    if (ii.hbmColor) {
        GetObject(ii.hbmColor, sizeof(BITMAP), &bmColor);
        w = bmColor.bmWidth;
        h = bmColor.bmHeight;
    } else if (ii.hbmMask) {
        GetObject(ii.hbmMask, sizeof(BITMAP), &bmColor);
        w = bmColor.bmWidth;
        h = bmColor.bmHeight / 2;
    }
    
    if (w <= 0 || h <= 0) {
        if (ii.hbmColor) DeleteObject(ii.hbmColor);
        if (ii.hbmMask) DeleteObject(ii.hbmMask);
        return;
    }
    
    std::vector<uint32_t> pixels(w * h, 0x00000000);
    HDC hdc = GetDC(nullptr);
    BITMAPINFO bmi = {};
    bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
    bmi.bmiHeader.biWidth = w;
    bmi.bmiHeader.biHeight = -h;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;
    
    if (ii.hbmColor) {
        GetDIBits(hdc, ii.hbmColor, 0, h, pixels.data(), &bmi, DIB_RGB_COLORS);
    } else {
        std::vector<uint32_t> maskPixels(w * h * 2);
        bmi.bmiHeader.biHeight = -(h * 2);
        GetDIBits(hdc, ii.hbmMask, 0, h * 2, maskPixels.data(), &bmi, DIB_RGB_COLORS);
        
        for (int i = 0; i < w * h; ++i) {
            uint32_t andMask = maskPixels[i];
            uint32_t xorMask = maskPixels[i + (w * h)];
            pixels[i] = (andMask == 0xFFFFFFFF) ? 0x00000000 : (0xFF000000 | xorMask);
        }
    }
    ReleaseDC(nullptr, hdc);
    
    if (ii.hbmColor) {
        bool hasAlpha = false;
        for (int i = 0; i < w * h; ++i) {
            if (((pixels[i] >> 24) & 0xFF) > 0) {
                hasAlpha = true;
                break;
            }
        }
        if (!hasAlpha) {
            for (int i = 0; i < w * h; ++i) {
                pixels[i] |= 0xFF000000;
            }
        }
    }
    
    for (int i = 0; i < w * h; ++i) {
        uint32_t p = pixels[i];
        uint8_t a = (p >> 24) & 0xFF;
        uint8_t r = (p >> 16) & 0xFF;
        uint8_t g = (p >> 8)  & 0xFF;
        uint8_t b = p & 0xFF;
        
        float alphaNorm = a / 255.0f;
        pixels[i] = (a << 24) | 
                    (static_cast<uint8_t>(r * alphaNorm) << 16) |
                    (static_cast<uint8_t>(g * alphaNorm) << 8)  |
                    static_cast<uint8_t>(b * alphaNorm);
    }
    
    D3D11_TEXTURE2D_DESC td = {};
    td.Width = w; td.Height = h; td.MipLevels = 1; td.ArraySize = 1;
    td.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
    td.SampleDesc.Count = 1; td.Usage = D3D11_USAGE_DEFAULT;
    td.BindFlags = D3D11_BIND_SHADER_RESOURCE;
    
    D3D11_SUBRESOURCE_DATA sd = {};
    sd.pSysMem = pixels.data();
    sd.SysMemPitch = w * sizeof(uint32_t);
    
    g_gpuCursor.texture.Reset();
    g_gpuCursor.srv.Reset();
    
    HRESULT hr = g_device->CreateTexture2D(&td, &sd, &g_gpuCursor.texture);
    if (SUCCEEDED(hr)) {
        g_device->CreateShaderResourceView(g_gpuCursor.texture.Get(), nullptr, &g_gpuCursor.srv);
    }
    
    g_gpuCursor.width = w;
    g_gpuCursor.height = h;
    g_gpuCursor.hotspotX = ii.xHotspot;
    g_gpuCursor.hotspotY = ii.yHotspot;
    g_gpuCursor.shouldRotate = isSystemNormal;
    
    if (ii.hbmColor) DeleteObject(ii.hbmColor);
    if (ii.hbmMask) DeleteObject(ii.hbmMask);
}

void UpdateGpuCursorAnimation() {
    if (!g_gpuCursor.isAnimating) return;
    
    auto now = std::chrono::steady_clock::now();
    auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - g_gpuCursor.animStartTime).count();
    float progress = static_cast<float>(elapsed) / static_cast<float>(g_settings.ripple.clickScaleDuration);
    
    if (progress >= 1.0f) {
        g_gpuCursor.currentScale = 1.0f;
        g_gpuCursor.isAnimating = false;
        return;
    }
    
    float bounce = std::sin(progress * float(M_PI));
    float maxScaleFactor = g_settings.ripple.clickScaleFactor / 100.0f;
    g_gpuCursor.currentScale = 1.0f + (maxScaleFactor - 1.0f) * bounce;
}

void DrawGpuCursor() {
    if (!g_gpuCursor.visible || !g_gpuCursor.srv) return;
    
    POINT mp;
    GetCursorPos(&mp);
    float mx = float(mp.x);
    float my = float(mp.y);
    
    float w = float(g_gpuCursor.width);
    float h = float(g_gpuCursor.height);
    float hx = float(g_gpuCursor.hotspotX);
    float hy = float(g_gpuCursor.hotspotY);
    
    // Smooth angle update
    float targetAngle = g_rawMouse.angle;
    bool shouldRotate = g_settings.rotateCursorWithMovement && g_gpuCursor.shouldRotate;
    
    if (!shouldRotate) {
        targetAngle = -135.0f * (float)M_PI / 180.0f; // default top-left angle in radians
    }
    
    // Angle interpolation
    float diff = targetAngle - g_gpuCursor.currentAngle;
    while (diff > (float)M_PI) diff -= 2.0f * (float)M_PI;
    while (diff < -(float)M_PI) diff += 2.0f * (float)M_PI;
    g_gpuCursor.currentAngle += diff * std::clamp(15.0f * g_deltaTime, 0.0f, 1.0f);
    
    UpdateGpuCursorAnimation();
    
    float scale = g_gpuCursor.currentScale;
    float angleOffset = shouldRotate ? (g_gpuCursor.currentAngle + 3.0f * (float)M_PI / 4.0f) : 0.0f;
    
    float cosA = std::cos(angleOffset);
    float sinA = std::sin(angleOffset);
    
    auto transformPoint = [&](float px, float py, float& rx, float& ry) {
        float lx = px - hx;
        float ly = py - hy;
        rx = (lx * cosA - ly * sinA) * scale;
        ry = (lx * sinA + ly * cosA) * scale;
    };
    
    float originX = float(g_vScreenX);
    float originY = float(g_vScreenY);
    
    GpuCursorVertex quad[4];
    
    float rx, ry;
    transformPoint(0.0f, 0.0f, rx, ry);
    quad[0] = { mx - originX + rx, my - originY + ry, 0.0f, 0.0f };
    
    transformPoint(w, 0.0f, rx, ry);
    quad[1] = { mx - originX + rx, my - originY + ry, 1.0f, 0.0f };
    
    transformPoint(0.0f, h, rx, ry);
    quad[2] = { mx - originX + rx, my - originY + ry, 0.0f, 1.0f };
    
    transformPoint(w, h, rx, ry);
    quad[3] = { mx - originX + rx, my - originY + ry, 1.0f, 1.0f };
    
    D3D11_MAPPED_SUBRESOURCE mapped = {};
    if (SUCCEEDED(g_context->Map(g_gpuCursorVB.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped))) {
        memcpy(mapped.pData, quad, sizeof(quad));
        g_context->Unmap(g_gpuCursorVB.Get(), 0);
    }
    
    g_context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
    g_context->IASetInputLayout(g_gpuCursorIL.Get());
    
    UINT stride = sizeof(GpuCursorVertex);
    UINT offset = 0;
    g_context->IASetVertexBuffers(0, 1, g_gpuCursorVB.GetAddressOf(), &stride, &offset);
    
    g_context->VSSetShader(g_gpuCursorVS.Get(), nullptr, 0);
    g_context->PSSetShader(g_gpuCursorPS.Get(), nullptr, 0);
    
    g_context->PSSetShaderResources(0, 1, g_gpuCursor.srv.GetAddressOf());
    g_context->PSSetSamplers(0, 1, g_cursorSampler.GetAddressOf());
    
    g_context->Draw(4, 0);
    
    ID3D11ShaderResourceView* nullSRV[1] = { nullptr };
    g_context->PSSetShaderResources(0, 1, nullSRV);
}

// =============================================================================
// 3D PYRAMID CURSOR
// =============================================================================

void Update3DPyramidCursor() {
    if (!g_settings.pyramidalCursor.enabled) {
        g_pyramidState.valid = false;
        return;
    }

    POINT mp;
    GetCursorPos(&mp);
    float mx = float(mp.x);
    float my = float(mp.y);

    // Direction from cursor movement
    float dx = g_rawMouse.vx;
    float dy = g_rawMouse.vy;
    float len = std::sqrt(dx * dx + dy * dy);
    if (len > 0.5f) {
        g_pyramidState.dirX = dx / len;
        g_pyramidState.dirY = dy / len;
    }
    if (g_pyramidState.dirX == 0.0f && g_pyramidState.dirY == 0.0f) {
        g_pyramidState.dirY = -1.0f;
    }

    // Accumulate spin
    float dt = g_deltaTime;
    g_pyramidState.spinAngle += g_settings.pyramidalCursor.spinSpeed * dt * (float)(M_PI / 180.0);

    float H = g_settings.pyramidalCursor.height;
    float R = g_settings.pyramidalCursor.baseRadius;
    float concave = g_settings.pyramidalCursor.concaveDepth;

    float dnx = g_pyramidState.dirX;
    float dny = g_pyramidState.dirY;

    // Base center = apex - direction * height
    float bcx = mx - dnx * H;
    float bcy = my - dny * H;

    // Perpendicular axes in the base plane
    // u = perpendicular to direction (width axis)
    float ux = -dny;
    float uy = dnx;
    // v = same as direction but foreshortened (depth axis for 3D look)
    float foreshorten = 0.4f;
    float vx = dnx * foreshorten;
    float vy = dny * foreshorten;

    float spin = g_pyramidState.spinAngle;

    // 3 base corners at 120-degree intervals
    float cornerX[3], cornerY[3];
    for (int i = 0; i < 3; ++i) {
        float a = spin + (float)i * 2.0f * (float)M_PI / 3.0f;
        float ca = std::cos(a), sa = std::sin(a);
        cornerX[i] = bcx + ux * R * ca + vx * R * sa;
        cornerY[i] = bcy + uy * R * ca + vy * R * sa;
    }

    g_pyramidState.cornerX[0] = cornerX[0];
    g_pyramidState.cornerY[0] = cornerY[0];
    g_pyramidState.cornerX[1] = cornerX[1];
    g_pyramidState.cornerY[1] = cornerY[1];
    g_pyramidState.cornerX[2] = cornerX[2];
    g_pyramidState.cornerY[2] = cornerY[2];

    // Concave center = base center pushed toward apex
    g_pyramidState.concX = bcx + dnx * concave;
    g_pyramidState.concY = bcy + dny * concave;

    // Face brightness based on spin angle and a fixed light direction
    float lx = 0.707f, ly = 0.707f; // light from top-right
    for (int i = 0; i < 3; ++i) {
        float midA = spin + ((float)i + 0.5f) * 2.0f * (float)M_PI / 3.0f;
        float nx = std::cos(midA);
        float ny = std::sin(midA);
        // Rotate normal to screen space using u,v basis
        float sx = ux * nx + vx * ny;
        float sy = uy * nx + vy * ny;
        float nl = std::sqrt(sx * sx + sy * sy);
        if (nl > 0.001f) { sx /= nl; sy /= nl; }
        float dot = sx * lx + sy * ly;
        g_pyramidState.faceBrightness[i] = 0.3f + 0.7f * std::max(0.0f, dot);
    }
    g_pyramidState.baseBrightness = 0.35f;
    g_pyramidState.valid = true;
}

void Draw3DPyramidCursor() {
    if (!g_pyramidState.valid || !g_pyramidVS || !g_pyramidPS || !g_pyramidVB || !g_pyramidDotVB) return;

    POINT mp;
    GetCursorPos(&mp);
    float apexX = float(mp.x) - float(g_vScreenX);
    float apexY = float(mp.y) - float(g_vScreenY);

    float originX = float(g_vScreenX);
    float originY = float(g_vScreenY);
    float ax = apexX;
    float ay = apexY;

    const float* cx = g_pyramidState.cornerX;
    const float* cy = g_pyramidState.cornerY;
    float ccx = g_pyramidState.concX - originX;
    float ccy = g_pyramidState.concY - originY;

    float baseR, baseG, baseB, baseA;
    UnpackPremul(g_settings.pyramidalCursor.colorARGB, baseR, baseG, baseB, baseA);

    struct PyramidVert { float x, y, r, g, b, a; };

    // Build triangle list: 3 faces (3 verts each) + 3 base triangles (3 verts each) = 18 verts
    PyramidVert verts[18];

    float dotR, dotG, dotB, dotA;
    UnpackPremul(g_settings.pyramidalCursor.dotColorARGB, dotR, dotG, dotB, dotA);

    int vi = 0;
    for (int i = 0; i < 3; ++i) {
        int ni = (i + 1) % 3;
        float bri = g_pyramidState.faceBrightness[i];
        float r = baseR * bri;
        float g = baseG * bri;
        float b = baseB * bri;
        float a = baseA;

        // Face triangle: apex, Ci, C(i+1)
        verts[vi++] = { ax, ay, r, g, b, a };
        verts[vi++] = { cx[i] - originX, cy[i] - originY, r, g, b, a };
        verts[vi++] = { cx[ni] - originX, cy[ni] - originY, r, g, b, a };
    }

    float bb = g_pyramidState.baseBrightness;
    float br = baseR * bb;
    float bg = baseG * bb;
    float bb2 = baseB * bb;
    float ba = baseA * 0.8f;
    for (int i = 0; i < 3; ++i) {
        int ni = (i + 1) % 3;
        // Base triangle: concave center, C(i+1), Ci (reversed winding for correct orientation)
        verts[vi++] = { ccx, ccy, br, bg, bb2, ba };
        verts[vi++] = { cx[ni] - originX, cy[ni] - originY, br, bg, bb2, ba };
        verts[vi++] = { cx[i] - originX, cy[i] - originY, br, bg, bb2, ba };
    }

    // Map and draw pyramid triangles
    D3D11_MAPPED_SUBRESOURCE mapped = {};
    if (SUCCEEDED(g_context->Map(g_pyramidVB.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped))) {
        memcpy(mapped.pData, verts, sizeof(verts));
        g_context->Unmap(g_pyramidVB.Get(), 0);
    }

    g_context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
    g_context->IASetInputLayout(g_pyramidIL.Get());
    UINT stride = sizeof(PyramidVert);
    UINT offset = 0;
    g_context->IASetVertexBuffers(0, 1, g_pyramidVB.GetAddressOf(), &stride, &offset);
    g_context->VSSetShader(g_pyramidVS.Get(), nullptr, 0);
    g_context->PSSetShader(g_pyramidPS.Get(), nullptr, 0);
    g_context->Draw(18, 0);

    // Draw 3 dots at base corners using circle pipeline
    {
        CircleInstance dots[3];
        for (int i = 0; i < 3; ++i) {
            float ds = g_settings.pyramidalCursor.dotSize;
            dots[i].cx = cx[i] - originX;
            dots[i].cy = cy[i] - originY;
            dots[i].radX = ds;
            dots[i].radY = ds;
            dots[i].angle = 0.0f;
            dots[i].thickness = -1.0f; // filled
            dots[i].r = dotR;
            dots[i].g = dotG;
            dots[i].b = dotB;
            dots[i].a = dotA;
        }
        D3D11_MAPPED_SUBRESOURCE dmapped = {};
        if (SUCCEEDED(g_context->Map(g_pyramidDotVB.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &dmapped))) {
            memcpy(dmapped.pData, dots, sizeof(dots));
            g_context->Unmap(g_pyramidDotVB.Get(), 0);
        }
        g_context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
        g_context->IASetInputLayout(g_circleIL.Get());
        ID3D11Buffer* vbs[2] = { g_circleQuadVB.Get(), g_pyramidDotVB.Get() };
        UINT strides[2] = { sizeof(float)*2, sizeof(CircleInstance) };
        UINT offsets[2] = { 0, 0 };
        g_context->IASetVertexBuffers(0, 2, vbs, strides, offsets);
        g_context->VSSetShader(g_circleVS.Get(), nullptr, 0);
        g_context->PSSetShader(g_circlePS.Get(), nullptr, 0);
        g_context->DrawInstanced(4, 3, 0, 0);
    }
}

// =============================================================================
// SAMPLE GENERATION (with normal stabilization)
// =============================================================================

void BuildSamplesGeneric(const std::vector<TrailPoint>& trail, std::vector<Sample>& samples) {
    samples.clear();
    if (trail.size() < 2) return;
    if (!g_settings.enableTrail) return;
    int n = int(trail.size());

    // Adaptive quality: reduce interpolation steps at high speed
    int steps = g_settings.interpSteps;
    if (g_settings.adaptiveQuality) {
        float speedFactor = std::clamp((g_maxRecentSpeed - 15.0f) / 50.0f, 0.0f, 1.0f);
        steps = std::max(1, int(g_settings.interpSteps * (1.0f - speedFactor * 0.8f)));
    }

    samples.reserve(size_t((n - 1) * steps + 1));
    auto pt = [&](int i) -> const TrailPoint& { return trail[std::clamp(i, 0, n - 1)]; };

    for (int i = 0; i < n - 1; ++i) {
        const TrailPoint& p0 = pt(i - 1), &p1 = pt(i), &p2 = pt(i + 1), &p3 = pt(i + 2);
        for (int s = 0; s < steps; ++s) {
            float t = float(s) / float(steps);
            Sample sam;
            CatmullRom(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y, p3.x, p3.y, t, sam.x, sam.y);
            sam.speed = p1.speed * (1.0f - t) + p2.speed * t;
            sam.progress = (float(i) + t) / float(n - 1);
            sam.nx = sam.ny = 0.0f;
            samples.push_back(sam);
        }
    }
    { Sample sam; sam.x = trail.back().x; sam.y = trail.back().y;
      sam.speed = trail.back().speed; sam.progress = 1.0f;
      sam.nx = 0; sam.ny = 1; samples.push_back(sam); }

    // Normal stabilization pass
    int m = int(samples.size());
    float prevNx = 0.0f, prevNy = 1.0f;
    for (int i = 0; i < m; ++i) {
        float ax, ay, bx, by;
        if (i == 0)          { ax = samples[0].x; ay = samples[0].y;
                               bx = samples[std::min(1,m-1)].x; by = samples[std::min(1,m-1)].y; }
        else if (i == m - 1) { ax = samples[i-1].x; ay = samples[i-1].y;
                               bx = samples[i].x; by = samples[i].y; }
        else                 { ax = samples[i-1].x; ay = samples[i-1].y;
                               bx = samples[i+1].x; by = samples[i+1].y; }
        float dx = bx - ax, dy = by - ay;
        float len = std::sqrt(dx * dx + dy * dy);
        if (len < 0.1f) {
            samples[i].nx = prevNx; samples[i].ny = prevNy;
        } else {
            float nx = -dy / len, ny = dx / len;
            if (nx * prevNx + ny * prevNy < 0.0f) { nx = -nx; ny = -ny; }
            samples[i].nx = nx; samples[i].ny = ny;
            prevNx = nx; prevNy = ny;
        }
    }
}

// =============================================================================
// RIBBON VERTEX EMISSION
// =============================================================================

UINT BuildLayerVerticesGeneric(TrailVertex* dst, const LayerSettings& layer, const std::vector<Sample>& samples, float alphaScalar, UINT maxVertices) {
    if (!layer.enabled || samples.size() < 2 || maxVertices == 0) return 0;
    const float minW = g_settings.minTrailWidth;
    const float originX = float(g_vScreenX), originY = float(g_vScreenY);
    UINT writeCount = 0;
    const int kCapSteps = 16; // Number of segments for the round caps

    // Rainbow override: compute start/end ARGB from current hue
    uint32_t startC = layer.startARGB, endC = layer.endARGB;
    if (g_settings.rainbowMode) {
        startC = HSLToARGB(g_rainbowHue, 1.0f, 0.5f);
        endC   = HSLToARGB(g_rainbowHue + 180.0f, 1.0f, 0.3f);
    }

    size_t numSamples = samples.size();
    for (size_t i = 0; i < numSamples; ++i) {
        const Sample& s = samples[i];
        
        float fade = ApplyFadeCurve(s.progress, g_settings.fadeMode);
        float currentBlur = layer.startBlur + (layer.endBlur - layer.startBlur) * s.progress;
        float normSpeed = std::min(s.speed / 20.0f, 1.0f);
        float velWidth = 1.0f + normSpeed * g_settings.velWidthMul;
        float velAlpha = 1.0f + normSpeed * g_settings.velAlphaMul;
        float w = g_settings.cursorSize * layer.widthFactor * fade * velWidth;
        if (w < minW) w = minW;
        float halfW = 0.5f * w;

        float cr, cg, cb, ca;
        if (g_settings.enableGradient)
            LerpARGBPremul(startC, endC, s.progress, cr, cg, cb, ca);
        else
            UnpackPremul(startC, cr, cg, cb, ca);

        float aMul = std::clamp(layer.alphaFactor * fade * velAlpha * alphaScalar, 0.0f, 1.0f);
        cr *= aMul; cg *= aMul; cb *= aMul; ca *= aMul;
        float lx = s.x - originX, ly = s.y - originY;

        // --- HEAD CAP (Start of trail) ---
        if (i == 0) {
            float alpha = std::atan2(s.ny, s.nx);
            
            float startAngle = alpha;
            float endAngle = alpha + 2.0f * (float)M_PI;

            for (int j = 0; j <= kCapSteps; ++j) {
                if (writeCount + 2 > maxVertices) break;
                float t = float(j) / kCapSteps;
                float theta = startAngle + (endAngle - startAngle) * t;
                
                TrailVertex& center = dst[writeCount++];
                center.x = lx; center.y = ly;
                center.r = cr; center.g = cg; center.b = cb; center.a = ca;
                center.u = currentBlur; center.v = 0.0f;

                TrailVertex& arc = dst[writeCount++];
                arc.x = lx + std::cos(theta) * halfW;
                arc.y = ly + std::sin(theta) * halfW;
                arc.r = cr; arc.g = cg; arc.b = cb; arc.a = ca;
                arc.u = currentBlur; arc.v = 1.0f;
            }
            // Duplicate the last cap vertex to initiate a degenerate cut
            if (writeCount < maxVertices && writeCount > 0) {
                dst[writeCount] = dst[writeCount - 1];
                writeCount++;
            }
        }

        // --- MAIN RIBBON ---
        if (i == 0) {
            // Duplicate the first ribbon vertex to complete the degenerate bridge
            if (writeCount < maxVertices) {
                dst[writeCount++] = TrailVertex{ lx + s.nx * halfW, ly + s.ny * halfW, cr, cg, cb, ca, currentBlur, 1.0f };
            }
        }
        if (writeCount + 2 > maxVertices) break;
        
        TrailVertex& a = dst[writeCount++];
        a.x = lx + s.nx * halfW; a.y = ly + s.ny * halfW;
        a.r = cr; a.g = cg; a.b = cb; a.a = ca;
        a.u = currentBlur; a.v = 1.0f;
        
        TrailVertex& b = dst[writeCount++];
        b.x = lx - s.nx * halfW; b.y = ly - s.ny * halfW;
        b.r = cr; b.g = cg; b.b = cb; b.a = ca;
        b.u = currentBlur; b.v = -1.0f;

        // --- TAIL CAP (End of trail) ---
        if (i == numSamples - 1) {
            // Duplicate the last ribbon vertex to start the trailing degenerate cut
            if (writeCount < maxVertices && writeCount > 0) {
                dst[writeCount] = dst[writeCount - 1];
                writeCount++;
            }

            float alpha = std::atan2(s.ny, s.nx);
            
            float startAngle = alpha;
            float endAngle = alpha + 2.0f * (float)M_PI;

            // Duplicate the upcoming first cap vertex to close the degenerate bridge
            if (writeCount < maxVertices) {
                dst[writeCount++] = TrailVertex{ lx + std::cos(startAngle) * halfW, ly + std::sin(startAngle) * halfW, cr, cg, cb, ca, currentBlur, 1.0f };
            }

            for (int j = 0; j <= kCapSteps; ++j) {
                if (writeCount + 2 > maxVertices) break;
                float t = float(j) / kCapSteps;
                float theta = startAngle + (endAngle - startAngle) * t;
                
                TrailVertex& arc = dst[writeCount++];
                arc.x = lx + std::cos(theta) * halfW;
                arc.y = ly + std::sin(theta) * halfW;
                arc.r = cr; arc.g = cg; arc.b = cb; arc.a = ca;
                arc.u = currentBlur; arc.v = 1.0f;

                TrailVertex& center = dst[writeCount++];
                center.x = lx; center.y = ly;
                center.r = cr; center.g = cg; center.b = cb; center.a = ca;
                center.u = currentBlur; center.v = 0.0f;
            }
        }
    }
    return writeCount;
}

// =============================================================================
// PORTED GDI+ FEATURES: KEYSTROKE PROCESSING & TEXT CORE
// =============================================================================

std::wstring GetKeyName(DWORD vkCode) {
    switch (vkCode) {
        case VK_SHIFT: case VK_LSHIFT: case VK_RSHIFT: return L"Shift";
        case VK_CONTROL: case VK_LCONTROL: case VK_RCONTROL: return L"Ctrl";
        case VK_MENU: case VK_LMENU: case VK_RMENU: return L"Alt";
        case VK_LWIN: case VK_RWIN: return L"Win";
        case VK_TAB: return L"Tab";
        case VK_CAPITAL: return L"Caps Lock";
        case VK_ESCAPE: return L"Esc";
        case VK_SPACE: return L"Space";
        case VK_PRIOR: return L"Page Up";
        case VK_NEXT: return L"Page Down";
        case VK_END: return L"End";
        case VK_HOME: return L"Home";
        case VK_LEFT: return L"Left";
        case VK_UP: return L"Up";
        case VK_RIGHT: return L"Right";
        case VK_DOWN: return L"Down";
        case VK_INSERT: return L"Insert";
        case VK_DELETE: return L"Delete";
        default:
            UINT scanCode = MapVirtualKey(vkCode, MAPVK_VK_TO_VSC);
            LONG lParam = scanCode << 16;
            WCHAR keyName[256];
            if (GetKeyNameTextW(lParam, keyName, sizeof(keyName) / sizeof(WCHAR)) > 0) {
                return keyName;
            }
            return L"";
    }
}

void UpdateKeystrokeDisplayString() {
    std::vector<std::wstring> parts;
    static const DWORD modifiers[] = {VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN};
    for (DWORD mod_vk : modifiers) {
        if (KeystrokeDisplay::pressedKeys.count(mod_vk) ||
            (mod_vk == VK_CONTROL && (KeystrokeDisplay::pressedKeys.count(VK_LCONTROL) || KeystrokeDisplay::pressedKeys.count(VK_RCONTROL))) ||
            (mod_vk == VK_MENU && (KeystrokeDisplay::pressedKeys.count(VK_LMENU) || KeystrokeDisplay::pressedKeys.count(VK_RMENU))) ||
            (mod_vk == VK_SHIFT && (KeystrokeDisplay::pressedKeys.count(VK_LSHIFT) || KeystrokeDisplay::pressedKeys.count(VK_RSHIFT)))) {
            parts.push_back(GetKeyName(mod_vk));
        }
    }
    for (DWORD vkCode : KeystrokeDisplay::pressedKeys) {
        bool isModifier = (vkCode >= VK_LSHIFT && vkCode <= VK_RMENU) ||
                          vkCode == VK_LWIN || vkCode == VK_RWIN ||
                          vkCode == VK_SHIFT || vkCode == VK_CONTROL || vkCode == VK_MENU;
        if (!isModifier) {
            std::wstring name = GetKeyName(vkCode);
            if (!name.empty()) parts.push_back(name);
        }
    }
    KeystrokeDisplay::displayString.clear();
    for (size_t i = 0; i < parts.size(); ++i) {
        KeystrokeDisplay::displayString += parts[i];
        if (i < parts.size() - 1) KeystrokeDisplay::displayString += L" + ";
    }
}

void Get3x5Font(wchar_t c, uint8_t cols[3]) {
    c = towupper(c);
    switch (c) {
        case L'0': cols[0] = 0x1F; cols[1] = 0x11; cols[2] = 0x1F; break;
        case L'1': cols[0] = 0x00; cols[1] = 0x1F; cols[2] = 0x00; break;
        case L'2': cols[0] = 0x1D; cols[1] = 0x15; cols[2] = 0x17; break;
        case L'3': cols[0] = 0x15; cols[1] = 0x15; cols[2] = 0x1F; break;
        case L'4': cols[0] = 0x07; cols[1] = 0x04; cols[2] = 0x1F; break;
        case L'5': cols[0] = 0x17; cols[1] = 0x15; cols[2] = 0x1D; break;
        case L'6': cols[0] = 0x1F; cols[1] = 0x15; cols[2] = 0x1D; break;
        case L'7': cols[0] = 0x01; cols[1] = 0x01; cols[2] = 0x1F; break;
        case L'8': cols[0] = 0x1F; cols[1] = 0x15; cols[2] = 0x1F; break;
        case L'9': cols[0] = 0x17; cols[1] = 0x15; cols[2] = 0x1F; break;
        case L'A': cols[0] = 0x1F; cols[1] = 0x05; cols[2] = 0x1F; break;
        case L'B': cols[0] = 0x1F; cols[1] = 0x15; cols[2] = 0x0A; break;
        case L'C': cols[0] = 0x1F; cols[1] = 0x11; cols[2] = 0x11; break;
        case L'D': cols[0] = 0x1F; cols[1] = 0x11; cols[2] = 0x0E; break;
        case L'E': cols[0] = 0x1F; cols[1] = 0x15; cols[2] = 0x11; break;
        case L'F': cols[0] = 0x1F; cols[1] = 0x05; cols[2] = 0x01; break;
        case L'G': cols[0] = 0x1F; cols[1] = 0x11; cols[2] = 0x1D; break;
        case L'H': cols[0] = 0x1F; cols[1] = 0x04; cols[2] = 0x1F; break;
        case L'I': cols[0] = 0x11; cols[1] = 0x1F; cols[2] = 0x11; break;
        case L'J': cols[0] = 0x10; cols[1] = 0x10; cols[2] = 0x1F; break;
        case L'K': cols[0] = 0x1F; cols[1] = 0x04; cols[2] = 0x1B; break;
        case L'L': cols[0] = 0x1F; cols[1] = 0x10; cols[2] = 0x10; break;
        case L'M': cols[0] = 0x1F; cols[1] = 0x02; cols[2] = 0x1F; break;
        case L'N': cols[0] = 0x1F; cols[1] = 0x04; cols[2] = 0x1F; break;
        case L'O': cols[0] = 0x1F; cols[1] = 0x11; cols[2] = 0x1F; break;
        case L'P': cols[0] = 0x1F; cols[1] = 0x05; cols[2] = 0x07; break;
        case L'Q': cols[0] = 0x1F; cols[1] = 0x11; cols[2] = 0x1F; break;
        case L'R': cols[0] = 0x1F; cols[1] = 0x05; cols[2] = 0x1A; break;
        case L'S': cols[0] = 0x17; cols[1] = 0x15; cols[2] = 0x1D; break;
        case L'T': cols[0] = 0x01; cols[1] = 0x1F; cols[2] = 0x01; break;
        case L'U': cols[0] = 0x1F; cols[1] = 0x10; cols[2] = 0x1F; break;
        case L'V': cols[0] = 0x0F; cols[1] = 0x10; cols[2] = 0x0F; break;
        case L'W': cols[0] = 0x1F; cols[1] = 0x08; cols[2] = 0x1F; break;
        case L'X': cols[0] = 0x1B; cols[1] = 0x04; cols[2] = 0x1B; break;
        case L'Y': cols[0] = 0x07; cols[1] = 0x18; cols[2] = 0x07; break;
        case L'Z': cols[0] = 0x19; cols[1] = 0x15; cols[2] = 0x13; break;
        case L'!': cols[0] = 0x02; cols[1] = 0x00; cols[2] = 0x02; break;
        case L'@': cols[0] = 0x0A; cols[1] = 0x15; cols[2] = 0x0A; break;
        case L'#': cols[0] = 0x0A; cols[1] = 0x1F; cols[2] = 0x0A; break;
        case L'$': cols[0] = 0x0C; cols[1] = 0x15; cols[2] = 0x06; break;
        case L'%': cols[0] = 0x0C; cols[1] = 0x12; cols[2] = 0x06; break;
        case L'^': cols[0] = 0x04; cols[1] = 0x0A; cols[2] = 0x11; break;
        case L'&': cols[0] = 0x0E; cols[1] = 0x11; cols[2] = 0x16; break;
        case L'*': cols[0] = 0x15; cols[1] = 0x0E; cols[2] = 0x15; break;
        case L'(': cols[0] = 0x04; cols[1] = 0x11; cols[2] = 0x04; break;
        case L')': cols[0] = 0x04; cols[1] = 0x08; cols[2] = 0x04; break;
        case L'_': cols[0] = 0x00; cols[1] = 0x00; cols[2] = 0x1F; break;
        case L'+': cols[0] = 0x04; cols[1] = 0x1F; cols[2] = 0x04; break;
        case L'=': cols[0] = 0x00; cols[1] = 0x1F; cols[2] = 0x1F; break;
        case L'[': cols[0] = 0x1F; cols[1] = 0x11; cols[2] = 0x1F; break;
        case L']': cols[0] = 0x1F; cols[1] = 0x08; cols[2] = 0x1F; break;
        case L'{': cols[0] = 0x0E; cols[1] = 0x11; cols[2] = 0x0E; break;
        case L'}': cols[0] = 0x0E; cols[1] = 0x08; cols[2] = 0x0E; break;
        case L'\\':cols[0] = 0x10; cols[1] = 0x0A; cols[2] = 0x05; break;
        case L'|': cols[0] = 0x04; cols[1] = 0x04; cols[2] = 0x04; break;
        case L':': cols[0] = 0x04; cols[1] = 0x00; cols[2] = 0x04; break;
        case L';': cols[0] = 0x0C; cols[1] = 0x00; cols[2] = 0x04; break;
        case L'\'':cols[0] = 0x04; cols[1] = 0x00; cols[2] = 0x00; break;
        case L'"': cols[0] = 0x05; cols[1] = 0x00; cols[2] = 0x05; break;
        case L'<': cols[0] = 0x04; cols[1] = 0x0A; cols[2] = 0x11; break;
        case L'>': cols[0] = 0x11; cols[1] = 0x0A; cols[2] = 0x04; break;
        case L',': cols[0] = 0x0C; cols[1] = 0x04; cols[2] = 0x00; break;
        case L'/': cols[0] = 0x04; cols[1] = 0x0A; cols[2] = 0x11; break;
        case L'~': cols[0] = 0x06; cols[1] = 0x00; cols[2] = 0x18; break;
        case L'`': cols[0] = 0x10; cols[1] = 0x04; cols[2] = 0x00; break;
        case L'-': cols[0] = 0x04; cols[1] = 0x04; cols[2] = 0x04; break;
        case L'.': cols[0] = 0x00; cols[1] = 0x10; cols[2] = 0x00; break;
        case L'?': cols[0] = 0x01; cols[1] = 0x15; cols[2] = 0x03; break;
        default:   cols[0] = 0x00; cols[1] = 0x00; cols[2] = 0x00; break;
    }
}

void DrawTextD3D(CircleInstance* dst, UINT& count, const std::wstring& text, float startX, float startY, float pixelSize, float r, float g, float b, float a, bool centerHorizontal, int maxInst) {
    if (text.empty()) return;
    float charWidth = 3.0f * pixelSize;
    float charSpacing = 1.0f * pixelSize;
    float totalWidth = text.length() * charWidth + (text.length() - 1) * charSpacing;
    float x = startX;
    if (centerHorizontal) {
        x -= totalWidth * 0.5f;
    }
    for (wchar_t c : text) {
        uint8_t cols[3];
        Get3x5Font(c, cols);
        for (int col = 0; col < 3; ++col) {
            for (int row = 0; row < 5; ++row) {
                if ((cols[col] >> row) & 1) {
                    if ((int)count >= maxInst) return;
                    CircleInstance& inst = dst[count++];
                    inst.cx = x + col * pixelSize + pixelSize * 0.5f;
                    inst.cy = startY + row * pixelSize + pixelSize * 0.5f;
                    inst.radX = pixelSize * 0.5f;
                    inst.radY = pixelSize * 0.5f;
                    inst.angle = 0.0f;
                    inst.thickness = -1.0f; // Filled mode
                    inst.r = r; inst.g = g; inst.b = b; inst.a = a;
                }
            }
        }
        x += charWidth + charSpacing;
    }
}

// =============================================================================
// CIRCLE INSTANCE BUILDING (ripples + satellites + cursor head + orbit ring)
// =============================================================================

UINT BuildCircleInstances(CircleInstance* dst) {
    UINT count = 0;
    auto now = std::chrono::steady_clock::now();
    float originX = float(g_vScreenX), originY = float(g_vScreenY);

    POINT mp;
    GetCursorPos(&mp);
    float cx = float(mp.x) - originX;
    float cy = float(mp.y) - originY;

    // --- Ripples ---
    if (g_settings.ripple.enabled) {
        std::lock_guard<std::mutex> lk(g_effectsMutex);
        g_ripples.erase(std::remove_if(g_ripples.begin(), g_ripples.end(),
            [&](const Ripple& r) {
                return std::chrono::duration_cast<std::chrono::milliseconds>(now - r.startTime).count()
                       > g_settings.ripple.durationMs;
            }), g_ripples.end());

        for (const Ripple& r : g_ripples) {
            if (count >= kMaxRipples) break;
            auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(now - r.startTime).count();
            float progress = std::clamp(float(ms) / float(g_settings.ripple.durationMs), 0.0f, 1.0f);
            float fadeFactor = ApplyFadeCurve(progress, g_settings.fadeMode);
            float progressEase = 1.0f - powf(1.0f - progress, 3.0f);
            float diameter = g_settings.ripple.maxDiameter * progressEase;
            if (diameter < 0.1f) diameter = 0.1f; // Prevent division-by-zero / NaN inside circle pixel shader
            float width = g_settings.ripple.startWidth * fadeFactor;
            if (width < 0.5f) continue;
            float cr, cg, cb, ca;
            UnpackPremul(r.argb, cr, cg, cb, ca);
            cr *= fadeFactor; cg *= fadeFactor; cb *= fadeFactor; ca *= fadeFactor;
            CircleInstance& inst = dst[count++];
            inst.cx = float(r.pos.x) - originX; inst.cy = float(r.pos.y) - originY;
            inst.radX = 0.5f * diameter; inst.radY = 0.5f * diameter;
            inst.angle = 0.0f;
            inst.thickness = width;
            inst.r = cr; inst.g = cg; inst.b = cb; inst.a = ca;
        }
    }

    // --- Squishy Cursor Head ---
    if (g_settings.cursorHead.enabled && count < kMaxCircleInst) {
        float scaleX = 1.0f + g_squishy.currentScale;
        float scaleY = std::max(0.3f, 1.0f - g_squishy.currentScale * 0.5f);
        float baseRadius = g_settings.cursorHead.size * 0.5f;
        if (baseRadius < 0.1f) baseRadius = 0.1f; // Safety clamp against structural zero settings
        float cr, cg, cb, ca;
        UnpackPremul(g_settings.cursorHead.colorARGB, cr, cg, cb, ca);
        CircleInstance& inst = dst[count++];
        inst.cx = g_squishy.posX - originX;
        inst.cy = g_squishy.posY - originY;
        inst.radX = baseRadius * scaleX;
        inst.radY = baseRadius * scaleY;
        inst.angle = g_squishy.currentAngle;
        inst.thickness = g_settings.cursorHead.filled ? -1.0f : g_settings.cursorHead.outlineWidth;
        inst.r = cr; inst.g = cg; inst.b = cb; inst.a = ca;
    }

    // --- Satellites ---
    if (g_settings.satellite.enabled) {
        float orbitR = g_settings.satellite.orbitDiameter * 0.5f;
        float satR   = g_settings.satellite.satelliteSize * 0.5f;
        float cr, cg, cb, ca;
        UnpackPremul(g_settings.satellite.colorARGB, cr, cg, cb, ca);

        // Orbit ring
        if (g_settings.satellite.showOrbitRing && count < kMaxCircleInst) {
            float rr, rg, rb, ra;
            UnpackPremul(g_settings.satellite.ringColorARGB, rr, rg, rb, ra);
            CircleInstance& ring = dst[count++];
            ring.cx = cx; ring.cy = cy;
            ring.radX = orbitR; ring.radY = orbitR;
            ring.angle = 0.0f;
            ring.thickness = g_settings.satellite.ringWidth;
            ring.r = rr; ring.g = rg; ring.b = rb; ring.a = ra;
        }

        // Primary ring satellites
        for (const auto& s : g_satellites) {
            if (count >= kMaxCircleInst) break;
            CircleInstance& inst = dst[count++];
            inst.cx = cx + orbitR * std::cos(s.angle);
            inst.cy = cy + orbitR * std::sin(s.angle);
            inst.radX = satR; inst.radY = satR;
            inst.angle = 0.0f;
            inst.thickness = g_settings.satellite.filled ? -1.0f : g_settings.satellite.outlineWidth;
            inst.r = cr; inst.g = cg; inst.b = cb; inst.a = ca;
        }

        // Dual ring satellites
        if (g_settings.satellite.enableDualRing) {
            for (const auto& s : g_satellites) {
                if (count >= kMaxCircleInst) break;
                CircleInstance& inst = dst[count++];
                inst.cx = cx + orbitR * std::cos(s.mirrorAngle);
                inst.cy = cy + orbitR * std::sin(s.mirrorAngle);
                inst.radX = satR; inst.radY = satR;
                inst.angle = 0.0f;
                inst.thickness = g_settings.satellite.filled ? -1.0f : g_settings.satellite.outlineWidth;
                inst.r = cr; inst.g = cg; inst.b = cb; inst.a = ca;
            }
        }
    }

    // --- Ported On-Screen Text Rendering Passes ---
    if (g_settings.fpsCounter.enabled) {
        std::wstring fpsStr = L"FPS: " + std::to_wstring(FpsTracker::lastFps.load());
        float fx = cx;
        if (!g_settings.layout.centerToCursorX) {
            if (g_settings.fpsCounter.alignRight) {
                fx = cx + 25.0f;
            } else {
                float totalWidth = fpsStr.length() * 3.0f * 2.5f + (fpsStr.length() - 1) * 1.0f * 2.5f;
                fx = cx - 25.0f - totalWidth;
            }
        }
        float fy = g_settings.fpsCounter.alignBottom ? (cy + 35.0f) : (cy - 55.0f);
        DrawTextD3D(dst, count, fpsStr, fx, fy, 2.5f, 1.0f, 1.0f, 0.0f, 1.0f, g_settings.layout.centerToCursorX, kMaxCircleInst);
    }

    if (g_settings.keystroke.enabled) {
        std::wstring keyStr;
        { std::lock_guard<std::mutex> lock(KeystrokeDisplay::keyMutex); keyStr = KeystrokeDisplay::displayString; }
        if (!keyStr.empty()) {
            float kx = cx;
            if (!g_settings.layout.centerToCursorX) {
                if (g_settings.fpsCounter.enabled ? !g_settings.fpsCounter.alignRight : false) {
                    kx = cx + 25.0f;
                } else {
                    float totalWidth = keyStr.length() * 3.0f * 3.5f + (keyStr.length() - 1) * 1.0f * 3.5f;
                    kx = cx - 25.0f - totalWidth;
                }
            }
            float ky = g_settings.fpsCounter.enabled ? (g_settings.fpsCounter.alignBottom ? (cy - 55.0f) : (cy + 35.0f)) : (cy + 35.0f);
            DrawTextD3D(dst, count, keyStr, kx, ky, 3.5f, 1.0f, 1.0f, 1.0f, 1.0f, g_settings.layout.centerToCursorX, kMaxCircleInst);
        }
    }

    if (g_settings.mouseClick.enabled) {
        std::wstring clickStr;
        float clickAlpha = 1.0f;
        bool showClick = false;
        {
            std::lock_guard<std::mutex> lock(MouseClickDisplay::clickMutex);
            auto clickNow = std::chrono::steady_clock::now();
            auto clickElapsed = std::chrono::duration_cast<std::chrono::milliseconds>(clickNow - MouseClickDisplay::startTime).count();
            if (clickElapsed < g_settings.mouseClick.duration) {
                clickStr = MouseClickDisplay::displayString;
                showClick = true;
                clickAlpha = 1.0f - ((float)clickElapsed / (float)g_settings.mouseClick.duration);
            }
        }
        if (showClick && !clickStr.empty()) {
            float mcx = cx;
            if (!g_settings.layout.centerToCursorX) {
                if (g_settings.fpsCounter.enabled ? !g_settings.fpsCounter.alignRight : false) {
                    mcx = cx + 25.0f;
                } else {
                    float totalWidth = clickStr.length() * 3.0f * 3.0f + (clickStr.length() - 1) * 1.0f * 3.0f;
                    mcx = cx - 25.0f - totalWidth;
                }
            }
            float mcy = cy + 60.0f;
            if (g_settings.fpsCounter.enabled) {
                if (g_settings.fpsCounter.alignBottom) {
                    mcy = g_settings.keystroke.enabled ? (cy - 85.0f) : (cy - 55.0f);
                } else {
                    mcy = g_settings.keystroke.enabled ? (cy + 65.0f) : (cy + 35.0f);
                }
            } else {
                mcy = g_settings.keystroke.enabled ? (cy + 65.0f) : (cy + 35.0f);
            }
            mcy -= 20.0f * (1.0f - clickAlpha); // Drift upward as it fades
            DrawTextD3D(dst, count, clickStr, mcx, mcy, 3.0f, 1.0f, 0.4f, 0.4f, clickAlpha, g_settings.layout.centerToCursorX, kMaxCircleInst);
        }
    }

    return count;
}

// =============================================================================
// PARTICLE BURST
// =============================================================================

UINT BuildParticleInstances(ParticleVertex* dst) {
    if (!g_settings.particleBurst.enabled) return 0;
    std::lock_guard<std::mutex> lk(g_effectsMutex);
    auto now = std::chrono::steady_clock::now();
    float originX = float(g_vScreenX), originY = float(g_vScreenY);

    // Step particles and remove expired
    float dt = g_deltaTime;
    float fric = powf(1.0f - g_settings.particleBurst.friction / 100.0f, dt * 120.0f);
    float grav = g_settings.particleBurst.gravity;

    g_particles.erase(std::remove_if(g_particles.begin(), g_particles.end(),
        [&](const Particle& p) {
            return std::chrono::duration_cast<std::chrono::milliseconds>(now - p.startTime).count()
                   > g_settings.particleBurst.lifetimeMs;
        }), g_particles.end());

    UINT count = 0;
    for (auto& p : g_particles) {
        if (count >= kMaxParticles) break;
        // Step physics
        p.vy += grav * dt;
        p.vx *= fric; p.vy *= fric;
        p.x += p.vx * dt; p.y += p.vy * dt;

        auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(now - p.startTime).count();
        float progress = std::clamp(float(ms) / float(g_settings.particleBurst.lifetimeMs), 0.0f, 1.0f);
        float fade = ApplyFadeCurve(progress, g_settings.fadeMode);

        float cr, cg, cb, ca;
        UnpackPremul(p.argb, cr, cg, cb, ca);
        cr *= fade; cg *= fade; cb *= fade; ca *= fade;

        ParticleVertex& pv = dst[count++];
        pv.x = p.x - originX; pv.y = p.y - originY;
        pv.size = g_settings.particleBurst.size * fade;
        pv.r = cr; pv.g = cg; pv.b = cb; pv.a = ca;
    }
    return count;
}

// =============================================================================
// RENDER FRAME
// =============================================================================

void RenderFrame() {
    if (g_unloading || !g_swapChain || !g_rtv || !g_context || !g_device) return;

    if (g_settings.bypassSystemCursor) {
        UpdateGpuCursorTexture();
    }

    // Rainbow hue advance
    if (g_settings.rainbowMode)
        g_rainbowHue = fmodf(g_rainbowHue + (float)g_settings.rainbowSpeed, 360.0f);

    BuildSamplesGeneric(g_trail, g_samples);

    // --- Ported On-Screen FPS Calculator Step ---
    FpsTracker::frameCount++;
    auto fpsNow = std::chrono::steady_clock::now();
    auto fpsElapsed = std::chrono::duration_cast<std::chrono::milliseconds>(fpsNow - FpsTracker::lastFpsTime).count();
    if (fpsElapsed >= g_settings.fpsCounter.refreshRate) {
        FpsTracker::lastFps = static_cast<int>((FpsTracker::frameCount.load() * 1000.0) / (fpsElapsed + 1e-9));
        FpsTracker::frameCount = 0;
        FpsTracker::lastFpsTime = fpsNow;
    }

    for (int i = 0; i < 10; ++i) {
        if (g_touchTrails[i].fadeAlpha > 0.0f) {
            BuildSamplesGeneric(g_touchTrails[i].trail, g_touchTrails[i].samples);
        }
    }

    struct InstancedDraw {
        UINT start;
        UINT count;
    };
    static std::vector<InstancedDraw> activeDraws;
    activeDraws.clear();

    // Map trail VB
    {
        D3D11_MAPPED_SUBRESOURCE mapped = {};
        if (FAILED(g_context->Map(g_trailVB.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped))) return;
        TrailVertex* base = reinterpret_cast<TrailVertex*>(mapped.pData);
        UINT cursor = 0;

        if (g_settings.enableTrail) {
            for (int L = 0; L < kMaxLayers; ++L) {
                UINT n = BuildLayerVerticesGeneric(base + cursor, g_settings.layers[L], g_samples, 1.0f, kMaxTotalVertices - cursor);
                if (n >= 4) activeDraws.push_back({ cursor, n });
                cursor += n;
            }
        }

        for (int i = 0; i < 10; ++i) {
            if (g_touchTrails[i].fadeAlpha > 0.0f && !g_touchTrails[i].samples.empty()) {
                for (int L = 0; L < kMaxLayers; ++L) {
                    UINT n = BuildLayerVerticesGeneric(base + cursor, g_settings.layers[L], g_touchTrails[i].samples, g_touchTrails[i].fadeAlpha, kMaxTotalVertices - cursor);
                    if (n >= 4) activeDraws.push_back({ cursor, n });
                    cursor += n;
                }
            }
        }
        g_context->Unmap(g_trailVB.Get(), 0);
    }

    // Map circle instance VB
    UINT circleCount = 0;
    {
        D3D11_MAPPED_SUBRESOURCE mapped = {};
        if (FAILED(g_context->Map(g_circleInstVB.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped))) return;
        circleCount = BuildCircleInstances(reinterpret_cast<CircleInstance*>(mapped.pData));
        g_context->Unmap(g_circleInstVB.Get(), 0);
    }

    // Map particle instance VB
    UINT particleCount = 0;
    {
        D3D11_MAPPED_SUBRESOURCE mapped = {};
        if (FAILED(g_context->Map(g_particleInstVB.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped))) return;
        particleCount = BuildParticleInstances(reinterpret_cast<ParticleVertex*>(mapped.pData));
        g_context->Unmap(g_particleInstVB.Get(), 0);
    }

    // Constants — Map/Unmap on DYNAMIC buffer: no driver copy/sync stall vs UpdateSubresource.
    FrameCB cb = {}; cb.invHalfResX = 2.0f / float(g_vScreenW); cb.invHalfResY = 2.0f / float(g_vScreenH);
    {
        D3D11_MAPPED_SUBRESOURCE mapped = {};
        if (SUCCEEDED(g_context->Map(g_frameCB.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &mapped))) {
            memcpy(mapped.pData, &cb, sizeof(cb));
            g_context->Unmap(g_frameCB.Get(), 0);
        }
    }

    // Setup render target
    g_context->OMSetRenderTargets(1, g_rtv.GetAddressOf(), nullptr);
    const FLOAT clear[4] = { 0, 0, 0, 0 };
    g_context->ClearRenderTargetView(g_rtv.Get(), clear);
    D3D11_VIEWPORT vp = {}; vp.Width = float(g_vScreenW); vp.Height = float(g_vScreenH); vp.MaxDepth = 1.0f;
    g_context->RSSetViewports(1, &vp);
    const FLOAT bf[4] = { 1, 1, 1, 1 };
    g_context->OMSetBlendState(g_blendPremul.Get(), bf, 0xFFFFFFFF);
    g_context->RSSetState(g_rasterState.Get());
    g_context->VSSetConstantBuffers(0, 1, g_frameCB.GetAddressOf());

    // Draw trail layers
    g_context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
    g_context->IASetInputLayout(g_trailIL.Get());
    { UINT stride = sizeof(TrailVertex), offset = 0;
      g_context->IASetVertexBuffers(0, 1, g_trailVB.GetAddressOf(), &stride, &offset); }
    g_context->VSSetShader(g_trailVS.Get(), nullptr, 0);
    g_context->PSSetShader(g_trailPS.Get(), nullptr, 0);
    for (const auto& draw : activeDraws) {
        g_context->Draw(draw.count, draw.start);
    }

    // Draw circles (ripples, cursor head, satellites)
    if (circleCount > 0) {
        g_context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
        g_context->IASetInputLayout(g_circleIL.Get());
        ID3D11Buffer* vbs[2] = { g_circleQuadVB.Get(), g_circleInstVB.Get() };
        UINT strides[2] = { sizeof(float)*2, sizeof(CircleInstance) };
        UINT offsets[2] = { 0, 0 };
        g_context->IASetVertexBuffers(0, 2, vbs, strides, offsets);
        g_context->VSSetShader(g_circleVS.Get(), nullptr, 0);
        g_context->PSSetShader(g_circlePS.Get(), nullptr, 0);
        g_context->DrawInstanced(4, circleCount, 0, 0);
    }

    // Draw particles
    if (particleCount > 0) {
        g_context->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP);
        g_context->IASetInputLayout(g_particleIL.Get());
        ID3D11Buffer* vbs[2] = { g_particleQuadVB.Get(), g_particleInstVB.Get() };
        UINT strides[2] = { sizeof(float)*2, sizeof(ParticleVertex) };
        UINT offsets[2] = { 0, 0 };
        g_context->IASetVertexBuffers(0, 2, vbs, strides, offsets);
        g_context->VSSetShader(g_particleVS.Get(), nullptr, 0);
        g_context->PSSetShader(g_particlePS.Get(), nullptr, 0);
        g_context->DrawInstanced(4, particleCount, 0, 0);
    }

    if (g_settings.bypassSystemCursor) {
        DrawGpuCursor();
    }

    if (g_settings.pyramidalCursor.enabled) {
        Draw3DPyramidCursor();
    }

    // Present
    HRESULT hr = g_swapChain->Present(0, 0);
    if (hr == DXGI_ERROR_DEVICE_REMOVED || hr == DXGI_ERROR_DEVICE_RESET) HandleDeviceLost();

    // Diagnostic
    if (g_settings.diagnosticLog) {
        g_diagFrameCount++;
        auto now2 = std::chrono::steady_clock::now();
        auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now2 - g_diagLastLogTime).count();
        if (elapsed >= 10000) {
            float fps = float(g_diagFrameCount * 1000) / float(elapsed);
            Wh_Log(L"[DIAG] FPS:%.1f dt:%.2fms trail:%zu samples:%zu circles:%u particles:%u",
                   fps, g_deltaTime*1000.0f, g_trail.size(), g_samples.size(), circleCount, particleCount);
            g_diagFrameCount = 0; g_diagLastLogTime = now2;
        }
    }
}

// =============================================================================
// MOUSE HOOK
// =============================================================================

LRESULT CALLBACK LowLevelMouseProc(int nCode, WPARAM wParam, LPARAM lParam) {
    if (nCode == HC_ACTION) {
        bool isClick = (wParam == WM_LBUTTONDOWN || wParam == WM_RBUTTONDOWN || wParam == WM_MBUTTONDOWN);
        if (isClick) {
            std::lock_guard<std::mutex> lk(g_effectsMutex);
            if (g_overlayWnd) PostMessageW(g_overlayWnd, WM_NULL, 0, 0);
            if (g_settings.ripple.enableClickScaling) {
                if (g_settings.bypassSystemCursor) {
                    g_gpuCursor.animStartTime = std::chrono::steady_clock::now();
                    g_gpuCursor.isAnimating = true;
                } else {
                    ScaleAndSetCursor(g_settings.ripple.clickScaleFactor);
                }
            }
            const MSLLHOOKSTRUCT* p = reinterpret_cast<MSLLHOOKSTRUCT*>(lParam);
            uint32_t argb = (wParam == WM_LBUTTONDOWN) ? g_settings.ripple.leftARGB
                          : (wParam == WM_RBUTTONDOWN) ? g_settings.ripple.rightARGB
                          :                              g_settings.ripple.middleARGB;

            // Ripple
            if (g_settings.ripple.enabled && g_ripples.size() < size_t(kMaxRipples)) {
                Ripple r; r.pos = p->pt;
                r.startTime = std::chrono::steady_clock::now(); r.argb = argb;
                g_ripples.push_back(r);
            }

            // Particle burst
            if (g_settings.particleBurst.enabled) {
                int count = g_settings.particleBurst.count;
                float speed = g_settings.particleBurst.speed;
                auto now = std::chrono::steady_clock::now();

                static thread_local uint32_t rngState = []() {
                    uint32_t seed = uint32_t(std::chrono::steady_clock::now().time_since_epoch().count());
                    return seed ? seed : 0xACE1u;
                }();
                auto nextRandomFloat = [&]() {
                    rngState ^= rngState << 13;
                    rngState ^= rngState >> 17;
                    rngState ^= rngState << 5;
                    return float(rngState & 0xFFFF) / 65535.0f;
                };

                for (int i = 0; i < count && g_particles.size() < size_t(kMaxParticles); ++i) {
                    float angle = 2.0f * (float)M_PI * float(i) / float(count);
                    float jitterAngle = angle + (nextRandomFloat() - 0.5f) * 0.5f;
                    float jitterSpeed = speed * (0.7f + nextRandomFloat() * 0.6f);
                    Particle part;
                    part.x = float(p->pt.x); part.y = float(p->pt.y);
                    part.vx = std::cos(jitterAngle) * jitterSpeed;
                    part.vy = std::sin(jitterAngle) * jitterSpeed;
                    part.startTime = now;
                    part.argb = argb;
                    g_particles.push_back(part);
                }
            }

            {
                std::lock_guard<std::mutex> lock(MouseClickDisplay::clickMutex);
                MouseClickDisplay::displayString = (wParam == WM_LBUTTONDOWN) ? L"Left Click"
                                                 : (wParam == WM_RBUTTONDOWN) ? L"Right Click"
                                                 :                              L"Middle Click";
                MouseClickDisplay::startTime = std::chrono::steady_clock::now();
                MouseClickDisplay::clickChanged.store(true);
            }
        }
    }
    return CallNextHookEx(g_mouseHook, nCode, wParam, lParam);
}

void MouseHookThreadFunc() {
    g_mouseHookThreadId.store(GetCurrentThreadId());
    HMODULE hModule = GetCurrentModuleHandle();
    g_mouseHook = SetWindowsHookExW(WH_MOUSE_LL, LowLevelMouseProc, hModule, 0);
    if (!g_mouseHook) { Wh_Log(L"[HOOK] Failed: %lu", GetLastError()); return; }
    MSG msg;
    while (GetMessage(&msg, nullptr, 0, 0) > 0) { TranslateMessage(&msg); DispatchMessage(&msg); }
    UnhookWindowsHookEx(g_mouseHook); g_mouseHook = nullptr;
}

LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam) {
    if (nCode == HC_ACTION && g_keystrokeEnabled.load()) {
        KBDLLHOOKSTRUCT* pkbhs = reinterpret_cast<KBDLLHOOKSTRUCT*>(lParam);
        const DWORD vkCode = pkbhs->vkCode;
        bool stateChanged = false;
        if (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN) {
            if ((pkbhs->flags & LLKHF_UP) == 0) {
                std::lock_guard<std::mutex> lock(KeystrokeDisplay::keyMutex);
                auto [_, inserted] = KeystrokeDisplay::pressedKeys.insert(vkCode);
                if (inserted) stateChanged = true;
            }
        } else if (wParam == WM_KEYUP || wParam == WM_SYSKEYUP) {
            std::lock_guard<std::mutex> lock(KeystrokeDisplay::keyMutex);
            if (KeystrokeDisplay::pressedKeys.erase(vkCode) > 0) {
                stateChanged = true;
            }
        }
        if (stateChanged) {
            UpdateKeystrokeDisplayString();
            Wh_Log(L"[KEY] Keystroke state updated: %s", KeystrokeDisplay::displayString.c_str());
            KeystrokeDisplay::keystrokeChanged.store(true);
            if (g_overlayWnd) PostMessageW(g_overlayWnd, WM_NULL, 0, 0);
        }
    }
    return CallNextHookEx(g_keyboardHook, nCode, wParam, lParam);
}

void KeyboardHookThreadFunc() {
    g_keyboardHookThreadId.store(GetCurrentThreadId());
    HMODULE hModule = GetCurrentModuleHandle();
    g_keyboardHook = SetWindowsHookExW(WH_KEYBOARD_LL, LowLevelKeyboardProc, hModule, 0);
    if (!g_keyboardHook) {
        Wh_Log(L"[HOOK] Failed to install low-level keyboard hook. Error: %lu", GetLastError());
        return;
    }
    Wh_Log(L"[HOOK] Low-level keyboard hook installed successfully.");
    MSG msg;
    while (GetMessage(&msg, nullptr, 0, 0) > 0) { TranslateMessage(&msg); DispatchMessage(&msg); }
    UnhookWindowsHookEx(g_keyboardHook); g_keyboardHook = nullptr;
    Wh_Log(L"[HOOK] Keyboard hook thread exiting.");
}

// =============================================================================
// WINDOW PROC
// =============================================================================

LRESULT CALLBACK OverlayWndProc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam) {
    switch (uMsg) {
        case WM_CLOSE:
            DestroyWindow(hWnd);
            return 0;
        case WM_DESTROY:
            g_running = false;
            return 0;
        case WM_NCHITTEST: return HTTRANSPARENT;
        case WM_SETCURSOR: return TRUE;
        case WM_DISPLAYCHANGE:
            if (!g_unloading) SetTimer(hWnd, TIMER_ID_DISPLAY_CHANGE, 200, nullptr);
            return 0;
        case WM_INPUT: {
            UINT dwSize = 0;
            GetRawInputData(reinterpret_cast<HRAWINPUT>(lParam), RID_INPUT, nullptr, &dwSize, sizeof(RAWINPUTHEADER));
            if (dwSize > 0) {
                BYTE stackBuf[512];
                BYTE* pBuffer = (dwSize <= sizeof(stackBuf)) ? stackBuf : static_cast<BYTE*>(_malloca(dwSize));
                if (pBuffer && GetRawInputData(reinterpret_cast<HRAWINPUT>(lParam), RID_INPUT, pBuffer, &dwSize, sizeof(RAWINPUTHEADER)) == dwSize) {
                    RAWINPUT* raw = reinterpret_cast<RAWINPUT*>(pBuffer);
                    if (raw->header.dwType == RIM_TYPEHID) {
                        ProcessRawTouch(raw);
                    }
                }
                if (pBuffer && pBuffer != stackBuf) _freea(pBuffer);
            }
            break;
        }
        case WM_TIMER:
            if (wParam == TIMER_ID_DISPLAY_CHANGE) { KillTimer(hWnd, TIMER_ID_DISPLAY_CHANGE); HandleDisplayChange(); }
            return 0;
    }
    return DefWindowProc(hWnd, uMsg, wParam, lParam);
}

// =============================================================================
// RAW HID TOUCH PROCESSING MODULE
// =============================================================================

void ProcessRawTouch(RAWINPUT* raw) {
    UINT size = 0;
    if (GetRawInputDeviceInfoW(raw->header.hDevice, RIDI_PREPARSEDDATA, nullptr, &size) != 0 || size == 0) return;

    BYTE stackPreparsed[1024];
    BYTE* pPreparsedBuf = (size <= sizeof(stackPreparsed)) ? stackPreparsed : static_cast<BYTE*>(_malloca(size));
    if (!pPreparsedBuf) return;

    PHIDP_PREPARSED_DATA pData = reinterpret_cast<PHIDP_PREPARSED_DATA>(pPreparsedBuf);
    if (GetRawInputDeviceInfoW(raw->header.hDevice, RIDI_PREPARSEDDATA, pData, &size) == static_cast<UINT>(-1)) {
        if (pPreparsedBuf != stackPreparsed) _freea(pPreparsedBuf);
        return;
    }

    HIDP_CAPS caps;
    if (HidP_GetCaps(pData, &caps) != HIDP_STATUS_SUCCESS || caps.NumberInputValueCaps == 0) {
        if (pPreparsedBuf != stackPreparsed) _freea(pPreparsedBuf);
        return;
    }

    USHORT vcCount = caps.NumberInputValueCaps;
    HIDP_VALUE_CAPS stackVCaps[32];
    HIDP_VALUE_CAPS* pVCaps = (vcCount <= 32) ? stackVCaps : static_cast<HIDP_VALUE_CAPS*>(_malloca(vcCount * sizeof(HIDP_VALUE_CAPS)));
    if (!pVCaps) {
        if (pPreparsedBuf != stackPreparsed) _freea(pPreparsedBuf);
        return;
    }

    if (HidP_GetValueCaps(HidP_Input, pVCaps, &vcCount, pData) != HIDP_STATUS_SUCCESS) {
        if (pVCaps != stackVCaps) _freea(pVCaps);
        if (pPreparsedBuf != stackPreparsed) _freea(pPreparsedBuf);
        return;
    }

    struct TouchContact {
        ULONG x = 0, y = 0, id = 0;
        bool hasX = false, hasY = false, hasID = false;
    };
    TouchContact contacts[32] = {};
    ULONG maxCapsX = 32767, maxCapsY = 32767;

    for (USHORT i = 0; i < vcCount; ++i) {
        USHORT lc = pVCaps[i].LinkCollection;
        if (lc >= 32) continue;

        ULONG val = 0;
        USAGE usage = pVCaps[i].IsRange ? pVCaps[i].Range.UsageMin : pVCaps[i].NotRange.Usage;

        if (HidP_GetUsageValue(HidP_Input, pVCaps[i].UsagePage, lc, usage, &val, pData, reinterpret_cast<char*>(raw->data.hid.bRawData), raw->data.hid.dwSizeHid) == HIDP_STATUS_SUCCESS) {
            if (pVCaps[i].UsagePage == 0x01 && usage == 0x30) { contacts[lc].x = val; contacts[lc].hasX = true; maxCapsX = pVCaps[i].LogicalMax; }
            if (pVCaps[i].UsagePage == 0x01 && usage == 0x31) { contacts[lc].y = val; contacts[lc].hasY = true; maxCapsY = pVCaps[i].LogicalMax; }
            if (pVCaps[i].UsagePage == 0x0D && usage == 0x51) { contacts[lc].id = val; contacts[lc].hasID = true; }
        }
    }

    bool fingerDown[32] = {};
    ULONG maxUsageCount = HidP_MaxUsageListLength(HidP_Input, 0x0D, pData);
    if (maxUsageCount == 0) maxUsageCount = 32;
    USAGE stackUsages[64];
    USAGE* pUsages = (maxUsageCount <= 64) ? stackUsages : static_cast<USAGE*>(_malloca(maxUsageCount * sizeof(USAGE)));

    if (pUsages) {
        for (USHORT lc = 0; lc < 32; ++lc) {
            ULONG usageCount = maxUsageCount;
            if (HidP_GetUsages(HidP_Input, 0x0D, lc, pUsages, &usageCount, pData, reinterpret_cast<char*>(raw->data.hid.bRawData), raw->data.hid.dwSizeHid) == HIDP_STATUS_SUCCESS) {
                for (ULONG u = 0; u < usageCount; ++u) {
                    if (pUsages[u] == 0x42) { // Tip Switch usage ID
                        fingerDown[lc] = true;
                        break;
                    }
                }
            }
        }
        if (pUsages != stackUsages) _freea(pUsages);
    }

    for (int lc = 0; lc < 32; ++lc) {
        auto& c = contacts[lc];
        if (!c.hasX || !c.hasY || !c.hasID || c.id == 0) continue;

        float sx = g_vScreenX + (float(c.x) / float(maxCapsX > 0 ? maxCapsX : 1)) * g_vScreenW;
        float sy = g_vScreenY + (float(c.y) / float(maxCapsY > 0 ? maxCapsY : 1)) * g_vScreenH;
        POINT pt = { (int)sx, (int)sy };
        DWORD pid = c.id;
        bool isDown = fingerDown[lc];

        int slot = -1;
        for (int i = 0; i < 10; ++i) {
            if (g_touchTrails[i].active && g_touchTrails[i].pointerId == pid) { slot = i; break; }
        }
        if (slot == -1 && isDown) {
            for (int i = 0; i < 10; ++i) {
                if (!g_touchTrails[i].active && g_touchTrails[i].fadeAlpha <= 0.0f) { slot = i; break; }
            }
        }

        if (slot != -1) {
            if (isDown) {
                if (!g_touchTrails[slot].active) {
                    if (g_settings.diagnosticLog) Wh_Log(L"[TOUCH] Activated slots index %d for pointer ID %u", slot, pid);
                    g_touchTrails[slot].pointerId = pid;
                    g_touchTrails[slot].active = true;
                    g_touchTrails[slot].fadeAlpha = 1.0f;
                    g_touchTrails[slot].trail.assign(g_settings.trailLength, TrailPoint{ sx, sy, 0, 0, 0 });
                }
                g_touchTrails[slot].lastPos = pt;
            } else if (g_touchTrails[slot].active) {
                if (g_settings.diagnosticLog) Wh_Log(L"[TOUCH] Lifted slot index %d (ID %u)", slot, pid);
                g_touchTrails[slot].active = false;
            }
        }
    }

    if (pVCaps != stackVCaps) _freea(pVCaps);
    if (pPreparsedBuf != stackPreparsed) _freea(pPreparsedBuf);
}

// =============================================================================
// RENDER THREAD
// =============================================================================

void RenderThreadFunc() {
    HMODULE hModule = GetCurrentModuleHandle();
    WNDCLASSW wc = {};
    wc.lpfnWndProc = OverlayWndProc; wc.hInstance = hModule;
    wc.lpszClassName = OVERLAY_WINDOW_CLASS; wc.hCursor = LoadCursor(nullptr, IDC_ARROW);
    if (!RegisterClassW(&wc)) return;

    RefreshVirtualScreenMetrics();
    g_overlayWnd = CreateWindowEx(
        WS_EX_LAYERED | WS_EX_NOREDIRECTIONBITMAP | WS_EX_TRANSPARENT
            | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
        OVERLAY_WINDOW_CLASS, nullptr, WS_POPUP | WS_VISIBLE,
        g_vScreenX, g_vScreenY, g_vScreenW, g_vScreenH,
        nullptr, nullptr, hModule, nullptr);
    if (!g_overlayWnd) { UnregisterClassW(OVERLAY_WINDOW_CLASS, hModule); return; }
    SetLayeredWindowAttributes(g_overlayWnd, 0, 255, LWA_ALPHA);

    if (!InitDirectX() || !CreatePipelineObjects() || !CreateOverlayResources()) {
        DestroyWindow(g_overlayWnd); g_overlayWnd = nullptr;
        ReleaseOverlayResources(); ReleasePipelineObjects(); UninitDirectX();
        UnregisterClassW(OVERLAY_WINDOW_CLASS, hModule); return;
    }

    RAWINPUTDEVICE rid;
    rid.usUsagePage = 0x0D; // Digitizers
    rid.usUsage     = 0x04; // Touch Screen
    rid.dwFlags     = RIDEV_INPUTSINK;
    rid.hwndTarget  = g_overlayWnd;
    if (!RegisterRawInputDevices(&rid, 1, sizeof(rid))) {
        Wh_Log(L"[TOUCH] RegisterRawInputDevices failed. Error: %lu", GetLastError());
    } else {
        Wh_Log(L"[TOUCH] Registered global Raw Input for Digitizer Touch Screens.");
    }

    POINT initPt = {0, 0}; GetCursorPos(&initPt); InitTrail(initPt);
    g_rawMouse.x = float(initPt.x);
    g_rawMouse.y = float(initPt.y);

    DWORD mmcssTask = 0;
    HANDLE hMMCSS = AvSetMmThreadCharacteristicsW(L"DisplayPostProcessing", &mmcssTask);

    g_lastFrameTime = std::chrono::steady_clock::now();
    g_diagLastLogTime = g_lastFrameTime;
    g_running = true;
    int clearFramesRemaining = 2;

    while (g_running && !g_unloading) {
        if (g_settingsChanged.exchange(false)) {
            bool wasHidingCursor = g_settings.cursorHead.hideSystemCursor;
            LoadSettings();
            bool isHidingCursor = g_settings.cursorHead.hideSystemCursor;
            if (wasHidingCursor != isHidingCursor) {
                SetSystemCursorVisibility(!isHidingCursor);
            }
        }

        MSG msg;
        while (PeekMessage(&msg, nullptr, 0, 0, PM_REMOVE)) {
            if (msg.message == WM_QUIT) { g_running = false; break; }
            TranslateMessage(&msg); DispatchMessage(&msg);
        }
        if (!g_running || !g_overlayWnd) break;

        auto now = std::chrono::steady_clock::now();
        g_deltaTime = std::clamp(std::chrono::duration<float>(now - g_lastFrameTime).count(), 0.001f, 0.1f);
        g_lastFrameTime = now;

        // Pre-seed pt with last known position so that if GetCursorPos fails
        // (e.g., during session lock or fast-user-switch transitions), code
        // further down that reads pt.x / pt.y has well-defined values.
        POINT pt = { g_lastUsedMousePos.x, g_lastUsedMousePos.y };
        bool mouseMoved = false;
        if (GetCursorPos(&pt)) {
            if (pt.x != g_lastUsedMousePos.x || pt.y != g_lastUsedMousePos.y) {
                mouseMoved = true;
                
                float raw_vx = float(pt.x) - g_rawMouse.x;
                float raw_vy = float(pt.y) - g_rawMouse.y;
                
                g_rawMouse.x = float(pt.x);
                g_rawMouse.y = float(pt.y);
                
                float smoothing = (float)g_settings.cursorRotationSmoothing;
                float alpha = 1.0f - (1.0f / std::max(smoothing, 1.0f));
                
                g_rawMouse.vx = g_rawMouse.vx * alpha + raw_vx * (1.0f - alpha);
                g_rawMouse.vy = g_rawMouse.vy * alpha + raw_vy * (1.0f - alpha);
                
                float dist = std::sqrt(g_rawMouse.vx * g_rawMouse.vx + g_rawMouse.vy * g_rawMouse.vy);
                if (dist > 0.1f) {
                    g_rawMouse.angle = std::atan2(g_rawMouse.vy, g_rawMouse.vx);
                    g_rawMouse.isMoving = true;
                }
            } else {
                g_rawMouse.isMoving = false;
                float smoothing = (float)g_settings.cursorRotationSmoothing;
                float alpha = 1.0f - (1.0f / std::max(smoothing, 1.0f));
                g_rawMouse.vx *= alpha;
                g_rawMouse.vy *= alpha;
            }
            if (g_settings.enableTrail) {
                UpdateTrailPhysics(pt);
            } else {
                g_lastUsedMousePos = pt;
            }
            UpdateSquishyCursor(pt);
        }
        UpdateSatellites();
        Update3DPyramidCursor();

        if (g_settings.ripple.enableClickScaling && !g_settings.bypassSystemCursor) {
            std::lock_guard<std::mutex> lk(g_effectsMutex);
            UpdateAndApplyCursorScaling();
        }

        bool cursorRotating = false;
        if (g_settings.bypassSystemCursor) {
            float targetAngle = g_rawMouse.angle;
            bool shouldRotate = g_settings.rotateCursorWithMovement && g_gpuCursor.shouldRotate;
            if (!shouldRotate) {
                targetAngle = -135.0f * (float)M_PI / 180.0f;
            }
            if (std::abs(g_gpuCursor.currentAngle - targetAngle) > 0.001f) {
                cursorRotating = true;
            }
        }

        bool effectsActive = g_settings.satellite.enabled || g_settings.rainbowMode || 
                             g_settings.pyramidalCursor.enabled ||
                             (!g_settings.bypassSystemCursor && CursorScaling::isScalingActive) ||
                             (g_settings.bypassSystemCursor && (g_gpuCursor.isAnimating || cursorRotating));
        if (!effectsActive) {
            std::lock_guard<std::mutex> lk(g_effectsMutex);
            if (!g_ripples.empty() || !g_particles.empty()) {
                effectsActive = true;
            }
        }
        bool trailMoving = false;
        if (g_settings.enableTrail && !effectsActive && !g_trail.empty()) {
            for (const auto& p : g_trail) {
                if (std::abs(p.vx) > 0.05f || std::abs(p.vy) > 0.05f) {
                    trailMoving = true;
                    break;
                }
            }
            if (!trailMoving) {
                if (std::abs(g_trail[0].x - float(pt.x)) > 0.1f || std::abs(g_trail[0].y - float(pt.y)) > 0.1f) {
                    trailMoving = true;
                }
            }
        }

        bool touchActive = false;
        for (int i = 0; i < 10; ++i) {
            if (g_touchTrails[i].fadeAlpha > 0.0f) {
                touchActive = true;
                UpdateTouchTrailPhysics(i);
            }
        }

        bool keysActive = false;
        if (g_settings.keystroke.enabled) {
            std::lock_guard<std::mutex> lock(KeystrokeDisplay::keyMutex);
            if (!KeystrokeDisplay::pressedKeys.empty()) keysActive = true;
        }

        bool clickActive = false;
        if (g_settings.mouseClick.enabled) {
            std::lock_guard<std::mutex> lock(MouseClickDisplay::clickMutex);
            auto clickNow = std::chrono::steady_clock::now();
            auto clickElapsed = std::chrono::duration_cast<std::chrono::milliseconds>(clickNow - MouseClickDisplay::startTime).count();
            if (clickElapsed < g_settings.mouseClick.duration) clickActive = true;
        }

        if (KeystrokeDisplay::keystrokeChanged.exchange(false) || MouseClickDisplay::clickChanged.exchange(false)) {
            clearFramesRemaining = 2;
        }

        if (mouseMoved || effectsActive || trailMoving || touchActive || keysActive || clickActive) {
            clearFramesRemaining = 2; 
        }

        if (clearFramesRemaining > 0) {
            RenderFrame();
            clearFramesRemaining--;
            if (FAILED(DwmFlush())) Sleep(16);
        }

        DWORD waitMs = (clearFramesRemaining > 0) ? 0 : 16;
        MsgWaitForMultipleObjectsEx(0, nullptr, waitMs, QS_ALLINPUT, MWMO_INPUTAVAILABLE);
    }

    if (hMMCSS) AvRevertMmThreadCharacteristics(hMMCSS);
    if (g_overlayWnd) { DestroyWindow(g_overlayWnd); g_overlayWnd = nullptr; }
    ReleaseOverlayResources(); ReleasePipelineObjects(); UninitDirectX();
    UnregisterClassW(OVERLAY_WINDOW_CLASS, hModule);
}

// =============================================================================
// MOD LIFECYCLE
// =============================================================================

BOOL Wh_ModInit() {
    Wh_Log(L"=== D3D Cursor Overlay v0.1.3 Ultimate INIT ===");
    g_unloading = false; g_running = false; g_frameCounter = 0;
    g_originalCursors.clear();
    g_sharedCursors.clear();
    g_trail.clear(); g_samples.clear(); g_rainbowHue = 0.0f;
    FpsTracker::lastFpsTime = std::chrono::steady_clock::now();
    { std::lock_guard<std::mutex> lk(g_effectsMutex); g_ripples.clear(); g_particles.clear(); }
    for (int i = 0; i < 10; ++i) {
        g_touchTrails[i] = TouchTrailState{};
    }
    { std::lock_guard<std::mutex> lock(MouseClickDisplay::clickMutex); MouseClickDisplay::displayString.clear(); }
    LoadSettings();
    if (g_settings.cursorHead.hideSystemCursor) {
        SetSystemCursorVisibility(false);
    }
    try {
        g_renderThread = std::thread(RenderThreadFunc);
    } catch (const std::system_error&) {
        Wh_Log(L"Failed to start render thread.");
        return FALSE;
    }
    try {
        g_mouseHookThread = std::thread(MouseHookThreadFunc);
    } catch (const std::system_error&) {
        Wh_Log(L"Failed to start mouse hook thread.");
    }
    try {
        g_keyboardHookThread = std::thread(KeyboardHookThreadFunc);
    } catch (const std::system_error&) {
        Wh_Log(L"Failed to start keyboard hook thread.");
    }
    return TRUE;
}

void Wh_ModUninit() {
    Wh_Log(L"=== D3D Cursor Overlay UNINIT ===");
    g_unloading = true;
    if (g_overlayWnd) PostMessageW(g_overlayWnd, WM_CLOSE, 0, 0);
    g_running = false;
    if (g_renderThread.joinable()) {
        DWORD tid = GetThreadId(g_renderThread.native_handle());
        if (tid) PostThreadMessage(tid, WM_QUIT, 0, 0);
        if (g_renderThread.joinable()) g_renderThread.join();
    }
    if (g_mouseHookThread.joinable()) {
        DWORD tid = g_mouseHookThreadId.load();
        if (tid) PostThreadMessage(tid, WM_QUIT, 0, 0);
        if (g_mouseHookThread.joinable()) g_mouseHookThread.join();
    }
    if (g_keyboardHookThread.joinable()) {
        DWORD tid = g_keyboardHookThreadId.load();
        if (tid) PostThreadMessage(tid, WM_QUIT, 0, 0);
        if (g_keyboardHookThread.joinable()) g_keyboardHookThread.join();
    }
    { std::lock_guard<std::mutex> lk(g_effectsMutex); g_ripples.clear(); g_particles.clear(); }
    { std::lock_guard<std::mutex> lock(KeystrokeDisplay::keyMutex); KeystrokeDisplay::pressedKeys.clear(); KeystrokeDisplay::displayString.clear(); }
    { std::lock_guard<std::mutex> lock(MouseClickDisplay::clickMutex); MouseClickDisplay::displayString.clear(); }

    {
        std::lock_guard<std::mutex> lk(CursorScaling::mutex);
        if (CursorScaling::isScalingActive) {
            CursorScaling::isScalingActive = false;
            CursorScaling::lastAppliedWidth = -1;
            CursorScaling::lastAppliedHeight = -1;
            SystemParametersInfo(SPI_SETCURSORS, 0, nullptr, 0);
        }

        if (CursorScaling::hCursorToScale) {
            DestroyCursor(CursorScaling::hCursorToScale);
            CursorScaling::hCursorToScale = nullptr;
        }
    }
    if (!g_originalCursors.empty()) {
        RestoreSystemCursors();
    }
    for (auto& pair : g_originalCursors) {
        if (pair.second) DestroyCursor(pair.second);
    }
    g_originalCursors.clear();
    g_sharedCursors.clear();

    if (g_hOriginalCursor) {
        SetSystemCursorVisibility(true);
    }

    g_trail.clear(); g_samples.clear(); g_satellites.clear();
    for (int i = 0; i < 10; ++i) {
        g_touchTrails[i].trail.clear();
        g_touchTrails[i].samples.clear();
    }
}

void Wh_ModSettingsChanged() {
    Wh_Log(L"[SETTINGS] Reloading signaled...");
    g_settingsChanged.store(true);
    if (g_overlayWnd) PostMessageW(g_overlayWnd, WM_NULL, 0, 0);
}