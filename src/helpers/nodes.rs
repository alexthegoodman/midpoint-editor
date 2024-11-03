#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    // Data Nodes
    State {
        name: String,
        data_type: DataType,
        value: String,    // Serialized value
        persistent: bool, // Should this persist between sessions?
    },
    Array {
        name: String,
        element_type: DataType,
        size: usize,
    },
    Dictionary {
        name: String,
        key_type: DataType,
        value_type: DataType,
    },

    // Control Flow Nodes
    Effect {
        dependencies: Vec<String>,
        execution_order: i32,
        parallel: bool,
    },
    Event {
        event_type: EventType,
        filters: Vec<String>,
        priority: i32,
    },
    Conditional {
        condition_type: ConditionType,
        comparison: ComparisonOp,
        else_branch: bool,
    },
    Loop {
        loop_type: LoopType,
        max_iterations: Option<u32>,
        break_condition: Option<String>,
    },
    Gate {
        gate_type: GateType,
        default_state: bool,
    },
    Sequence {
        steps: Vec<String>,
        auto_progress: bool,
    },

    // Render Nodes
    Render {
        render_type: RenderType,
        layer: i32,
        blend_mode: BlendMode,
        shader: Option<String>,
    },
    Camera {
        projection_type: ProjectionType,
        viewport: Rect,
        priority: i32,
    },
    UI {
        element_type: UIElementType,
        layout: LayoutType,
        style: String, // CSS-like styling
    },

    // Operation Nodes
    MathOp {
        operation: MathOperation,
        precision: NumericPrecision,
    },
    VectorOp {
        operation: VectorOperation,
        dimensions: u8,
    },
    StringOp {
        operation: StringOperation,
        case_sensitive: bool,
    },
    PhysicsOp {
        operation: PhysicsOperation,
        affects_collision: bool,
        // force_type: ForceType,
    },
    AnimationOp {
        animation_type: AnimationType,
        duration: u32,
        easing: EasingFunction,
    },
    AudioOp {
        operation: AudioOperation,
        channel: String,
        spatial: bool,
    },

    // System Nodes
    Behavior {
        behavior_type: BehaviorType,
        update_frequency: UpdateFrequency,
        priority: i32,
    },
    Spawner {
        template: String,
        spawn_rules: SpawnRules,
        max_instances: Option<u32>,
    },
    Collision {
        shape: CollisionShape,
        layer: u32,
        mask: u32,
        trigger: bool,
    },
    Timer {
        duration: u32,
        repeat: bool,
        start_on_create: bool,
    },
    GameState {
        state_type: GameStateType,
        transitions: Vec<String>,
        persistent: bool,
    },
}

// Supporting types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataType {
    Boolean,
    Integer,
    Float,
    String,
    Vector2,
    Vector3,
    Color,
    Entity,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    Input(InputType),
    Collision,
    Timer,
    Custom(String),
    Message,
    Scene,
    Animation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputType {
    Keyboard(String), // Key code
    Mouse(String),
    // Mouse(MouseButton),
    // Gamepad(GamepadButton),
    // Touch(u32), // Touch point index
    // Gesture(GestureType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ConditionType {
    Value,
    State,
    Distance,
    Time,
    Random,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoopType {
    ForEach,
    While,
    Fixed,
    Until,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RenderType {
    Sprite,
    Model,
    Particle,
    Text,
    Shape,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Root,
    Min,
    Max,
    Lerp,
    Clamp,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BehaviorType {
    AI,
    Physics,
    Navigation,
    Animation,
    Interaction,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UpdateFrequency {
    EveryFrame,
    Fixed(u32), // Frames between updates
    OnDemand,
}

// Comparison operations for conditionals
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Contains,
    NotContains,
    Between,
    Matches, // Regex/pattern matching
}

// Types of flow control gates
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GateType {
    And,      // All inputs must be true
    Or,       // Any input must be true
    Xor,      // Exactly one input must be true
    Not,      // Inverts the input
    Latch,    // Maintains state until reset
    Cooldown, // Time-based gate
    Random,   // Random chance to pass
}

// Blend modes for rendering
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    Alpha,
    Custom(String),
}

// Camera projection types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProjectionType {
    Orthographic,
    Perspective,
    Isometric,
    Custom(String),
}

// Rectangle definition for UI and rendering
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub anchor: RectAnchor,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RectAnchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

// UI element types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UIElementType {
    Container,
    Button,
    Text,
    Image,
    Slider,
    Toggle,
    InputField,
    Dropdown,
    ScrollView,
    ProgressBar,
    Panel,
    Custom(String),
}

// Layout types for UI elements
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LayoutType {
    Vertical,
    Horizontal,
    Grid {
        columns: u32,
        spacing: u32,
    },
    Flex {
        wrap: bool,
        justify: FlexJustify,
        align: FlexAlign,
    },
    Absolute,
    Custom(String),
}

// Numeric precision for math operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NumericPrecision {
    Integer,
    Float,
    Double,
    Fixed(u8), // Fixed-point with specified decimal places
}

// Vector operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VectorOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Dot,
    Cross,
    Normalize,
    Length,
    Distance,
    Lerp,
    Reflect,
    Project,
}

// String operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StringOperation {
    Concat,
    Split,
    Replace,
    Substring,
    ToUpper,
    ToLower,
    Trim,
    Contains,
    Format,
    Regex(String),
}

// Physics operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsOperation {
    ApplyForce { continuous: bool },
    ApplyImpulse,
    SetVelocity,
    AddTorque,
    SetPosition,
    SetRotation,
    Raycast,
    SetMass,
    SetFriction,
    SetRestitution,
    Sleep,
    WakeUp,
}

// Animation types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimationType {
    Sprite {
        frames: u32,
        loop_type: AnimationLoop,
    },
    Skeletal {
        blend_type: BlendType,
    },
    Property {
        property: String,
    },
    State {
        transitions: Vec<String>,
    },
    Morph,
    Particle,
}

// Easing functions for animations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EasingFunction {
    Linear,
    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    ElasticIn,
    ElasticOut,
    BounceIn,
    BounceOut,
    Custom(String),
}

// Audio operations
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AudioOperation {
    Play { loop_type: AudioLoop },
    Stop,
    Pause,
    Resume,
    SetVolume,
    SetPitch,
    SetPosition, // For spatial audio
    Fade { duration: u32, target_volume: u32 },
    PlayOneShot,
}

// Spawn rules for entity creation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpawnRules {
    Immediate,
    Interval {
        time: u32,
        variance: u32,
    },
    Wave {
        count: u32,
        delay: u32,
    },
    Triggered {
        event: String,
    },
    Pattern {
        pattern_type: SpawnPattern,
        spacing: u32,
    },
}

// Collision shapes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CollisionShape {
    Box { width: u32, height: u32, depth: u32 },
    Sphere { radius: u32 },
    Capsule { radius: u32, height: u32 },
    Cylinder { radius: u32, height: u32 },
    Mesh { path: String, convex: bool },
    Compound(Vec<CollisionShape>),
}

// Game state types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameStateType {
    Menu,
    Loading,
    Playing,
    Paused,
    GameOver,
    Cutscene,
    Dialog,
    Inventory,
    Combat,
    Custom(String),
}

// Additional supporting types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimationLoop {
    None,
    Loop,
    PingPong,
    ClampForever,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BlendType {
    Override,
    Additive,
    Multiplicative,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AudioLoop {
    None,
    Loop,
    LoopWithIntro { intro_duration: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpawnPattern {
    Circle,
    Line,
    Grid,
    Random,
    Spiral,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlexJustify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}
