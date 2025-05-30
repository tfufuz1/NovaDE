use glib::prelude::*;
use glib::subclass::prelude::*;
use glib::{Object, ParamSpec, Properties, Value}; // Added ParamSpec, Properties, Value
use std::cell::RefCell;
// Removed Rc as UIState itself will be a GObject (which is ref-counted)

// Placeholder for window information (can remain as a simple struct)
#[derive(Debug, Clone, Default)]
pub struct WindowInfo {
    pub id: String,
    pub title: String,
    pub is_active: bool,
}

mod imp {
    use super::*;
    use std::cell::Cell; // For simple mutable fields like u32

    #[derive(Properties, Debug, Default)]
    #[properties(wrapper_type = super::UIState)]
    pub struct UIStatePriv {
        #[property(get, set, name = "window-count", nick = "Window Count", blurb = "Number of open windows", minimum = 0, maximum = 1000, default = 0)]
        pub window_count: Cell<u32>,

        // You could have more complex fields wrapped in RefCell if they need interior mutability
        // and are not simple Copy types. For example:
        // pub open_windows: RefCell<Vec<super::WindowInfo>>,
        // pub active_username: RefCell<Option<String>>,
        // However, for properties exposed to GObject system, they typically need to be
        // gettable/settable via GValue, which is simpler for basic types or GObjects.
        // For Vec<WindowInfo>, you might expose methods to modify it and then emit a
        // notification signal if other parts of UI need to react to changes in the Vec itself,
        // or expose a GObjectListModel.
        // For this example, we focus on window_count.
        pub other_internal_data: RefCell<String>, // Example of other data not directly exposed as property
    }

    #[glib::object_subclass]
    impl ObjectSubclass for UIStatePriv {
        const NAME: &'static str = "NovaUIState";
        type Type = super::UIState;
        type ParentType = glib::Object;

        // You can initialize default values for properties here if needed,
        // though #[property(default = ...)] often handles it.
        // fn new() -> Self {
        //     Self {
        //         window_count: Cell::new(0),
        //         other_internal_data: RefCell::new("Initial internal data".to_string()),
        //     }
        // }
    }

    impl ObjectImpl for UIStatePriv {
        // If you had custom signals, you'd define them here.
        // fn signals() -> &'static [glib::subclass::Signal] {
        //     static SIGNALS: std::sync::OnceLock<Box<[glib::subclass::Signal]>> = std::sync::OnceLock::new();
        //     SIGNALS.get_or_init(|| Box::from([]))
        // }

        // If you override constructed, you can do post-construction setup
        // fn constructed(&self) {
        //    self.parent_constructed(); // Always call parent
        //    // ... your setup
        // }
    }
}

glib::wrapper! {
    pub struct UIState(ObjectSubclass<imp::UIStatePriv>);
}

// Public constructor and methods for UIState GObject
impl UIState {
    pub fn new() -> Self {
        let obj: Self = Object::builder().build();
        // Initialize non-property fields if any, or set default property values programmatically if needed
        // obj.imp().other_internal_data.replace("Initialized via new".to_string());
        obj
    }

    // Example method to interact with internal data not exposed as a property
    pub fn get_other_data(&self) -> String {
        self.imp().other_internal_data.borrow().clone()
    }

    pub fn set_other_data(&self, data: &str) {
        self.imp().other_internal_data.replace(data.to_string());
    }

    // For properties like window_count, you use obj.property("window-count") or generated accessors
    // The #[property] macro in UIStatePriv generates:
    // obj.window_count() -> u32
    // obj.set_window_count(val: u32)
    // obj.notify_window_count()
    // obj.find_property("window-count")
    // obj.list_properties()
    
    // Example of a method that might change a property and emit notification
    // (though direct set via `obj.set_window_count()` handles notification automatically)
    pub fn increment_window_count_manual(&self) {
        let current_count = self.window_count();
        self.set_window_count(current_count + 1);
        // self.notify_window_count(); // Not needed if using generated setter
        tracing::info!("Window count incremented to: {}", self.window_count());
    }
}

impl Default for UIState {
    fn default() -> Self {
        Self::new()
    }
}

// Previous SharedUIState and new_shared_ui_state are no longer needed
// as UIState is now a GObject and inherently reference-counted (like Rc<T>).
// You directly pass around clones of UIState.
// pub type SharedUIState = Rc<RefCell<UIState>>; // Old
// pub fn new_shared_ui_state() -> SharedUIState { // Old
//     Rc::new(RefCell::new(UIState::new()))
// }
