use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use midpoint_engine::core::RendererState::RendererState;
use midpoint_engine::core::RendererState::{ObjectConfig, ObjectProperty};
use midpoint_engine::floem::keyboard::ModifiersState;
use midpoint_engine::floem::reactive::{RwSignal, SignalUpdate};
use midpoint_engine::helpers::saved_data::{File, LandscapeData, SavedState};
use tokio::sync::mpsc::UnboundedSender;
use undo::Edit;
use undo::Record;
use uuid::Uuid;

#[derive(Debug)]
pub struct ObjectEdit {
    pub object_id: Uuid,
    pub field_name: String,
    pub old_value: ObjectProperty,
    pub new_value: ObjectProperty,
    pub signal: Option<RwSignal<String>>,
}

impl Edit for ObjectEdit {
    type Target = RecordState;
    type Output = ();

    fn edit(&mut self, record_state: &mut RecordState) {
        let mut renderer_state = record_state.renderer_state.lock().unwrap();

        match &self.new_value {
            ObjectProperty::Width(w) => {
                // editor.update_object(self.object_id, "width", InputValue::Number(*w));

                // let mut width = w.to_string();
                // self.signal.expect("signal error").set(width);
            }
        }
    }

    fn undo(&mut self, record_state: &mut RecordState) {
        let mut renderer_state = record_state.renderer_state.lock().unwrap();

        match &self.old_value {
            ObjectProperty::Width(w) => {
                // editor.update_object(self.object_id, "width", InputValue::Number(*w));

                // let mut width = w.to_string();
                // self.signal.expect("signal error").set(width);
            }
        }
    }
}

pub struct MouseState {
    pub is_first_mouse: bool,
    pub last_mouse_x: f64,
    pub last_mouse_y: f64,
    pub right_mouse_pressed: bool,
}

pub struct EditorState {
    pub renderer_state: Arc<Mutex<RendererState>>,
    pub record: Arc<Mutex<Record<ObjectEdit>>>,
    pub record_state: RecordState,
    pub object_selected: bool,
    pub selected_object_id: Uuid,
    pub value_signals: Arc<Mutex<HashMap<String, RwSignal<String>>>>,
    pub current_modifiers: ModifiersState,
    pub mouse_state: MouseState,
}

pub struct RecordState {
    pub renderer_state: Arc<Mutex<RendererState>>,
    // pub record: Arc<Mutex<Record<ObjectEdit>>>,
}

impl EditorState {
    pub fn new(
        renderer_state: Arc<Mutex<RendererState>>,
        record: Arc<Mutex<Record<ObjectEdit>>>,
    ) -> Self {
        Self {
            renderer_state: Arc::clone(&renderer_state),
            record: Arc::clone(&record),
            record_state: RecordState {
                renderer_state: Arc::clone(&renderer_state),
                // record: Arc::clone(&record),
            },
            object_selected: false,
            selected_object_id: Uuid::nil(),
            value_signals: Arc::new(Mutex::new(HashMap::new())),
            current_modifiers: ModifiersState::empty(),
            mouse_state: MouseState {
                last_mouse_x: 0.0,
                last_mouse_y: 0.0,
                is_first_mouse: true,
                right_mouse_pressed: false,
            },
        }
    }

    // Helper method to register a new signal
    pub fn register_signal(&mut self, name: String, signal: RwSignal<String>) {
        let mut signals = self.value_signals.lock().unwrap();
        signals.insert(name + &self.selected_object_id.to_string(), signal);
    }

    // pub fn update_width(&mut self, new_width_str: &str) -> Result<(), String> {
    //     let new_width =
    //         string_to_f32(new_width_str).map_err(|_| "Couldn't convert string to f32")?;

    //     let old_width = {
    //         let editor = self.record_state.editor.lock().unwrap();
    //         editor.get_object_width(self.selected_object_id)
    //     };

    //     let edit = ObjectEdit {
    //         object_id: self.selected_object_id,
    //         old_value: ObjectProperty::Width(old_width),
    //         new_value: ObjectProperty::Width(new_width),
    //         field_name: "width".to_string(),
    //         signal: Some(
    //             self.value_signals
    //                 .lock()
    //                 .unwrap()
    //                 .get(&format!("width{}", self.selected_object_id))
    //                 .cloned()
    //                 .expect("Couldn't get width value signal"),
    //         ),
    //     };

    //     let mut record = self.record.lock().unwrap();
    //     record.edit(&mut self.record_state, edit);

    //     Ok(())
    // }

    pub fn undo(&mut self) {
        let mut record = self.record.lock().unwrap();

        if record.undo(&mut self.record_state).is_some() {
            println!("Undo successful");
        }
    }

    pub fn redo(&mut self) {
        let mut record = self.record.lock().unwrap();

        if record.redo(&mut self.record_state).is_some() {
            println!("Redo successful");
        }
    }
}

pub struct NamedSignals {
    pub texture_browser: Option<RwSignal<Vec<File>>>,
    pub model_browser: Option<RwSignal<Vec<File>>>,
    pub landscape_browser: Option<RwSignal<Vec<LandscapeData>>>,
    pub concept_browser: Option<RwSignal<Vec<File>>>,
}

pub struct StateHelper {
    pub renderer_state: Option<Arc<Mutex<RendererState>>>,
    pub saved_state: Option<Arc<Mutex<SavedState>>>,
    pub project_selected_signal: Option<RwSignal<Uuid>>,
    pub auth_token: String,
    // pub simple_singals: Arc<Mutex<HashMap<String, RwSignal<String>>>>
    // pub named_signals: Arc<Mutex<NamedSignals>>,
    pub file_signals: Arc<Mutex<HashMap<String, Arc<UnboundedSender<UIMessage>>>>>,
    pub object_selected_signal: Option<RwSignal<bool>>,
    pub selected_object_id_signal: Option<RwSignal<Uuid>>,
    pub selected_object_data_signal: Option<RwSignal<ObjectConfig>>,
}

#[derive(Clone, Debug)]
pub enum UIMessage {
    UpdateTextures(Vec<File>),
    AddTexture(File),
    // ... other UI updates
}

impl StateHelper {
    pub fn new(auth_token: String) -> Self {
        Self {
            renderer_state: None,
            saved_state: None,
            project_selected_signal: None,
            auth_token,
            file_signals: Arc::new(Mutex::new(HashMap::new())),
            object_selected_signal: None,
            selected_object_id_signal: None,
            selected_object_data_signal: None,
        }
    }

    // Helper method to register a new signal
    pub fn register_file_signal(&mut self, name: String, signal: Arc<UnboundedSender<UIMessage>>) {
        let mut signals = self.file_signals.lock().unwrap();
        signals.insert(name, signal);
    }
}
