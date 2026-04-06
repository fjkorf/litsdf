use serde::{Deserialize, Serialize};

/// All node types in the SDF node graph.
///
/// Each variant's fields are the default values for unconnected inputs.
/// When an input pin has a connection, the connected value overrides the default.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SdfNode {
    // ── Value Generators ──
    /// Outputs the current time in seconds. No inputs.
    Time,

    /// Sine oscillator: amplitude * sin(time * frequency * TAU + phase)
    SinOscillator {
        amplitude: f32,
        frequency: f32,
        phase: f32,
    },

    /// Constant float value.
    Constant { value: f32 },

    /// Constant Vec3 value.
    ConstantVec3 { value: [f32; 3] },

    // ── Math ──
    /// Add two floats.
    Add,

    /// Multiply two floats.
    Multiply,

    /// Linear interpolation: mix(a, b, factor)
    Mix { factor: f32 },

    /// Clamp a value to a range.
    Clamp { min: f32, max: f32 },

    /// Negate a float.
    Negate,

    // ── Vec3 Operations ──
    /// Compose 3 floats into a Vec3.
    Vec3Compose,

    /// Decompose a Vec3 into 3 floats.
    Vec3Decompose,

    // ── Wave Generators ──
    /// Square wave oscillator.
    SquareWave { amplitude: f32, frequency: f32, phase: f32 },

    /// Triangle wave oscillator.
    TriangleWave { amplitude: f32, frequency: f32, phase: f32 },

    /// Sawtooth wave oscillator.
    SawtoothWave { amplitude: f32, frequency: f32, phase: f32 },

    // ── Advanced Math ──
    /// Cubic ease in/out: x^exp for x<0.5, 1-(1-x)^exp for x>=0.5
    EaseInOut { exponent: f32 },

    /// Remap value from [in_min, in_max] to [out_min, out_max].
    Remap { in_min: f32, in_max: f32, out_min: f32, out_max: f32 },

    /// Absolute value.
    Abs,

    /// Modulo (wrapping).
    Modulo { divisor: f32 },

    /// Cosine palette: a + b * cos(2π(c*t + d)). Inputs/outputs are Vec3.
    CosinePalette,

    // ── Animation Shaping ──
    /// Exponential impulse: k * x * exp(1 - k*x). Sharp attack, smooth decay.
    ExpImpulse { k: f32 },

    /// Smoothstep: hermite interpolation from edge0 to edge1.
    SmoothStep { edge0: f32, edge1: f32 },

    /// 1D noise sampled at time * frequency. Organic random motion.
    Noise1D { frequency: f32 },

    // ── Logic Nodes ──
    /// Compare two values. Mode: 0=GT, 1=LT, 2=EQ, 3=GTE, 4=LTE. Output 1.0 or 0.0.
    Compare { mode: u32 },

    /// Pass Value through when Control > 0.5, else output 0.0.
    Gate,

    /// Boolean math. Op: 0=AND, 1=OR, 2=NOT (ignores B).
    BoolMath { op: u32 },

    // ── Physics Input Nodes ──
    /// Outputs 1.0 if the bone is touching any collider, 0.0 otherwise.
    IsColliding,

    /// Outputs the contact surface normal X/Y/Z.
    ContactNormal,

    /// Outputs downward raycast hit distance and normal.
    RaycastDown,

    /// Persistent per-bone float. Reads previous frame value, writes current.
    StateVar { index: u32 },

    /// Outputs bone's linear velocity X/Y/Z from Avian physics.
    BoneVelocity,

    /// Outputs bone's angular velocity X/Y/Z from Avian physics.
    BoneAngularVelocity,

    /// Outputs bone's world position X/Y/Z from Avian physics.
    BoneWorldPosition,

    /// Outputs scalar speed (length of velocity vector).
    BoneSpeed,

    // ── Output / Sink Nodes ──
    /// Shape property output. Collects final values to write into shape properties.
    ShapeOutput,

    /// Bone property output. Collects final values to write into bone transform + physics forces.
    BoneOutput,
}

impl SdfNode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Time => "Time",
            Self::SinOscillator { .. } => "Sin Oscillator",
            Self::Constant { .. } => "Constant",
            Self::ConstantVec3 { .. } => "Constant Vec3",
            Self::Add => "Add",
            Self::Multiply => "Multiply",
            Self::Mix { .. } => "Mix",
            Self::Clamp { .. } => "Clamp",
            Self::Negate => "Negate",
            Self::Vec3Compose => "Vec3 Compose",
            Self::Vec3Decompose => "Vec3 Decompose",
            Self::SquareWave { .. } => "Square Wave",
            Self::TriangleWave { .. } => "Triangle Wave",
            Self::SawtoothWave { .. } => "Sawtooth Wave",
            Self::EaseInOut { .. } => "Ease In/Out",
            Self::Remap { .. } => "Remap",
            Self::Abs => "Abs",
            Self::Modulo { .. } => "Modulo",
            Self::CosinePalette => "Cosine Palette",
            Self::ExpImpulse { .. } => "Exp Impulse",
            Self::SmoothStep { .. } => "Smooth Step",
            Self::Noise1D { .. } => "Noise 1D",
            Self::Compare { .. } => "Compare",
            Self::Gate => "Gate",
            Self::BoolMath { .. } => "Bool Math",
            Self::IsColliding => "Is Colliding",
            Self::ContactNormal => "Contact Normal",
            Self::RaycastDown => "Raycast Down",
            Self::StateVar { .. } => "State Var",
            Self::BoneVelocity => "Bone Velocity",
            Self::BoneAngularVelocity => "Bone Angular Vel",
            Self::BoneWorldPosition => "Bone World Pos",
            Self::BoneSpeed => "Bone Speed",
            Self::ShapeOutput => "Shape Output",
            Self::BoneOutput => "Bone Output",
        }
    }

    /// Number of input pins.
    pub fn input_count(&self) -> usize {
        match self {
            Self::Time => 0,
            Self::SinOscillator { .. } => 4,
            Self::Constant { .. } => 0,
            Self::ConstantVec3 { .. } => 0,
            Self::Add | Self::Multiply => 2,
            Self::Mix { .. } => 3,
            Self::Clamp { .. } => 3,
            Self::Negate => 1,
            Self::Vec3Compose => 3,
            Self::Vec3Decompose => 1,
            Self::SquareWave { .. } | Self::TriangleWave { .. } | Self::SawtoothWave { .. } => 4, // amp, freq, phase, time
            Self::EaseInOut { .. } => 2,     // value, exponent
            Self::Remap { .. } => 5,         // value, in_min, in_max, out_min, out_max
            Self::Abs => 1,
            Self::Modulo { .. } => 2,        // value, divisor
            Self::CosinePalette => 5,        // t, a, b, c, d (all vec3 except t)
            Self::ExpImpulse { .. } => 2,    // value, k
            Self::SmoothStep { .. } => 3,    // value, edge0, edge1
            Self::Noise1D { .. } => 2,       // time, frequency
            Self::Compare { .. } => 2,       // A, B
            Self::Gate => 2,                // Value, Control
            Self::BoolMath { .. } => 2,     // A, B
            Self::IsColliding | Self::ContactNormal | Self::RaycastDown => 0,
            Self::StateVar { .. } => 2,     // Write (bool), Value
            Self::BoneVelocity | Self::BoneAngularVelocity | Self::BoneWorldPosition | Self::BoneSpeed => 0,
            Self::ShapeOutput => 27,
            Self::BoneOutput => 13,   // transform(7) + force(3) + torque(3)
        }
    }

    /// Number of output pins.
    pub fn output_count(&self) -> usize {
        match self {
            Self::Time => 1,
            Self::SinOscillator { .. } => 1,
            Self::Constant { .. } => 1,
            Self::ConstantVec3 { .. } => 1,
            Self::Add | Self::Multiply => 1,
            Self::Mix { .. } => 1,
            Self::Clamp { .. } => 1,
            Self::Negate => 1,
            Self::Vec3Compose => 1,
            Self::Vec3Decompose => 3,
            Self::SquareWave { .. } | Self::TriangleWave { .. } | Self::SawtoothWave { .. } => 1,
            Self::EaseInOut { .. } | Self::Remap { .. } | Self::Abs | Self::Modulo { .. }
            | Self::ExpImpulse { .. } | Self::SmoothStep { .. } | Self::Noise1D { .. } => 1,
            Self::CosinePalette => 1,
            Self::Compare { .. } | Self::Gate | Self::BoolMath { .. } | Self::IsColliding | Self::StateVar { .. } => 1,
            Self::ContactNormal => 3,
            Self::RaycastDown => 4,  // Distance, Normal X, Y, Z
            Self::BoneVelocity | Self::BoneAngularVelocity | Self::BoneWorldPosition => 3,
            Self::BoneSpeed => 1,
            Self::ShapeOutput | Self::BoneOutput => 0,
        }
    }

    /// Input pin label.
    pub fn input_label(&self, index: usize) -> &'static str {
        match self {
            Self::SinOscillator { .. } => match index {
                0 => "Amplitude",
                1 => "Frequency",
                2 => "Phase",
                3 => "Time",
                _ => "?",
            },
            Self::Add | Self::Multiply => match index {
                0 => "A",
                1 => "B",
                _ => "?",
            },
            Self::Mix { .. } => match index {
                0 => "A",
                1 => "B",
                2 => "Factor",
                _ => "?",
            },
            Self::Clamp { .. } => match index {
                0 => "Value",
                1 => "Min",
                2 => "Max",
                _ => "?",
            },
            Self::Negate => "Value",
            Self::Vec3Compose => match index {
                0 => "X",
                1 => "Y",
                2 => "Z",
                _ => "?",
            },
            Self::Vec3Decompose => "Vec3",
            Self::SquareWave { .. } | Self::TriangleWave { .. } | Self::SawtoothWave { .. } => match index {
                0 => "Amplitude",
                1 => "Frequency",
                2 => "Phase",
                3 => "Time",
                _ => "?",
            },
            Self::EaseInOut { .. } => match index {
                0 => "Value",
                1 => "Exponent",
                _ => "?",
            },
            Self::Remap { .. } => match index {
                0 => "Value",
                1 => "In Min",
                2 => "In Max",
                3 => "Out Min",
                4 => "Out Max",
                _ => "?",
            },
            Self::Abs => "Value",
            Self::Modulo { .. } => match index {
                0 => "Value",
                1 => "Divisor",
                _ => "?",
            },
            Self::ExpImpulse { .. } => match index {
                0 => "Value",
                1 => "K (sharpness)",
                _ => "?",
            },
            Self::SmoothStep { .. } => match index {
                0 => "Value",
                1 => "Edge 0",
                2 => "Edge 1",
                _ => "?",
            },
            Self::Noise1D { .. } => match index {
                0 => "Time",
                1 => "Frequency",
                _ => "?",
            },
            Self::CosinePalette => match index {
                0 => "t",
                1 => "a (bias)",
                2 => "b (amp)",
                3 => "c (freq)",
                4 => "d (phase)",
                _ => "?",
            },
            Self::ShapeOutput => match index {
                0 => "Pos X",
                1 => "Pos Y",
                2 => "Pos Z",
                3 => "Rot X",
                4 => "Rot Y",
                5 => "Rot Z",
                6 => "Scale",
                7 => "Red",
                8 => "Green",
                9 => "Blue",
                10 => "Roughness",
                11 => "Metallic",
                12 => "Fresnel",
                13 => "Noise Amp",
                14 => "Noise Freq",
                15 => "Noise Oct",
                16 => "Symmetry",
                17 => "Rounding",
                18 => "Onion",
                19 => "Twist",
                20 => "Bend",
                21 => "Elongate X",
                22 => "Elongate Y",
                23 => "Elongate Z",
                24 => "Repeat X",
                25 => "Repeat Y",
                26 => "Repeat Z",
                _ => "?",
            },
            Self::Compare { .. } => match index { 0 => "A", 1 => "B", _ => "?" },
            Self::Gate => match index { 0 => "Value", 1 => "Control", _ => "?" },
            Self::BoolMath { .. } => match index { 0 => "A", 1 => "B", _ => "?" },
            Self::StateVar { .. } => match index { 0 => "Write", 1 => "Value", _ => "?" },
            Self::BoneOutput => match index {
                0 => "Pos X",
                1 => "Pos Y",
                2 => "Pos Z",
                3 => "Rot X",
                4 => "Rot Y",
                5 => "Rot Z",
                6 => "Scale",
                7 => "Force X",
                8 => "Force Y",
                9 => "Force Z",
                10 => "Torque X",
                11 => "Torque Y",
                12 => "Torque Z",
                _ => "?",
            },
            _ => "In",
        }
    }

    /// Output pin label.
    pub fn output_label(&self, index: usize) -> &'static str {
        match self {
            Self::Time => "Seconds",
            Self::SinOscillator { .. } => "Value",
            Self::Constant { .. } => "Value",
            Self::ConstantVec3 { .. } => "Vec3",
            Self::Add | Self::Multiply | Self::Mix { .. } | Self::Clamp { .. } | Self::Negate
            | Self::SquareWave { .. } | Self::TriangleWave { .. } | Self::SawtoothWave { .. }
            | Self::EaseInOut { .. } | Self::Remap { .. } | Self::Abs | Self::Modulo { .. }
            | Self::ExpImpulse { .. } | Self::SmoothStep { .. } | Self::Noise1D { .. } => "Result",
            Self::CosinePalette => "Color",
            Self::Vec3Compose => "Vec3",
            Self::Vec3Decompose => match index {
                0 => "X",
                1 => "Y",
                2 => "Z",
                _ => "?",
            },
            Self::Compare { .. } | Self::Gate | Self::BoolMath { .. } | Self::IsColliding | Self::StateVar { .. } => "Result",
            Self::ContactNormal => match index { 0 => "X", 1 => "Y", 2 => "Z", _ => "?" },
            Self::RaycastDown => match index { 0 => "Distance", 1 => "Normal X", 2 => "Normal Y", 3 => "Normal Z", _ => "?" },
            Self::BoneVelocity | Self::BoneAngularVelocity | Self::BoneWorldPosition => match index {
                0 => "X",
                1 => "Y",
                2 => "Z",
                _ => "?",
            },
            Self::BoneSpeed => "Speed",
            Self::ShapeOutput | Self::BoneOutput => "?",
        }
    }
}

/// Pin type for type checking connections and visual distinction.
/// All types are f32 under the hood — Bool and Int are visual/semantic only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinType {
    Float,
    Vec3,
    Bool,
    Int,
}

impl PinType {
    /// Whether two pin types can be connected (with auto-coercion).
    pub fn compatible(from: PinType, to: PinType) -> bool {
        use PinType::*;
        matches!((from, to),
            (Float, Float) | (Vec3, Vec3)
            | (Bool, Bool) | (Int, Int)
            // Auto-coerce between scalar types
            | (Bool, Float) | (Float, Bool)
            | (Int, Float) | (Float, Int)
            | (Bool, Int) | (Int, Bool)
        )
    }
}

impl SdfNode {
    pub fn input_type(&self, index: usize) -> PinType {
        match self {
            Self::Vec3Decompose => PinType::Vec3,
            Self::CosinePalette if index >= 1 => PinType::Vec3,
            // Logic node inputs
            Self::Gate if index == 1 => PinType::Bool, // Control
            Self::BoolMath { .. } => PinType::Bool,
            // Output sinks accept Float
            Self::ShapeOutput | Self::BoneOutput => PinType::Float,
            _ => PinType::Float,
        }
    }

    pub fn output_type(&self, _index: usize) -> PinType {
        match self {
            Self::ConstantVec3 { .. } | Self::Vec3Compose | Self::CosinePalette => PinType::Vec3,
            // Logic/sensing nodes output Bool
            Self::Compare { .. } | Self::BoolMath { .. } | Self::IsColliding => PinType::Bool,
            _ => PinType::Float,
        }
    }

    /// Whether this node is a physics input (only meaningful in bone graphs).
    pub fn is_physics_node(&self) -> bool {
        matches!(self, Self::BoneVelocity | Self::BoneAngularVelocity | Self::BoneWorldPosition | Self::BoneSpeed)
    }

    /// Human-readable description for tooltips.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Time => "Outputs elapsed time in seconds.",
            Self::SinOscillator { .. } => "Sine wave: amplitude * sin(time * frequency * TAU + phase).",
            Self::Constant { .. } => "Outputs a constant float value.",
            Self::ConstantVec3 { .. } => "Outputs a constant Vec3 value.",
            Self::Add => "Adds two float values (A + B).",
            Self::Multiply => "Multiplies two float values (A * B).",
            Self::Mix { .. } => "Linear interpolation: A*(1-f) + B*f.",
            Self::Clamp { .. } => "Clamps value to [min, max] range.",
            Self::Negate => "Negates a float value (-V).",
            Self::Vec3Compose => "Composes three floats into a Vec3.",
            Self::Vec3Decompose => "Decomposes a Vec3 into X, Y, Z floats.",
            Self::SquareWave { .. } => "Square wave oscillator (±amplitude).",
            Self::TriangleWave { .. } => "Triangle wave oscillator.",
            Self::SawtoothWave { .. } => "Sawtooth wave oscillator.",
            Self::EaseInOut { .. } => "Cubic ease in/out curve.",
            Self::Remap { .. } => "Remaps value from [in_min,in_max] to [out_min,out_max].",
            Self::Abs => "Absolute value of a float.",
            Self::Modulo { .. } => "Modulo (wrapping) operation.",
            Self::CosinePalette => "Cosine color palette: a + b*cos(2π(c*t + d)).",
            Self::ExpImpulse { .. } => "Exponential impulse: sharp attack, smooth decay.",
            Self::SmoothStep { .. } => "Hermite smoothstep interpolation.",
            Self::Noise1D { .. } => "1D hash-based noise sampled at time * frequency.",
            Self::Compare { .. } => "Compares A and B. Outputs true (1) or false (0).",
            Self::Gate => "Passes Value when Control is true, else outputs 0.",
            Self::BoolMath { .. } => "Boolean logic: AND, OR, or NOT.",
            Self::IsColliding => "True if this bone is touching any surface.",
            Self::ContactNormal => "Surface contact normal direction (X, Y, Z).",
            Self::RaycastDown => "Downward raycast: distance to ground and hit normal.",
            Self::StateVar { .. } => "Persistent per-bone float. Stores value across frames.",
            Self::BoneVelocity => "Bone linear velocity (X, Y, Z) from physics.",
            Self::BoneAngularVelocity => "Bone angular velocity (X, Y, Z) from physics.",
            Self::BoneWorldPosition => "Bone world position (X, Y, Z) from physics.",
            Self::BoneSpeed => "Scalar speed (length of velocity vector).",
            Self::ShapeOutput => "Writes values to shape properties (color, material, etc).",
            Self::BoneOutput => "Writes values to bone transform and physics forces.",
        }
    }
}
