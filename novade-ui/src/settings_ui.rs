use adw::prelude::*;
use adw::{ActionRow, ComboRow, PreferencesGroup, PreferencesPage, PreferencesWindow};
use gtk::{glib, StringList, Switch};
use gtk::subclass::prelude::*;
use tracing; // For logging interactions

mod imp {
    use super::*;
    // No GObject properties needed for this simple version, so no need for Properties derive yet.
    // No template children needed if we construct everything programmatically in instance_init or new.
    #[derive(Default)]
    pub struct NovaSettingsWindowPriv;

    #[glib::object_subclass]
    impl ObjectSubclass for NovaSettingsWindowPriv {
        const NAME: &'static str = "NovaSettingsWindow";
        type Type = super::NovaSettingsWindow;
        type ParentType = PreferencesWindow; // Parent is AdwPreferencesWindow
    }

    impl ObjectImpl for NovaSettingsWindowPriv {}
    impl WidgetImpl for NovaSettingsWindowPriv {} // AdwPreferencesWindow -> ... -> Widget
    impl WindowImpl for NovaSettingsWindowPriv {} // AdwPreferencesWindow -> ... -> Window
    impl PreferencesWindowImpl for NovaSettingsWindowPriv {} // Specific to AdwPreferencesWindow
}

glib::wrapper! {
    pub struct NovaSettingsWindow(ObjectSubclass<imp::NovaSettingsWindowPriv>)
        @extends PreferencesWindow, adw::Window, gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl NovaSettingsWindow {
    pub fn new(parent: &impl IsA<gtk::Window>) -> Self {
        let window: Self = glib::Object::builder()
            .property("transient-for", parent)
            .property("modal", true) // Usually settings windows are modal
            .build();
        window.setup_settings_ui();
        window
    }

    fn setup_settings_ui(&self) {
        // Use gettext for localizing strings
        use gettextrs::gettext;

        // Create a PreferencesPage
        let page = PreferencesPage::new();
        self.add(&page); // Add page to the PreferencesWindow

        // --- Appearance Group ---
        let appearance_group = PreferencesGroup::builder()
            .title(&gettext("Appearance")) // i18n
            .description(&gettext("Customize the look and feel.")) // i18n
            .build();
        page.add(&appearance_group);

        // Dark Theme Toggle
        let dark_theme_row = ActionRow::builder()
            .title(&gettext("Enable Dark Theme")) // i18n
            .subtitle(&gettext("Toggle between light and dark system-wide themes.")) // i18n
            .build();
        let dark_theme_switch = Switch::builder()
            .valign(gtk::Align::Center)
            .build();
        dark_theme_row.add_suffix(&dark_theme_switch);
        dark_theme_row.set_activatable_widget(Some(&dark_theme_switch)); // Activate switch when row is clicked

        // In a real app, bind `dark_theme_switch.active` to AdwStyleManager or GSettings
        dark_theme_switch.connect_state_set(move |_switch, active| {
            tracing::info!("Dark theme switch toggled: {}", active);
            // Example: adw::StyleManager::default().set_color_scheme(if active { adw::ColorScheme::ForceDark } else { adw::ColorScheme::ForceLight });
            // This needs to be handled carefully, might need to be done from main context or similar.
            // For now, just log. It's important that this returns Inhibit(false) or glib::Propagation::Stop
            // if we handle it, or Inhibit(true)/glib::Propagation::Proceed if we want the default handler too.
            // For a simple switch, usually default behavior is fine unless we are intercepting.
            glib::Propagation::Stop // We are "handling" it by logging.
        });
        appearance_group.add(&dark_theme_row);

        // Font Size Selector
        let font_size_row = ComboRow::builder()
            .title(&gettext("Interface Font Size")) // i18n
            .subtitle(&gettext("Choose the general font size for the interface.")) // i18n
            .build();
        // These options are typically translatable.
        let font_options = [
            &gettext("Small"), 
            &gettext("Medium (Default)"), 
            &gettext("Large"), 
            &gettext("Extra Large")
        ];
        let string_list = StringList::new(&font_options);
        font_size_row.set_model(Some(&string_list));
        font_size_row.set_selected(1); // Default to "Medium (Default)"
        
        font_size_row.connect_selected_notify(|combo_row| {
            tracing::info!("Font size selection changed: Index {}, Value: '{}'", 
                combo_row.selected(),
                combo_row.selected_item().map_or("N/A".to_string(), |item| item.string().map_or("N/A".to_string(), |s| s.to_string()))
            );
        });
        appearance_group.add(&font_size_row);

        // --- Behavior Group (Example) ---
        let behavior_group = PreferencesGroup::builder()
            .title(&gettext("Behavior")) // i18n
            .description(&gettext("Customize system behavior.")) // i18n
            .build();
        page.add(&behavior_group);

        let placeholder_row = ActionRow::builder()
            .title(&gettext("Placeholder Setting")) // i18n
            .subtitle(&gettext("This is just a placeholder for future settings.")) // i18n
            .build();
        let placeholder_button = gtk::Button::with_label(&gettext("Click Action")); // i18n
        placeholder_button.connect_clicked(|_| {
            tracing::info!("Placeholder action button clicked.");
        });
        placeholder_row.add_suffix(&placeholder_button);
        placeholder_row.set_activatable_widget(Some(&placeholder_button));
        behavior_group.add(&placeholder_row);

        self.set_search_enabled(true); // Allow searching through preferences
        self.set_title(Some(&gettext("NovaDE Settings"))); // i18n Set window title
    }
}
