use std::borrow::{Borrow, BorrowMut};
use std::rc::{Rc, Weak};
use std::sync::{Arc, Mutex, MutexGuard};

use bytemuck::Contiguous;
use cgmath::Vector4;
use editor_state::{EditorState, ObjectEdit, StateHelper, UIMessage};
use floem::common::{nav_button, option_button, small_button};
use floem::kurbo::Size;
use floem::window::WindowConfig;
use floem_renderer::gpu_resources::{self, GpuResources};
use floem_winit::dpi::{LogicalSize, PhysicalSize};
use floem_winit::event::{ElementState, KeyEvent, Modifiers, MouseButton, MouseScrollDelta};
use helpers::auth::read_auth_token;
use helpers::websocket::{Call, WebSocketManager};
use midpoint_engine::core::RendererState::{Point, RendererState, WindowSize};
use midpoint_engine::core::Viewport::Viewport;
use midpoint_engine::startup::{get_camera, handle_key_press, handle_mouse_move, Vertex};
use uuid::Uuid;
use views::app::app_view;
// use winit::{event_loop, window};
use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;
use wgpu::util::DeviceExt;

use floem::context::PaintState;
use floem::{Application, CustomRenderCallback};
use floem::{GpuHelper, View, WindowHandle};
use undo::{Edit, Record};

pub mod editor_state;
pub mod gql;
pub mod helpers;
pub mod views;

type RenderCallback<'a> = dyn for<'b> Fn(
        wgpu::CommandEncoder,
        wgpu::SurfaceTexture,
        wgpu::TextureView,
        wgpu::TextureView,
        &WindowHandle,
    ) + 'a;

fn create_render_callback<'a>() -> Box<RenderCallback<'a>> {
    Box::new(
        move |mut encoder: wgpu::CommandEncoder,
              frame: wgpu::SurfaceTexture,
              view: wgpu::TextureView,
              resolve_view: wgpu::TextureView,
              window_handle: &WindowHandle| {
            let mut handle = window_handle.borrow();

            let engine = handle
                .user_engine
                .as_ref()
                .expect("Couldn't get user engine")
                .lock()
                .unwrap();

            if let Some(gpu_resources) = &handle.gpu_resources {
                if engine.current_view == "scene".to_string() {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: Some(&resolve_view),
                            ops: wgpu::Operations {
                                // load: wgpu::LoadOp::Clear(wgpu::Color {
                                //     // grey background
                                //     r: 0.15,
                                //     g: 0.15,
                                //     b: 0.15,
                                //     // white background
                                //     // r: 1.0,
                                //     // g: 1.0,
                                //     // b: 1.0,
                                //     a: 1.0,
                                // }),
                                // load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        // depth_stencil_attachment: None,
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &handle
                                .gpu_helper
                                .as_ref()
                                .expect("Couldn't get gpu helper")
                                .lock()
                                .unwrap()
                                .depth_view
                                .as_ref()
                                .expect("Couldn't fetch depth view"), // This is the depth texture view
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0), // Clear to max depth
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None, // Set this if using stencil
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    // println!("Render frame...");

                    // Render partial screen content
                    // render_pass.set_viewport(100.0, 100.0, 200.0, 200.0, 0.0, 1.0);
                    render_pass.set_scissor_rect(
                        400,
                        0,
                        window_handle
                            .window_width
                            .expect("Couldn't get window width")
                            - 400,
                        window_handle
                            .window_height
                            .expect("Couldn't get window height"),
                    );

                    render_pass.set_pipeline(
                        &handle
                            .render_pipeline
                            .as_ref()
                            .expect("Couldn't fetch render pipeline"),
                    );

                    let viewport = engine.viewport.lock().unwrap();
                    let window_size = WindowSize {
                        width: viewport.width as u32,
                        height: viewport.height as u32,
                    };

                    let mut camera = get_camera();

                    // TODO: bad to call on every frame?
                    camera.update();

                    let camera_matrix = camera.view_projection_matrix;
                    gpu_resources.queue.write_buffer(
                        &engine.camera_uniform_buffer,
                        0,
                        bytemuck::cast_slice(camera_matrix.as_slice()),
                    );

                    // draw utility grids
                    for grid in &engine.grids {
                        render_pass.set_bind_group(0, &engine.camera_bind_group, &[]);
                        render_pass.set_bind_group(1, &grid.bind_group, &[]);
                        render_pass.set_bind_group(2, &grid.texture_bind_group, &[]);

                        render_pass.set_vertex_buffer(0, grid.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            grid.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );

                        render_pass.draw_indexed(0..grid.index_count, 0, 0..1);
                    }

                    // // draw pyramids
                    // for pyramid in &state.pyramids {
                    //     pyramid.update_uniform_buffer(&queue);
                    //     render_pass.set_bind_group(0, &camera_bind_group, &[]);
                    //     render_pass.set_bind_group(1, &pyramid.bind_group, &[]);

                    //     render_pass.set_vertex_buffer(0, pyramid.vertex_buffer.slice(..));
                    //     render_pass.set_index_buffer(pyramid.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

                    //     render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
                    // }

                    // draw cubes
                    for cube in &engine.cubes {
                        cube.transform.update_uniform_buffer(&gpu_resources.queue);
                        render_pass.set_bind_group(0, &engine.camera_bind_group, &[]);
                        render_pass.set_bind_group(1, &cube.bind_group, &[]);

                        render_pass.set_vertex_buffer(0, cube.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            cube.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint16,
                        );

                        render_pass.draw_indexed(0..cube.index_count as u32, 0, 0..1);
                    }

                    for model in &engine.models {
                        for mesh in &model.meshes {
                            mesh.transform.update_uniform_buffer(&gpu_resources.queue);
                            render_pass.set_bind_group(0, &engine.camera_bind_group, &[]);
                            render_pass.set_bind_group(1, &mesh.bind_group, &[]);
                            render_pass.set_bind_group(2, &mesh.texture_bind_group, &[]);

                            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(
                                mesh.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint16,
                            );

                            render_pass.draw_indexed(0..mesh.index_count as u32, 0, 0..1);
                        }
                    }

                    for landscape in &engine.landscapes {
                        if (landscape.texture_bind_group.is_some()) {
                            landscape
                                .transform
                                .update_uniform_buffer(&gpu_resources.queue);
                            render_pass.set_bind_group(0, &engine.camera_bind_group, &[]);
                            render_pass.set_bind_group(1, &landscape.bind_group, &[]);
                            render_pass.set_bind_group(
                                2,
                                &landscape
                                    .texture_bind_group
                                    .as_ref()
                                    .expect("No landscape texture bind group"),
                                &[],
                            );

                            render_pass.set_vertex_buffer(0, landscape.vertex_buffer.slice(..));
                            render_pass.set_index_buffer(
                                landscape.index_buffer.slice(..),
                                wgpu::IndexFormat::Uint32,
                            );

                            render_pass.draw_indexed(0..landscape.index_count as u32, 0, 0..1);
                        }
                    }
                }

                let command_buffer = encoder.finish();
                gpu_resources.queue.submit(Some(command_buffer));
                gpu_resources.device.poll(wgpu::Maintain::Poll);
                frame.present();
            } else {
                println!("GPU resources not available yet");
            }
            // }
        },
    )
}

fn handle_cursor_moved(
    mut editor_state: Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn Fn(f64, f64, f64, f64)>> {
    Some(Box::new(
        move |position_x: f64, position_y: f64, logPosX: f64, logPoxY: f64| {
            let mut editor_state = editor_state.lock().unwrap();

            if editor_state.mouse_state.is_first_mouse {
                editor_state.mouse_state.last_mouse_x = position_x as f64;
                editor_state.mouse_state.last_mouse_y = position_y as f64;
                editor_state.mouse_state.is_first_mouse = false;
                return;
            }

            let dx = position_x - editor_state.mouse_state.last_mouse_x as f64;
            let dy = position_y - editor_state.mouse_state.last_mouse_y as f64;

            editor_state.mouse_state.last_mouse_x = position_x;
            editor_state.mouse_state.last_mouse_y = position_y;

            // Only update camera if right mouse button is pressed
            if editor_state.mouse_state.right_mouse_pressed {
                // editor_state.update_camera_rotation(dx as f32, dy as f32);
                handle_mouse_move(dx as f32, dy as f32);
            }
        },
    ))
}

fn handle_mouse_input(
    mut editor_state: Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
    record: Arc<Mutex<Record<ObjectEdit>>>,
) -> Option<Box<dyn Fn(MouseButton, ElementState)>> {
    Some(Box::new(move |button, state| {
        let mut editor_state = editor_state.lock().unwrap();

        if button == MouseButton::Right {
            let edit_config = match state {
                ElementState::Pressed => editor_state.mouse_state.right_mouse_pressed = true,
                ElementState::Released => editor_state.mouse_state.right_mouse_pressed = false,
            };
        }

        // let mut editor_orig = Arc::clone(&editor);
        // let mut editor = editor.lock().unwrap();
        // let viewport = viewport.lock().unwrap();
        // let window_size = WindowSize {
        //     width: viewport.width as u32,
        //     height: viewport.height as u32,
        // };
        // if button == MouseButton::Left {
        //     let edit_config = match state {
        //         ElementState::Pressed => editor.handle_mouse_down(
        //             // mouse_position.0,
        //             // mouse_position.1,
        //             &window_size,
        //             &gpu_resources.device,
        //         ),
        //         ElementState::Released => editor.handle_mouse_up(),
        //     };

        //     drop(editor);

        //     // if (edit_config.is_some()) {
        //     //     let edit_config = edit_config.expect("Couldn't get polygon edit config");

        //     //     let mut editor_state = editor_state.lock().unwrap();

        //     //     let edit = PolygonEdit {
        //     //         polygon_id: edit_config.polygon_id,
        //     //         old_value: edit_config.old_value,
        //     //         new_value: edit_config.new_value,
        //     //         field_name: edit_config.field_name,
        //     //         signal: None,
        //     //     };

        //     //     let mut record_state = RecordState {
        //     //         editor: editor_orig,
        //     //         // record: Arc::clone(&record),
        //     //     };

        //     //     let mut record = record.lock().unwrap();
        //     //     record.edit(&mut record_state, edit);
        //     // }
        // }
    }))
}

fn handle_window_resize(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    gpu_helper: std::sync::Arc<Mutex<GpuHelper>>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(PhysicalSize<u32>, LogicalSize<f64>)>> {
    Some(Box::new(move |size, logical_size| {
        // let mut editor = editor.lock().unwrap();

        // let window_size = WindowSize {
        //     width: size.width,
        //     height: size.height,
        // };

        // let mut viewport = viewport.lock().unwrap();

        // viewport.width = size.width as f32;
        // viewport.height = size.height as f32;

        // let mut camera = editor.camera.expect("Couldn't get camera on resize");

        // camera.window_size.width = size.width;
        // camera.window_size.height = size.height;

        // editor.update_date_from_window_resize(&window_size, &gpu_resources.device);

        // gpu_helper
        //     .lock()
        //     .unwrap()
        //     .recreate_depth_view(&gpu_resources, &window_size);
    }))
}

fn handle_mouse_wheel(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(MouseScrollDelta)>> {
    Some(Box::new(move |delta: MouseScrollDelta| {
        // let mut editor = editor.lock().unwrap();

        // let mouse_pos = Point {
        //     x: editor.global_top_left.x,
        //     y: editor.global_top_left.y,
        // };

        // match delta {
        //     MouseScrollDelta::LineDelta(_x, y) => {
        //         // y is positive for scrolling up/away from user
        //         // negative for scrolling down/toward user
        //         // let zoom_factor = if y > 0.0 { 1.1 } else { 0.9 };
        //         editor.handle_wheel(y, mouse_pos, &gpu_resources.queue);
        //     }
        //     MouseScrollDelta::PixelDelta(pos) => {
        //         // Convert pixel delta if needed
        //         let y = pos.y as f32;
        //         // let zoom_factor = if y > 0.0 { 1.1 } else { 0.9 };
        //         editor.handle_wheel(y, mouse_pos, &gpu_resources.queue);
        //     }
        // }
    }))
}

fn handle_modifiers_changed(
    editor_state: std::sync::Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(Modifiers)>> {
    Some(Box::new(move |modifiers: Modifiers| {
        let mut editor_state = editor_state.lock().unwrap();
        println!("modifiers changed");
        let modifier_state = modifiers.state();
        editor_state.current_modifiers = modifier_state;
    }))
}

use floem_winit::keyboard::NamedKey;
use floem_winit::keyboard::{Key, SmolStr};

fn handle_keyboard_input(
    editor_state: Arc<Mutex<EditorState>>,
    gpu_resources: std::sync::Arc<GpuResources>,
    viewport: std::sync::Arc<Mutex<Viewport>>,
) -> Option<Box<dyn FnMut(KeyEvent)>> {
    Some(Box::new(move |event: KeyEvent| {
        if event.state != ElementState::Pressed {
            return;
        }

        let mut editor_state = editor_state.lock().unwrap();
        // Check for Ctrl+Z (undo)
        let modifiers = editor_state.current_modifiers;

        let logical_key_text = event.logical_key.to_text().unwrap_or_default();
        match logical_key_text {
            "z" => {
                if modifiers.control_key() {
                    if modifiers.shift_key() {
                        editor_state.redo(); // Ctrl+Shift+Z
                    } else {
                        editor_state.undo(); // Ctrl+Z
                    }
                }
            }
            "y" => {
                if modifiers.control_key() {
                    editor_state.redo(); // Ctrl+Y
                }
            }
            _ => {}
        }

        handle_key_press(
            Arc::clone(&editor_state.renderer_state),
            logical_key_text,
            true,
        );
    }))
}

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Initialize logging
    // tracing::fmt::init();

    let auth_token = read_auth_token();

    // TODO: show alert if auth_token is empty
    println!("auth_token {:?}", auth_token);

    let app = Application::new();

    // Get the primary monitor's size
    let monitor = app.primary_monitor().expect("Couldn't get primary monitor");
    let monitor_size = monitor.size();

    // Calculate a reasonable window size (e.g., 80% of the screen size)
    let window_width = (monitor_size.width.into_integer() as f32 * 0.8) as u32;
    let window_height = (monitor_size.height.into_integer() as f32 * 0.8) as u32;

    let window_size = WindowSize {
        width: window_width,
        height: window_height,
    };

    let mut gpu_helper = Arc::new(Mutex::new(GpuHelper::new()));
    let mut state_helper = Arc::new(Mutex::new(StateHelper::new(auth_token)));

    let state_2 = Arc::clone(&state_helper);
    let state_3 = Arc::clone(&state_helper);
    let state_4 = Arc::clone(&state_helper);

    let gpu_cloned = Arc::clone(&gpu_helper);
    let gpu_cloned2 = Arc::clone(&gpu_helper);

    let viewport = Arc::new(Mutex::new(Viewport::new(
        window_size.width as f32,
        window_size.height as f32,
    )));

    let viewport_2 = Arc::clone(&viewport);
    let viewport_3 = Arc::clone(&viewport);
    let viewport_4 = Arc::clone(&viewport);

    let record: Arc<Mutex<Record<ObjectEdit>>> = Arc::new(Mutex::new(Record::new()));

    let record_2 = Arc::clone(&record);

    let mut manager = WebSocketManager::new();

    if let Err(e) = manager
        .connect(state_3, {
            let state_helper = state_4.clone();

            move |signals_category, signal_name, signal_value| {
                // main thread!? no.
                println!(
                    "Handling WebSocket message: {} {}",
                    signals_category, signal_name
                );
            }
        })
        .await
    {
        eprintln!("Failed to connect: {:?}", e);
        // return;
    }

    let manager = Arc::new(manager);

    // // Disconnect when done
    // manager.disconnect();

    let (mut app, window_id) = app.window(
        move |_| {
            app_view(
                // Arc::clone(&editor_state),
                // Arc::clone(&editor),
                Arc::clone(&state_helper),
                Arc::clone(&gpu_helper),
                Arc::clone(&viewport),
                Arc::clone(&manager),
            )
        },
        Some(
            WindowConfig::default()
                .size(Size::new(
                    window_size.width as f64,
                    window_size.height as f64,
                ))
                .title("CommonOS Midpoint"),
        ),
    );

    let window_id = window_id.expect("Couldn't get window id");

    {
        let app_handle = app.handle.as_mut().expect("Couldn't get handle");
        let window_handle = app_handle
            .window_handles
            .get_mut(&window_id)
            .expect("Couldn't get window handle");

        // Create and set the render callback
        let render_callback = create_render_callback();

        window_handle.set_encode_callback(render_callback);
        // window_handle.window_size = Some(window_size);
        window_handle.window_width = Some(window_width);
        window_handle.window_height = Some(window_height);

        println!("Ready...");

        // window_handle.user_editor = Some(cloned); // set engine after pipeline setup

        // Receive and store GPU resources
        // match &mut window_handle.paint_state {
        //     PaintState::PendingGpuResources { rx, .. } =>
        if let PaintState::PendingGpuResources { rx, .. } = &mut window_handle.paint_state {
            async {
                let gpu_resources = Arc::new(rx.recv().unwrap().unwrap());

                println!("Initializing pipeline...");

                // let camera = Camera::new(window_size);
                // let camera_binding = CameraBinding::new(&gpu_resources.device);

                // editor.camera = Some(camera);
                // editor.camera_binding = Some(camera_binding);

                let camera = get_camera();

                camera.update_aspect_ratio(window_width as f32 / window_height as f32);
                camera.update_view_projection_matrix();

                let camera_matrix = camera.view_projection_matrix;
                let camera_uniform_buffer =
                    gpu_resources
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Camera Uniform Buffer"),
                            contents: bytemuck::cast_slice(camera_matrix.as_slice()),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });

                let camera_uniform_buffer = Arc::new(camera_uniform_buffer);

                // let sampler = gpu_resources
                //     .device
                //     .create_sampler(&wgpu::SamplerDescriptor {
                //         address_mode_u: wgpu::AddressMode::ClampToEdge,
                //         address_mode_v: wgpu::AddressMode::ClampToEdge,
                //         mag_filter: wgpu::FilterMode::Linear,
                //         min_filter: wgpu::FilterMode::Linear,
                //         mipmap_filter: wgpu::FilterMode::Nearest,
                //         ..Default::default()
                //     });

                gpu_cloned.lock().unwrap().recreate_depth_view(
                    &gpu_resources,
                    window_width,
                    window_height,
                );

                // let depth_stencil_state = wgpu::DepthStencilState {
                //     format: wgpu::TextureFormat::Depth24Plus,
                //     depth_write_enabled: true,
                //     depth_compare: wgpu::CompareFunction::Less,
                //     stencil: wgpu::StencilState::default(),
                //     bias: wgpu::DepthBiasState::default(),
                // };

                // // let camera_binding = editor
                // //     .camera_binding
                // //     .as_ref()
                // //     .expect("Couldn't get camera binding");

                // // Define the layouts
                // let pipeline_layout =
                //     gpu_resources
                //         .device
                //         .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                //             label: Some("Pipeline Layout"),
                //             // bind_group_layouts: &[&bind_group_layout],
                //             bind_group_layouts: &[], // No bind group layouts
                //             push_constant_ranges: &[],
                //         });

                // Create the bind group for the uniform buffer
                let camera_bind_group_layout = gpu_resources.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        label: Some("Camera Bind Group Layout"),
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                    },
                );

                let model_bind_group_layout = gpu_resources.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        entries: &[wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }],
                        label: Some("model_bind_group_layout"),
                    },
                );

                let model_bind_group_layout = Arc::new(model_bind_group_layout);

                let texture_bind_group_layout = gpu_resources.device.create_bind_group_layout(
                    &wgpu::BindGroupLayoutDescriptor {
                        entries: &[
                            wgpu::BindGroupLayoutEntry {
                                binding: 0,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Texture {
                                    multisampled: false,
                                    view_dimension: wgpu::TextureViewDimension::D2Array,
                                    sample_type: wgpu::TextureSampleType::Float {
                                        filterable: true,
                                    },
                                },
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 1,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                                count: None,
                            },
                            wgpu::BindGroupLayoutEntry {
                                binding: 2,
                                visibility: wgpu::ShaderStages::FRAGMENT,
                                ty: wgpu::BindingType::Buffer {
                                    ty: wgpu::BufferBindingType::Uniform,
                                    has_dynamic_offset: false,
                                    min_binding_size: None,
                                },
                                count: None,
                            },
                        ],
                        label: Some("Texture Bind Group Layout"),
                    },
                );

                let texture_bind_group_layout = Arc::new(texture_bind_group_layout);

                let camera_bind_group =
                    gpu_resources
                        .device
                        .create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &camera_bind_group_layout,
                            entries: &[wgpu::BindGroupEntry {
                                binding: 0,
                                resource: camera_uniform_buffer.as_entire_binding(),
                            }],
                            label: Some("Camera Bind Group"),
                        });

                let camera_bind_group = Arc::new(camera_bind_group);

                let color_render_mode_buffer =
                    gpu_resources
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Color Render Mode Buffer"),
                            contents: bytemuck::cast_slice(&[0i32]), // Default to normal mode
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });

                let color_render_mode_buffer = Arc::new(color_render_mode_buffer);

                let texture_render_mode_buffer =
                    gpu_resources
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Texture Render Mode Buffer"),
                            contents: bytemuck::cast_slice(&[1i32]), // Default to text mode
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });

                let texture_render_mode_buffer = Arc::new(texture_render_mode_buffer);

                let pipeline_layout =
                    gpu_resources
                        .device
                        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                            label: Some("Render Pipeline Layout"),
                            bind_group_layouts: &[
                                &camera_bind_group_layout,
                                &model_bind_group_layout,
                                &texture_bind_group_layout,
                            ],
                            push_constant_ranges: &[],
                        });

                // let depth_texture = gpu_resources
                //     .device
                //     .create_texture(&wgpu::TextureDescriptor {
                //         size: wgpu::Extent3d {
                //             width: window_width,
                //             height: window_height,
                //             depth_or_array_layers: 1,
                //         },
                //         mip_level_count: 1,
                //         sample_count: 1,
                //         dimension: wgpu::TextureDimension::D2,
                //         format: wgpu::TextureFormat::Depth24Plus,
                //         usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                //             | wgpu::TextureUsages::TEXTURE_BINDING,
                //         label: Some("Depth Texture"),
                //         view_formats: &[],
                //     });

                // let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

                let depth_stencil_state = wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                };

                // Load the shaders
                let shader_module_vert_primary =
                    gpu_resources
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Primary Vert Shader"),
                            source: wgpu::ShaderSource::Wgsl(
                                include_str!("shaders/primary_vertex.wgsl").into(),
                            ),
                        });

                let shader_module_frag_primary =
                    gpu_resources
                        .device
                        .create_shader_module(wgpu::ShaderModuleDescriptor {
                            label: Some("Primary Frag Shader"),
                            source: wgpu::ShaderSource::Wgsl(
                                include_str!("shaders/primary_fragment.wgsl").into(),
                            ),
                        });

                // let swapchain_capabilities = gpu_resources
                //     .surface
                //     .get_capabilities(&gpu_resources.adapter);
                // let swapchain_format = swapchain_capabilities.formats[0]; // Choosing the first available format
                let swapchain_format = wgpu::TextureFormat::Bgra8UnormSrgb; // hardcode for now

                // Configure the render pipeline
                let render_pipeline =
                    gpu_resources
                        .device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("Midpoint Primary Render Pipeline"),
                            layout: Some(&pipeline_layout),
                            multiview: None,
                            cache: None,
                            vertex: wgpu::VertexState {
                                module: &shader_module_vert_primary,
                                entry_point: "main", // name of the entry point in your vertex shader
                                buffers: &[Vertex::desc()], // Make sure your Vertex::desc() matches your vertex structure
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader_module_frag_primary,
                                entry_point: "main", // name of the entry point in your fragment shader
                                targets: &[Some(wgpu::ColorTargetState {
                                    format: swapchain_format,
                                    // blend: Some(wgpu::BlendState::REPLACE),
                                    blend: Some(wgpu::BlendState {
                                        color: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::SrcAlpha,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                        alpha: wgpu::BlendComponent {
                                            src_factor: wgpu::BlendFactor::One,
                                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                            operation: wgpu::BlendOperation::Add,
                                        },
                                    }),
                                    write_mask: wgpu::ColorWrites::ALL,
                                })],
                                compilation_options: wgpu::PipelineCompilationOptions::default(),
                            }),
                            // primitive: wgpu::PrimitiveState::default(),
                            // depth_stencil: None,
                            // multisample: wgpu::MultisampleState::default(),
                            primitive: wgpu::PrimitiveState {
                                conservative: false,
                                topology: wgpu::PrimitiveTopology::TriangleList, // how vertices are assembled into geometric primitives
                                // strip_index_format: Some(wgpu::IndexFormat::Uint32),
                                strip_index_format: None,
                                front_face: wgpu::FrontFace::Ccw, // Counter-clockwise is considered the front face
                                // none cull_mode
                                cull_mode: None,
                                polygon_mode: wgpu::PolygonMode::Fill,
                                // Other properties such as conservative rasterization can be set here
                                unclipped_depth: false,
                            },
                            depth_stencil: Some(depth_stencil_state), // Optional, only if you are using depth testing
                            multisample: wgpu::MultisampleState {
                                count: 4, // effect performance
                                mask: !0,
                                alpha_to_coverage_enabled: false,
                            },
                        });

                window_handle.render_pipeline = Some(render_pipeline);
                // window_handle.depth_view = gpu_helper.depth_view;

                println!("Initialized...");

                let state = RendererState::new(
                    viewport_2.clone(),
                    &gpu_resources.device,
                    &gpu_resources.queue,
                    model_bind_group_layout.clone(),
                    texture_bind_group_layout.clone(),
                    texture_render_mode_buffer.clone(),
                    color_render_mode_buffer.clone(),
                    camera_uniform_buffer.clone(),
                    camera_bind_group.clone(),
                )
                .await;

                let renderer_state = Arc::new(Mutex::new(state));

                let renderer_state_2 = Arc::clone(&renderer_state);
                let renderer_state_3 = Arc::clone(&renderer_state);

                // initialize_renderer_state(state);

                let mut state_helper = state_2.lock().unwrap();

                state_helper.renderer_state = Some(renderer_state_3);

                let editor_state = Arc::new(Mutex::new(EditorState::new(renderer_state, record)));

                window_handle.user_engine = Some(renderer_state_2);

                window_handle.handle_cursor_moved = handle_cursor_moved(
                    editor_state.clone(),
                    gpu_resources.clone(),
                    viewport_3.clone(),
                );
                window_handle.handle_mouse_input = handle_mouse_input(
                    editor_state.clone(),
                    gpu_resources.clone(),
                    viewport_4.clone(),
                    record_2.clone(),
                );
                // window_handle.handle_window_resized = handle_window_resize(
                //     cloned7,
                //     gpu_resources.clone(),
                //     gpu_cloned3,
                //     cloned_viewport3.clone(),
                // );
                // window_handle.handle_mouse_wheel =
                //     handle_mouse_wheel(cloned11, gpu_resources.clone(), cloned_viewport3.clone());
                // window_handle.handle_modifiers_changed = handle_modifiers_changed(
                //     state_3,
                //     gpu_resources.clone(),
                //     cloned_viewport3.clone(),
                // );
                window_handle.handle_keyboard_input = handle_keyboard_input(
                    editor_state.clone(),
                    gpu_resources.clone(),
                    viewport_4.clone(),
                );

                // // *** TODO: Test Scene *** //

                // editor.update_camera_binding(&gpu_resources.queue);

                gpu_cloned2.lock().unwrap().gpu_resources = Some(Arc::clone(&gpu_resources));
                // editor.gpu_resources = Some(Arc::clone(&gpu_resources));
                window_handle.gpu_resources = Some(gpu_resources);
                window_handle.gpu_helper = Some(gpu_cloned);
                // editor.window = window_handle.window.clone();
            }
            .await;
        }
        //     PaintState::Initialized { .. } => async {
        //         println!("Renderer is already initialized");
        //     }
        // }
    }

    app.run();
}
