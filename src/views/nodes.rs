use midpoint_engine::floem::event::EventListener;
use midpoint_engine::floem::event::EventPropagation;
use midpoint_engine::floem::peniko::Color;
use midpoint_engine::floem::reactive::SignalGet;
use midpoint_engine::floem::reactive::SignalUpdate;
use midpoint_engine::floem::reactive::*;
use midpoint_engine::floem::style::CursorStyle;
use midpoint_engine::floem::taffy::FlexDirection;
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

use crate::helpers::nodes::NodeComponent;
use crate::helpers::nodes::NodeInputs;
use crate::helpers::nodes::NodeType;
use crate::helpers::nodes::Port;

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
                let connected_to = port.connected_to;

                let all_nodes = all_nodes.get();
                let connected_to_port = if connected_to.is_some() {
                    let connected_to = connected_to.as_ref().expect("Couldn't get connected_to");
                    if port.is_output {
                        all_nodes.iter().find_map(|n| {
                            n.ui_inputs.iter().find(|i| i.id == *connected_to).cloned()
                        })
                    } else {
                        all_nodes.iter().find_map(|n| {
                            n.ui_outputs.iter().find(|i| i.id == *connected_to).cloned()
                        })
                    }
                } else {
                    None
                };

                h_stack((
                    if left {
                        empty().into_any()
                    } else {
                        label(move || port.display_name.clone())
                            .style(|s| s.selectable(false))
                            .into_any()
                    },
                    if connected_to_port.is_some() {
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
                            move |s| {
                                s.background(node.get_type_color())
                                    .color(Color::WHITE_SMOKE)
                                    .margin_bottom(2.0)
                            }
                        })
                        .style(|s| s.selectable(false))
                        .into_any()
                    } else {
                        empty().into_any()
                    },
                    if left {
                        label(move || display_name_2.clone())
                            .style(|s| s.selectable(false))
                            .into_any()
                    } else {
                        empty().into_any()
                    },
                ))
            }),
        ))
        .style(|s| s.flex_direction(FlexDirection::Column)),
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
    let inputs = node.ui_inputs.clone();
    let outputs = node.ui_outputs.clone();
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

pub fn create_test_nodes() -> Vec<NodeComponent> {
    vec![
        // setting up state
        NodeComponent {
            id: "stringa".to_string(),
            title: "Name (String)".to_string(),
            initial_position: [50, 50],
            node_type: NodeType::String,
            ui_inputs: vec![],
            ui_outputs: vec![Port {
                id: "string_value1".to_string(),
                input_name: Some("value".to_string()),
                connected_to: Some("name_54".to_string()),
                display_name: "String Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "valuea".to_string(),
            title: "Value (String)".to_string(),
            initial_position: [250, 50],
            node_type: NodeType::String,
            ui_inputs: vec![],
            ui_outputs: vec![Port {
                id: "string_value2".to_string(),
                input_name: Some("value".to_string()),
                connected_to: Some("value_54".to_string()),
                display_name: "String Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "datatypea".to_string(),
            title: "Data Type (DateType)".to_string(),
            initial_position: [450, 50],
            node_type: NodeType::DataType,
            ui_inputs: vec![],
            ui_outputs: vec![Port {
                id: "datatype_value2".to_string(),
                input_name: Some("datatype".to_string()),
                connected_to: Some("datatype_54".to_string()),
                display_name: "Data Type Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "persistenta".to_string(),
            title: "Persistent (Boolean)".to_string(),
            initial_position: [650, 50],
            node_type: NodeType::Boolean,
            ui_inputs: vec![],
            ui_outputs: vec![Port {
                id: "bool_value2".to_string(),
                input_name: Some("boolean".to_string()),
                connected_to: Some("persistent_54".to_string()),
                display_name: "Boolean Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "state1".to_string(),
            title: "Player Health (ReactiveState)".to_string(),
            initial_position: [100, 200],
            node_type: NodeType::ReactiveState,
            // node_inputs: NodeInputs::State {
            //     name: "Health State".to_string(),
            //     value: "80".to_string(),
            //     data_type: crate::helpers::nodes::DataType::String,
            //     persistent: false,
            // },
            ui_inputs: vec![
                Port {
                    id: "name_54".to_string(),
                    input_name: Some("name".to_string()),
                    connected_to: Some("stringa".to_string()),
                    display_name: "Name".to_string(),
                    // connection_type: PortType::Variable,
                    is_output: false,
                },
                Port {
                    id: "value_54".to_string(),
                    input_name: Some("value".to_string()),
                    connected_to: Some("string_value2".to_string()),
                    display_name: "Value".to_string(),
                    // connection_type: PortType::Variable,
                    is_output: false,
                },
                Port {
                    id: "datatype_54".to_string(),
                    input_name: Some("data_type".to_string()),
                    connected_to: Some("datatype_value2".to_string()),
                    display_name: "Data Type".to_string(),
                    // connection_type: PortType::Variable,
                    is_output: false,
                },
                Port {
                    id: "persistent_54".to_string(),
                    input_name: Some("persistent".to_string()),
                    connected_to: Some("bool_value2".to_string()),
                    display_name: "Persistent".to_string(),
                    // connection_type: PortType::Variable,
                    is_output: false,
                },
            ],
            ui_outputs: vec![Port {
                id: "health_out".to_string(),
                input_name: None,
                connected_to: Some("dependencies1_in".to_string()),
                display_name: "ReactiveState Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        // setting up effect
        NodeComponent {
            id: "execution_order1a".to_string(),
            title: "Execution Order (Integer)".to_string(),
            initial_position: [50, 250],
            node_type: NodeType::Integer,
            ui_inputs: vec![],
            ui_outputs: vec![Port {
                id: "integer1".to_string(),
                input_name: Some("integer".to_string()),
                connected_to: Some("exec_order1_in".to_string()),
                display_name: "Integer Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "parallel1b".to_string(),
            title: "Parallel (Boolean)".to_string(),
            initial_position: [250, 250],
            node_type: NodeType::Boolean,
            ui_inputs: vec![],
            ui_outputs: vec![Port {
                id: "bool45".to_string(),
                input_name: Some("boolean".to_string()),
                connected_to: Some("parallel1_in".to_string()),
                display_name: "Boolean Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "effect1".to_string(),
            title: "Detect Warning (Effect)".to_string(),
            initial_position: [350, 500],
            node_type: NodeType::Effect,
            // node_inputs: NodeInputs::Effect {
            //     dependencies: Vec::new(),
            //     execution_order: 0,
            //     parallel: false,
            // },
            ui_inputs: vec![
                //     Port {
                //     id: "effect1_in".to_string(),
                //     input_name: Some("TBD".to_string()),
                //     connected_to: Some("health_out".to_string()),
                //     display_name: "Effect 1 In".to_string(),
                //     is_output: false,
                // }
                Port {
                    id: "dependencies1_in".to_string(),
                    input_name: Some("dependencies".to_string()),
                    connected_to: Some("health_out".to_string()), // connects to State
                    display_name: "Dependencies".to_string(),
                    is_output: false,
                },
                Port {
                    id: "exec_order1_in".to_string(),
                    input_name: Some("exec_order".to_string()),
                    connected_to: Some("integer1".to_string()),
                    display_name: "Exec. Order".to_string(),
                    is_output: false,
                },
                Port {
                    id: "parallel1_in".to_string(),
                    input_name: Some("parallel".to_string()),
                    connected_to: Some("bool45".to_string()),
                    display_name: "Parallel".to_string(),
                    is_output: false,
                },
            ],
            ui_outputs: vec![Port {
                id: "warning_out".to_string(),
                input_name: None,
                connected_to: Some("disval1_in".to_string()),
                display_name: "On Effect".to_string(),
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        // displaying to ui
        NodeComponent {
            id: "elemtyp23".to_string(),
            title: "Element Type (Elem. Type)".to_string(),
            initial_position: [50, 350],
            node_type: NodeType::UIElementType,
            ui_inputs: vec![],
            ui_outputs: vec![Port {
                id: "elemtypeoutput2".to_string(),
                input_name: Some("elem_type".to_string()),
                connected_to: Some("eltype1_in".to_string()),
                display_name: "Elem. Type Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "stylehere2".to_string(),
            title: "Bar Style (Style)".to_string(),
            initial_position: [50, 350],
            node_type: NodeType::Style,
            ui_inputs: vec![],
            ui_outputs: vec![Port {
                id: "styleout".to_string(),
                input_name: Some("style".to_string()),
                connected_to: Some("style1_in".to_string()),
                display_name: "Style Output".to_string(),
                // connection_type: PortType::Variable,
                is_output: true,
            }],
            parent: None,
            children: Vec::new(),
        },
        NodeComponent {
            id: "ui1".to_string(),
            title: "Update Health Bar (UI)".to_string(),
            initial_position: [600, 500],
            node_type: NodeType::UI,
            // node_inputs: NodeInputs::UI {
            //     element_type: crate::helpers::nodes::UIElementType::ProgressBar,
            //     layout: crate::helpers::nodes::LayoutType::Absolute,
            //     style: "temp_style".to_string(),
            // },
            ui_inputs: vec![
                //     Port {
                //     id: "ui1_in".to_string(),
                //     input_name: Some("TBD".to_string()),
                //     display_name: "UI In".to_string(),
                //     connected_to: Some("warning_out".to_string()),
                //     is_output: false,
                // }
                Port {
                    id: "disval1_in".to_string(),
                    input_name: Some("display_value".to_string()),
                    display_name: "Display Value".to_string(),
                    connected_to: Some("warning_out".to_string()),
                    is_output: false,
                },
                Port {
                    id: "eltype1_in".to_string(),
                    input_name: Some("element_type".to_string()),
                    display_name: "Element Type".to_string(),
                    connected_to: Some("elemtypeoutput2".to_string()),
                    is_output: false,
                },
                Port {
                    id: "style1_in".to_string(),
                    input_name: Some("style".to_string()),
                    display_name: "Style".to_string(),
                    connected_to: Some("styleout".to_string()),
                    is_output: false,
                },
            ],
            ui_outputs: vec![],
            parent: None,
            children: Vec::new(),
        },
    ]
}
