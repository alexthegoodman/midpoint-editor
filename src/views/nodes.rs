use midpoint_engine::floem::event::EventListener;
use midpoint_engine::floem::event::EventPropagation;
use midpoint_engine::floem::peniko::Color;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::reactive::*;
use midpoint_engine::floem::style::CursorStyle;
use midpoint_engine::floem::taffy::Position;
use midpoint_engine::floem::unit::Pct;
use midpoint_engine::floem::unit::Px;
use midpoint_engine::floem::views::container;
use midpoint_engine::floem::views::dyn_stack;
use midpoint_engine::floem::views::empty;
use midpoint_engine::floem::views::h_stack;
use midpoint_engine::floem::views::label;
use midpoint_engine::floem::views::v_stack;
use midpoint_engine::floem::views::Decorators;
use midpoint_engine::floem::IntoView;
use midpoint_engine::floem::{GpuHelper, View, WindowHandle};
use nalgebra::Vector2;
use nalgebra_glm::Vec2;

use crate::helpers::nodes::NodeType;

struct NodeCanvas {
    offset: RwSignal<Vec2>,
    start_pan_pos: RwSignal<Vec2>,
    is_panning: RwSignal<bool>,
    nodes: RwSignal<Vec<NodeComponent>>,
}

pub fn node_ports(
    all_nodes: RwSignal<Vec<NodeComponent>>,
    node: NodeComponent,
    ports: Vec<Port>,
    left: bool,
) -> impl View {
    let ports = create_rw_signal(ports);

    container(
        (dyn_stack(
            move || ports.get(),
            move |port| port.id.clone(),
            (move |port| {
                let left = left.clone();
                let display_name_2 = port.display_name.clone();
                let connected_to = port
                    .connected_to
                    .as_ref()
                    .expect("Couldn't get connected_to");
                let all_nodes = all_nodes.get();
                let connected_to_port = if port.is_output {
                    all_nodes
                        .iter()
                        .find_map(|n| n.inputs.iter().find(|i| i.id == *connected_to).cloned())
                } else {
                    all_nodes
                        .iter()
                        .find_map(|n| n.outputs.iter().find(|i| i.id == *connected_to).cloned())
                };

                h_stack((
                    if left {
                        empty().into_any()
                    } else {
                        label(move || port.display_name.clone()).into_any()
                    },
                    if port.connected_to.is_some() && connected_to_port.is_some() {
                        label(move || {
                            format!(
                                "{}",
                                connected_to_port
                                    .as_ref()
                                    .expect("Couldn't get connected_to")
                                    .display_name
                            )
                        })
                        .style({
                            let node = node.clone();
                            move |s| s.background(node.get_type_color()).margin_bottom(2.0)
                        })
                        .into_any()
                    } else {
                        empty().into_any()
                    },
                    if left {
                        label(move || display_name_2.clone()).into_any()
                    } else {
                        empty().into_any()
                    },
                ))
            }),
        )),
    )
    .style(|s| s.position(Position::Relative).selectable(false))
}

pub fn node_item(
    node: NodeComponent,
    offset: RwSignal<Vec2>,
    start_pan_pos: RwSignal<Vec2>,
    is_panning: RwSignal<bool>,
    nodes: RwSignal<Vec<NodeComponent>>,
    dragging_node_id: RwSignal<Option<String>>,
) -> impl View {
    let node_position = create_rw_signal(Vec2::new(
        *node
            .initial_position
            .get(0)
            .expect("Couldn't get the position") as f32,
        *node
            .initial_position
            .get(1)
            .expect("Couldn't get the position") as f32,
    ));

    let is_dragging = create_rw_signal(false);
    let drag_start_pos = create_rw_signal(Vec2::identity());
    let initial_node_pos = create_rw_signal(Vec2::identity());
    let last_mouse_pos = create_rw_signal(Vec2::identity());

    let node_title = node.title.clone();
    let node_id = node.id.clone();
    let inputs = node.inputs.clone();
    let outputs = node.outputs.clone();
    let node_type = node.node_type.clone();

    let node_2 = node.clone();
    let node_3 = node.clone();

    v_stack((
        label(move || format!("{}", node_title.clone())).style(move |s| {
            s.background(node.get_type_color())
                .margin_bottom(4.0)
                .padding(4.0)
                .color(Color::WHITE_SMOKE)
                .selectable(false)
        }),
        node_ports(nodes, node_2.clone(), inputs, true),
        node_ports(nodes, node_3.clone(), outputs, false),
    ))
    .draggable()
    .style(move |s| {
        s.position(Position::Absolute)
            .margin_top(Px((node_position.get().y + offset.get().y) as f64))
            .margin_left(Px((node_position.get().x + offset.get().x) as f64))
            .width(150.0)
            .height(100.0)
            .border_radius(15.0)
            .background(Color::LIGHT_BLUE)
            .cursor(CursorStyle::Pointer)
            .box_shadow_blur(3)
            .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
            .box_shadow_spread(2)
    })
    .on_event(EventListener::DragStart, move |evt| {
        is_dragging.set(true);
        let scale_factor = 1.25; // hardcode test
        let position = Vec2::new(
            evt.point().expect("Couldn't get point").x as f32 / scale_factor,
            evt.point().expect("Couldn't get point").y as f32 / scale_factor,
        );
        // Calculate the offset between click and node position
        // let click_offset = Vec2::new(
        //     position.x - node_position.get().x,
        //     position.y - node_position.get().y,
        // );
        drag_start_pos.set(position);
        initial_node_pos.set(node_position.get());
        is_panning.set(false); // Prevent canvas panning while dragging node
        dragging_node_id.set(Some(node_id.clone()));
        last_mouse_pos.set(position);

        EventPropagation::Stop
    })
    .on_event(EventListener::DragEnd, move |evt| {
        is_dragging.set(false);
        dragging_node_id.set(None);
        let scale_factor = 1.25; // hardcode test
        let position = Vec2::new(
            evt.point().expect("Couldn't get point").x as f32 / scale_factor,
            evt.point().expect("Couldn't get point").y as f32 / scale_factor,
        );
        let delta = (position - drag_start_pos.get());

        let new_pos = Vec2::new(
            initial_node_pos.get().x + delta.x,
            initial_node_pos.get().y + delta.y, // Inverted y-axis delta
        );

        node_position.set(new_pos);
        EventPropagation::Continue
    })
    .style(move |s| {
        s.apply_if(is_dragging.get(), |s| {
            s.box_shadow_blur(3)
                .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
                .box_shadow_spread(2)
        })
    })
}

pub fn node_canvas() -> impl View {
    let offset = create_rw_signal(Vec2::identity());
    let start_pan_pos = create_rw_signal(Vec2::identity());
    let is_panning = create_rw_signal(false);
    let nodes: RwSignal<Vec<NodeComponent>> = create_rw_signal(create_test_nodes());
    let dragging_node_id = create_rw_signal(None::<String>);

    container(
        (dyn_stack(
            move || nodes.get(),
            move |node| node.id.clone(),
            (move |node| {
                let node_id = node.id.clone();
                node_item(
                    node,
                    offset,
                    start_pan_pos,
                    is_panning,
                    nodes,
                    dragging_node_id,
                )
            }),
        )),
    )
    .style(|s| s.width_full().height_full().background(Color::GRAY))
    // Canvas panning
    // .on_event(EventListener::PointerDown, move |evt| {
    //     if dragging_node_id.get().is_none() {
    //         is_panning.set(true);
    //         start_pan_pos.set(Vec2::new(
    //             evt.point().expect("Couldn't get point").x as f32,
    //             evt.point().expect("Couldn't get point").y as f32,
    //         ));
    //     }
    //     EventPropagation::Continue
    // })
    // .on_event(EventListener::PointerMove, move |evt| {
    //     if is_panning.get() && dragging_node_id.get().is_none() {
    //         let position = Vec2::new(
    //             evt.point().expect("Couldn't get point").x as f32,
    //             evt.point().expect("Couldn't get point").y as f32,
    //         );
    //         let delta = position - start_pan_pos.get();
    //         offset.update(|off| *off += delta);
    //         start_pan_pos.set(position);
    //     }
    //     EventPropagation::Continue
    // })
    // .on_event(EventListener::PointerUp, move |_| {
    //     is_panning.set(false);
    //     EventPropagation::Continue
    // })
}

// Node components would also benefit from signals for dynamic properties
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeComponent {
    pub id: String,
    pub title: String,
    pub node_type: NodeType,
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub initial_position: [u32; 2],
}

impl NodeComponent {
    pub fn new(id: String, node_type: NodeType, position: Vec2) -> Self {
        Self {
            id,
            title: String::new(),
            node_type,
            inputs: Vec::new(),
            outputs: Vec::new(),
            parent: None,
            children: Vec::new(),
            initial_position: [0, 0],
        }
    }
}

impl NodeComponent {
    pub fn get_type_color(&self) -> Color {
        match self.node_type {
            // data
            NodeType::State { .. } => Color::BLUE,
            NodeType::Array { .. } => Color::BLUE,
            NodeType::Dictionary { .. } => Color::BLUE,
            // control flow
            NodeType::Effect { .. } => Color::GREEN,
            NodeType::Event { .. } => Color::GREEN,
            NodeType::Conditional { .. } => Color::GREEN,
            NodeType::Loop { .. } => Color::GREEN,
            NodeType::Gate { .. } => Color::GREEN,
            NodeType::Sequence { .. } => Color::GREEN,
            // render
            NodeType::Render { .. } => Color::RED,
            NodeType::Camera { .. } => Color::RED,
            NodeType::UI { .. } => Color::RED,
            // operations
            NodeType::MathOp { .. } => Color::YELLOW,
            NodeType::VectorOp { .. } => Color::YELLOW,
            NodeType::StringOp { .. } => Color::YELLOW,
            NodeType::PhysicsOp { .. } => Color::YELLOW,
            NodeType::AnimationOp { .. } => Color::YELLOW,
            NodeType::AudioOp { .. } => Color::YELLOW,
            // systems
            NodeType::Behavior { .. } => Color::BLACK,
            NodeType::Spawner { .. } => Color::BLACK,
            NodeType::Collision { .. } => Color::BLACK,
            NodeType::Timer { .. } => Color::BLACK,
            NodeType::GameState { .. } => Color::BLACK,
        }
    }
}

// Port system using labels instead of paths
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Port {
    pub id: String,
    pub display_name: String,
    pub connected_to: Option<String>, // ID of connected port
    // pub connection_type: NodeType,
    pub is_output: bool,
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub enum PortType {
//     State,
//     Effect,
//     // etc
// }

pub fn create_test_nodes() -> Vec<NodeComponent> {
    vec![
        NodeComponent {
            id: "state1".to_string(),
            title: "Player Health (State)".to_string(),
            initial_position: [100, 100],
            node_type: NodeType::State {
                name: "Health State".to_string(),
                value: "80".to_string(),
                data_type: crate::helpers::nodes::DataType::String,
                persistent: false,
            },
            inputs: vec![],
            outputs: vec![Port {
                id: "health_out".to_string(),
                connected_to: Some("effect1_in".to_string()),
                display_name: "Health Out".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "effect1".to_string(),
            title: "Detect Warning (Effect)".to_string(),
            initial_position: [350, 100],
            node_type: NodeType::Effect {
                dependencies: Vec::new(),
                execution_order: 0,
                parallel: false,
            },
            inputs: vec![Port {
                id: "effect1_in".to_string(),
                connected_to: Some("health_out".to_string()),
                display_name: "Effect 1 In".to_string(),
                // connection_type: PortType::Variable,
                is_output: false,
            }],
            outputs: vec![Port {
                id: "warning_out".to_string(),
                connected_to: Some("ui1_in".to_string()),
                display_name: "Warning Out".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "ui1".to_string(),
            title: "Update Health Bar (UI)".to_string(),
            initial_position: [600, 100],
            node_type: NodeType::UI {
                element_type: crate::helpers::nodes::UIElementType::ProgressBar,
                layout: crate::helpers::nodes::LayoutType::Absolute,
                style: "temp_style".to_string(),
            },
            inputs: vec![Port {
                id: "ui1_in".to_string(),
                display_name: "UI In".to_string(),
                // connection_type: PortType::Variable,
                connected_to: Some("warning_out".to_string()),
                is_output: false,
            }],
            outputs: vec![],
            parent: None,
            children: Vec::new(),
        },
    ]
}
