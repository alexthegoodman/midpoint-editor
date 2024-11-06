use midpoint_engine::animations::motion_path::{create_test_motion_paths, EasingType};
use midpoint_engine::floem::event::EventListener;
use midpoint_engine::floem::reactive::{create_rw_signal, RwSignal, SignalGet, SignalUpdate};
use midpoint_engine::floem::taffy::Position;
use midpoint_engine::floem_renderer::Renderer;
use midpoint_engine::{
    animations::motion_path::MotionPath,
    floem::{
        self,
        context::{ComputeLayoutCx, EventCx, LayoutCx, PaintCx, StyleCx, UpdateCx},
        event::{Event, EventPropagation},
        kurbo::{self, Line, Point, Rect},
        peniko::{Brush, Color},
        style::Style,
        taffy::{Display, Layout, NodeId, TaffyTree},
        text::{Attrs, AttrsList, TextLayout},
        unit::UnitExt,
        views::{container, label, stack, Decorators},
        AppState, View, ViewId,
    },
};

use std::time::Duration;

/// State for the timeline component
#[derive(Debug, Clone)]
pub struct TimelineState {
    current_time: Duration,
    zoom_level: f64,
    scroll_offset: f64,
    selected_keyframes: Vec<KeyframeId>,
    // dragging: Option<DragState>,
    dragging: Option<DragOperation>,
    hovered_keyframe: Option<(String, Duration)>,
    property_expansions: im::HashMap<String, bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyframeId {
    property_path: String,
    time: Duration,
}

#[derive(Debug, Clone)]
enum DragState {
    Keyframe(KeyframeId),
    TimeSlider(f64),
    Scrolling { start_x: f64, start_offset: f64 },
}

/// Configuration for the timeline
#[derive(Debug, Clone)]
pub struct TimelineConfig {
    width: f64,
    height: f64,
    header_height: f64,
    property_width: f64,
    row_height: f64,
    // Add offset parameters
    offset_x: f64,
    offset_y: f64,
}

impl Default for TimelineConfig {
    fn default() -> Self {
        Self {
            width: 800.0,
            height: 400.0,
            header_height: 30.0,
            property_width: 150.0,
            row_height: 24.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnimationData {
    /// All motion paths in the animation
    pub paths: Vec<MotionPath>,
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
#[derive(Clone, Debug)]
pub struct UIKeyframe {
    /// Time of the keyframe
    pub time: Duration,
    /// Value at this keyframe (could be position, rotation, etc)
    pub value: KeyframeValue,
    /// Type of interpolation to next keyframe
    pub easing: EasingType,
}

/// Possible values for keyframes
#[derive(Clone, Debug)]
pub enum KeyframeValue {
    Position([f32; 3]),
    Rotation([f32; 4]),
    Scale([f32; 3]),
    Custom(Vec<f32>),
}

/// Helper to convert MotionPath data into UI-friendly AnimationData
impl AnimationData {
    pub fn from_motion_paths(paths: Vec<MotionPath>) -> Self {
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
                position_prop.children[0].keyframes.push(UIKeyframe {
                    time: keyframe.time,
                    value: KeyframeValue::Position([keyframe.position[0], 0.0, 0.0]),
                    easing: keyframe
                        .easing
                        .as_ref()
                        .expect("Couldn't get easing")
                        .clone(),
                });
                position_prop.children[1].keyframes.push(UIKeyframe {
                    time: keyframe.time,
                    value: KeyframeValue::Position([0.0, keyframe.position[1], 0.0]),
                    easing: keyframe
                        .easing
                        .as_ref()
                        .expect("Couldn't get easing")
                        .clone(),
                });
                position_prop.children[2].keyframes.push(UIKeyframe {
                    time: keyframe.time,
                    value: KeyframeValue::Position([0.0, 0.0, keyframe.position[2]]),
                    easing: keyframe
                        .easing
                        .as_ref()
                        .expect("Couldn't get easing")
                        .clone(),
                });
                // }

                // Update rotation keyframes
                // if let Some(rot) = keyframe.rotation {
                rotation_prop.keyframes.push(UIKeyframe {
                    time: keyframe.time,
                    value: KeyframeValue::Rotation(keyframe.rotation),
                    easing: keyframe
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

pub struct TimelineGridView {
    id: ViewId,
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: AnimationData,
    style: Style,
}

impl TimelineGridView {
    pub fn new(
        state: TimelineState,
        config: TimelineConfig,
        animation_data: AnimationData,
    ) -> Self {
        Self {
            id: ViewId::new(),
            state: create_rw_signal(state),
            config: config.clone(),
            animation_data,
            // style: Style::default(),
            style: Style::new()
                .margin_left(300.0)
                .margin_top(100.0)
                .position(Position::Absolute)
                .width(config.clone().width)
                .height(config.clone().height),
        }
        .on_event(EventListener::PointerMove, move |e| {
            println!("event recieved");
            EventPropagation::Continue
        })
    }

    fn draw_time_grid(&self, cx: &mut PaintCx) {
        let duration = self.animation_data.duration.as_secs_f64();
        let step = (0.5 / self.state.get().zoom_level).max(0.1);

        for time in (0..(duration / step) as i32).map(|i| i as f64 * step) {
            let x = self.config.offset_x
                + time_to_x(
                    self.state,
                    self.config.clone(),
                    Duration::from_secs_f64(time),
                );

            // Vertical grid line
            cx.stroke(
                &Line::new(
                    Point::new(x, self.config.offset_y),
                    Point::new(x, self.config.offset_y + self.config.height),
                ),
                &Color::GRAY,
                1.0,
            );

            // Time label
            let mut attrs_list = AttrsList::new(Attrs::new().color(Color::BLACK));
            let mut text_layout = TextLayout::new();
            text_layout.set_text(&format!("{:.1}s", time), attrs_list);

            cx.draw_text(&text_layout, Point::new(x, self.config.offset_y));
        }
    }

    fn draw_keyframe(&self, cx: &mut PaintCx, center: Point, selected: bool) {
        let size = 6.0;
        let color = if selected {
            Color::rgb8(66, 135, 245)
        } else {
            Color::rgb8(245, 166, 35)
        };

        // Draw diamond shape
        // let path = kurbo::BezPath::from_vec(vec![
        //     kurbo::PathEl::MoveTo(center + kurbo::Vec2::new(0.0, -size)),
        //     kurbo::PathEl::LineTo(center + kurbo::Vec2::new(size, 0.0)),
        //     kurbo::PathEl::LineTo(center + kurbo::Vec2::new(0.0, size)),
        //     kurbo::PathEl::LineTo(center + kurbo::Vec2::new(-size, 0.0)),
        //     kurbo::PathEl::ClosePath,
        // ]);
        let path = kurbo::BezPath::from_vec(vec![
            kurbo::PathEl::MoveTo(Point::new(
                center.x + self.config.offset_x,
                center.y + self.config.offset_y - size,
            )),
            kurbo::PathEl::LineTo(Point::new(
                center.x + self.config.offset_x + size,
                center.y + self.config.offset_y,
            )),
            kurbo::PathEl::LineTo(Point::new(
                center.x + self.config.offset_x,
                center.y + self.config.offset_y + size,
            )),
            kurbo::PathEl::LineTo(Point::new(
                center.x + self.config.offset_x - size,
                center.y + self.config.offset_y,
            )),
            kurbo::PathEl::ClosePath,
        ]);

        cx.fill(&path, color, 1.0);
    }

    fn draw_keyframes(&self, cx: &mut PaintCx) {
        // Track visible vertical space used
        let mut current_y = self.config.header_height;

        // Draw keyframes for each property
        for property in &self.animation_data.properties {
            if let Some(y) = self.draw_property_keyframes(cx, property, current_y) {
                current_y = y;
            }
        }
    }

    fn draw_property_keyframes(
        &self,
        cx: &mut PaintCx,
        property: &AnimationProperty,
        start_y: f64,
    ) -> Option<f64> {
        let mut current_y = start_y;

        // Check if property is visible (based on scroll position and view height)
        if current_y > self.config.height {
            return None;
        }

        // Draw property's own keyframes
        for keyframe in &property.keyframes {
            let x = time_to_x(self.state, self.config.clone(), keyframe.time);

            // Skip if outside visible area
            if x < -10.0 || x > self.config.width + 10.0 {
                continue;
            }

            let selected = self.state.get().selected_keyframes.contains(&KeyframeId {
                property_path: property.property_path.clone(),
                time: keyframe.time,
            });

            // Draw the keyframe marker
            self.draw_keyframe(
                cx,
                Point::new(x, (current_y + self.config.row_height / 2.0)),
                selected,
            );

            // draw connecting lines between keyframes
            // (only if there's a previous keyframe in our visible range)
            if let Some(prev_keyframe) = property
                .keyframes
                .iter()
                .filter(|k| k.time < keyframe.time)
                .last()
            {
                let prev_x = time_to_x(self.state, self.config.clone(), prev_keyframe.time);
                if prev_x >= -10.0 {
                    cx.stroke(
                        &Line::new(
                            Point::new(
                                self.config.offset_x + prev_x,
                                self.config.offset_y + (current_y + self.config.row_height / 2.0),
                            ),
                            Point::new(
                                self.config.offset_x + x,
                                self.config.offset_y + (current_y + self.config.row_height / 2.0),
                            ),
                        ),
                        &Color::DARK_GRAY,
                        1.0,
                    );
                }
            }
        }

        current_y += self.config.row_height;

        // If the property is expanded, draw child properties
        if self
            .state
            .get()
            .property_expansions
            .get(&property.property_path)
            .copied()
            .unwrap_or(false)
        {
            for child in &property.children {
                if let Some(new_y) = self.draw_property_keyframes(cx, child, current_y) {
                    current_y = new_y;
                } else {
                    // Child property was outside visible area, we can stop here
                    break;
                }
            }
        }

        Some(current_y)
    }

    /// Calculate the Y position for a given property path
    fn get_property_y_position(&self, property_path: &str) -> f64 {
        let mut y_position = self.config.header_height;

        // Helper function to search through properties recursively
        fn find_property_y(
            properties: &[AnimationProperty],
            target_path: &str,
            current_y: &mut f64,
            row_height: f64,
            property_expansions: &im::HashMap<String, bool>,
        ) -> Option<f64> {
            for property in properties {
                // Check if this is the property we're looking for
                if property.property_path == target_path {
                    return Some(*current_y);
                }

                // Move to next row
                *current_y += row_height;

                // If this property is expanded and has children, search them
                if property_expansions
                    .get(&property.property_path)
                    .copied()
                    .unwrap_or(false)
                {
                    if let Some(y) = find_property_y(
                        &property.children,
                        target_path,
                        current_y,
                        row_height,
                        property_expansions,
                    ) {
                        return Some(y);
                    }
                }
            }
            None
        }

        // Search through properties and return the found Y position or a default
        let y = find_property_y(
            &self.animation_data.properties,
            property_path,
            &mut y_position,
            self.config.row_height,
            &self.state.get().property_expansions,
        );

        // Add the offset_y to the final position
        self.config.offset_y + y.unwrap_or(y_position) + (self.config.row_height / 2.0)
    }

    pub fn request_repaint(&self) {
        self.id.request_paint();
    }
}

fn hit_test_keyframe(
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: AnimationData,
    point: Point,
) -> Option<(String, Duration)> {
    let current_y = config.header_height;
    let row_height = config.row_height.clone();
    let hit_radius = 6.0;

    for property in &animation_data.properties {
        // Check if point is within the property's vertical bounds
        let property_height = row_height;
        let y_center = current_y + property_height / 2.0;

        if (point.y - y_center).abs() <= hit_radius {
            // Check keyframes
            for keyframe in &property.keyframes {
                let x = time_to_x(state, config.clone(), keyframe.time);
                let keyframe_point = Point::new(x, y_center);

                if point.distance(keyframe_point) <= hit_radius {
                    return Some((property.property_path.clone(), keyframe.time));
                }
            }
        }
    }
    None
}

fn time_to_x(state: RwSignal<TimelineState>, config: TimelineConfig, time: Duration) -> f64 {
    let time_secs = time.as_secs_f64();
    let base_spacing = config.property_width; // pixels per second at zoom level 1.0
    (time_secs * base_spacing * state.get().zoom_level) - state.get().scroll_offset
}

fn x_to_time(state: RwSignal<TimelineState>, config: TimelineConfig, x: f64) -> Duration {
    let base_spacing = config.property_width;
    let time_secs = (x + state.get().scroll_offset) / (base_spacing * state.get().zoom_level);
    Duration::from_secs_f64(time_secs.max(0.0))
}

#[derive(Clone, Debug)]
enum DragOperation {
    Playhead(f64),
    Keyframe {
        property_path: String,
        original_time: Duration,
        start_x: f64,
    },
    None,
}

impl View for TimelineGridView {
    fn id(&self) -> ViewId {
        // println!("get id");
        self.id
    }

    fn view_style(&self) -> Option<Style> {
        // println!("view_style");
        Some(self.style.clone())
    }

    fn debug_name(&self) -> std::borrow::Cow<'static, str> {
        "TimelineGridView".into()
    }

    fn paint(&mut self, cx: &mut PaintCx) {
        // println!("paint");
        // Draw background
        // Draw background using a rectangle path instead of bounds
        // let background_rect = kurbo::Rect::new(0.0, 0.0, self.config.width, self.config.height);
        let background_rect = kurbo::Rect::new(
            self.config.offset_x,
            self.config.offset_y,
            self.config.offset_x + self.config.width,
            self.config.offset_y + self.config.height,
        );
        cx.fill(&background_rect, Color::WHITE, 1.0);

        // cx.fill(cx.bounds(), &Color::WHITE, 1.0);

        // Draw grid
        self.draw_time_grid(cx);

        // Draw keyframes
        self.draw_keyframes(cx);

        // Draw playhead with offset
        let playhead_x = self.config.offset_x
            + time_to_x(
                self.state,
                self.config.clone(),
                self.state.get().current_time,
            );
        cx.stroke(
            &Line::new(
                Point::new(playhead_x, self.config.offset_y),
                Point::new(playhead_x, self.config.offset_y + self.config.height),
            ),
            &Color::rgb8(255, 0, 0),
            2.0,
        );

        // Add hover effects
        if let Some((property_path, time)) = &self.state.get().hovered_keyframe {
            let y = self.get_property_y_position(property_path);
            let x = time_to_x(self.state, self.config.clone(), *time);

            // Draw hover highlight
            let hover_size = 8.0;
            cx.stroke(
                &kurbo::Circle::new(Point::new(x, y), hover_size),
                &Color::rgba8(255, 165, 0, 128), // Semi-transparent orange
                2.0,
            );
        }
    }

    // fn event_before_children(&mut self, _cx: &mut EventCx, event: &Event) -> EventPropagation {
    //     let layout = self.id.get_layout();
    //     println!("event_before_children {:?}", layout);
    //     match event {
    //         Event::PointerDown(e) if e.button.is_primary() => self.handle_mouse_down(e.pos),
    //         Event::PointerMove(e) => self.handle_mouse_move(e.pos),
    //         Event::PointerUp(e) if e.button.is_primary() => self.handle_mouse_up(e.pos),
    //         Event::PointerWheel(e) => self.handle_scroll(e.delta.y),
    //         _ => EventPropagation::Continue,
    //     }
    // }

    // fn compute_layout(&mut self, cx: &mut ComputeLayoutCx) -> Option<Rect> {
    //     // println!("compute_layout");
    //     Some(Rect::new(0.0, 0.0, self.config.width, self.config.height))
    // }

    // Make sure compute_layout returns proper bounds
    fn compute_layout(&mut self, _cx: &mut ComputeLayoutCx) -> Option<Rect> {
        println!("compute_layout");
        Some(Rect::new(
            self.config.offset_x,
            self.config.offset_y,
            self.config.offset_x + self.config.width,
            self.config.offset_y + self.config.height,
        ))
    }

    // Implement other required View trait methods with default behavior
    fn update(&mut self, _cx: &mut UpdateCx, _state: Box<dyn std::any::Any>) {
        println!("update");
        self.id.request_layout();
    }
    // fn style_pass(&mut self, _cx: &mut StyleCx) {}
    fn layout(&mut self, _cx: &mut LayoutCx) -> NodeId {
        // println!("layout");
        // NodeId::new(0) // You'll need proper node ID management
        let node = self.id().new_taffy_node();
        node
    }
    // fn event_after_children(&mut self, _cx: &mut EventCx, _event: &Event) -> EventPropagation {
    //     EventPropagation::Continue
    // }
    fn scroll_to(&mut self, _cx: &mut AppState, _target: ViewId, _rect: Option<Rect>) -> bool {
        false
    }
}

// Create test data for the timeline
pub fn create_test_timeline() -> impl View {
    let state = TimelineState {
        current_time: Duration::from_secs_f64(0.0),
        zoom_level: 1.0,
        scroll_offset: 0.0,
        selected_keyframes: Vec::new(),
        property_expansions: im::HashMap::from_iter([
            ("position".to_string(), true),
            ("rotation".to_string(), true),
        ]),
        dragging: None,
        hovered_keyframe: None,
    };

    let config = TimelineConfig {
        width: 1200.0,
        height: 300.0,
        header_height: 30.0,
        property_width: 200.0,
        row_height: 24.0,
        // offset_x: 325.0,
        // offset_y: 300.0,
        offset_x: 0.0,
        offset_y: 0.0,
    };

    // Create some test keyframes for position
    let position_x_keyframes = vec![
        UIKeyframe {
            time: Duration::from_secs_f64(0.0),
            value: KeyframeValue::Position([0.0, 0.0, 0.0]),
            easing: EasingType::Linear,
        },
        UIKeyframe {
            time: Duration::from_secs_f64(1.5),
            value: KeyframeValue::Position([100.0, 0.0, 0.0]),
            easing: EasingType::EaseInOut,
        },
        UIKeyframe {
            time: Duration::from_secs_f64(3.0),
            value: KeyframeValue::Position([-50.0, 0.0, 0.0]),
            easing: EasingType::EaseIn,
        },
    ];

    let position_y_keyframes = vec![
        UIKeyframe {
            time: Duration::from_secs_f64(0.0),
            value: KeyframeValue::Position([0.0, 0.0, 0.0]),
            easing: EasingType::Linear,
        },
        UIKeyframe {
            time: Duration::from_secs_f64(2.0),
            value: KeyframeValue::Position([0.0, 150.0, 0.0]),
            easing: EasingType::EaseOut,
        },
    ];

    // Create test keyframes for rotation
    let rotation_keyframes = vec![
        UIKeyframe {
            time: Duration::from_secs_f64(0.5),
            value: KeyframeValue::Rotation([0.0, 0.0, 0.0, 1.0]),
            easing: EasingType::Linear,
        },
        UIKeyframe {
            time: Duration::from_secs_f64(2.5),
            value: KeyframeValue::Rotation([0.0, 0.0, 0.707, 0.707]),
            easing: EasingType::EaseInOut,
        },
    ];

    // Create property hierarchy
    let animation_data = AnimationData {
        paths: create_test_motion_paths(), // We can add actual MotionPath data if needed
        duration: Duration::from_secs_f64(4.0),
        properties: vec![
            AnimationProperty {
                name: "Position".to_string(),
                property_path: "position".to_string(),
                children: vec![
                    AnimationProperty {
                        name: "X".to_string(),
                        property_path: "position.x".to_string(),
                        children: Vec::new(),
                        keyframes: position_x_keyframes,
                        depth: 1,
                    },
                    AnimationProperty {
                        name: "Y".to_string(),
                        property_path: "position.y".to_string(),
                        children: Vec::new(),
                        keyframes: position_y_keyframes,
                        depth: 1,
                    },
                    AnimationProperty {
                        name: "Z".to_string(),
                        property_path: "position.z".to_string(),
                        children: Vec::new(),
                        keyframes: Vec::new(), // Empty for testing
                        depth: 1,
                    },
                ],
                keyframes: Vec::new(),
                depth: 0,
            },
            AnimationProperty {
                name: "Rotation".to_string(),
                property_path: "rotation".to_string(),
                children: Vec::new(),
                keyframes: rotation_keyframes,
                depth: 0,
            },
        ],
    };

    create_timeline(state, config, animation_data)
}

#[derive(Clone)]
struct TimelineHandle {
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: AnimationData,
    view_id: ViewId,
}

pub fn create_timeline(
    state: TimelineState,
    config: TimelineConfig,
    animation_data: AnimationData,
) -> impl View {
    let test = TimelineGridView::new(state, config, animation_data);

    let view_id = test.id;

    // Create a lightweight handle for events
    let handle = TimelineHandle {
        state: test.state.clone(),
        config: test.config.clone(),
        animation_data: test.animation_data.clone(),
        view_id,
    };

    let handle_move = handle.clone();
    let handle_up = handle.clone();
    let handle_wheel = handle.clone();

    container((test))
        .style(|s| {
            s.width(1200.0)
                .height(300.0)
                .margin_top(50.0)
                .margin_left(50.0)
                .background(Color::LIGHT_CORAL)
        })
        .on_event(EventListener::PointerDown, move |e| {
            println!("PointerDown");
            let scale_factor = 1.25; // hardcode test
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );

            handle_mouse_down(
                handle.state,
                handle.config.clone(),
                handle.animation_data.clone(),
                position,
            );
            handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerMove, move |e| {
            // println!("PointerMove");
            let scale_factor = 1.25;
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );

            handle_mouse_move(
                handle_move.state,
                handle_move.config.clone(),
                handle_move.animation_data.clone(),
                position,
            );
            handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerUp, move |e| {
            println!("PointerUp");
            let scale_factor = 1.25;
            let position = Point::new(
                e.point().expect("Couldn't get point").x as f64,
                e.point().expect("Couldn't get point").y as f64,
            );
            handle_mouse_up(handle_up.state, position);
            handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
        .on_event(EventListener::PointerWheel, move |e| {
            println!("PointerWheel");
            // Add wheel handling
            handle_scroll(handle_wheel.state, 0.1);
            handle.view_id.request_paint(); // Request repaint after state change
            EventPropagation::Continue
        })
}

fn handle_mouse_down(
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: AnimationData,
    pos: Point,
) -> EventPropagation {
    println!("handle_mouse_down");
    let state_data = state.get();
    // Check if clicking on a keyframe
    if let Some((property_path, time)) =
        hit_test_keyframe(state, config.clone(), animation_data, pos)
    {
        // state_data.dragging = Some(DragOperation::Keyframe {
        //     property_path,
        //     original_time: time,
        //     start_x: pos.x,
        // });
        println!("start move keyframe {:?}", time);
        state.update(|s| {
            s.dragging = Some(DragOperation::Keyframe {
                property_path,
                original_time: time,
                start_x: pos.x,
            })
        });
        return EventPropagation::Stop;
    }

    // Check if clicking on timeline (for playhead)
    if pos.y <= config.header_height {
        let time = x_to_time(state, config, pos.x);
        // state_data.current_time = time;
        // state_data.dragging = Some(DragOperation::Playhead(pos.x));
        println!("start move playhead {:?}", time);
        state.update(|s| s.current_time = time);
        state.update(|s| s.dragging = Some(DragOperation::Playhead(pos.x)));
        return EventPropagation::Stop;
    }

    EventPropagation::Continue
}

fn handle_mouse_move(
    state: RwSignal<TimelineState>,
    config: TimelineConfig,
    animation_data: AnimationData,
    pos: Point,
) -> EventPropagation {
    // println!("handle_mouse_move");
    let state_data = state.get();
    if (state_data.dragging.is_some()) {
        let dragging = state_data.dragging.as_ref().expect("Couldn't get dragging");
        match dragging {
            DragOperation::Playhead(_) => {
                // state_data.current_time = x_to_time(state, config.clone(), pos.x);
                println!("moving playhead");
                let value = x_to_time(state, config.clone(), pos.x);
                state.update(|s| s.current_time = value);
                return EventPropagation::Stop;
            }
            DragOperation::Keyframe {
                property_path,
                original_time,
                start_x,
            } => {
                let delta_x = pos.x - start_x;
                let new_time = x_to_time(
                    state,
                    config.clone(),
                    time_to_x(state, config.clone(), *original_time) + delta_x,
                );

                println!("moving keyframe {:?}", new_time);

                // TODO: update_keyframe_time
                // self.update_keyframe_time(property_path, *original_time, new_time);

                return EventPropagation::Stop;
            }
            _ => {
                // Update hover state
                if let Some((property_path, time)) =
                    hit_test_keyframe(state, config.clone(), animation_data, pos)
                {
                    // state_data.hovered_keyframe = Some((property_path, time));
                    state.update(|s| s.hovered_keyframe = Some((property_path, time)));
                } else {
                    // state_data.hovered_keyframe = None;
                    state.update(|s| s.hovered_keyframe = None);
                }
                return EventPropagation::Continue;
            }
        }
    } else {
        return EventPropagation::Continue;
    }
}

fn handle_mouse_up(state: RwSignal<TimelineState>, _pos: Point) -> EventPropagation {
    // let state = state.get();

    // state.dragging = None;
    state.update(|s| s.dragging = None);
    EventPropagation::Stop
}

fn handle_scroll(state: RwSignal<TimelineState>, delta: f64) -> EventPropagation {
    let state_data = state.get();

    println!("handle_scroll");
    if delta != 0.0 {
        // Adjust zoom level based on scroll
        let old_zoom = state_data.zoom_level;
        // state.zoom_level = (state.zoom_level * (1.0 + delta * 0.001))
        //     .max(0.1)
        //     .min(10.0);
        state.update(|s| {
            s.zoom_level = (state_data.zoom_level * (1.0 + delta * 0.001))
                .max(0.1)
                .min(10.0)
        });

        // Adjust scroll offset to keep the timeline position under the cursor
        // You might want to use the cursor position for more precise zooming
        let zoom_ratio = state_data.zoom_level / old_zoom;
        // state.scroll_offset *= zoom_ratio;
        state.update(|s| {
            s.scroll_offset *= zoom_ratio;
        });

        EventPropagation::Stop
    } else {
        EventPropagation::Continue
    }
}
