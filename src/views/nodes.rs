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

struct NodeCanvas {
    offset: RwSignal<Vec2>,
    start_pan_pos: RwSignal<Vec2>,
    is_panning: RwSignal<bool>,
    nodes: RwSignal<Vec<NodeComponent>>,
}

pub fn node_ports(ports: Vec<Port>) -> impl View {
    let ports = create_rw_signal(ports);

    container(
        (dyn_stack(
            move || ports.get(),
            move |port| port.id.clone(),
            (move |port| {
                h_stack((
                    label(move || format!("IN({})", port.id)),
                    if port.connected_to.is_some() {
                        label(move || {
                            format!(
                                "← {}",
                                port.connected_to
                                    .as_ref()
                                    .expect("Couldn't get connected_to")
                            )
                        })
                        .into_any()
                    } else {
                        empty().into_any()
                    },
                ))
            }),
        )),
    )
}

// pub fn node_item(
//     node: NodeComponent,
//     offset: RwSignal<Vec2>,
//     start_pan_pos: RwSignal<Vec2>,
//     is_panning: RwSignal<bool>,
//     nodes: RwSignal<Vec<NodeComponent>>,
//     dragging_node_id: RwSignal<Option<String>>,
// ) -> impl View {
//     let node_position = create_rw_signal(Vec2::new(
//         *node
//             .initial_position
//             .get(0)
//             .expect("Couldn't get the position") as f32,
//         *node
//             .initial_position
//             .get(1)
//             .expect("Couldn't get the position") as f32,
//     ));

//     let node_title = node.title.clone();
//     let node_id = node.id.clone();
//     let node_id_c = node.id.clone();
//     let inputs = node.inputs.clone();
//     let outputs = node.outputs.clone();

//     v_stack((
//         label(move || node_title.clone()).style(move |s| s.background(node.get_type_color())),
//         node_ports(inputs),
//         node_ports(outputs),
//     ))
//     .style(move |s| {
//         s.position(Position::Absolute)
//             .margin_top(node_position.get().x + offset.get().x)
//             .margin_left(node_position.get().y + offset.get().y)
//             .width(150.0)
//             .height(150.0)
//             .background(Color::OLIVE)
//             .cursor(CursorStyle::Pointer)
//     })
//     .draggable()
//     .on_event(EventListener::DragStart, move |_| {
//         println!("Drag start");
//         dragging_node_id.set(Some(node_id.clone()));
//         EventPropagation::Continue
//     })
//     .on_event(EventListener::DragOver, {
//         println!("Drag over");
//         let node_id = node_id_c.clone();

//         move |evt| {
//             if let Some(drag_id) = dragging_node_id.get() {
//                 if drag_id == node_id.clone() {
//                     let position = Vec2::new(
//                         evt.point().expect("Couldn't get point").x as f32,
//                         evt.point().expect("Couldn't get point").y as f32,
//                     );
//                     // Update node position based on drag delta
//                     let delta = position - start_pan_pos.get();
//                     node_position.update(|p| *p += delta);
//                     start_pan_pos.set(position);
//                 }
//             }
//             EventPropagation::Continue
//         }
//     })
//     .on_event(EventListener::DragEnd, move |_| {
//         println!("Drag end");
//         dragging_node_id.set(None);
//         EventPropagation::Continue
//     })
//     .dragging_style(|s| {
//         s.box_shadow_blur(3)
//             .box_shadow_color(Color::rgba(100.0, 100.0, 100.0, 0.5))
//             .box_shadow_spread(2)
//     })
// }

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

    v_stack((
        label(move || node_title.clone()).style(move |s| s.background(node.get_type_color())),
        node_ports(inputs),
        node_ports(outputs),
    ))
    .draggable()
    .style(move |s| {
        s.position(Position::Absolute)
            .margin_top(Px((node_position.get().y + offset.get().y) as f64))
            .margin_left(Px((node_position.get().x + offset.get().x) as f64))
            .width(150.0)
            .height(100.0)
            .background(Color::LIGHT_BLUE)
            .cursor(CursorStyle::Pointer)
    })
    .on_event(EventListener::DragStart, move |evt| {
        is_dragging.set(true);
        let scale_factor = 1.25; // hardcode test
        let position = Vec2::new(
            evt.point().expect("Couldn't get point").x as f32 / scale_factor,
            evt.point().expect("Couldn't get point").y as f32 / scale_factor,
        );
        // Calculate the offset between click and node position
        let click_offset = Vec2::new(
            position.x - node_position.get().x,
            position.y - node_position.get().y,
        );
        println!(
            "DragStart offset: {:?} {:?} {:?}",
            position,
            node_position.get(),
            click_offset
        );
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

// Different node types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeType {
    State { name: String, value: String },
    Effect { dependencies: Vec<String> },
    Context { name: String },
}

impl NodeComponent {
    pub fn get_type_color(&self) -> Color {
        match self.node_type {
            NodeType::State { .. } => Color::BLUE,
            NodeType::Effect { .. } => Color::GREEN,
            NodeType::Context { .. } => Color::PURPLE,
        }
    }
}

// Port system using labels instead of paths
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Port {
    pub id: String,
    pub display_name: String,
    pub connected_to: Option<String>, // ID of connected port
    pub connection_type: PortType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortType {
    State,
    Effect,
    Context,
    Variable,
    // etc
}

pub fn create_test_nodes() -> Vec<NodeComponent> {
    vec![
        NodeComponent {
            id: "state1".to_string(),
            title: "Player Health".to_string(),
            initial_position: [100, 100],
            node_type: NodeType::State {
                name: "Health State".to_string(),
                value: "80".to_string(),
            },
            inputs: vec![],
            outputs: vec![Port {
                id: "health_out".to_string(),
                connected_to: Some("effect1_in".to_string()),
                display_name: "Port 1".to_string(),
                connection_type: PortType::Variable,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "effect1".to_string(),
            title: "Show Warning".to_string(),
            initial_position: [300, 100],
            node_type: NodeType::Effect {
                dependencies: Vec::new(),
            },
            inputs: vec![Port {
                id: "effect1_in".to_string(),
                connected_to: Some("health_out".to_string()),
                display_name: "Port 1".to_string(),
                connection_type: PortType::Variable,
            }],
            outputs: vec![Port {
                id: "warning_out".to_string(),
                connected_to: Some("ui1_in".to_string()),
                display_name: "Port 2".to_string(),
                connection_type: PortType::Variable,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "ui1".to_string(),
            title: "Health Bar".to_string(),
            initial_position: [500, 100],
            node_type: NodeType::Context {
                name: "Health Context?".to_string(),
            },
            inputs: vec![Port {
                id: "ui1_in".to_string(),
                display_name: "Port 1".to_string(),
                connection_type: PortType::Variable,
                connected_to: Some("warning_out".to_string()),
            }],
            outputs: vec![],
            parent: None,
            children: Vec::new(),
        },
    ]
}
