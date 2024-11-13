use midpoint_engine::animations::motion_path::{EasingType, SkeletonMotionPath};
use uuid::Uuid;

use std::time::Duration;

#[derive(Clone, Debug)]
pub struct AnimationData {
    /// All motion paths in the animation
    pub paths: Vec<SkeletonMotionPath>,
    /// Total duration of the animation
    pub duration: Duration,
    /// Hierarchical property structure for UI
    pub properties: Vec<AnimationProperty>,
}

/// Represents a property that can be animated in the UI
#[derive(Clone, Debug)]
pub struct AnimationProperty {
    /// Name of the property (e.g., "Position.X", "Rotation.Z")
    pub name: String,
    /// Path to this property in the data (for linking to MotionPath data)
    pub property_path: String,
    /// Nested properties (if any)
    pub children: Vec<AnimationProperty>,
    /// Direct keyframes for this property
    pub keyframes: Vec<UIKeyframe>,
    /// Visual depth in the property tree
    pub depth: u32,
}

/// Represents a keyframe in the UI
#[derive(PartialEq, Clone, Debug)]
pub struct UIKeyframe {
    /// Used to associate with this speciifc UI Keyframe
    pub id: String,
    /// Used to associate with the SkeletonKeyframe for updates
    pub skel_key_id: String,
    /// Time of the keyframe
    pub time: Duration,
    /// Value at this keyframe (could be position, rotation, etc)
    pub value: KeyframeValue,
    /// Type of interpolation to next keyframe
    pub easing: EasingType,
}

/// Possible values for keyframes
#[derive(PartialEq, Clone, Debug)]
pub enum KeyframeValue {
    Position([f32; 3]),
    Rotation([f32; 4]),
    Scale([f32; 3]),
    Custom(Vec<f32>),
}

/// Helper to convert MotionPath data into UI-friendly AnimationData
impl AnimationData {
    pub fn from_motion_paths(paths: Vec<SkeletonMotionPath>) -> Self {
        let mut max_duration = Duration::from_secs(0);
        let mut properties = Vec::new();

        // Create top-level property groups
        let mut position_prop = AnimationProperty {
            name: "Position".to_string(),
            property_path: "position".to_string(),
            children: vec![
                AnimationProperty {
                    name: "X".to_string(),
                    property_path: "position.x".to_string(),
                    children: Vec::new(),
                    keyframes: Vec::new(),
                    depth: 1,
                },
                AnimationProperty {
                    name: "Y".to_string(),
                    property_path: "position.y".to_string(),
                    children: Vec::new(),
                    keyframes: Vec::new(),
                    depth: 1,
                },
                AnimationProperty {
                    name: "Z".to_string(),
                    property_path: "position.z".to_string(),
                    children: Vec::new(),
                    keyframes: Vec::new(),
                    depth: 1,
                },
            ],
            keyframes: Vec::new(),
            depth: 0,
        };

        let mut rotation_prop = AnimationProperty {
            name: "Rotation".to_string(),
            property_path: "rotation".to_string(),
            children: Vec::new(),
            keyframes: Vec::new(),
            depth: 0,
        };

        // Process each motion path
        for path in &paths {
            // Update max duration
            if path.duration > max_duration {
                max_duration = path.duration;
            }

            // Process keyframes into UI properties
            for keyframe in &path.keyframes {
                // Update position keyframes
                // if let Some(pos) = keyframe.position {
                let uuid1 = Uuid::new_v4();
                position_prop.children[0].keyframes.push(UIKeyframe {
                    id: uuid1.to_string(),
                    skel_key_id: keyframe.id.clone(),
                    time: keyframe.base.time,
                    value: KeyframeValue::Position([keyframe.base.position[0], 0.0, 0.0]),
                    easing: keyframe
                        .base
                        .easing
                        .as_ref()
                        .expect("Couldn't get easing")
                        .clone(),
                });
                let uuid2 = Uuid::new_v4();
                position_prop.children[1].keyframes.push(UIKeyframe {
                    id: uuid2.to_string(),
                    skel_key_id: keyframe.id.clone(),
                    time: keyframe.base.time,
                    value: KeyframeValue::Position([0.0, keyframe.base.position[1], 0.0]),
                    easing: keyframe
                        .base
                        .easing
                        .as_ref()
                        .expect("Couldn't get easing")
                        .clone(),
                });
                let uuid3 = Uuid::new_v4();
                position_prop.children[2].keyframes.push(UIKeyframe {
                    id: uuid3.to_string(),
                    skel_key_id: keyframe.id.clone(),
                    time: keyframe.base.time,
                    value: KeyframeValue::Position([0.0, 0.0, keyframe.base.position[2]]),
                    easing: keyframe
                        .base
                        .easing
                        .as_ref()
                        .expect("Couldn't get easing")
                        .clone(),
                });
                // }

                // Update rotation keyframes
                // if let Some(rot) = keyframe.rotation {
                let uuid4 = Uuid::new_v4();
                rotation_prop.keyframes.push(UIKeyframe {
                    id: uuid4.to_string(),
                    skel_key_id: keyframe.id.clone(),
                    time: keyframe.base.time,
                    value: KeyframeValue::Rotation(keyframe.base.rotation),
                    easing: keyframe
                        .base
                        .easing
                        .as_ref()
                        .expect("Couldn't get easing")
                        .clone(),
                });
                // }
            }
        }

        properties.push(position_prop);
        properties.push(rotation_prop);

        Self {
            paths,
            duration: max_duration,
            properties,
        }
    }
}
