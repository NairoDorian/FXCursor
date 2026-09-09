// ==WindhawkMod==
// @id              custom-cursor-effects
// @name            Custom Cursor Effects
// @description     Adds a highly customizable, physics-based trail, a 'squishy' cursor head, animated click ripples, and more to the mouse cursor, rendered with GDI+.
// @version         0.1 Alpha
// @author          Nairod
// @github          https://github.com/
// @include         dwm.exe
// @compilerOptions -lgdiplus -lgdi32 -lole32 -luser32
// ==/WindhawkMod==

// ==WindhawkModReadme==
/*
# Custom Cursor Effects: Magic Trail, Squishy Cursor & Click Ripples
 
Enhances your Windows experience with a highly configurable set of effects for the Mouse & Touch Cursor, rendered with GDI+ for high-quality, anti-aliased visuals.
It currently features several major effects:
- **Physics-Driven Trail:** A beautiful, smooth trail that follows your cursor with realistic physics. Customize its length, stiffness, color gradients, and up to 4 distinct visual layers.
- **Squishy Cursor Head:** A fun, deformable ellipse that squishes and stretches based on your cursor's speed and direction, adding a touch of personality to your pointer.
- **Click Ripples:** An animated ripple effect that emanates from the cursor on every mouse click, providing satisfying visual feedback. Left and right clicks can have different colors.
- **Satellite Orbitals:** Add one or two rings of orbiting "satellites" around your cursor for a cosmic feel. Customize their speed, size, and appearance.
- **Keystroke & FPS Display:** On-screen overlays to display currently pressed keys and monitor rendering performance.

All effects are rendered with GDI+ for high-quality, anti-aliased visuals and are fully configurable through the settings menu. The mod is highly optimized, using dirty-rectangle updates and adaptive quality to minimize CPU usage.

### ⚠ Important Usage Note ⚠
For the effects to render correctly, you **must** add `dwm.exe` (the Desktop Window Manager) to Windhawk's process inclusion list in the advanced settings. This allows the mod to draw the cursor overlay directly on the desktop.
*/
// ==/WindhawkModReadme==

// ==WindhawkModSettings==
/*
- headSpring: 100
  $name: Head Spring Strength
  $description: "Defines the spring force pulling the trail's head towards the cursor. Higher values create a more responsive, 'snappier' connection, making the trail catch up faster. Lower values result in a 'looser', more delayed feel."
- headFriction: 30
  $name: Head Friction/Damping
  $description: "Controls the damping (friction) for the trail's head. This resists motion, preventing the head from overshooting the cursor and oscillating. Higher values make its movement smoother and less bouncy. Lower values can cause it to feel more 'springy'."
- spring: 100
  $name: Trail Spring Strength
  $description: "Defines the spring force connecting each segment of the trail to the one before it. Higher values result in a tighter, more rigid trail. Lower values create a more flexible, 'noodle-like' effect that wobbles more."
- friction: 30
  $name: Trail Friction/Damping
  $description: "Controls the damping (friction) for the trail's segments. This resists motion between segments, smoothing out the trail's movement. Higher values create a more viscous, 'gelatinous' trail. Lower values allow it to be more wobbly and energetic."
- trailLength: 5
  $name: Trail Length
  $description: "The number of physics-based segments in the trail. More segments create a longer and smoother trail but may increase CPU usage. Recommended range: 20-50."
- positionHistorySkip: 1
  $name: Position History Skip
  $description: "Skips N mouse position updates before updating the trail's target. 0 means the trail follows every update (smoothest). A value of 1 skips every other update, creating a slightly delayed/staggered effect. Higher values increase this effect, making the trail feel 'chunkier' or 'stepped'."
- cursorSize: 15
  $name: Base Cursor Size (px)
  $description: "The master diameter of the cursor head and the maximum width of the trail in pixels. This value acts as a global size control, which is then scaled by each layer's specific width factor. It's the 100% reference size."
- minTrailWidth: 2
  $name: Minimum Trail Width (px)
  $description: "The absolute minimum width in pixels that the trail can shrink to at its tail end. This prevents the trail from becoming completely invisible, even with strong fading effects or when the cursor is stationary."
- squishIntensity: 2
  $name: Squish Intensity
  $description: "Controls how much the cursor head deforms in response to cursor velocity. At 0, it remains a perfect circle. Higher values create a more dramatic 'stretch and squish' effect, making it look more elastic."
- squishSmoothing: 50
  $name: Squish Smoothing
  $description: "Controls the interpolation speed for the cursor head's follow behavior and squish animation. Higher values make it follow the mouse more tightly and react faster, feeling more 'digital'. Lower values create a more delayed, 'lagging', and 'gooey' feel."
- velocityWidthMultiplier: 0
  $name: Velocity Width Boost
  $description: "Dynamically increases the trail's width based on cursor speed. A value greater than 0 makes the trail thicker during fast movements, creating a motion blur-like effect. Set to 0 to disable. (Value is a multiplier)."
- velocityAlphaMultiplier: 5
  $name: Velocity Opacity Boost
  $description: "Dynamically increases the trail's opacity based on cursor speed. A value greater than 0 makes the trail brighter and more opaque during fast movements, enhancing its visual impact. Set to 0 to disable. (Value is a multiplier)."
- interpolationSteps: 3
  $name: Curve Smoothness
  $description: "The number of interpolation steps between each physics point to create a smooth curve (using Catmull-Rom splines). Higher values produce a smoother, more refined trail but are more computationally expensive. A good value is 3-5. See 'Adaptive Quality'."
- fadeMode: 2
  $name: Fade Curve
  $description: "The algorithm used for fading the trail's opacity and width. 0=Linear (uniform fade), 1=Ease-Out (fades fast then slows, natural), 2=Exponential (sharp, quick fade), 3=Sigmoid (S-curve, slow at ends, fast in middle)."
- adaptiveQuality: true
  $name: Adaptive Quality
  $description: "If enabled, automatically reduces 'Curve Smoothness' during fast cursor movements to maintain performance and prevent visual lag. It lowers interpolation steps when speed is high, which is less noticeable in motion. Highly recommended for performance."
- enableGradient: true
  $name: Enable Color Gradient
  $description: "If enabled, the trail layers will smoothly transition from their 'Start Color' to their 'End Color' along the trail's length. If disabled, only the 'Start Color' is used for the entire trail."
- hideSystemCursor: false
  $name: Hide System Cursor
  $description: "If enabled, replaces the default system cursor (e.g., the arrow) with a transparent one, allowing the 'Squishy Cursor Head' to act as the primary pointer. The original cursor is restored when the mod is disabled or this setting is turned off."
- layer1:
  - enabled: true
    $name: Enable Layer 1
  - color: '#FFFFFF'
    $name: Start Color
    $description: "The color of this layer at the head of the trail. If 'Enable Color Gradient' is active, this is the starting color of the gradient."
  - endColor: '#000000'
    $name: End Color (Gradient)
    $description: "The color of this layer at the tail end of the trail. This is the target color for the gradient if 'Enable Color Gradient' is active."
  - widthFactor: 120
    $name: Width Factor (%)
    $description: "A multiplier for the 'Base Cursor Size', as a percentage. E.g., 100 makes this layer's base width equal to the cursor size, while 50 makes it half the size."
  - alphaFactor: 50
    $name: Base Opacity (%)
    $description: "The base opacity of this layer, as a percentage (100% = fully opaque). This is the maximum opacity before any fading or velocity-based effects are applied."
  $name: Layer 1 (Outer Glow)
- layer2:
  - enabled: true
    $name: Enable Layer 2
  - color: '#000000'
    $name: Start Color
    $description: "The color of this layer at the head of the trail. If 'Enable Color Gradient' is active, this is the starting color of the gradient."
  - endColor: '#000000'
    $name: End Color (Gradient)
    $description: "The color of this layer at the tail end of the trail. This is the target color for the gradient if 'Enable Color Gradient' is active."
  - widthFactor: 90
    $name: Width Factor (%)
    $description: "A multiplier for the 'Base Cursor Size', as a percentage. E.g., 100 makes this layer's base width equal to the cursor size, while 50 makes it half the size."
  - alphaFactor: 100
    $name: Base Opacity (%)
    $description: "The base opacity of this layer, as a percentage (100% = fully opaque). This is the maximum opacity before any fading or velocity-based effects are applied."
  $name: Layer 2 (Mid Layer)
- layer3:
  - enabled: true
    $name: Enable Layer 3
  - color: '#FFFFFF'
    $name: Start Color
    $description: "The color of this layer at the head of the trail. If 'Enable Color Gradient' is active, this is the starting color of the gradient."
  - endColor: '#000000'
    $name: End Color (Gradient)
    $description: "The color of this layer at the tail end of the trail. This is the target color for the gradient if 'Enable Color Gradient' is active."
  - widthFactor: 40
    $name: Width Factor (%)
    $description: "A multiplier for the 'Base Cursor Size', as a percentage. E.g., 100 makes this layer's base width equal to the cursor size, while 50 makes it half the size."
  - alphaFactor: 100
    $name: Base Opacity (%)
    $description: "The base opacity of this layer, as a percentage (100% = fully opaque). This is the maximum opacity before any fading or velocity-based effects are applied."
  $name: Layer 3 (Core)
- layer4:
  - enabled: true
    $name: Enable Layer 4
  - color: '#000000'
    $name: Start Color
    $description: "The color of this layer at the head of the trail. If 'Enable Color Gradient' is active, this is the starting color of the gradient."
  - endColor: '#000000'
    $name: End Color (Gradient)
    $description: "The color of this layer at the tail end of the trail. This is the target color for the gradient if 'Enable Color Gradient' is active."
  - widthFactor: 5
    $name: Width Factor (%)
    $description: "A multiplier for the 'Base Cursor Size', as a percentage. E.g., 100 makes this layer's base width equal to the cursor size, while 50 makes it half the size."
  - alphaFactor: 100
    $name: Base Opacity (%)
    $description: "The base opacity of this layer, as a percentage (100% = fully opaque). This is the maximum opacity before any fading or velocity-based effects are applied."
  $name: Layer 4 (Inner Core)
- cursorHead:
  - enabled: false
    $name: Enable Squishy Cursor Head
  - filled: false
    $name: Filled (true) or Outlined (false)
    $description: "Determines if the cursor head is rendered as a solid, filled shape (when checked) or as a hollow outline (when unchecked)."
  - color: '#FFFFFF'
    $name: Cursor Color
    $description: "The color of the cursor head's shape (if filled) or its outline."
  - size: 15
    $name: Cursor Head Size (px)
    $description: "The base diameter of the cursor head in pixels when it is stationary (not moving)."
  - outlineWidth: 1
    $name: Outline Width (px)
    $description: "If the cursor is not filled, this sets the thickness of its outline in pixels. Only applies if 'Filled' is unchecked."
  - alpha: 100
    $name: Cursor Opacity (%)
    $description: "The opacity of the cursor head, as a percentage. 100% is fully opaque, while 0% is fully transparent."
  $name: Squishy Cursor Head
- rippleEffect:
  - enabled: true
    $name: Enable Click Ripple Effect
  - maxDiameter: 80
    $name: Ripple Max Diameter (px)
    $description: "The final diameter the ripple expands to before it finishes its animation and fades out completely."
  - startWidth: 8
    $name: Start Width (px)
    $description: "The initial thickness of the ripple's outline at the moment of the click. The outline will shrink as the ripple expands."
  - duration: 500
    $name: Duration (ms)
    $description: "The total time in milliseconds for the ripple animation to complete, from click to fade-out."
  - leftClickColor: '#FFFFFF'
    $name: Left-Click Color
    $description: "The color of the ripple effect when the left mouse button is clicked."
  - rightClickColor: '#FFFFFF'
    $name: Right-Click Color
    $description: "The color of the ripple effect when the right mouse button is clicked."
  - middleClickColor: '#FFFFFF'
    $name: Middle-Click Color
    $description: "The color of the ripple effect when the middle mouse button is clicked."
  - clickScaleFactor: 150
    $name: Cursor Click Scale Factor (%)
    $description: "The factor by which the system cursor will enlarge on click (e.g., 150% = 1.5x size). Only active if 'Enable Cursor Click Scaling' is on."
  - clickScaleDuration: 150
    $name: Cursor Click Scale Duration (ms)
    $description: "The duration in milliseconds for the cursor to scale up and back down on click."
  - enableClickScaling: true
    $name: Enable System Cursor Click Scaling
    $description: "If enabled, the system cursor will briefly enlarge on every click to provide visual feedback. This works best when 'Hide System Cursor' is disabled."
  $name: Click Ripple Effect
- satelliteEffect:
  - enabled: false
    $name: Enable Satellite Effect
  - ringDiameter: 41
    $name: Ring Diameter (px)
    $description: "The diameter of the circle on which the satellites orbit."
  - ringVisible: false
    $name: Show Orbit Ring
    $description: "If enabled, the ring that the satellites orbit on is visible."
  - ringWidth: 1
    $name: Ring Width (px)
    $description: "The thickness of the orbit ring, if visible."
  - ringColor: '#FFFFFF'
    $name: Ring Color
    $description: "The color of the orbit ring."
  - satelliteCount: 4
    $name: Number of Satellites
    $description: "How many satellites orbit the cursor."
  - satelliteDiameter: 8
    $name: Satellite Diameter (px)
    $description: "The size of each individual satellite."
  - satelliteWidth: 1
    $name: Satellite Outline Width (px)
    $description: "The thickness of the satellite's outline."
  - satelliteFilled: false
    $name: Filled Satellites
    $description: "If enabled, satellites are filled circles. If disabled, they are outlines."
  - satelliteColor: '#FFFFFF'
    $name: Satellite Color
    $description: "The color of the satellites."
  - rotationSpeed: 1
    $name: Rotation Speed
    $description: "How fast the satellites orbit the cursor. Can be negative for reverse direction."
  - reverseRotation: false
    $name: Reverse Rotation Direction
    $description: "If enabled, reverses the primary direction of the satellite orbit."
  - enableDualRing: false
    $name: Enable Mirrored Dual Ring
    $description: "If enabled, a second ring of satellites will be created, orbiting in the opposite direction to the primary ring, creating a mirrored effect."
  - dualRingRotationSpeed: 2
    $name: Mirrored Ring Rotation Speed
    $description: "Sets the rotation speed for the second, mirrored ring of satellites. This is independent of the primary ring's speed and direction. Only active if 'Enable Mirrored Dual Ring' is on."
  $name: Satellite Effect
- layout:
  - centerToCursorX: true
    $name: Center Horizontally to Cursor
    $description: "If enabled, the FPS counter and Keystroke overlay will be horizontally centered relative to the cursor's X position, overriding their individual left/right alignment settings."
  $name: Layout
- keystrokeOverlay:
  - enabled: true
    $name: Display Keystrokes
    $description: "If enabled, shows currently pressed keys on screen. Its position is determined by the Layout and FPS Counter settings to avoid overlap."
  $name: Keystroke Overlay
- fpsCounter:
  - enabled: true
    $name: Show FPS Counter
    $description: "Displays a real-time Frames Per Second (FPS) counter on the screen to monitor rendering performance. The FPS reflects the rate at which the effects are being drawn."
  - logFps: false
    $name: Log FPS to Debug Console
    $description: "If enabled, logs the current FPS to the Windhawk debug console at the specified refresh rate. Useful for performance monitoring without an on-screen display."
  - alignBottom: true
    $name: Align to Bottom
    $description: "If enabled, the FPS counter is aligned to the bottom of the redrawn area. Otherwise, it's at the top. The Keystroke Overlay will be positioned opposite to this."
  - alignRight: true
    $name: Align to Right
    $description: "If enabled, the FPS counter is aligned to the right of the redrawn area. Otherwise, it's on the left. The Keystroke Overlay will be positioned opposite to this."
  - refreshRate: 500
    $name: Display Refresh Rate (ms)
    $description: "The time interval in milliseconds at which the displayed FPS value updates. A lower value updates more frequently. Default is 500ms."
  $name: FPS Counter
- debugOverlay:
  - enabled: true
    $name: Enable Debug Overlay (Default ON for testing)
    $description: "Draws a red rectangle around the 'dirty region' that is being redrawn each frame. Useful for performance tuning and debugging."
  $name: Debug Overlay (Default ON for testing)
*/
// ==/WindhawkModSettings==


#include <windhawk_utils.h>
#include <gdiplus.h>
#include <winuser.h>
#include <thread>
#include <atomic>
#include <chrono>
#include <vector>
#include <string>
#include <algorithm>
#include <cmath>
#include <mutex>
#include <map>
#include <set>

#ifndef OCR_NORMAL
#define OCR_NORMAL 32512
#endif

// Define OEM cursor resources if they aren't already, to ensure the code compiles
// even if OEMRESOURCE isn't specified in the build flags.
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

#ifndef M_PI
#define M_PI 3.14159265358979323846
#endif

using namespace Gdiplus;

// Represents a single point in the physics-based trail.
// Each point acts as a node in a spring-mass-damper system, forming a flexible chain.
// It stores kinematic properties that are updated in each physics tick to simulate realistic motion,
// creating effects like inertia and wobble.
struct TrailPoint {
    PointF pos;          // The current position (x, y) of the point in screen coordinates.
    PointF vel;          // The current velocity vector (vx, vy), indicating the point's speed and direction of travel.
    float speed;         // The scalar magnitude of the velocity vector, calculated each frame for use in visual effects like dynamic width/opacity.
    float acceleration;  // The rate of change of speed. Positive values mean speeding up, negative mean slowing down. Used for more nuanced visual effects.
};

// Represents the state of the deforming cursor head.
// This struct tracks the properties needed to animate the "squish-and-stretch" effect,
// which is achieved by rendering a transformed ellipse. The `current` values smoothly
// interpolate towards the `target` values each frame to create a fluid, organic animation.
struct SquishyCircle {
    PointF pos;          // The current smoothed position of the circle's center. It lags slightly behind the actual cursor for a more fluid feel.
    PointF prevPos;      // The position from the previous frame, used to calculate the velocity and direction of movement.
    float currentScale;  // The current, smoothly interpolated scale factor for the stretch effect (0 = no stretch, >0 = stretch). This is the value used for rendering.
    float currentAngle;  // The current, smoothly interpolated rotation angle in radians, aligning the ellipse with the direction of movement. This is the value used for rendering.
    float targetScale;   // The target scale factor, calculated directly from the current cursor velocity. `currentScale` smoothly moves towards this value.
    float targetAngle;   // The target rotation angle, calculated directly from the current cursor direction. `currentAngle` smoothly moves towards this value.
};

// Represents a single ripple effect triggered by a mouse click.
// Each ripple is an independent animation with its own position, start time, and color.
// A new instance of this struct is created for every click when the effect is enabled.
struct Ripple {
    POINT pos;                                     // The screen coordinates where the click occurred, serving as the ripple's origin.
    std::chrono::steady_clock::time_point startTime; // The timestamp when the ripple was created, used to calculate animation progress (0.0 to 1.0).
    Color color;                                   // The color of the ripple, determined by the click type (left, right, or middle).
};

// Represents a single satellite in the orbital effect.
struct Satellite {
    float currentAngle; // The current angle of the satellite in its orbit, in radians.
    float mirroredAngle; // The angle for the mirrored satellite, updated independently.
};

// Holds all global state for the cursor effects, including settings, physics data, and system handles.
// Using a namespace encapsulates all related variables, preventing pollution of the global scope and making the code more organized.
namespace CursorState {
    std::atomic<bool> isThreadRunning = false; // Controls the main loop of the rendering thread. Set to false to signal a graceful shutdown.
    std::thread cursorThread;                  // The dedicated thread for performing physics updates and rendering all visual effects.
    HWND hWndCursor = nullptr;                 // Handle to the transparent, screen-sized overlay window where all effects are drawn.
    RECT prevDirtyRect = {0};                  // The screen area that was updated in the previous frame. Used for redraw optimization (dirty rectangle update).
    std::vector<TrailPoint> prevTrailPoints;   // A copy of the trail points from the previous frame, used for accurate dirty rectangle calculation.

    // State for the dedicated mouse hook thread. A separate thread is crucial for a low-level
    // mouse hook (WH_MOUSE_LL) because the system sends hook messages to the thread that
    // installed the hook. To avoid blocking the main UI or rendering threads, this hook
    // runs its own message loop on a dedicated background thread.
    std::thread mouseHookThread;
    std::atomic<bool> isMouseHookThreadRunning = false;
    DWORD mouseHookThreadId = 0;

    // State for the dedicated keyboard hook thread, following the same pattern as the mouse hook.
    std::thread keyboardHookThread;
    std::atomic<bool> isKeyboardHookThreadRunning = false;
    DWORD keyboardHookThreadId = 0;
    
    std::vector<TrailPoint> trailPoints; // The collection of points that form the physics-based trail. This is the core data structure for the trail effect.
    SquishyCircle squishyCircle = {{0, 0}, {0, 0}, 0, 0, 0, 0}; // The state of the animated "squishy" cursor head.
    std::vector<Satellite> satellites;   // A collection of all satellites for the satellite effect.
    long long frameCounter = 0;          // A simple frame counter, used for the 'Position History Skip' logic.
    POINT lastUsedMousePos = {0};        // The last mouse position used as a target for the trail head.
 
    float maxRecentSpeed = 0.0f;         // Tracks the highest speed achieved in the last few frames. This value decays over time and is used for the adaptive quality feature.

    // A struct to hold all user-configurable settings, loaded from the Windhawk UI.
    // This keeps all settings neatly organized in one place.
    struct Settings {
        // Physics
        float spring;
        float friction;
        float headSpring;
        float headFriction;
        int trailLength;
        int positionHistorySkip;
        // Appearance
        int cursorSize;
        int minTrailWidth;
        float squishIntensity;
        float squishSmoothing;
        float velocityWidthMultiplier;
        float velocityAlphaMultiplier;
        // Quality & General
        int interpolationSteps;
        int fadeMode;
        bool adaptiveQuality;
        bool enableGradient;
        bool hideSystemCursor;

        // Trail Layers
        struct Layer {
            bool enabled;
            WindhawkUtils::StringSetting color;
            WindhawkUtils::StringSetting endColor;
            float widthFactor;
            float alphaFactor;
        } layer1, layer2, layer3, layer4;

        // Cursor Head
        struct CursorHead {
            bool enabled;
            bool filled;
            WindhawkUtils::StringSetting color;
            int size;
            int outlineWidth;
            float alpha;
        } cursorHead;

        // Ripple Effect
        struct RippleEffect {
            bool enabled;
            int maxDiameter;
            int startWidth;
            int duration;
            WindhawkUtils::StringSetting leftClickColor;
            WindhawkUtils::StringSetting rightClickColor;
            WindhawkUtils::StringSetting middleClickColor;
            bool enableClickScaling;
            int clickScaleFactor;
            int clickScaleDuration;
        } rippleEffect;

        struct SatelliteEffect {
            bool enabled;
            int ringDiameter;
            bool ringVisible;
            int ringWidth;
            WindhawkUtils::StringSetting ringColor;
            int satelliteCount;
            int satelliteDiameter;
            int satelliteWidth;
            bool satelliteFilled;
            WindhawkUtils::StringSetting satelliteColor;
            float rotationSpeed;
            bool reverseRotation;
            bool enableDualRing;
            float dualRingRotationSpeed;
        } satelliteEffect;

        // FPS Counter
        struct FpsCounter {
            bool enabled;
            bool logFps; // Log FPS to debug console
            bool alignBottom; // Align to bottom (vs. top)
            bool alignRight; // Align to right (vs. left)
            int refreshRate; // How often the displayed number updates
        } fpsCounter;

        // Debug Overlay
        struct DebugOverlay {
            bool enabled;
        } debugOverlay;

        // Layout
        struct Layout {
            bool centerToCursorX;
        } layout;
    } settings;

    ULONG_PTR gdiplusToken; // Token for GDI+ session management. Must be held for the lifetime of the mod.
    HICON hOriginalCursor = nullptr; // Handle to the original system cursor, saved when `hideSystemCursor` is enabled so it can be restored on uninit.

    // Global state for click detection and ripple animations.
    HHOOK hMouseHook = nullptr;    // Handle to the low-level mouse hook (WH_MOUSE_LL) used to detect clicks system-wide.
    HHOOK hKeyboardHook = nullptr; // Handle to the low-level keyboard hook (WH_KEYBOARD_LL) used to detect key presses.
    std::vector<Ripple> ripples;       // A list of all currently active ripple animations. New ripples are added by the hook thread, and they are rendered and removed by the render thread.
    std::mutex ripplesMutex;           // A mutex to protect concurrent access to the `ripples` vector from both the hook and render threads, preventing data races.
}

// Namespace for cursor scaling state
namespace CursorScaling {
    // A map to store the original cursors for all types we are overriding.
    // We need to replace multiple cursor types (arrow, ibeam, hand, etc.)
    // to ensure our scaled cursor is shown consistently.
    std::map<UINT, HCURSOR> hOriginalCursors;

    // Stores a copy of the cursor that was active at the moment of the click.
    // This is the cursor we will be scaling.
    HCURSOR hCursorToScale = nullptr;

    // Stores a copy of the user's default arrow cursor, captured once at mod startup.
    // This is the "true original" we will restore to after the animation.
    HCURSOR hOriginalArrowCursor = nullptr;

    // A list of all system cursor IDs we will override during the scaling animation.
    const std::vector<UINT> kCursorTypesToOverride = {
        OCR_NORMAL, OCR_IBEAM, OCR_WAIT, OCR_CROSS, OCR_UP, OCR_SIZENWSE,
        OCR_SIZENESW, OCR_SIZEWE, OCR_SIZENS, OCR_SIZEALL, OCR_NO, OCR_HAND
    };

    std::chrono::steady_clock::time_point scaleStartTime;
    bool isScalingActive = false;
    int targetScaleFactor = 150; // in %, Default, will be loaded from settings.
}

// A dedicated namespace for the keystroke overlay feature.
namespace KeystrokeDisplay {
    // Use a set to store the virtual key codes of all currently pressed keys.
    // This allows us to track multiple simultaneous key presses.
    std::set<DWORD> pressedKeys;
    // The string that will be rendered on screen, representing the combination of pressed keys.
    std::wstring displayString; // e.g., "Ctrl + Shift + A"

    std::mutex keyMutex;

    // Helper function to get a user-friendly name for a given virtual key code.
    std::wstring GetKeyName(DWORD vkCode);
}
// A dedicated namespace to encapsulate all variables and logic for tracking frames per second (FPS).
// This helps keep the FPS calculation logic separate and organized.
namespace FpsTracker {
    std::atomic<int> frameCount = 0; // Number of frames rendered since the last FPS calculation.
    std::atomic<int> lastFps = 0; // The most recently calculated FPS value, ready for display.
    std::chrono::steady_clock::time_point lastFpsTime = std::chrono::steady_clock::now();
}

// The unique class name for our transparent overlay window.
constexpr WCHAR kCursorWindowClassName[] = L"WindhawkCustomGdiplusCursorTrail";

// Forward declarations of functions used in the mod.
LRESULT CALLBACK CursorWndProc(HWND, UINT, WPARAM, LPARAM); // Window procedure for the overlay window.
void UpdateTrailPhysics(POINT currentMousePos);             // Updates the physics simulation for the trail.
void UpdateSquishyCircle();                                  // Updates the animation state for the squishy cursor head.
void RenderUltimateTrail(Graphics& g, const RECT& dirtyRect, const CursorState::Settings::Layer& layer); // Renders one layer of the trail.
void RenderSquishyCircle(Graphics& g, const RECT& dirtyRect); // Renders the squishy cursor head.
void LoadSettings();                                         // Loads and applies all settings from the Windhawk UI.
Color ColorFromHexString(const std::wstring& hex);           // Converts a hex color string (e.g., L"#FFFFFF") to a GDI+ Color object.
Color LerpColor(const Color& start, const Color& end, float t); // Linearly interpolates between two GDI+ colors.
void SetSystemCursorVisibility(bool visible);                // Hides or shows the default system cursor.
UINT GetDpiForWindowSafe(HWND hWnd);                         // Safely gets the DPI for a given window, with a fallback for older systems.
float ApplyFadeCurve(float progress, int mode);              // Applies a selected fading algorithm to a 0.0-1.0 progress value.
PointF CatmullRomInterpolate(const PointF& p0, const PointF& p1, const PointF& p2, const PointF& p3, float t); // Calculates a point on a Catmull-Rom spline for smooth curves.
void RenderRipples(Graphics& g, const RECT& dirtyRect);         // Renders all active click ripple animations.
LRESULT CALLBACK LowLevelMouseProc(int nCode, WPARAM wParam, LPARAM lParam); // The callback for the low-level mouse hook.
void UpdateSatellites();                                     // Updates the rotation of the satellites.
void RenderSatellites(Graphics& g, const RECT& dirtyRect);   // Renders the orbiting satellites.

// Forward declaration for the keyboard hook procedure.
LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam);
void RenderKeystrokeOverlay(Graphics& g, const SIZE& bmpSize, const RECT& totalDirtyRect);

// Forward declarations for cursor scaling.
void ScaleAndSetCursor(int scaleFactor);
void UpdateAndApplyCursorScaling();
 
// A dedicated namespace for keystroke overlay settings, to avoid polluting the global scope.
namespace KeystrokeOverlaySettings {
    bool enabled = false;
}

void UpdateKeystrokeDisplayString();

/**
 * @brief Manages the cursor rendering thread's lifecycle and main loop.
 *
 * This function is the entry point for the dedicated rendering thread. It performs one-time
 * setup for the cursor rendering environment, which includes:
 * 1. Registering a custom window class for the overlay.
 * 2. Creating a transparent, screen-sized, click-through window that will host the GDI+ drawings.
 * 3. Initializing the cursor and trail positions to the current mouse location to prevent them from flying in from (0,0) on startup.
 *
 * After setup, it enters a continuous loop that drives the animation. In each iteration, it
 * updates the physics simulation for all effects and then triggers a
 * repaint of the window (`WM_PAINT`). The loop is rate-limited to control the frame rate of the effect,
 * balancing smoothness with CPU usage.
 */
void CursorThreadFunc() {
    // Get a handle to the current module (the DLL of this mod). This is necessary to register
    // a window class, as the system needs to know which module owns the window procedure.
    HMODULE hModule = nullptr;
    if (!GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, (LPCWSTR)&CursorThreadFunc, &hModule)) {
        Wh_Log(L"GetModuleHandleEx failed in CursorThreadFunc.");
        return;
    }

    // Register the window class for our overlay window.
    WNDCLASSW wc = {};
    wc.lpfnWndProc = CursorWndProc;
    wc.hInstance = hModule;
    wc.lpszClassName = kCursorWindowClassName;
    if (!RegisterClassW(&wc)) {
        Wh_Log(L"RegisterClassW failed with error %lu.", GetLastError());
        return;
    }

    // Get the dimensions of the entire virtual screen to cover all monitors.
    int vScreenX = GetSystemMetrics(SM_XVIRTUALSCREEN);
    int vScreenY = GetSystemMetrics(SM_YVIRTUALSCREEN);
    int vScreenWidth = GetSystemMetrics(SM_CXVIRTUALSCREEN);
    int vScreenHeight = GetSystemMetrics(SM_CYVIRTUALSCREEN);

    // Create a transparent, top-most, layered window that covers the entire screen.
    // WS_EX_TOPMOST:       Ensures the window stays above all other non-topmost windows.
    // WS_EX_LAYERED:       Enables per-pixel alpha transparency, which is essential for drawing smooth, anti-aliased shapes on the desktop.
    // WS_EX_TRANSPARENT:   Makes the window "click-through," allowing mouse events to pass to the windows underneath it.
    // WS_EX_TOOLWINDOW:    Hides the window from the taskbar and Alt+Tab switcher, making it feel like an overlay rather than an application.
    CursorState::hWndCursor = CreateWindowExW( // clang-format off
        WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW,
        wc.lpszClassName, L"Custom GDI+ Cursor Ultimate Squishy", WS_POPUP,
        vScreenX, vScreenY, vScreenWidth, vScreenHeight,
        nullptr, nullptr, hModule, nullptr
    );

    if (!CursorState::hWndCursor) {
        Wh_Log(L"Failed to create cursor window with error %lu.", GetLastError());
        UnregisterClassW(wc.lpszClassName, hModule);
        return;
    }

    ShowWindow(CursorState::hWndCursor, SW_SHOWNOACTIVATE);

    CursorState::isThreadRunning = true;

    // Initialize the squishy circle's position to the current cursor location to avoid a jump on start.
    POINT pt;
    GetCursorPos(&pt);
    CursorState::squishyCircle.pos = PointF((float)pt.x, (float)pt.y);
    CursorState::squishyCircle.prevPos = CursorState::squishyCircle.pos;

    // --- High-Precision Frame Rate Limiter ---
    // We aim for a target frame time (e.g., for 165 FPS) and dynamically sleep for the
    // remaining time in each frame's budget. This allows the loop to run as fast as possible up to the target,
    // rather than being artificially capped at a low rate.
    constexpr auto targetFrameTime = std::chrono::duration<double, std::milli>(1000.0 / 165.0); // Target frame time for ~165 FPS

    // Main rendering loop.
    while (CursorState::isThreadRunning) {
        auto frameStart = std::chrono::steady_clock::now();
        
        // Update and apply cursor scaling animation if enabled.
        if (CursorState::settings.rippleEffect.enableClickScaling) {
            UpdateAndApplyCursorScaling();
        }

        GetCursorPos(&pt);
        UpdateTrailPhysics(pt);
        UpdateSquishyCircle();
        UpdateSatellites();

        // Trigger a repaint of the overlay window. This will send a WM_PAINT message.
        InvalidateRect(CursorState::hWndCursor, nullptr, FALSE); // Invalidate to mark for redraw.
        UpdateWindow(CursorState::hWndCursor); // Forces an immediate WM_PAINT message.

        auto frameEnd = std::chrono::steady_clock::now();
        auto frameDuration = frameEnd - frameStart;

        // --- High-Precision Wait ---
        // std::this_thread::sleep_for can be imprecise due to system timer resolution,
        // especially for short durations. To ensure a stable frame rate,
        // we sleep for most of the duration and then "spin-wait" for the final millisecond.
        if (frameDuration < targetFrameTime) {
            auto remainingTime = targetFrameTime - frameDuration;
            // Sleep if we have more than ~1.5ms to wait.
            if (remainingTime > std::chrono::milliseconds(1) + std::chrono::microseconds(500)) {
                std::this_thread::sleep_for(remainingTime - std::chrono::milliseconds(1));
            }
            // Spin-wait for the final moment to achieve high precision.
            while (std::chrono::steady_clock::now() - frameStart < targetFrameTime);
        }
    }

    // Reset FPS counter on exit
    { // Use a block to scope the lock
        std::lock_guard<std::mutex> lock(CursorState::ripplesMutex); // Reuse mutex for thread-safe access
        FpsTracker::frameCount = 0;
        FpsTracker::lastFps = 0;
        FpsTracker::lastFpsTime = std::chrono::steady_clock::now();
    }

    // Cleanup resources when the thread exits.
    DestroyWindow(CursorState::hWndCursor);
    UnregisterClassW(wc.lpszClassName, hModule);
}

/**
 * @brief Manages the mouse hook thread's lifecycle.
 *
 * This function is the entry point for a dedicated thread responsible for handling a
 * low-level mouse hook (`WH_MOUSE_LL`). A separate thread is essential because the system
 * dispatches hook notifications to the specific thread that called `SetWindowsHookEx`.
 * To prevent these messages from interfering with the main application or rendering
 * thread, this function installs the hook and then enters a standard Windows message loop
 * (`GetMessage`/`TranslateMessage`/`DispatchMessage`). This loop's only job is to process hook-related messages.
 * When the mod is uninitialized, a `WM_QUIT` message is posted to this
 * thread to terminate the loop, allowing for a clean unhooking and thread exit.
 */
void MouseHookThreadFunc() {
    HMODULE hModule = nullptr;
    GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                       (LPCWSTR)&MouseHookThreadFunc, &hModule);

    // Install a global, low-level mouse hook. This allows us to intercept mouse events
    // system-wide, which is necessary to detect clicks anywhere on the screen.
    CursorState::hMouseHook = SetWindowsHookExW(WH_MOUSE_LL, LowLevelMouseProc, hModule, 0);
    if (!CursorState::hMouseHook) {
        Wh_Log(L"Failed to set mouse hook with error %lu.", GetLastError());
        return;
    }
    
    Wh_Log(L"Low-level mouse hook installed successfully.");

    // This message loop is crucial. It waits for messages from the system, including
    // notifications for our mouse hook, and dispatches them to the hook procedure (`LowLevelMouseProc`).
    // The loop runs until GetMessage returns 0, which happens when it receives a WM_QUIT message.
    MSG msg;
    while (GetMessage(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    // Cleanup the hook when the message loop is terminated.
    if (CursorState::hMouseHook) {
        UnhookWindowsHookEx(CursorState::hMouseHook);
        CursorState::hMouseHook = nullptr;
    }
    Wh_Log(L"Mouse hook thread exiting.");
}

// A dedicated thread for the low-level keyboard hook, following the same pattern as the mouse hook thread for thread safety and responsiveness.
void KeyboardHookThreadFunc() {
    HMODULE hModule = nullptr;
    GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                       (LPCWSTR)&KeyboardHookThreadFunc, &hModule);

    // Install the low-level keyboard hook.
    CursorState::hKeyboardHook = SetWindowsHookExW(WH_KEYBOARD_LL, LowLevelKeyboardProc, hModule, 0);
    if (!CursorState::hKeyboardHook) {
        Wh_Log(L"Failed to set keyboard hook with error %lu.", GetLastError());
        return;
    }
    
    Wh_Log(L"Low-level keyboard hook installed successfully.");

    // This message loop is essential for the hook to receive messages.
    MSG msg;
    while (GetMessage(&msg, nullptr, 0, 0)) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }

    // Cleanup the hook when the thread is terminated.
    if (CursorState::hKeyboardHook) {
        UnhookWindowsHookEx(CursorState::hKeyboardHook);
        CursorState::hKeyboardHook = nullptr;
    }
    Wh_Log(L"Keyboard hook thread exiting.");
}

/**
 * @brief Updates the physics simulation for each point in the trail using a spring-damper system.
 *
 * This function implements a simple Verlet integration-like physics model. It iterates
 * through each point in the trail, applying spring and friction (damping) forces to
 * simulate a flexible, connected chain.
 * - The head of the trail (index 0) is attracted to the real cursor position.
 * - Each subsequent segment is attracted to the one before it, creating the follow-the-leader motion.
 * This creates the characteristic lagging and wobbling motion. It also calculates
 * the speed and acceleration of each point, which are used for dynamic visual effects such as
 * changing the trail's width and opacity based on velocity.
 *
 * @param currentMousePos The current position of the system mouse cursor, which acts as the
 *                        target for the trail's head.
 */
void UpdateTrailPhysics(POINT currentMousePos) {
    if (CursorState::trailPoints.empty()) return;

    // Position history skip logic using a modulo operator for a correct repeating pattern.
    // The trail's target position is updated only on frames where the counter is a multiple of (skip + 1),
    // creating a pattern like P, S, S, P... (1 Position Update, N Skips).
    int skipCycleLength = CursorState::settings.positionHistorySkip + 1;
    if (CursorState::frameCounter % skipCycleLength == 0) {
        CursorState::lastUsedMousePos = currentMousePos;
    }
    CursorState::frameCounter++;

    // Normalize settings values from the UI (e.g., 0-100) to ranges suitable for physics calculations.
    float spring = CursorState::settings.spring / 1000.0f;
    // Convert friction from a 0-100 UI scale to a 0.0-1.0 damping factor.
    // A friction setting of 100 in the UI corresponds to a damping factor of 0.0 (maximum friction, no velocity retained),
    // while a setting of 0 corresponds to a factor of 1.0 (no friction). We invert the UI value for the physics calculation.
    float friction = 1.0f - (CursorState::settings.friction / 100.0f);
    float headSpring = CursorState::settings.headSpring / 1000.0f;
    float headFriction = 1.0f - (CursorState::settings.headFriction / 100.0f);

    // --- Update Head Point ---
    // The first point in the trail is attracted directly to the mouse cursor.
    TrailPoint& head = CursorState::trailPoints[0];

    // Apply spring force: accelerate the head towards the mouse cursor.
    head.vel.X += (CursorState::lastUsedMousePos.x - head.pos.X) * headSpring;
    head.vel.Y += (CursorState::lastUsedMousePos.y - head.pos.Y) * headSpring;

    // Apply friction: dampen the head's velocity to prevent overshooting and create a smoother follow.
    head.vel.X *= headFriction;
    head.vel.Y *= headFriction;

    // Update position based on the new velocity.
    head.pos.X += head.vel.X;
    head.pos.Y += head.vel.Y;

    // Calculate speed and acceleration for rendering effects.
    float oldSpeedHead = head.speed;
    head.speed = std::sqrt(head.vel.X * head.vel.X + head.vel.Y * head.vel.Y);
    head.acceleration = head.speed - oldSpeedHead;

    // --- Update Tail Segments ---
    // Each subsequent point is attracted to the point immediately preceding it.
    for (size_t i = 1; i < CursorState::trailPoints.size(); ++i) {
        TrailPoint& current = CursorState::trailPoints[i];
        const TrailPoint& prev = CursorState::trailPoints[i - 1];

        // A small influence from the point two segments ahead (prevPrev) is added. This acts as a
        // secondary, weaker spring. It helps create a more stable and organic "wobble" in the trail,
        // preventing it from collapsing into a straight line too quickly and propagating motion more naturally through the chain.
        float influence = 0.3f; // A small fraction to keep the influence subtle.
        if (i > 1) {
            const TrailPoint& prevPrev = CursorState::trailPoints[i - 2];
            current.vel.X += (prevPrev.pos.X - current.pos.X) * spring * influence;
            current.vel.Y += (prevPrev.pos.Y - current.pos.Y) * spring * influence;
        }

        // Apply main spring force: accelerate towards the previous point.
        current.vel.X += (prev.pos.X - current.pos.X) * spring;
        current.vel.Y += (prev.pos.Y - current.pos.Y) * spring;

        // Apply friction.
        current.vel.X *= friction;
        current.vel.Y *= friction;

        // Update position.
        current.pos.X += current.vel.X;
        current.pos.Y += current.vel.Y;

        // Calculate speed and acceleration.
        float oldSpeed = current.speed;
        current.speed = std::sqrt(current.vel.X * current.vel.X + current.vel.Y * current.vel.Y);
        current.acceleration = current.speed - oldSpeed;
    }

    // Track the maximum speed recently achieved by any point in the trail.
    // This is used for the adaptive quality feature and decays slowly over time.
    CursorState::maxRecentSpeed *= 0.95f;
    for (const auto& p : CursorState::trailPoints)
        CursorState::maxRecentSpeed = std::max(CursorState::maxRecentSpeed, p.speed);
}


/**
 * @brief Updates the animation state of the squishy cursor head.
 *
 * This function drives the squish-and-stretch animation of the cursor head. It first
 * smoothly interpolates the visible position of the circle towards the real-time mouse
 * cursor, creating a slight "lag" effect for a more organic feel.
 *
 * Based on the change in this smoothed position, it calculates a velocity. The velocity
 * determines a `targetScale` (for stretching) and a `targetAngle` (to align with the
 * direction of movement). The function then smoothly interpolates the circle's `currentScale`
 * and `currentAngle` towards these target values, creating a fluid, responsive animation
 * that deforms based on cursor movement.
 */
void UpdateSquishyCircle() {
    if (CursorState::trailPoints.empty()) return;
    
    // Get the real-time mouse position for the most responsive tracking.
    POINT mousePos;
    GetCursorPos(&mousePos);
    
    float smoothing = CursorState::settings.squishSmoothing / 100.0f;
    
    // Smoothly interpolate the circle's display position towards the actual cursor position.
    // This creates the "lagging" effect.
    CursorState::squishyCircle.pos.X += (mousePos.x - CursorState::squishyCircle.pos.X) * smoothing;
    CursorState::squishyCircle.pos.Y += (mousePos.y - CursorState::squishyCircle.pos.Y) * smoothing;
    
    // Calculate the distance moved since the last frame to determine velocity.
    float deltaX = CursorState::squishyCircle.pos.X - CursorState::squishyCircle.prevPos.X;
    float deltaY = CursorState::squishyCircle.pos.Y - CursorState::squishyCircle.prevPos.Y;
    float velocity = std::sqrt(deltaX * deltaX + deltaY * deltaY);
    
    // Store the current position for the next frame's calculation.
    CursorState::squishyCircle.prevPos = CursorState::squishyCircle.pos;
    
    // --- Calculate Target Scale ---
    // The amount of "squish" is proportional to the cursor's velocity.
    float squishIntensity = CursorState::settings.squishIntensity / 100.0f;
    // Amplify velocity to make the effect more noticeable even with small mouse movements.
    // The magic numbers (8.0, 200.0, 15.0) were found through experimentation to create a visually pleasing effect.
    float amplifiedVelocity = velocity * 8.0f;
    // Clamp the velocity to prevent extreme, jarring stretching on very fast mouse flicks.
    float clampedVelocity = std::min(amplifiedVelocity, 200.0f);
    // Convert the final velocity into a scale factor, modulated by the user's intensity setting.
    float scaleValue = (clampedVelocity / 15.0f) * squishIntensity;
    CursorState::squishyCircle.targetScale = scaleValue;
    
    // Smoothly interpolate the current scale towards the target scale.
    CursorState::squishyCircle.currentScale += (CursorState::squishyCircle.targetScale - CursorState::squishyCircle.currentScale) * smoothing;
    
    // --- Calculate Target Angle ---
    // The circle rotates to align with the direction of movement.
    if (velocity > 0.5f) { // Only update the angle if there's noticeable movement to prevent jitter.
        CursorState::squishyCircle.targetAngle = std::atan2(deltaY, deltaX); // atan2 gives the angle in radians.
    }
    
    // Smoothly interpolate the current angle towards the target angle.
    // This is more complex than linear interpolation because angles wrap around (e.g., from +PI to -PI).
    // The code calculates the shortest rotational distance between the current and target angles
    // to prevent the ellipse from taking the "long way around" when crossing the 180-degree boundary.
    float angleDiff = CursorState::squishyCircle.targetAngle - CursorState::squishyCircle.currentAngle;
    while (angleDiff > M_PI) angleDiff -= 2.0f * M_PI;  // Normalize to (-PI, PI] range
    while (angleDiff < -M_PI) angleDiff += 2.0f * M_PI; // Normalize to [-PI, PI) range
    CursorState::squishyCircle.currentAngle += angleDiff * smoothing;
}

/**
 * @brief Updates the rotation angles for all satellites in the orbital effect.
 *
 * This function is called each frame to advance the position of the satellites in their orbits based on the user-defined speeds.
 * It handles both the primary and the optional mirrored ring, updating their angles for the rendering step.
 */
void UpdateSatellites() {
    if (!CursorState::settings.satelliteEffect.enabled) return;

    // --- Update Primary Ring ---
    float baseRotationSpeed = CursorState::settings.satelliteEffect.rotationSpeed / 100.0f;
    if (CursorState::settings.satelliteEffect.reverseRotation) {
        baseRotationSpeed *= -1.0f;
    }

    // --- Update Mirrored Ring (if enabled) ---
    // The mirrored ring's speed is independent and rotates based on its own setting.
    float mirroredRotationSpeed = -(CursorState::settings.satelliteEffect.dualRingRotationSpeed / 100.0f);

    for (auto& satellite : CursorState::satellites) {
        // Update primary angle
        satellite.currentAngle += baseRotationSpeed;
        if (satellite.currentAngle > 2.0f * M_PI) {
            satellite.currentAngle -= 2.0f * M_PI;
        } else if (satellite.currentAngle < 0.0f) {
            satellite.currentAngle += 2.0f * M_PI;
        }

        // Update mirrored angle if the effect is active
        if (CursorState::settings.satelliteEffect.enableDualRing) {
            satellite.mirroredAngle += mirroredRotationSpeed;
            if (satellite.mirroredAngle > 2.0f * M_PI) {
                satellite.mirroredAngle -= 2.0f * M_PI;
            } else if (satellite.mirroredAngle < 0.0f) {
                satellite.mirroredAngle += 2.0f * M_PI;
            }
        }
    }
}

/**
 * @brief Applies a non-linear fading curve to a linear progress value.
 *
 * This function transforms a linear progress value (where 0.0 is the trail head and 1.0 is the tail)
 * using a selected mathematical function. This allows for more visually interesting fade-outs
 * for the trail's width and opacity compared to a simple linear falloff. The output value
 * is typically inverted (1.0 at the head, 0.0 at the tail) to be used as a multiplier.
 *
 * @param progress The linear progress along the trail, normalized from 0.0 (head) to 1.0 (tail).
 * @param mode The selected fade mode from settings (0=Linear, 1=Ease-Out, etc.).
 * @return The transformed fade factor, typically mapping progress=0.0 to a result of 1.0 and progress=1.0 to a result of 0.0.
 */
float ApplyFadeCurve(float progress, int mode) {
    switch (mode) {
        case 1: // Ease-Out (Quadratic): Starts fading fast, then slows down. A common, natural-looking fade.
            return 1.0f - progress * progress;
        case 2: // Exponential: A very sharp, pronounced fade-out that quickly becomes transparent.
            return std::exp(-progress * 3.0f);
        case 3: // Sigmoid: An 'S'-shaped curve, with a slow fade at the start and end, but fast in the middle.
            return 1.0f / (1.0f + std::exp(8.0f * (progress - 0.5f)));
        default: // Case 0, Linear: A constant, uniform rate of fading from head to tail.
            return 1.0f - progress;
    }
}

/**
 * @brief Calculates an interpolated point on a Catmull-Rom spline.
 *
 * This function generates a smooth curve that passes through a series of control points
 * (our physics-based `TrailPoint`s). Given four consecutive points (p0, p1, p2, p3), it calculates
 * an interpolated point on the curve segment that lies *between p1 and p2*. This is
 * the core of the trail smoothing algorithm, turning the discrete physics points into a
 * continuous, visually pleasing curve.
 *
 * @param p0 The control point before the segment starts (the "history" point).
 * @param p1 The starting point of the curve segment.
 * @param p2 The ending point of the curve segment.
 * @param p3 The control point after the segment ends (the "future" point).
 * @param t The interpolation factor (from 0.0 to 1.0) along the segment from p1 to p2.
 * @return The calculated PointF on the curve for the given `t` value.
 */
PointF CatmullRomInterpolate(const PointF& p0, const PointF& p1, const PointF& p2, const PointF& p3, float t) {
    float t2 = t * t;
    float t3 = t2 * t;
    
    PointF result;
    result.X = 0.5f * ((2.0f * p1.X) +
                       (-p0.X + p2.X) * t +
                       (2.0f * p0.X - 5.0f * p1.X + 4.0f * p2.X - p3.X) * t2 +
                       (-p0.X + 3.0f * p1.X - 3.0f * p2.X + p3.X) * t3);
    result.Y = 0.5f * ((2.0f * p1.Y) +
                       (-p0.Y + p2.Y) * t +
                       (2.0f * p0.Y - 5.0f * p1.Y + 4.0f * p2.Y - p3.Y) * t2 +
                       (-p0.Y + 3.0f * p1.Y - 3.0f * p2.Y + p3.Y) * t3);
    return result;
}

/**
 * @brief Linearly interpolates between two GDI+ Color objects.
 *
 * @param start The starting color (when t=0.0).
 * @param end The ending color (when t=1.0).
 * @param t The interpolation factor, clamped between 0.0 and 1.0.
 * @return The resulting interpolated GDI+ Color object.
 */
Color LerpColor(const Color& start, const Color& end, float t) {
    t = std::clamp(t, 0.0f, 1.0f);
    return Color(
        (BYTE)(start.GetAlpha() + (end.GetAlpha() - start.GetAlpha()) * t),
        (BYTE)(start.GetR() + (end.GetR() - start.GetR()) * t),
        (BYTE)(start.GetG() + (end.GetG() - start.GetG()) * t),
        (BYTE)(start.GetB() + (end.GetB() - start.GetB()) * t)
    );
}

/**
 * @brief Renders the squishy cursor head as a transformed ellipse.
 *
 * This function draws the cursor head based on the current `SquishyCircle` state.
 * It uses GDI+ transformations (translation, rotation) to render the
 * ellipse, which is highly efficient as the GPU can handle the complex matrix math.
 * The process is:
 * 1. Save the current graphics state.
 * 2. Translate the coordinate system to the circle's smoothed position.
 * 3. Rotate the coordinate system to match the circle's angle of movement.
 * 4. Draw the ellipse, scaled based on the `currentScale`, at the new origin (0,0).
 * 5. Restore the graphics state to not affect subsequent drawing operations.
 *
 * @param g A reference to the GDI+ Graphics object to draw on.
 * @param dirtyRect The rectangular area of the screen being updated, used to calculate
 *                  the relative drawing coordinates within the off-screen bitmap.
 */
void RenderSquishyCircle(Graphics& g, const RECT& dirtyRect) {
    if (!CursorState::settings.cursorHead.enabled) return;
    
    UINT dpi = GetDpiForWindowSafe(CursorState::hWndCursor);
    float baseSize = (float)MulDiv(CursorState::settings.cursorHead.size, dpi, 96);
    
    Color color = ColorFromHexString(CursorState::settings.cursorHead.color.get());
    BYTE alpha = (BYTE)(color.GetAlpha() * CursorState::settings.cursorHead.alpha / 100.0f);
    
    // The squish effect is achieved by scaling an ellipse. It stretches along the
    // axis of movement (scaleX) and compresses along the perpendicular axis (scaleY).
    float scaleX = 1.0f + CursorState::squishyCircle.currentScale;
    float scaleY = std::max(0.3f, 1.0f - CursorState::squishyCircle.currentScale); // Prevent the ellipse from becoming invisibly thin during a hard stretch.
    
    // Calculate the final dimensions of the ellipse.
    float width = baseSize * scaleX;
    float height = baseSize * scaleY;
    
    // Calculate the center position relative to the top-left of the dirty rectangle.
    float centerX = CursorState::squishyCircle.pos.X - dirtyRect.left;
    float centerY = CursorState::squishyCircle.pos.Y - dirtyRect.top;
    
    // Save the current graphics state so we can restore it after our transformations.
    GraphicsState state = g.Save();
    
    // Apply transformations in reverse order of effect:
    // 1. Translate the coordinate system's origin to the center of our ellipse.
    g.TranslateTransform(centerX, centerY);
    // 2. Rotate the coordinate system to match the direction of cursor movement.
    g.RotateTransform(CursorState::squishyCircle.currentAngle * 180.0f / M_PI);
    
    // Now, we can draw the ellipse centered at (0,0) in our new transformed space, and it will appear
    // correctly scaled, rotated, and positioned on the screen.
    if (CursorState::settings.cursorHead.filled) {
        SolidBrush brush(Color(alpha, color.GetR(), color.GetG(), color.GetB()));
        g.FillEllipse(&brush, -width / 2.0f, -height / 2.0f, width, height);
    } else {
        float outlineWidth = (float)MulDiv(CursorState::settings.cursorHead.outlineWidth, dpi, 96);
        Pen pen(Color(alpha, color.GetR(), color.GetG(), color.GetB()), outlineWidth);
        pen.SetAlignment(PenAlignmentInset); // Ensures the pen's width is drawn inside the ellipse's bounds, preventing the shape from becoming larger than intended.
        g.DrawEllipse(&pen, -width / 2.0f, -height / 2.0f, width, height);
    }
    
    // Restore the original graphics state to undo the translation and rotation for subsequent drawing operations.
    g.Restore(state);
}

/**
 * @brief Renders the orbiting satellites around the cursor.
 *
 * @param g A reference to the GDI+ Graphics object to draw on.
 * @param dirtyRect The rectangular area of the screen being updated, used for coordinate offsetting.
 */
void RenderSatellites(Graphics& g, const RECT& dirtyRect) {
    if (!CursorState::settings.satelliteEffect.enabled || CursorState::satellites.empty()) return;

    UINT dpi = GetDpiForWindowSafe(CursorState::hWndCursor);
    POINT mousePos;
    GetCursorPos(&mousePos);

    float centerX = (float)mousePos.x - dirtyRect.left;
    float centerY = (float)mousePos.y - dirtyRect.top;

    // Render the orbit ring if visible.
    if (CursorState::settings.satelliteEffect.ringVisible) {
        float ringDiameter = (float)MulDiv(CursorState::settings.satelliteEffect.ringDiameter, dpi, 96);
        float ringWidth = (float)MulDiv(CursorState::settings.satelliteEffect.ringWidth, dpi, 96);
        Color ringColor = ColorFromHexString(CursorState::settings.satelliteEffect.ringColor.get());
        Pen ringPen(ringColor, ringWidth);
        g.DrawEllipse(&ringPen, centerX - ringDiameter / 2, centerY - ringDiameter / 2, ringDiameter, ringDiameter);
    }

    // Prepare GDI+ objects for rendering the satellites.
    float satelliteDiameter = (float)MulDiv(CursorState::settings.satelliteEffect.satelliteDiameter, dpi, 96);
    float satelliteWidth = (float)MulDiv(CursorState::settings.satelliteEffect.satelliteWidth, dpi, 96);
    Color satelliteColor = ColorFromHexString(CursorState::settings.satelliteEffect.satelliteColor.get());
    SolidBrush satelliteBrush(satelliteColor);
    Pen satellitePen(satelliteColor, satelliteWidth);

    float orbitRadius = (float)MulDiv(CursorState::settings.satelliteEffect.ringDiameter / 2, dpi, 96);

    for (const auto& satellite : CursorState::satellites) {
        // Calculate the satellite's position on the circle using trigonometry.
        // cos(angle) for X, sin(angle) for Y.
        float satelliteX = centerX + orbitRadius * std::cos(satellite.currentAngle) - satelliteDiameter / 2;
        float satelliteY = centerY + orbitRadius * std::sin(satellite.currentAngle) - satelliteDiameter / 2;
        
        if (CursorState::settings.satelliteEffect.satelliteFilled) {
            g.FillEllipse(&satelliteBrush, satelliteX, satelliteY, satelliteDiameter, satelliteDiameter);
        } else {
            g.DrawEllipse(&satellitePen, satelliteX, satelliteY, satelliteDiameter, satelliteDiameter);
        }
    }

    // If dual ring is enabled, render the second set of satellites rotating in the opposite direction.
    if (CursorState::settings.satelliteEffect.enableDualRing) {
        for (const auto& satellite : CursorState::satellites) {
            // Use the independent 'mirroredAngle' for the second ring's position.
            float satelliteX = centerX + orbitRadius * std::cos(satellite.mirroredAngle) - satelliteDiameter / 2;
            float satelliteY = centerY + orbitRadius * std::sin(satellite.mirroredAngle) - satelliteDiameter / 2;
            if (CursorState::settings.satelliteEffect.satelliteFilled) {
                g.FillEllipse(&satelliteBrush, satelliteX, satelliteY, satelliteDiameter, satelliteDiameter);
            } else {
                g.DrawEllipse(&satellitePen, satelliteX, satelliteY, satelliteDiameter, satelliteDiameter);
            }
        }
    }
}

/**
 * @brief Renders all active ripple animations.
 *
 * This function iterates through the list of active ripples. For each ripple, it calculates
 * the current state of its animation based on the time elapsed since it was created. It
 * determines the ripple's current diameter, outline width, and opacity, then draws it as
 * a transparent, expanding circle. Ripples that have exceeded their duration are
 * removed from the list using the erase-remove idiom to maintain performance.
 *
 * @param g A reference to the GDI+ Graphics object to draw on.
 * @param dirtyRect The rectangular area of the screen being updated, used for coordinate offsetting.
 */
void RenderRipples(Graphics& g, const RECT& dirtyRect) {
    if (!CursorState::settings.rippleEffect.enabled) return;

    // Lock the mutex to ensure thread-safe access to the ripples vector, which can be
    // modified by the mouse hook thread at any time.
    std::lock_guard<std::mutex> lock(CursorState::ripplesMutex);
    auto now = std::chrono::steady_clock::now();
    UINT dpi = GetDpiForWindowSafe(CursorState::hWndCursor);
    
    // Remove old ripples that have completed their animation.
    // This uses the erase-remove idiom for efficient element removal from a vector.
    CursorState::ripples.erase(
        std::remove_if(CursorState::ripples.begin(), CursorState::ripples.end(),
            [&](const Ripple& r) {
                auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - r.startTime).count();
                return elapsed > CursorState::settings.rippleEffect.duration;
            }),
        CursorState::ripples.end()
    );

    // Draw each remaining active ripple.
    for (const auto& ripple : CursorState::ripples) {
        // Calculate the animation's progress from 0.0 (start) to 1.0 (end).
        auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - ripple.startTime).count();
        float progress = (float)elapsed / CursorState::settings.rippleEffect.duration;
        progress = std::clamp(progress, 0.0f, 1.0f);

        // The ripple's width and alpha use a "ping-pong" animation curve. They start at max,
        // decrease to zero at the halfway point, and would increase again if allowed. We use a fade curve
        // on this progress to make the width and alpha fade out smoothly over the animation's duration.
        // The result is an animation that fades in and then fades out, which we use for both the ripple's width and its opacity.
        float pingPongProgress = 1.0f - 2.0f * std::abs(progress - 0.5f);
        float animatedFactor = ApplyFadeCurve(1.0f - pingPongProgress, CursorState::settings.fadeMode);

        // The diameter expands linearly from 0 to the max diameter over the animation's duration.
        float maxDiameter = (float)MulDiv(CursorState::settings.rippleEffect.maxDiameter, dpi, 96);
        float currentDiameter = maxDiameter * progress;
        
        // The outline width shrinks based on the animated factor.
        float startWidth = (float)MulDiv(CursorState::settings.rippleEffect.startWidth, dpi, 96);
        float currentWidth = startWidth * animatedFactor;

        if (currentWidth < 0.5f) continue; // Skip drawing if it's too thin to be visible.

        // The alpha also fades out based on the animated factor.
        BYTE currentAlpha = (BYTE)(255.0f * animatedFactor);
        
        Pen pen(Color(currentAlpha, ripple.color.GetR(), ripple.color.GetG(), ripple.color.GetB()), currentWidth);
        pen.SetAlignment(PenAlignmentCenter); // Draw the pen stroke centered on the ellipse's geometric path.
        
        // Calculate the top-left corner for drawing the ellipse, relative to the dirty rect.
        float x = (float)ripple.pos.x - currentDiameter / 2.0f - dirtyRect.left;
        float y = (float)ripple.pos.y - currentDiameter / 2.0f - dirtyRect.top;
        
        g.DrawEllipse(&pen, x, y, currentDiameter, currentDiameter);
    }
}

/**
 * @brief Renders a single, complete layer of the cursor trail.
 *
 * This is the heart of the visual output. It generates a smooth, dynamic trail based
 * on the raw physics points. The process for each layer is:
 * 1. It iterates through the list of `TrailPoint`s.
 * 2. For each segment between two physics points, it uses Catmull-Rom interpolation to generate a
 *    smooth curve, which is approximated by breaking it down into many small, straight line segments.
 * 3. For each of these small, interpolated segments, it calculates its properties (width, color, opacity) by
 *    linearly interpolating the values from the nearest physics points.
 * 4. These properties are then modified by user settings like fade curves, velocity multipliers,
 *    and gradients to create the final dynamic appearance.
 * 5. Finally, it draws the stylized line segment to the off-screen buffer.
 *
 * @param g A reference to the GDI+ Graphics object to draw on.
 * @param dirtyRect The rectangular area of the screen being updated, used for coordinate offsetting.
 * @param layer The settings for the specific layer to be rendered (e.g., `settings.layer1`).
 */
void RenderUltimateTrail(Graphics& g, const RECT& dirtyRect, const CursorState::Settings::Layer& layer) {
    if (!layer.enabled || CursorState::trailPoints.size() < 2) return;

    // Scale sizes based on the window's DPI for consistent appearance on high-DPI displays.
    UINT dpi = GetDpiForWindowSafe(CursorState::hWndCursor);
    int scaledCursorSize = MulDiv(CursorState::settings.cursorSize, dpi, 96);
    int scaledMinWidth = MulDiv(CursorState::settings.minTrailWidth, dpi, 96);
    
    Color startColor = ColorFromHexString(layer.color.get());
    Color endColor = CursorState::settings.enableGradient ?
                     ColorFromHexString(layer.endColor.get()) : startColor;
    
    // Configure the GDI+ Pen for drawing the trail lines.
    Pen pen(startColor);
    pen.SetStartCap(LineCapRound);
    pen.SetEndCap(LineCapRound);
    pen.SetLineJoin(LineJoinRound);
    
    // --- Adaptive Quality ---
    // Adjust the number of interpolation steps based on cursor speed if the setting is enabled.
    int adaptiveInterpSteps = CursorState::settings.interpolationSteps;
    if (CursorState::settings.adaptiveQuality) {
        // This logic smoothly reduces rendering detail during fast movements to improve performance when it is least noticeable.
        const float speedThreshold = 15.0f;     // The cursor speed (in pixels per frame) at which quality reduction begins.
        const float reductionScale = 50.0f;     // Range of speed over which quality drops to its minimum.
        const float maxReductionFactor = 0.80f; // Max percentage of steps to remove (e.g., 80%).

        // Calculate a 'speed factor' from 0.0 (slow) to 1.0 (fast).
        float speedFactor = std::clamp((CursorState::maxRecentSpeed - speedThreshold) / reductionScale, 0.0f, 1.0f);
        
        // Determine the reduction percentage based on the speed factor.
        float reduction = speedFactor * maxReductionFactor;
        
        // Apply the reduction, ensuring at least one step remains.
        adaptiveInterpSteps = std::max(1, static_cast<int>(CursorState::settings.interpolationSteps * (1.0f - reduction)));
    }
    
    // Fallback to simple line drawing if there aren't enough points for a Catmull-Rom spline, which requires at least 4 points.
    if (CursorState::trailPoints.size() < 4) {
        for (size_t i = 0; i < CursorState::trailPoints.size() - 1; ++i) {
            float progress = (float)i / (CursorState::trailPoints.size() - 1);
            float fadeFactor = ApplyFadeCurve(progress, CursorState::settings.fadeMode);
            
            float normalizedSpeed = std::min(CursorState::trailPoints[i].speed / 20.0f, 1.0f);
            float velocityBoost = 1.0f + (normalizedSpeed * CursorState::settings.velocityWidthMultiplier);
            float width = std::max((float)scaledMinWidth,
                          scaledCursorSize * layer.widthFactor * fadeFactor * velocityBoost);
            
            float baseAlpha = 255.0f * layer.alphaFactor * fadeFactor;
            float opacityBoost = 1.0f;
            if (baseAlpha > 10.0f) {
                opacityBoost = 1.0f + (normalizedSpeed * CursorState::settings.velocityAlphaMultiplier);
            }

            Color segmentColor = CursorState::settings.enableGradient ?
                            LerpColor(startColor, endColor, progress) : startColor;
            BYTE alpha = (BYTE)std::clamp(baseAlpha * opacityBoost, 0.0f, 255.0f);

            pen.SetColor(Color(alpha, segmentColor.GetR(), segmentColor.GetG(), segmentColor.GetB()));
            pen.SetWidth(width);
            
            g.DrawLine(&pen,
                CursorState::trailPoints[i].pos.X - dirtyRect.left,
                CursorState::trailPoints[i].pos.Y - dirtyRect.top,
                CursorState::trailPoints[i + 1].pos.X - dirtyRect.left,
                CursorState::trailPoints[i + 1].pos.Y - dirtyRect.top);
        }
        return;
    }

    // Main rendering loop: iterate through each segment of the trail.
    // To draw a smooth curve between point `p1` (at index `i`) and `p2` (at `i+1`), the Catmull-Rom
    // algorithm requires a control point before (`p0`) and one after (`p3`). This loop structure
    // provides these four points for each segment of the trail, ensuring a continuous, smooth curve.
    for (size_t i = 1; i < CursorState::trailPoints.size() - 2; ++i) { // Loop must be within bounds that allow for p0, p1, p2, p3.
        // Get the four control points needed for the Catmull-Rom spline for the segment between p1 and p2.
        // Clamping is used at the ends of the trail to provide valid control points.
        const PointF& p0 = CursorState::trailPoints[std::max(0, (int)i - 1)].pos;
        const PointF& p1 = CursorState::trailPoints[i].pos;
        const PointF& p2 = CursorState::trailPoints[i + 1].pos;
        const PointF& p3 = CursorState::trailPoints[std::min(CursorState::trailPoints.size() - 1, i + 2)].pos;
        
        // Draw the curve by breaking it into smaller, straight line segments.
        for (int step = 0; step < adaptiveInterpSteps; ++step) {
            float t = (float)step / adaptiveInterpSteps;
            float nextT = (float)(step + 1) / adaptiveInterpSteps;
            
            PointF current = CatmullRomInterpolate(p0, p1, p2, p3, t);
            PointF next = CatmullRomInterpolate(p0, p1, p2, p3, nextT);
            
            // Calculate properties for this specific interpolated segment.
            float globalProgress = ((float)i + t) / (CursorState::trailPoints.size() - 1);
            float fadeFactor = ApplyFadeCurve(globalProgress, CursorState::settings.fadeMode);
            
            // Interpolate speed and acceleration between the two nearest real trail points for smooth transitions.
            // Since the interpolated points generated by Catmull-Rom do not have their own physics state (like velocity),
            // we perform a linear interpolation (lerp) of these properties from the two real physics points
            // that define the segment (p1 and p2). This ensures a smooth gradient of width and opacity along the interpolated curve.
            float blendedSpeed = CursorState::trailPoints[i].speed * (1.0f - t) +
                                 CursorState::trailPoints[i + 1].speed * t;
            float blendedAccel = CursorState::trailPoints[i].acceleration * (1.0f - t) +
                                 CursorState::trailPoints[i + 1].acceleration * t;
            
            // Calculate width boost from velocity.
            float normalizedSpeed = std::min(blendedSpeed / 20.0f, 1.0f);
            float velocityBoost = 1.0f;
            if (CursorState::settings.velocityWidthMultiplier > 0.0f) {
                velocityBoost += (normalizedSpeed * CursorState::settings.velocityWidthMultiplier * 0.1);
            }

            float width = std::max((float)scaledMinWidth,
                          scaledCursorSize * layer.widthFactor * fadeFactor * velocityBoost);
            
            if (width < 0.3f) continue; // Skip drawing invisibly thin lines to save performance.
            
            // Calculate opacity boost from velocity.
            float opacityBoost = 1.0f + (normalizedSpeed * CursorState::settings.velocityAlphaMultiplier);
            
            // Determine the color, either solid or from a gradient.
            Color segmentColor = CursorState::settings.enableGradient ?
                               LerpColor(startColor, endColor, globalProgress) : startColor;
            
            // Combine all factors to get the final alpha value.
            BYTE alpha = (BYTE)std::min(255.0f, segmentColor.GetAlpha() * layer.alphaFactor * fadeFactor * opacityBoost);
            pen.SetColor(Color(alpha, segmentColor.GetR(), segmentColor.GetG(), segmentColor.GetB()));
            pen.SetWidth(width);
            
            // Draw the line segment, offsetting by the dirty rect's position to draw on the correct part of our bitmap.
            g.DrawLine(&pen,
                current.X - dirtyRect.left, current.Y - dirtyRect.top,
                next.X - dirtyRect.left, next.Y - dirtyRect.top);
        }
    }
}

/**
 * @brief The window procedure for the transparent cursor overlay window.
 *
 * This function handles messages sent to our overlay window. Its primary responsibility
 * is to orchestrate the rendering process in response to `WM_PAINT` messages. It uses
 * two key optimizations for performance and visual quality:
 *
 * 1. **Dirty Rectangle Update:** Instead of redrawing the entire screen every frame, it
 *    calculates the smallest possible rectangle ("dirty rect") that encloses both the
 *    trail's previous and current positions, plus any active effects. Only this area is redrawn.
 *
 * 2. **Double Buffering:** All drawing operations are performed on an off-screen bitmap
 *    in memory. Once rendering is complete, this bitmap is blitted to the screen in a
 *    single operation using `UpdateLayeredWindow`, which prevents flickering.
 */
LRESULT CALLBACK CursorWndProc(HWND hWnd, UINT message, WPARAM wParam, LPARAM lParam) {
    // REMOVED: Hotkey handling is no longer done here. The low-level hook captures keys instead.
    if (message == WM_DESTROY) {
        CursorState::prevTrailPoints.clear();
    }

    if (message == WM_PAINT) {
        if (CursorState::trailPoints.empty()) {
            return 0;
        }
        
        // --- Dirty Rectangle Calculation ---
        // To optimize rendering, we only redraw the part of the screen that has changed.
        // This "dirty rectangle" is the bounding box of all effects from the last frame
        // and the current frame combined, ensuring we erase the old trail and draw the new one.
        
        UINT dpi = GetDpiForWindowSafe(hWnd); // Get DPI for proper scaling.

        // Dynamically calculate padding based on the widest enabled trail layer.
        // This ensures the bounding box is snug and not excessively large.
        float maxWidthFactor = 0.0f;
        if (CursorState::settings.layer1.enabled) maxWidthFactor = std::max(maxWidthFactor, CursorState::settings.layer1.widthFactor);
        if (CursorState::settings.layer2.enabled) maxWidthFactor = std::max(maxWidthFactor, CursorState::settings.layer2.widthFactor);
        if (CursorState::settings.layer3.enabled) maxWidthFactor = std::max(maxWidthFactor, CursorState::settings.layer3.widthFactor);
        if (CursorState::settings.layer4.enabled) maxWidthFactor = std::max(maxWidthFactor, CursorState::settings.layer4.widthFactor);
        int padding = MulDiv((int)(CursorState::settings.cursorSize * maxWidthFactor), dpi, 96);
        
        // --- Dirty Rectangle Calculation ---
        // Calculate the bounding box for the *current* trail state.
        RECT currentRect = { LONG_MAX, LONG_MAX, LONG_MIN, LONG_MIN };
        for (const auto& pt : CursorState::trailPoints) {
            currentRect.left   = std::min(currentRect.left,   (LONG)pt.pos.X);
            currentRect.top    = std::min(currentRect.top,    (LONG)pt.pos.Y);
            currentRect.right  = std::max(currentRect.right,  (LONG)pt.pos.X);
            currentRect.bottom = std::max(currentRect.bottom, (LONG)pt.pos.Y);
        }

        // Calculate the bounding box for the *previous* trail state.
        RECT prevRect = { LONG_MAX, LONG_MAX, LONG_MIN, LONG_MIN };
        for (const auto& pt : CursorState::prevTrailPoints) {
            prevRect.left   = std::min(prevRect.left,   (LONG)pt.pos.X);
            prevRect.top    = std::min(prevRect.top,    (LONG)pt.pos.Y);
            prevRect.right  = std::max(prevRect.right,  (LONG)pt.pos.X);
            prevRect.bottom = std::max(prevRect.bottom, (LONG)pt.pos.Y);
        }

        // If ripples are enabled, expand the dirty rectangle to include their area of effect.
        if (CursorState::settings.rippleEffect.enabled) {
            auto now = std::chrono::steady_clock::now();
            std::lock_guard<std::mutex> lock(CursorState::ripplesMutex);

            for (const auto& ripple : CursorState::ripples) {
                // Calculate the ripple's current animation progress.
                auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - ripple.startTime).count();
                float progress = std::clamp((float)elapsed / CursorState::settings.rippleEffect.duration, 0.0f, 1.0f);

                // Calculate the ripple's current diameter and pen width for this frame.
                // This logic must EXACTLY match the rendering logic in RenderRipples to ensure a tight bounding box.
                float currentDiameter = (float)MulDiv(CursorState::settings.rippleEffect.maxDiameter, dpi, 96) * progress;
                
                // The width shrinks over the animation's lifetime. We must calculate this to get a tight bounding box.
                float pingPongProgress = 1.0f - 2.0f * std::abs(progress - 0.5f); // Triangle wave from 0->1->0
                float animatedFactor = ApplyFadeCurve(1.0f - pingPongProgress, CursorState::settings.fadeMode);
                float currentWidth = (float)MulDiv(CursorState::settings.rippleEffect.startWidth, dpi, 96) * animatedFactor;

                // The total radius is the path radius + half the pen width.
                int currentPadding = (int)(currentDiameter / 2.0f + currentWidth / 2.0f);

                // Create a tight bounding box for this specific ripple's *current* state.
                RECT rippleRect = {
                    ripple.pos.x - currentPadding,
                    ripple.pos.y - currentPadding,
                    ripple.pos.x + currentPadding,
                    ripple.pos.y + currentPadding
                };
                // Union this ripple's bounding box with the main dirty rect.
                UnionRect(&currentRect, &currentRect, &rippleRect);
            }
        }

        // Expand dirty rect to include satellites.
        if (CursorState::settings.satelliteEffect.enabled) {
            POINT mousePos;
            GetCursorPos(&mousePos);
            int satellitePadding = MulDiv(CursorState::settings.satelliteEffect.ringDiameter / 2, dpi, 96) + MulDiv(CursorState::settings.satelliteEffect.satelliteDiameter / 2, dpi, 96);
            RECT satelliteRect = {
                mousePos.x - satellitePadding,
                mousePos.y - satellitePadding,
                mousePos.x + satellitePadding,
                mousePos.y + satellitePadding
            };
            UnionRect(&currentRect, &currentRect, &satelliteRect);
        }

        // Combine the current and previous bounding boxes to get the total area that needs redrawing.
        RECT totalDirtyRect; // This is the final "dirty" area for this frame.
        UnionRect(&totalDirtyRect, &currentRect, &prevRect);

        // Add padding to prevent clipping at the edges when the trail is thick or moving fast.
        InflateRect(&totalDirtyRect, padding, padding);

        SIZE bmpSize = { totalDirtyRect.right - totalDirtyRect.left,
                         totalDirtyRect.bottom - totalDirtyRect.top };
        if (bmpSize.cx <= 0 || bmpSize.cy <= 0) {
            // Before exiting, save the current trail state for the next frame's calculation.
            CursorState::prevTrailPoints = CursorState::trailPoints;
            return 0;
        }

        // --- FPS Counter Logic ---
        // This block calculates the frames per second based on how many frames were rendered
        // since the last time the counter was updated.
        FpsTracker::frameCount++;
        auto now = std::chrono::steady_clock::now();
        auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - FpsTracker::lastFpsTime).count();
        if (elapsed >= CursorState::settings.fpsCounter.refreshRate) {
            // Calculate FPS by scaling the frame count over the elapsed time.
            // This correctly calculates frames per *second*, not just frames per refresh interval.
            // For example, if 60 frames are counted over 500ms, the FPS is (60 * 1000) / 500 = 120.
            // We add a small epsilon to elapsed to avoid division by zero if the interval is extremely short.
            FpsTracker::lastFps = static_cast<int>((FpsTracker::frameCount.load() * 1000.0) / (elapsed + 1e-9));
            if (CursorState::settings.fpsCounter.logFps) {
                Wh_Log(L"FPS: %d (calculated over %lld ms interval)", FpsTracker::lastFps.load(), elapsed);
            }
            FpsTracker::frameCount = 0; // Reset after calculation
            FpsTracker::lastFpsTime = now; // Reset timer
        }

        // --- Double Buffering ---
        // We draw everything to an off-screen bitmap in memory first, and then
        // copy the finished image to the screen all at once, which prevents flickering.
        HDC hdcScreen = GetDC(hWnd);
        HDC hdcMemory = CreateCompatibleDC(hdcScreen);
        HBITMAP hBitmap = CreateCompatibleBitmap(hdcScreen, bmpSize.cx, bmpSize.cy);
        
        if (hBitmap) {
            HGDIOBJ hOldBitmap = SelectObject(hdcMemory, hBitmap);
            
            // Create a GDI+ Graphics object from our in-memory device context.
            // Set high-quality rendering modes for smooth, anti-aliased output.
            Graphics memoryGraphics(hdcMemory);
            memoryGraphics.SetSmoothingMode(SmoothingModeAntiAlias);
            memoryGraphics.SetCompositingQuality(CompositingQualityHighQuality);
            memoryGraphics.SetPixelOffsetMode(PixelOffsetModeHighQuality);

            // Render all enabled layers, from back to front (layer 1 is bottom, layer 4 is top).
            RenderUltimateTrail(memoryGraphics, totalDirtyRect, CursorState::settings.layer1);
            RenderUltimateTrail(memoryGraphics, totalDirtyRect, CursorState::settings.layer2);
            RenderUltimateTrail(memoryGraphics, totalDirtyRect, CursorState::settings.layer3);
            RenderUltimateTrail(memoryGraphics, totalDirtyRect, CursorState::settings.layer4);
            
            // Render the cursor head on top of all trail layers.
            RenderSquishyCircle(memoryGraphics, totalDirtyRect);

            // Render ripples on top of the trail and cursor head.
            RenderRipples(memoryGraphics, totalDirtyRect);

            // Render the keystroke overlay if enabled.
            RenderKeystrokeOverlay(memoryGraphics, bmpSize, totalDirtyRect);

            RenderSatellites(memoryGraphics, totalDirtyRect);

            // Render FPS counter if enabled
            if (CursorState::settings.fpsCounter.enabled) {
                FontFamily fontFamily(L"Arial");
                Font font(&fontFamily, 16, FontStyleRegular, UnitPixel);
                SolidBrush textBrush(Color(255, 255, 255, 0)); // Yellow text
                SolidBrush shadowBrush(Color(200, 0, 0, 0)); // Black shadow

                std::wstring fpsString = L"FPS: " + std::to_wstring(FpsTracker::lastFps.load());

                // Define the layout rectangle for the FPS counter. Its size is set to the
                // dimensions of the current off-screen bitmap (the "dirty rectangle"), so the text
                // will be aligned relative to the corners of the area being redrawn.
                RectF layoutRect(
                    0.0f, 0.0f,
                    (REAL)bmpSize.cx, (REAL)bmpSize.cy
                );
                StringFormat stringFormat;

                if (!CursorState::settings.fpsCounter.alignBottom) { // Top
                    stringFormat.SetLineAlignment(StringAlignmentNear);
                } else { // Bottom
                    stringFormat.SetLineAlignment(StringAlignmentFar);
                }
                // If centering is enabled, override horizontal alignment to center.
                if (CursorState::settings.layout.centerToCursorX) {
                    POINT mousePos;
                    GetCursorPos(&mousePos);
                    // Adjust the X position of the layout rectangle to be centered on the cursor's X coordinate.
                    layoutRect.X = (float)mousePos.x - totalDirtyRect.left - layoutRect.Width / 2.0f;
                    stringFormat.SetAlignment(StringAlignmentCenter);
                } else {
                    // Standard left/right alignment
                    stringFormat.SetAlignment(CursorState::settings.fpsCounter.alignRight ? StringAlignmentFar : StringAlignmentNear);
                }


                // Draw shadow first for better readability
                memoryGraphics.DrawString(fpsString.c_str(), -1, &font, layoutRect, &stringFormat, &shadowBrush);
                // Draw main text
                memoryGraphics.DrawString(fpsString.c_str(), -1, &font, layoutRect, &stringFormat, &textBrush);
            }

            if (CursorState::settings.debugOverlay.enabled) {
                Pen debugPen(Color(255, 255, 0, 0), 1.0f);
                memoryGraphics.DrawRectangle(&debugPen, 0, 0, bmpSize.cx - 1, bmpSize.cy - 1);
            }
            // --- Update Layered Window ---
            // Copy the contents of our in-memory bitmap to the layered window on screen.
            // This function handles the transparency and positioning automatically.
            POINT ptSrc = { 0, 0 };
            POINT ptDst = { totalDirtyRect.left, totalDirtyRect.top };
            BLENDFUNCTION blend = { AC_SRC_OVER, 0, 255, AC_SRC_ALPHA };
            UpdateLayeredWindow(hWnd, hdcScreen, &ptDst, &bmpSize, hdcMemory, &ptSrc, 0, &blend, ULW_ALPHA);

            // Clean up GDI objects.
            SelectObject(hdcMemory, hOldBitmap);
            DeleteObject(hBitmap);
        }

        DeleteDC(hdcMemory);
        ReleaseDC(hWnd, hdcScreen);

        // Save the current trail state for the next frame's dirty rect calculation.
        CursorState::prevTrailPoints = CursorState::trailPoints;
        return 0;
    }
    return DefWindowProc(hWnd, message, wParam, lParam);
}

/**
 * @brief Renders the currently pressed keys on the screen.
 *
 * This function draws the `displayString` (e.g., "Ctrl + S") onto the graphics context.
 * The position is determined by the layout settings, often positioned opposite to the
 * FPS counter for clear visual separation.
 * It also supports being horizontally centered relative to the cursor.
 *
 * @param g A reference to the GDI+ Graphics object to draw on.
 * @param bmpSize The size of the bitmap being drawn to, used for alignment.
 */
void RenderKeystrokeOverlay(Graphics& g, const SIZE& bmpSize, const RECT& totalDirtyRect) {
    if (!KeystrokeOverlaySettings::enabled) return;

    // Lock the mutex to safely access shared keystroke data.
    std::lock_guard<std::mutex> lock(KeystrokeDisplay::keyMutex);

    // Only render if there are keys currently pressed.
    if (!KeystrokeDisplay::pressedKeys.empty()) {
        FontFamily fontFamily(L"Arial");
        Font font(&fontFamily, 24, FontStyleBold, UnitPixel);
        SolidBrush textBrush(Color(255, 255, 255, 255)); // White text
        SolidBrush shadowBrush(Color(200, 0, 0, 0));     // Black shadow

        RectF layoutRect(0.0f, 0.0f, (REAL)bmpSize.cx, (REAL)bmpSize.cy);
        StringFormat stringFormat;

        // --- Horizontal Alignment ---
        if (CursorState::settings.layout.centerToCursorX) {
            // If centering is on, both overlays are centered horizontally relative to the cursor.
            POINT mousePos;
            GetCursorPos(&mousePos);
            layoutRect.X = (float)mousePos.x - totalDirtyRect.left - layoutRect.Width / 2.0f;
            stringFormat.SetAlignment(StringAlignmentCenter);
        } else {
            // Standard positioning: opposite the FPS counter, or default to center if FPS is off.
            if (CursorState::settings.fpsCounter.enabled) {
                stringFormat.SetAlignment(CursorState::settings.fpsCounter.alignRight ? StringAlignmentNear : StringAlignmentFar);
            } else {
                stringFormat.SetAlignment(StringAlignmentCenter); // Default to horizontal center
            }
        }

        // --- Vertical Alignment ---
        // Position is always opposite the FPS counter, or defaults to bottom if the counter is disabled.
        if (CursorState::settings.fpsCounter.enabled) {
            // Position vertically opposite to the FPS counter.
            stringFormat.SetLineAlignment(CursorState::settings.fpsCounter.alignBottom ? StringAlignmentNear : StringAlignmentFar);
        } else {
            // Default to bottom alignment when FPS counter is disabled.
            stringFormat.SetLineAlignment(StringAlignmentFar);
        }

        // Draw shadow first for a nice outline effect
        g.DrawString(KeystrokeDisplay::displayString.c_str(), -1, &font, layoutRect, &stringFormat, &shadowBrush);
        // Draw the main text
        g.DrawString(KeystrokeDisplay::displayString.c_str(), -1, &font, layoutRect, &stringFormat, &textBrush);
    }
}

/**
 * @brief Provides a more readable name for special virtual key codes.
 *
 * This function maps common non-printable virtual key codes (like modifiers, function keys, etc.)
 * to user-friendly strings. For other keys, it falls back to the system's `GetKeyNameTextW`.
 *
 * @param vkCode The virtual key code to translate.
 * @return A std::wstring with the friendly name of the key.
 */
std::wstring KeystrokeDisplay::GetKeyName(DWORD vkCode) {
    // Handle common modifiers and special keys with user-friendly names.
    switch (vkCode) {
        case VK_SHIFT:
        case VK_LSHIFT:
        case VK_RSHIFT:
            return L"Shift";
        case VK_CONTROL:
        case VK_LCONTROL:
        case VK_RCONTROL:
            return L"Ctrl";
        case VK_MENU:
        case VK_LMENU:
        case VK_RMENU:
            return L"Alt";
        case VK_LWIN:
        case VK_RWIN:
            return L"Win";
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
            // For other keys, use the system function to get their name.
            UINT scanCode = MapVirtualKey(vkCode, MAPVK_VK_TO_VSC);
            LONG lParam = scanCode << 16;
            WCHAR keyName[256];
            if (GetKeyNameTextW(lParam, keyName, sizeof(keyName) / sizeof(WCHAR)) > 0) {
                return keyName;
            }
            // Fallback for keys GetKeyNameTextW doesn't handle well.
            return L"";
    }
}

/**
 * @brief Updates the string that shows the currently pressed keys.
 *
 * This function is called whenever a key is pressed or released. It constructs a string
 * by combining the names of all keys currently in the `pressedKeys` set, separated by " + ".
 * The order is: Modifiers (Ctrl, Alt, Shift, Win), then any other pressed keys.
 */
void UpdateKeystrokeDisplayString() { // Called within a mutex lock.
    std::vector<std::wstring> parts;
    // Define the order of modifiers for consistent display.
    const std::vector<DWORD> modifiers = {VK_CONTROL, VK_MENU, VK_SHIFT, VK_LWIN, VK_RWIN};

    // Add modifier names first if they are pressed.
    for (DWORD mod_vk : modifiers) {
        if (KeystrokeDisplay::pressedKeys.count(mod_vk) ||
            (mod_vk == VK_CONTROL && (KeystrokeDisplay::pressedKeys.count(VK_LCONTROL) || KeystrokeDisplay::pressedKeys.count(VK_RCONTROL))) ||
            (mod_vk == VK_MENU && (KeystrokeDisplay::pressedKeys.count(VK_LMENU) || KeystrokeDisplay::pressedKeys.count(VK_RMENU))) ||
            (mod_vk == VK_SHIFT && (KeystrokeDisplay::pressedKeys.count(VK_LSHIFT) || KeystrokeDisplay::pressedKeys.count(VK_RSHIFT)))) {
            parts.push_back(KeystrokeDisplay::GetKeyName(mod_vk));
        }
    }

    // Add other (non-modifier) keys.
    for (DWORD vkCode : KeystrokeDisplay::pressedKeys) {
        bool isModifier = (vkCode >= VK_LSHIFT && vkCode <= VK_RMENU) || vkCode == VK_LWIN || vkCode == VK_RWIN;
        if (!isModifier) {
            std::wstring name = KeystrokeDisplay::GetKeyName(vkCode);
            if (!name.empty()) {
                parts.push_back(name);
            }
        }
    }

    // Join the parts with " + " to form the final display string.
    KeystrokeDisplay::displayString.clear();
    for (size_t i = 0; i < parts.size(); ++i) {
        KeystrokeDisplay::displayString += parts[i];
        if (i < parts.size() - 1) {
            KeystrokeDisplay::displayString += L" + ";
        }
    }
}

/**
 * @brief Converts a hexadecimal color string (e.g., L"#RRGGBB") to a GDI+ Color.
 *
 * @param hex The wide string containing the hex code.
 * @return A GDI+ Color object. Returns opaque white on parsing failure.
 */
Color ColorFromHexString(const std::wstring& hex) {
    if (hex.empty() || hex.length() < 2 || hex[0] != L'#') {
        return Color(255, 255, 255, 255);
    }
    try {
        if (hex.length() == 7) {
            long r = std::stol(hex.substr(1, 2), nullptr, 16);
            long g = std::stol(hex.substr(3, 2), nullptr, 16);
            long b = std::stol(hex.substr(5, 2), nullptr, 16);
            return Color(255, (BYTE)r, (BYTE)g, (BYTE)b);
        } else if (hex.length() == 9) {
            long a = std::stol(hex.substr(1, 2), nullptr, 16);
            long r = std::stol(hex.substr(3, 2), nullptr, 16);
            long g = std::stol(hex.substr(5, 2), nullptr, 16);
            long b = std::stol(hex.substr(7, 2), nullptr, 16);
            return Color((BYTE)a, (BYTE)r, (BYTE)g, (BYTE)b);
        }
        return Color(255, 255, 255, 255);
    } catch (...) {
        return Color(255, 255, 255, 255);
    }
}

/**
 * @brief Gets the DPI for a given window in a safe manner.
 *
 * It first tries to use the modern `GetDpiForWindow` function. If that's not
 * available on the system (e.g., older versions of Windows), it falls back to the
 * older `GetDeviceCaps` method. This ensures DPI awareness and correct scaling on a wide range of systems.
 *
 * @param hWnd The handle to the window.
 * @return The DPI value (e.g., 96 for 100% scaling, 144 for 150%).
 */
UINT GetDpiForWindowSafe(HWND hWnd) {
    UINT dpi = 96; // Default DPI for 100% scaling.
    // Dynamically load GetDpiForWindow for compatibility with older Windows versions.
    static auto pGetDpiForWindow = (UINT(WINAPI*)(HWND))GetProcAddress(
        GetModuleHandleW(L"user32.dll"), "GetDpiForWindow");
    if (pGetDpiForWindow) {
        dpi = pGetDpiForWindow(hWnd);
    } else {
        // Fallback method for older systems.
        HDC hdc = GetDC(nullptr);
        if (hdc) {
            dpi = GetDeviceCaps(hdc, LOGPIXELSX);
            ReleaseDC(nullptr, hdc);
        }
    }
    return dpi;
}

/**
 * @brief Loads all settings from the Windhawk UI into the global `CursorState::settings` struct.
 *
 * This function is called on mod initialization and whenever the user changes a setting in the UI.
 * It reads each value, clamps it to a valid range where necessary to prevent errors, and converts it
 * to the appropriate type for use in the physics and rendering calculations (e.g., converting
 * percentage-based integers from the UI to floating-point multipliers from 0.0 to 1.0).
 */
void LoadSettings() {
    // --- Physics Settings ---
    CursorState::settings.spring = (float)Wh_GetIntSetting(L"spring");
    CursorState::settings.friction = (float)Wh_GetIntSetting(L"friction");
    CursorState::settings.headSpring = (float)Wh_GetIntSetting(L"headSpring");
    CursorState::settings.headFriction = (float)Wh_GetIntSetting(L"headFriction");
    CursorState::settings.trailLength = std::clamp(Wh_GetIntSetting(L"trailLength"), 10, 500);
    CursorState::settings.positionHistorySkip = std::clamp(Wh_GetIntSetting(L"positionHistorySkip"), 0, 100);
    
    // --- Appearance Settings ---
    CursorState::settings.cursorSize = std::clamp(Wh_GetIntSetting(L"cursorSize"), 5, 100);
    CursorState::settings.minTrailWidth = std::clamp(Wh_GetIntSetting(L"minTrailWidth"), 1, 20);
    CursorState::settings.squishIntensity = (float)Wh_GetIntSetting(L"squishIntensity");
    CursorState::settings.squishSmoothing = (float)Wh_GetIntSetting(L"squishSmoothing");
    // The UI setting is an integer, so we divide by 10.0 to get a float value (e.g., a UI setting of 20 becomes a 2.0 multiplier).
    CursorState::settings.velocityWidthMultiplier = (float)Wh_GetIntSetting(L"velocityWidthMultiplier") / 10.0f;
    CursorState::settings.velocityAlphaMultiplier = (float)Wh_GetIntSetting(L"velocityAlphaMultiplier") / 10.0f;
    
    // --- Quality & General Settings ---
    CursorState::settings.interpolationSteps = std::clamp(Wh_GetIntSetting(L"interpolationSteps"), 1, 10);
    CursorState::settings.fadeMode = std::clamp(Wh_GetIntSetting(L"fadeMode"), 0, 3);
    CursorState::settings.adaptiveQuality = Wh_GetIntSetting(L"adaptiveQuality");
    CursorState::settings.enableGradient = Wh_GetIntSetting(L"enableGradient");
    CursorState::settings.hideSystemCursor = Wh_GetIntSetting(L"hideSystemCursor");

    // --- Layer 1 Settings ---
    CursorState::settings.layer1.enabled = Wh_GetIntSetting(L"layer1.enabled");
    CursorState::settings.layer1.color = WindhawkUtils::StringSetting::make(L"layer1.color");
    CursorState::settings.layer1.endColor = WindhawkUtils::StringSetting::make(L"layer1.endColor");
    // Convert percentage values (0-100) to float multipliers (0.0-1.0).
    CursorState::settings.layer1.widthFactor = (float)Wh_GetIntSetting(L"layer1.widthFactor") / 100.0f;
    CursorState::settings.layer1.alphaFactor = (float)Wh_GetIntSetting(L"layer1.alphaFactor") / 100.0f;

    // --- Layer 2 Settings ---
    CursorState::settings.layer2.enabled = Wh_GetIntSetting(L"layer2.enabled");
    CursorState::settings.layer2.color = WindhawkUtils::StringSetting::make(L"layer2.color");
    CursorState::settings.layer2.endColor = WindhawkUtils::StringSetting::make(L"layer2.endColor");
    // Convert percentage values (0-100) to float multipliers (0.0-1.0).
    CursorState::settings.layer2.widthFactor = (float)Wh_GetIntSetting(L"layer2.widthFactor") / 100.0f;
    CursorState::settings.layer2.alphaFactor = (float)Wh_GetIntSetting(L"layer2.alphaFactor") / 100.0f;

    // --- Layer 3 Settings ---
    CursorState::settings.layer3.enabled = Wh_GetIntSetting(L"layer3.enabled");
    CursorState::settings.layer3.color = WindhawkUtils::StringSetting::make(L"layer3.color");
    CursorState::settings.layer3.endColor = WindhawkUtils::StringSetting::make(L"layer3.endColor");
    // Convert percentage values (0-100) to float multipliers (0.0-1.0).
    CursorState::settings.layer3.widthFactor = (float)Wh_GetIntSetting(L"layer3.widthFactor") / 100.0f;
    CursorState::settings.layer3.alphaFactor = (float)Wh_GetIntSetting(L"layer3.alphaFactor") / 100.0f;

    // --- Layer 4 Settings ---
    CursorState::settings.layer4.enabled = Wh_GetIntSetting(L"layer4.enabled");
    CursorState::settings.layer4.color = WindhawkUtils::StringSetting::make(L"layer4.color");
    CursorState::settings.layer4.endColor = WindhawkUtils::StringSetting::make(L"layer4.endColor");
    // Convert percentage values (0-100) to float multipliers (0.0-1.0).
    CursorState::settings.layer4.widthFactor = (float)Wh_GetIntSetting(L"layer4.widthFactor") / 100.0f;
    CursorState::settings.layer4.alphaFactor = (float)Wh_GetIntSetting(L"layer4.alphaFactor") / 100.0f;

    // --- Cursor Head Settings ---
    CursorState::settings.cursorHead.enabled = Wh_GetIntSetting(L"cursorHead.enabled");
    CursorState::settings.cursorHead.filled = Wh_GetIntSetting(L"cursorHead.filled");
    CursorState::settings.cursorHead.color = WindhawkUtils::StringSetting::make(L"cursorHead.color");
    CursorState::settings.cursorHead.size = std::clamp(Wh_GetIntSetting(L"cursorHead.size"), 5, 50);
    CursorState::settings.cursorHead.outlineWidth = std::clamp(Wh_GetIntSetting(L"cursorHead.outlineWidth"), 1, 10);
    CursorState::settings.cursorHead.alpha = (float)Wh_GetIntSetting(L"cursorHead.alpha");
    
    // --- Ripple Effect Settings ---
    CursorState::settings.rippleEffect.enabled = Wh_GetIntSetting(L"rippleEffect.enabled");
    CursorState::settings.rippleEffect.maxDiameter = std::clamp(Wh_GetIntSetting(L"rippleEffect.maxDiameter"), 10, 500);
    CursorState::settings.rippleEffect.startWidth = std::clamp(Wh_GetIntSetting(L"rippleEffect.startWidth"), 1, 100);
    CursorState::settings.rippleEffect.duration = std::clamp(Wh_GetIntSetting(L"rippleEffect.duration"), 100, 2000);
    CursorState::settings.rippleEffect.leftClickColor = WindhawkUtils::StringSetting::make(L"rippleEffect.leftClickColor");
    CursorState::settings.rippleEffect.rightClickColor = WindhawkUtils::StringSetting::make(L"rippleEffect.rightClickColor");
    CursorState::settings.rippleEffect.middleClickColor = WindhawkUtils::StringSetting::make(L"rippleEffect.middleClickColor");    
    CursorState::settings.rippleEffect.clickScaleFactor = std::clamp(Wh_GetIntSetting(L"rippleEffect.clickScaleFactor") , 1, 500);
    CursorState::settings.rippleEffect.clickScaleDuration = std::clamp(Wh_GetIntSetting(L"rippleEffect.clickScaleDuration"), 50, 500);    
    CursorState::settings.rippleEffect.enableClickScaling = Wh_GetIntSetting(L"rippleEffect.enableClickScaling");

    // --- Satellite Effect Settings ---
    CursorState::settings.satelliteEffect.enabled = Wh_GetIntSetting(L"satelliteEffect.enabled");
    CursorState::settings.satelliteEffect.ringDiameter = std::clamp(Wh_GetIntSetting(L"satelliteEffect.ringDiameter"), 10, 1000);
    CursorState::settings.satelliteEffect.ringVisible = Wh_GetIntSetting(L"satelliteEffect.ringVisible");
    CursorState::settings.satelliteEffect.ringWidth = std::clamp(Wh_GetIntSetting(L"satelliteEffect.ringWidth"), 1, 50);
    CursorState::settings.satelliteEffect.ringColor = WindhawkUtils::StringSetting::make(L"satelliteEffect.ringColor");
    CursorState::settings.satelliteEffect.satelliteCount = std::clamp(Wh_GetIntSetting(L"satelliteEffect.satelliteCount"), 1, 50);
    CursorState::settings.satelliteEffect.satelliteDiameter = std::clamp(Wh_GetIntSetting(L"satelliteEffect.satelliteDiameter"), 1, 100);
    CursorState::settings.satelliteEffect.satelliteWidth = std::clamp(Wh_GetIntSetting(L"satelliteEffect.satelliteWidth"), 1, 50);
    CursorState::settings.satelliteEffect.satelliteFilled = Wh_GetIntSetting(L"satelliteEffect.satelliteFilled");
    CursorState::settings.satelliteEffect.satelliteColor = WindhawkUtils::StringSetting::make(L"satelliteEffect.satelliteColor");
    CursorState::settings.satelliteEffect.rotationSpeed = (float)Wh_GetIntSetting(L"satelliteEffect.rotationSpeed");
    CursorState::settings.satelliteEffect.reverseRotation = Wh_GetIntSetting(L"satelliteEffect.reverseRotation");
    CursorState::settings.satelliteEffect.enableDualRing = Wh_GetIntSetting(L"satelliteEffect.enableDualRing");
    CursorState::settings.satelliteEffect.dualRingRotationSpeed = (float)Wh_GetIntSetting(L"satelliteEffect.dualRingRotationSpeed");

    // --- FPS Counter Settings ---
    CursorState::settings.fpsCounter.enabled = Wh_GetIntSetting(L"fpsCounter.enabled");
    CursorState::settings.fpsCounter.logFps = Wh_GetIntSetting(L"fpsCounter.logFps");
    CursorState::settings.fpsCounter.alignBottom = Wh_GetIntSetting(L"fpsCounter.alignBottom");
    CursorState::settings.fpsCounter.alignRight = Wh_GetIntSetting(L"fpsCounter.alignRight");
    CursorState::settings.fpsCounter.refreshRate = Wh_GetIntSetting(L"fpsCounter.refreshRate");

    // --- Debug Overlay Settings ---
    CursorState::settings.debugOverlay.enabled = Wh_GetIntSetting(L"debugOverlay.enabled");

    // --- Layout Settings ---
    CursorState::settings.layout.centerToCursorX = Wh_GetIntSetting(L"layout.centerToCursorX");

    // --- Keystroke Overlay Settings ---
    KeystrokeOverlaySettings::enabled = Wh_GetIntSetting(L"keystrokeOverlay.enabled");


    if (CursorState::settings.debugOverlay.enabled) {
        // Force a full redraw when enabling the debug overlay to clear any artifacts.
        InvalidateRect(CursorState::hWndCursor, nullptr, TRUE);
    }
    // If the trail length has changed, resize the trailPoints vector to match.
    // This reinitializes the trail at the current cursor position to prevent visual glitches.
    if (CursorState::trailPoints.size() != (size_t)CursorState::settings.trailLength) {
        POINT pt;
        GetCursorPos(&pt);
        CursorState::trailPoints.assign(CursorState::settings.trailLength,
            {{ (float)pt.x, (float)pt.y }, {0.0f, 0.0f}, 0.0f, 0.0f});
        CursorState::lastUsedMousePos = pt;
        CursorState::prevTrailPoints = CursorState::trailPoints; // Also reset previous points
    }

    // If the satellite count has changed, resize and reinitialize the satellites.
    if (CursorState::satellites.size() != (size_t)CursorState::settings.satelliteEffect.satelliteCount) {
        CursorState::satellites.resize(CursorState::settings.satelliteEffect.satelliteCount);
        float angleIncrement = 2.0f * M_PI / CursorState::settings.satelliteEffect.satelliteCount;
        for (size_t i = 0; i < CursorState::satellites.size(); ++i) {
            CursorState::satellites[i].currentAngle = i * angleIncrement;
            CursorState::satellites[i].mirroredAngle = i * angleIncrement;
        }
    }
}

/**
 * @brief Hides or shows the default system cursor.
 *
 * This function replaces the standard system cursor (`OCR_NORMAL`, the default arrow)
 * with a custom icon. To hide it, a fully transparent 1x1 pixel icon is created
 * programmatically and set as the new system cursor. To show it, the original cursor,
 * which was saved beforehand, is restored. This is a common technique for replacing
 * the system cursor without complex API hooking.
 *
 * @param visible If `true`, restores the original system cursor. If `false`, hides it by replacing it with a transparent one.
 */
void SetSystemCursorVisibility(bool visible) {
    if (!visible) {
        // --- Hide the cursor ---
        if (CursorState::hOriginalCursor) return; // Already hidden.

        // Create a 1x1 transparent icon to replace the system cursor.
        // This is a standard technique for hiding the cursor without needing complex API hooking.
        HMODULE hModule = nullptr;
        GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS |
                           GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
                           (LPCWSTR)&SetSystemCursorVisibility, &hModule);
        // Create a 1x1 pixel monochrome icon. The visibility is controlled by two bitmasks:
        // - The AND mask (andMask) determines transparency. 0xFF means the background shows through (transparent).
        // - The XOR mask (xorMask) determines the pixel color. 0x00 means black (but is irrelevant here due to transparency).
        // The result is a 1x1 transparent pixel, which is effectively an invisible icon.
        BYTE andMask[] = { 0xFF };
        BYTE xorMask[] = { 0x00 };
        HICON hInvisibleIcon = CreateIcon(hModule, 1, 1, 1, 1, andMask, xorMask);
        
        if (hInvisibleIcon) {
            // Save a copy of the current cursor so we can restore it later.
            // CopyCursor is important because the system owns the handle returned by LoadCursor.
            CursorState::hOriginalCursor = CopyCursor(LoadCursor(nullptr, IDC_ARROW));
            // Set the system cursor to our invisible icon. We must pass a copy of the icon handle
            // because the system takes ownership of it and will destroy it later.
            if (!SetSystemCursor(CopyIcon(hInvisibleIcon), OCR_NORMAL)) {
                 Wh_Log(L"Failed to set system cursor to invisible.");
                 if(CursorState::hOriginalCursor) DestroyIcon(CursorState::hOriginalCursor);
                 CursorState::hOriginalCursor = nullptr;
            }
            DestroyIcon(hInvisibleIcon);
        }
    } else {
        // --- Show the cursor ---
        if (CursorState::hOriginalCursor) {
            // Restore the cursor we saved earlier.
            SetSystemCursor(CursorState::hOriginalCursor, OCR_NORMAL);
            CursorState::hOriginalCursor = nullptr;
        } else {
            // If we don't have a saved cursor for some reason, just tell Windows to reload the default cursor scheme.
            SystemParametersInfo(SPI_SETCURSORS, 0, nullptr, 0);
        }
    }
}

/** 
 * @brief The callback procedure for the low-level mouse hook (`WH_MOUSE_LL`).
 *
 * This function is called by the system on the dedicated hook thread whenever a mouse event occurs system-wide.
 * It checks if the event is a left or right mouse button down event. If it is,
 * and the corresponding effect is enabled, it creates a new `Ripple` object and adds it to the
 * thread-safe global `ripples` vector for the rendering thread to draw.
 *
 * @param nCode A code the hook procedure uses to determine how to process the message. Must be `HC_ACTION`.
 * @param wParam The identifier of the mouse message (e.g., `WM_LBUTTONDOWN`).
 * @param lParam A pointer to an `MSLLHOOKSTRUCT` structure containing details about the event, like cursor position.
 * @return The return value of `CallNextHookEx`, which is crucial for passing the event to the next hook in the chain.
 */
LRESULT CALLBACK LowLevelMouseProc(int nCode, WPARAM wParam, LPARAM lParam) {
    // If click scaling is enabled, trigger it on any mouse down event.
    if (nCode == HC_ACTION && CursorState::settings.rippleEffect.enableClickScaling && (wParam == WM_LBUTTONDOWN || wParam == WM_RBUTTONDOWN || wParam == WM_MBUTTONDOWN)) {
        ScaleAndSetCursor(CursorState::settings.rippleEffect.clickScaleFactor);
    }

    // HC_ACTION means we should process this message.
    if (nCode == HC_ACTION) {
        // Check for left, right, or middle mouse button down events.
        if (wParam == WM_LBUTTONDOWN || wParam == WM_RBUTTONDOWN || wParam == WM_MBUTTONDOWN) {
            if (CursorState::settings.rippleEffect.enabled) {
                // The lParam for a WH_MOUSE_LL hook is a pointer to an MSLLHOOKSTRUCT.
                // This struct contains detailed information about the mouse event, including its screen coordinates.
                MSLLHOOKSTRUCT* p = (MSLLHOOKSTRUCT*)lParam;

                // Log the click type and position to the Windhawk debug console.
                Wh_Log(L"Mouse %s click detected at (%d, %d)",
                       (wParam == WM_LBUTTONDOWN ? L"Left" : (wParam == WM_RBUTTONDOWN ? L"Right" : L"Middle")),
                       p->pt.x, p->pt.y);
                
                // Create a new ripple and configure it.
                Ripple newRipple;
                newRipple.pos = p->pt;
                newRipple.startTime = std::chrono::steady_clock::now();
                if (wParam == WM_LBUTTONDOWN) {
                    newRipple.color = ColorFromHexString(CursorState::settings.rippleEffect.leftClickColor.get());
                } else if (wParam == WM_RBUTTONDOWN) {
                    newRipple.color = ColorFromHexString(CursorState::settings.rippleEffect.rightClickColor.get());
                } else { // WM_MBUTTONDOWN
                    newRipple.color = ColorFromHexString(CursorState::settings.rippleEffect.middleClickColor.get());
                }
                
                // Lock the mutex before modifying the shared ripples vector.
                std::lock_guard<std::mutex> lock(CursorState::ripplesMutex);
                CursorState::ripples.push_back(newRipple);
            }
        }
    }
    
    // VERY IMPORTANT: Always call CallNextHookEx to pass the event to the next hook in the
    // chain. Failing to do so will break mouse input for other applications and hooks.
    return CallNextHookEx(CursorState::hMouseHook, nCode, wParam, lParam);
}

/**
 * @brief Scales the current system cursor and sets it.
 *
 * This function captures the current cursor (arrow, ibeam, etc.), starts the scaling
 * animation timer, and backs up the current system-wide cursor scheme.
 * 
 * ### The Bug and The Fix Explained
 * A common issue with cursor replacement is that only the standard arrow cursor (`OCR_NORMAL`)
 * gets replaced, while other cursors (like the I-beam for text or the hand for links) would not.
 * This happened because previous logic would only call `SetSystemCursor(..., OCR_NORMAL)`.
 *
 * When the mouse is over a textbox, Windows expects the `OCR_IBEAM` cursor. If we scale the
 * I-beam but only set it for the `OCR_NORMAL` slot, Windows will immediately switch back to
 * its default, unscaled `OCR_IBEAM` cursor, making the effect disappear instantly.
 * 
 * The fix involves these key steps, implemented here and in `UpdateAndApplyCursorScaling`:
 * 1.  **Backup All Cursors:** We first save a copy of *all* major system cursors (`kCursorTypesToOverride`)
 *     into the `hOriginalCursors` map. This is our "before" snapshot.
 * 2.  **Override All Cursors:** During the animation, we apply our scaled cursor to *every*
 *     cursor slot we backed up. This temporarily forces the entire system to use
 *     our animated cursor, regardless of what the UI element underneath requests.
 *
 * @param scaleFactor The factor in % by which to scale the cursor (e.g., 150).
 */
void ScaleAndSetCursor(int scaleFactor) {
    // If an animation is already running, just restart the timer. Otherwise, start a new one.
    CursorScaling::scaleStartTime = std::chrono::steady_clock::now();
    CursorScaling::isScalingActive = true;
    CursorScaling::targetScaleFactor = scaleFactor;

    // Step 1 of the fix: If this is a new animation, back up all original system cursors.
    // We only do this if the map is empty to avoid doing it on every frame of the animation.
    if (CursorScaling::hOriginalCursors.empty()) {
        for (UINT cursorId : CursorScaling::kCursorTypesToOverride) {
            // Get a copy of the current system cursor for this type.
            HCURSOR hCurrent = (HCURSOR)LoadImage(nullptr, MAKEINTRESOURCE(cursorId), IMAGE_CURSOR, 0, 0, LR_SHARED);
            if (hCurrent) {
                // Store a copy, as we must not destroy the shared system handle.
                CursorScaling::hOriginalCursors[cursorId] = CopyCursor(hCurrent);
            }
        }
    }

    // Destroy the previously captured cursor to prevent resource leaks.
    if (CursorScaling::hCursorToScale) {
        DestroyCursor(CursorScaling::hCursorToScale);
        CursorScaling::hCursorToScale = nullptr;
    }

    // Now, get the current cursor (which could be an I-beam, hand, etc.) and make a copy.
    // This copy will be the source for our scaling animation.
    CURSORINFO ci = { sizeof(CURSORINFO) };
    if (GetCursorInfo(&ci) && ci.hCursor != nullptr) {
        CursorScaling::hCursorToScale = CopyCursor(ci.hCursor);
    }
}

/**
 * @brief Updates the cursor scaling animation and applies the scaled cursor.
 *
 * This function is called repeatedly by the rendering thread. It calculates the current
 * scale factor based on animation progress, creates a scaled version of the original
 * cursor, and sets it as the system cursor. When the animation finishes, it restores
 * all the original cursors.
 */
void UpdateAndApplyCursorScaling() {
    if (!CursorScaling::isScalingActive) {
        return;
    }

    // ### The Bug and The Fix Explained (Restoration Phase)
    // When the animation finishes, it's crucial to restore the system to its original state.
    // The bug was that only the OCR_NORMAL cursor was being restored.
    //
    // The fix is to iterate through the `hOriginalCursors` map we created at the start
    // and restore each cursor to its proper system slot (e.g., put the original I-beam
    // back into the OCR_IBEAM slot). This ensures the cursor scheme is perfectly reset.

    CURSORINFO ci = { sizeof(CURSORINFO) };
    if (!GetCursorInfo(&ci)) {
        // If we can't get cursor info, something is wrong, stop scaling.
        CursorScaling::isScalingActive = false;
        return;
    }

    auto now = std::chrono::steady_clock::now();
    auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - CursorScaling::scaleStartTime).count();
    float progress = (float)elapsed / (float)CursorState::settings.rippleEffect.clickScaleDuration;

    if (progress >= 1.0f) {
        // Animation finished, restore all original cursors.
        CursorScaling::isScalingActive = false;
        // Step 2 of the fix: Restore ALL original cursors we backed up.
        // SetSystemCursor takes ownership of the handle, so we must pass a fresh copy.
        for (const auto& pair : CursorScaling::hOriginalCursors) {
            SetSystemCursor(CopyCursor(pair.second), pair.first);
            DestroyCursor(pair.second); // Clean up the copies we made at the start.
        }
        CursorScaling::hOriginalCursors.clear();

        // Destroy the temporary cursor we were scaling.
        if (CursorScaling::hCursorToScale) {
            DestroyCursor(CursorScaling::hCursorToScale);
            CursorScaling::hCursorToScale = nullptr;
        }
        return;
    }

    // Calculate current scale factor using a ping-pong curve (grows from 1.0 to max and back to 1.0).
    float pingPong = 1.0f - 2.0f * std::abs(progress - 0.5f);
    float currentScale = 1.0f + (CursorScaling::targetScaleFactor * 0.01f - 1.0f) * pingPong;

    // Ensure we have a base cursor to scale from. This should ideally be set by ScaleAndSetCursor.
    if (!CursorScaling::hCursorToScale) {
        // Fallback: if hCursorToScale is null, try to get the current system cursor.
        // This might happen if the mod starts with scaling active or if there's an unexpected state.
        if (ci.hCursor != nullptr) {
            CursorScaling::hCursorToScale = CopyCursor(ci.hCursor);
        } else {
            // Cannot proceed without a base cursor. Stop scaling.
            CursorScaling::isScalingActive = false;
            return;
        }
    }

    // Get the standard system cursor dimensions. This provides a reliable base size for scaling
    // and avoids using GetIconInfoExW, which is unreliable for some system cursors like the I-beam.
    int baseWidth = GetSystemMetrics(SM_CXCURSOR);
    int baseHeight = GetSystemMetrics(SM_CYCURSOR);

    int newWidth = static_cast<int>(round(baseWidth * currentScale));
    int newHeight = static_cast<int>(round(baseHeight * currentScale));

    // CopyImage correctly scales any GDI cursor handle, not just ones loaded from resources.
    HICON hNewCursor = (HICON)CopyImage(CursorScaling::hCursorToScale, IMAGE_CURSOR, newWidth, newHeight, LR_COPYRETURNORG);
    
    if (hNewCursor) {
        // Step 3 of the fix: Apply the newly scaled cursor to ALL cursor types we are overriding.
        // This is the magic that makes the scaling visible over any UI element.
        for (UINT cursorId : CursorScaling::kCursorTypesToOverride) {
            // We must pass a copy of the icon for each call, as SetSystemCursor takes ownership.
            SetSystemCursor(CopyIcon(hNewCursor), cursorId);
        }
        DestroyIcon(hNewCursor);
    } else {
        // If CopyImage fails, log the error. This might indicate a very unusual cursor.
        Wh_Log(L"CopyImage failed to scale cursor. Error: %lu", GetLastError());
    }
}

// The low-level keyboard hook procedure, called for every keyboard event in the system.
LRESULT CALLBACK LowLevelKeyboardProc(int nCode, WPARAM wParam, LPARAM lParam) {
    // We only process the event if nCode is HC_ACTION.
    if (nCode == HC_ACTION && KeystrokeOverlaySettings::enabled) {
        KBDLLHOOKSTRUCT* pkbhs = reinterpret_cast<KBDLLHOOKSTRUCT*>(lParam);
        const DWORD vkCode = pkbhs->vkCode;
        bool stateChanged = false;

        if (wParam == WM_KEYDOWN || wParam == WM_SYSKEYDOWN) {
            // Check if this is the first time the key is pressed (i.e., not an auto-repeat event).
            // The 30th bit of flags is 1 if the key was already down.
            if ((pkbhs->flags & LLKHF_UP) == 0) { // Key is being pressed down
                std::lock_guard<std::mutex> lock(KeystrokeDisplay::keyMutex);
                // Only consider it a change if the key wasn't already in our set.
                if (KeystrokeDisplay::pressedKeys.find(vkCode) == KeystrokeDisplay::pressedKeys.end()) {
                    KeystrokeDisplay::pressedKeys.insert(vkCode);
                    stateChanged = true;
                }
            }
        } else if (wParam == WM_KEYUP || wParam == WM_SYSKEYUP) {
            std::lock_guard<std::mutex> lock(KeystrokeDisplay::keyMutex);
            if (KeystrokeDisplay::pressedKeys.erase(vkCode) > 0) {
                stateChanged = true;
            }
        }

        if (stateChanged) {
            UpdateKeystrokeDisplayString();
            Wh_Log(L"Keys pressed: %s", KeystrokeDisplay::displayString.c_str());

            // A key state changed, so we must trigger a repaint to show/hide the overlay.
            // We invalidate a fixed rectangle at the bottom-center of the primary monitor
            // where the text is likely to be drawn. This ensures it updates even if the cursor is stationary.
            int screenWidth = GetSystemMetrics(SM_CXSCREEN);
            int screenHeight = GetSystemMetrics(SM_CYSCREEN);
            RECT textRect = { screenWidth / 2 - 200, screenHeight - 100, screenWidth / 2 + 200, screenHeight };
            InvalidateRect(CursorState::hWndCursor, &textRect, FALSE);
        }
    }
    
    // CRITICAL: Always call CallNextHookEx to pass the message to the next hook in the chain.
    // If you don't do this, you will block all keyboard input for the entire system.
    return CallNextHookEx(CursorState::hKeyboardHook, nCode, wParam, lParam);
}
// The following functions are the standard entry points for a Windhawk mod.
// They are called by Windhawk to manage the mod's lifecycle.

/**
 * @brief Called by Windhawk when the mod is first initialized.
 *
 * This function is the main entry point. It sets up global resources like GDI+,
 * loads the initial user settings, and starts the dedicated rendering and hook threads.
 */
BOOL Wh_ModInit() {
    Wh_Log(L"Initializing Custom GDI+ Cursor Trail");
    
    // Initialize the GDI+ library, required for all GDI+ drawing operations.
    GdiplusStartupInput gdiplusStartupInput;
    GdiplusStartup(&CursorState::gdiplusToken, &gdiplusStartupInput, nullptr);

    // Capture the user's default arrow cursor once at startup.
    // This will be our "true original" to restore after animations or on uninit.
    // We use CopyCursor because LoadCursor returns a shared handle that we must not modify or destroy.
    CursorScaling::hOriginalArrowCursor = CopyCursor(LoadCursor(nullptr, IDC_ARROW));
    
    // Load settings from the UI.
    LoadSettings();

    // Hide the system cursor if the setting is enabled at startup.
    if (CursorState::settings.hideSystemCursor) {
        SetSystemCursorVisibility(false);
    }
    // Start the mouse hook thread.
    CursorState::isMouseHookThreadRunning = true;
    CursorState::mouseHookThread = std::thread([]{
        CursorState::mouseHookThreadId = GetCurrentThreadId();
        MouseHookThreadFunc();
    });

    // Start the keyboard hook thread.
    CursorState::isKeyboardHookThreadRunning = true;
    CursorState::keyboardHookThread = std::thread([]{
        CursorState::keyboardHookThreadId = GetCurrentThreadId();
        KeyboardHookThreadFunc();
    });

    // Start the main rendering thread.
    CursorState::cursorThread = std::thread(CursorThreadFunc);
    
    return TRUE;
}


/**
 * @brief Called by Windhawk when the mod is unloaded or disabled.
 *
 * This function is responsible for cleaning up all resources, gracefully stopping
 * the rendering and hook threads, restoring the original system cursor(s),
 * and shutting down GDI+.
 */
void Wh_ModUninit() {
    Wh_Log(L"Uninitializing Custom GDI+ Cursor Trail");
    
    // Stop the rendering thread first.
    CursorState::isThreadRunning = false;
    if (CursorState::cursorThread.joinable()) {
        CursorState::cursorThread.join();
    }
    
    // Cleanly shut down the mouse hook thread by posting a WM_QUIT message to its message loop.
    if (CursorState::isMouseHookThreadRunning) {
        CursorState::isMouseHookThreadRunning = false;
        if (CursorState::mouseHookThreadId != 0) {
            PostThreadMessage(CursorState::mouseHookThreadId, WM_QUIT, 0, 0);
        }
        if (CursorState::mouseHookThread.joinable()) {
            CursorState::mouseHookThread.join();
        }
    }

    // Cleanly shut down the keyboard hook thread, following the same pattern.
    if (CursorState::isKeyboardHookThreadRunning) {
        CursorState::isKeyboardHookThreadRunning = false;
        if (CursorState::keyboardHookThreadId != 0) {
            PostThreadMessage(CursorState::keyboardHookThreadId, WM_QUIT, 0, 0);
        }
        if (CursorState::keyboardHookThread.joinable()) {
            CursorState::keyboardHookThread.join();
        }
    }
    
    // Clean up cursor scaling resources
    // If an animation was active or we still hold copies of the original cursors, restore them.
    for (const auto& pair : CursorScaling::hOriginalCursors) {
        SetSystemCursor(CopyCursor(pair.second), pair.first); // Restore original
        DestroyCursor(pair.second); // Clean up the copy we made
    }
    CursorScaling::hOriginalCursors.clear();
    if (CursorScaling::hCursorToScale) {
        DestroyCursor(CursorScaling::hCursorToScale);
        CursorScaling::hCursorToScale = nullptr;
    }
    if (CursorScaling::hOriginalArrowCursor) {
        DestroyCursor(CursorScaling::hOriginalArrowCursor);
        CursorScaling::hOriginalArrowCursor = nullptr;
    }
    // Restore the original system cursor if it was hidden.
    if (CursorState::hOriginalCursor) {
        SetSystemCursorVisibility(true);
    }
    
    // Clear any remaining ripples to free resources.
    {
        std::lock_guard<std::mutex> lock(CursorState::ripplesMutex);
        CursorState::ripples.clear();
    }

    // Shut down the GDI+ library.
    GdiplusShutdown(CursorState::gdiplusToken);
}

/**
 * @brief Called by Windhawk whenever the user changes a setting in the mod's UI.
 *
 * This function reloads all settings and applies any changes that need immediate
 * effect, such as resizing the trail or toggling the system cursor's visibility.
 */
void Wh_ModSettingsChanged() {
    Wh_Log(L"Settings changed, reloading.");
    
    // Check if the cursor visibility setting has changed.
    bool wasHidingCursor = CursorState::settings.hideSystemCursor;
    LoadSettings(); // Reload all settings into the global state.
    bool isHidingCursor = CursorState::settings.hideSystemCursor;

    // Apply the new visibility setting if it's different from the old one.
    if (wasHidingCursor != isHidingCursor) {
        SetSystemCursorVisibility(!isHidingCursor);
    }

    // If debug overlay was just turned off, we need to invalidate the whole screen
    // to make sure the red rectangles are erased.
    if (!CursorState::settings.debugOverlay.enabled) {
        InvalidateRect(CursorState::hWndCursor, nullptr, TRUE);
    }
}