use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use floem::keyboard::ModifiersState;
use floem::reactive::{RwSignal, SignalUpdate};
use midpoint_engine::core::RendererState::ObjectProperty;
use midpoint_engine::core::RendererState::RendererState;
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

pub struct EditorState {
    pub renderer_state: Arc<Mutex<RendererState>>,
    pub record: Arc<Mutex<Record<ObjectEdit>>>,
    pub record_state: RecordState,
    pub object_selected: bool,
    pub selected_object_id: Uuid,
    pub value_signals: Arc<Mutex<HashMap<String, RwSignal<String>>>>,
    pub current_modifiers: ModifiersState,
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
