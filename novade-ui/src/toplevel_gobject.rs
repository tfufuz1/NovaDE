use glib::Object;
use gtk::glib;
use crate::wayland_integration::toplevels::ToplevelInfo; // Adjust path as needed

mod imp {
    use super::*;
    use std::cell::RefCell;
    use glib::Properties; // For GObject properties

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::ToplevelListItemGObject)]
    pub struct ToplevelListItemGObject {
        #[property(get, set, name = "wayland-id", type = u32, member = wayland_id)]
        pub wayland_id: RefCell<u32>,
        #[property(get, set, name = "title", type = String, member = title)]
        pub title: RefCell<String>,
        #[property(get, set, name = "app-id", type = String, member = app_id)]
        pub app_id: RefCell<String>,
        // raw_states could be exposed as a GVariant or string if needed, or specific bool properties
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ToplevelListItemGObject {
        const NAME: &'static str = "NovaDEToplevelListItem";
        type Type = super::ToplevelListItemGObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ToplevelListItemGObject {}
}

glib::wrapper! {
    pub struct ToplevelListItemGObject(ObjectSubclass<imp::ToplevelListItemGObject>);
}

impl ToplevelListItemGObject {
    pub fn new(info: &ToplevelInfo) -> Self {
        let item: Self = Object::builder()
            .property("wayland-id", info.wayland_id)
            .property("title", info.title.clone().unwrap_or_default())
            .property("app-id", info.app_id.clone().unwrap_or_default())
            .build();
        item
    }

    pub fn update_from_info(&self, info: &ToplevelInfo) {
        self.set_title(info.title.clone().unwrap_or_default());
        self.set_app_id(info.app_id.clone().unwrap_or_default());
        // Update other properties if they exist (e.g., from raw_states)
    }

    pub fn wayland_id(&self) -> u32 {
        self.imp().properties.get("wayland-id").unwrap()
    }
}
