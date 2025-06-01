# SPEC-DEPENDENCY-MATRIX-v1.0.0: NovaDE Abhängigkeitsmatrix

```
SPEZIFIKATION: SPEC-DEPENDENCY-MATRIX-v1.0.0
VERSION: 1.0.0
STATUS: GENEHMIGT
ABHÄNGIGKEITEN: [SPEC-ROOT-v1.0.0]
AUTOR: Linus Wozniak Jobs
DATUM: 2025-05-31
ÄNDERUNGSPROTOKOLL: 
- 2025-05-31: Initiale Version (LWJ)
```

## 1. Zweck und Geltungsbereich

Dieses Dokument definiert die Abhängigkeitsmatrix für alle Module und Komponenten des NovaDE-Projekts. Es dient als zentrale Referenz für die Abhängigkeitsbeziehungen zwischen den verschiedenen Teilen des Systems und gewährleistet die Einhaltung der Schichtenhierarchie sowie die Vermeidung zirkulärer Abhängigkeiten.

## 2. Schichtenabhängigkeiten

Die folgende Tabelle zeigt die erlaubten Abhängigkeitsrichtungen zwischen den Hauptschichten:

| Quellschicht | Zielschicht | Erlaubt | Begründung |
|--------------|-------------|---------|------------|
| Core | Core | Ja | Interne Abhängigkeiten sind erlaubt |
| Core | Domain | Nein | Tiefere Schicht darf nicht von höherer abhängen |
| Core | System | Nein | Tiefere Schicht darf nicht von höherer abhängen |
| Core | UI | Nein | Tiefere Schicht darf nicht von höherer abhängen |
| Domain | Core | Ja | Höhere Schicht darf von tieferer abhängen |
| Domain | Domain | Ja | Interne Abhängigkeiten sind erlaubt |
| Domain | System | Nein | Tiefere Schicht darf nicht von höherer abhängen |
| Domain | UI | Nein | Tiefere Schicht darf nicht von höherer abhängen |
| System | Core | Ja | Höhere Schicht darf von tieferer abhängen |
| System | Domain | Ja | Höhere Schicht darf von tieferer abhängen |
| System | System | Ja | Interne Abhängigkeiten sind erlaubt |
| System | UI | Nein | Tiefere Schicht darf nicht von höherer abhängen |
| UI | Core | Ja | Höhere Schicht darf von tieferer abhängen |
| UI | Domain | Ja | Höhere Schicht darf von tieferer abhängen |
| UI | System | Ja | Höhere Schicht darf von tieferer abhängen |
| UI | UI | Ja | Interne Abhängigkeiten sind erlaubt |

## 3. Modulabhängigkeiten

### 3.1 Core Layer Modulabhängigkeiten

| Quellmodul | Zielmodul | Art der Abhängigkeit | Versionsbeschränkungen |
|------------|-----------|----------------------|------------------------|
| core::config | core::errors | Nutzung | =1.0.0 |
| core::config | core::types | Nutzung | =1.0.0 |
| core::logging | core::errors | Nutzung | =1.0.0 |
| core::utils | core::errors | Nutzung | =1.0.0 |
| core::utils | core::types | Nutzung | =1.0.0 |

### 3.2 Domain Layer Modulabhängigkeiten

| Quellmodul | Zielmodul | Art der Abhängigkeit | Versionsbeschränkungen |
|------------|-----------|----------------------|------------------------|
| domain::theming | core::errors | Nutzung | =1.0.0 |
| domain::theming | core::types | Nutzung | =1.0.0 |
| domain::theming | core::config | Nutzung | =1.0.0 |
| domain::theming | core::logging | Nutzung | =1.0.0 |
| domain::workspaces | core::errors | Nutzung | =1.0.0 |
| domain::workspaces | core::types | Nutzung | =1.0.0 |
| domain::workspaces | core::logging | Nutzung | =1.0.0 |
| domain::workspaces | domain::shared_types | Nutzung | =1.0.0 |
| domain::notifications_core | core::errors | Nutzung | =1.0.0 |
| domain::notifications_core | core::types | Nutzung | =1.0.0 |
| domain::notifications_core | core::logging | Nutzung | =1.0.0 |
| domain::notifications_core | domain::shared_types | Nutzung | =1.0.0 |
| domain::notifications_rules | domain::notifications_core | Nutzung | =1.0.0 |
| domain::notifications_rules | core::errors | Nutzung | =1.0.0 |
| domain::notifications_rules | core::logging | Nutzung | =1.0.0 |
| domain::global_settings_and_state_management | core::errors | Nutzung | =1.0.0 |
| domain::global_settings_and_state_management | core::types | Nutzung | =1.0.0 |
| domain::global_settings_and_state_management | core::config | Nutzung | =1.0.0 |
| domain::global_settings_and_state_management | core::logging | Nutzung | =1.0.0 |
| domain::window_policy_engine | core::errors | Nutzung | =1.0.0 |
| domain::window_policy_engine | core::types | Nutzung | =1.0.0 |
| domain::window_policy_engine | core::logging | Nutzung | =1.0.0 |
| domain::window_policy_engine | domain::workspaces | Nutzung | =1.0.0 |
| domain::user_centric_services::ai_interaction | core::errors | Nutzung | =1.0.0 |
| domain::user_centric_services::ai_interaction | core::types | Nutzung | =1.0.0 |
| domain::user_centric_services::ai_interaction | core::logging | Nutzung | =1.0.0 |
| domain::user_centric_services::ai_interaction | domain::shared_types | Nutzung | =1.0.0 |
| domain::common_events | core::types | Nutzung | =1.0.0 |
| domain::shared_types | core::types | Nutzung | =1.0.0 |

### 3.3 System Layer Modulabhängigkeiten

| Quellmodul | Zielmodul | Art der Abhängigkeit | Versionsbeschränkungen |
|------------|-----------|----------------------|------------------------|
| system::compositor | core::errors | Nutzung | =1.0.0 |
| system::compositor | core::types | Nutzung | =1.0.0 |
| system::compositor | core::logging | Nutzung | =1.0.0 |
| system::compositor | domain::window_policy_engine | Nutzung | =1.0.0 |
| system::compositor | domain::workspaces | Nutzung | =1.0.0 |
| system::input | core::errors | Nutzung | =1.0.0 |
| system::input | core::types | Nutzung | =1.0.0 |
| system::input | core::logging | Nutzung | =1.0.0 |
| system::input | system::compositor | Nutzung | =1.0.0 |
| system::dbus_interfaces | core::errors | Nutzung | =1.0.0 |
| system::dbus_interfaces | core::logging | Nutzung | =1.0.0 |
| system::audio_management | core::errors | Nutzung | =1.0.0 |
| system::audio_management | core::types | Nutzung | =1.0.0 |
| system::audio_management | core::logging | Nutzung | =1.0.0 |
| system::mcp_client | core::errors | Nutzung | =1.0.0 |
| system::mcp_client | core::logging | Nutzung | =1.0.0 |
| system::mcp_client | domain::user_centric_services::ai_interaction | Nutzung | =1.0.0 |
| system::portals | core::errors | Nutzung | =1.0.0 |
| system::portals | core::logging | Nutzung | =1.0.0 |
| system::portals | system::dbus_interfaces | Nutzung | =1.0.0 |
| system::power_management | core::errors | Nutzung | =1.0.0 |
| system::power_management | core::logging | Nutzung | =1.0.0 |
| system::power_management | system::dbus_interfaces | Nutzung | =1.0.0 |
| system::window_mechanics | core::errors | Nutzung | =1.0.0 |
| system::window_mechanics | core::types | Nutzung | =1.0.0 |
| system::window_mechanics | core::logging | Nutzung | =1.0.0 |
| system::window_mechanics | domain::window_policy_engine | Nutzung | =1.0.0 |
| system::window_mechanics | system::compositor | Nutzung | =1.0.0 |

### 3.4 UI Layer Modulabhängigkeiten

| Quellmodul | Zielmodul | Art der Abhängigkeit | Versionsbeschränkungen |
|------------|-----------|----------------------|------------------------|
| ui::shell | core::errors | Nutzung | =1.0.0 |
| ui::shell | core::types | Nutzung | =1.0.0 |
| ui::shell | core::logging | Nutzung | =1.0.0 |
| ui::shell | domain::workspaces | Nutzung | =1.0.0 |
| ui::shell | domain::notifications_core | Nutzung | =1.0.0 |
| ui::shell | domain::theming | Nutzung | =1.0.0 |
| ui::shell | system::compositor | Nutzung | =1.0.0 |
| ui::shell | ui::widgets | Nutzung | =1.0.0 |
| ui::control_center | core::errors | Nutzung | =1.0.0 |
| ui::control_center | core::types | Nutzung | =1.0.0 |
| ui::control_center | core::logging | Nutzung | =1.0.0 |
| ui::control_center | domain::global_settings_and_state_management | Nutzung | =1.0.0 |
| ui::control_center | domain::theming | Nutzung | =1.0.0 |
| ui::control_center | ui::widgets | Nutzung | =1.0.0 |
| ui::widgets | core::errors | Nutzung | =1.0.0 |
| ui::widgets | core::types | Nutzung | =1.0.0 |
| ui::widgets | domain::theming | Nutzung | =1.0.0 |
| ui::window_manager_frontend | core::errors | Nutzung | =1.0.0 |
| ui::window_manager_frontend | core::types | Nutzung | =1.0.0 |
| ui::window_manager_frontend | domain::workspaces | Nutzung | =1.0.0 |
| ui::window_manager_frontend | domain::window_policy_engine | Nutzung | =1.0.0 |
| ui::window_manager_frontend | system::compositor | Nutzung | =1.0.0 |
| ui::window_manager_frontend | system::window_mechanics | Nutzung | =1.0.0 |
| ui::notifications_frontend | core::errors | Nutzung | =1.0.0 |
| ui::notifications_frontend | core::types | Nutzung | =1.0.0 |
| ui::notifications_frontend | domain::notifications_core | Nutzung | =1.0.0 |
| ui::notifications_frontend | domain::notifications_rules | Nutzung | =1.0.0 |
| ui::notifications_frontend | ui::widgets | Nutzung | =1.0.0 |
| ui::theming_gtk | core::errors | Nutzung | =1.0.0 |
| ui::theming_gtk | domain::theming | Nutzung | =1.0.0 |
| ui::portals | core::errors | Nutzung | =1.0.0 |
| ui::portals | core::logging | Nutzung | =1.0.0 |
| ui::portals | system::portals | Nutzung | =1.0.0 |

## 4. Komponentenabhängigkeiten

Die Komponentenabhängigkeiten werden in den jeweiligen Modulspezifikationen detailliert beschrieben. Hier sind die wichtigsten Abhängigkeiten zwischen Komponenten verschiedener Module aufgeführt:

| Quellkomponente | Zielkomponente | Art der Abhängigkeit | Versionsbeschränkungen |
|-----------------|----------------|----------------------|------------------------|
| domain::theming::DefaultThemingEngine | core::config::ConfigProvider | Nutzung | =1.0.0 |
| domain::workspaces::DefaultWorkspaceManager | domain::global_settings_and_state_management::GlobalSettingsService | Nutzung | =1.0.0 |
| domain::notifications_core::DefaultNotificationService | domain::notifications_rules::NotificationRulesEngine | Nutzung | =1.0.0 |
| system::compositor::DesktopState | domain::window_policy_engine::WindowManagementPolicyService | Nutzung | =1.0.0 |
| system::input::SeatManager | system::compositor::DesktopState | Nutzung | =1.0.0 |
| system::window_mechanics::DefaultWindowMechanicsService | domain::window_policy_engine::WindowManagementPolicyService | Nutzung | =1.0.0 |
| ui::shell::ShellManager | domain::workspaces::WorkspaceManagerService | Nutzung | =1.0.0 |
| ui::shell::ShellManager | domain::notifications_core::NotificationService | Nutzung | =1.0.0 |
| ui::control_center::ControlCenterApplication | domain::global_settings_and_state_management::GlobalSettingsService | Nutzung | =1.0.0 |
| ui::notifications_frontend::NotificationCenter | domain::notifications_core::NotificationService | Nutzung | =1.0.0 |
| ui::window_manager_frontend::WindowManagerUI | system::window_mechanics::WindowMechanicsService | Nutzung | =1.0.0 |

## 5. Externe Abhängigkeiten

### 5.1 Core Layer Externe Abhängigkeiten

| Modul | Externe Abhängigkeit | Versionsbeschränkungen | Verwendungszweck |
|-------|----------------------|------------------------|------------------|
| core::errors | thiserror | ^1.0 | Fehlerbehandlung |
| core::config | serde | ^1.0 | Serialisierung/Deserialisierung |
| core::config | toml | ^0.8 | TOML-Konfigurationsdateien |
| core::config | once_cell | ^1.18 | Globale Singletons |
| core::logging | tracing | ^0.1 | Logging-Framework |
| core::logging | tracing-subscriber | ^0.3 | Logging-Subscriber |
| core::types | uuid | ^1.4 | Eindeutige Identifikatoren |
| core::types | chrono | ^0.4 | Zeitstempel und Zeitoperationen |

### 5.2 Domain Layer Externe Abhängigkeiten

| Modul | Externe Abhängigkeit | Versionsbeschränkungen | Verwendungszweck |
|-------|----------------------|------------------------|------------------|
| domain::theming | serde | ^1.0 | Serialisierung/Deserialisierung |
| domain::theming | serde_json | ^1.0 | JSON-Verarbeitung |
| domain::workspaces | uuid | ^1.4 | Eindeutige Identifikatoren |
| domain::workspaces | chrono | ^0.4 | Zeitstempel und Zeitoperationen |
| domain::notifications_core | uuid | ^1.4 | Eindeutige Identifikatoren |
| domain::notifications_core | chrono | ^0.4 | Zeitstempel und Zeitoperationen |
| domain::notifications_rules | regex | ^1.9 | Reguläre Ausdrücke |
| domain::global_settings_and_state_management | serde | ^1.0 | Serialisierung/Deserialisierung |
| domain::global_settings_and_state_management | serde_json | ^1.0 | JSON-Verarbeitung |
| domain::user_centric_services::ai_interaction | uuid | ^1.4 | Eindeutige Identifikatoren |
| domain::user_centric_services::ai_interaction | chrono | ^0.4 | Zeitstempel und Zeitoperationen |

### 5.3 System Layer Externe Abhängigkeiten

| Modul | Externe Abhängigkeit | Versionsbeschränkungen | Verwendungszweck |
|-------|----------------------|------------------------|------------------|
| system::compositor | smithay | ^0.3 | Wayland-Compositor-Framework |
| system::input | libinput | ^0.31 | Eingabeverarbeitung |
| system::input | xkbcommon | ^0.5 | Tastaturverarbeitung |
| system::dbus_interfaces | zbus | ^3.14 | D-Bus-Kommunikation |
| system::audio_management | pipewire-rs | ^0.7 | Audioverarbeitung |
| system::mcp_client | mcp_client_rs | ^1.0 | Model Context Protocol |
| system::portals | zbus | ^3.14 | D-Bus-Kommunikation |
| system::power_management | zbus | ^3.14 | D-Bus-Kommunikation |

### 5.4 UI Layer Externe Abhängigkeiten

| Modul | Externe Abhängigkeit | Versionsbeschränkungen | Verwendungszweck |
|-------|----------------------|------------------------|------------------|
| ui::shell | gtk4 | ^4.10 | Benutzeroberfläche |
| ui::shell | libadwaita | ^1.3 | Moderne GNOME-Widgets |
| ui::control_center | gtk4 | ^4.10 | Benutzeroberfläche |
| ui::control_center | libadwaita | ^1.3 | Moderne GNOME-Widgets |
| ui::widgets | gtk4 | ^4.10 | Benutzeroberfläche |
| ui::widgets | libadwaita | ^1.3 | Moderne GNOME-Widgets |
| ui::widgets | cairo | ^0.17 | Benutzerdefiniertes Zeichnen |
| ui::widgets | pango | ^0.17 | Textlayout und -rendering |
| ui::window_manager_frontend | gtk4 | ^4.10 | Benutzeroberfläche |
| ui::notifications_frontend | gtk4 | ^4.10 | Benutzeroberfläche |
| ui::notifications_frontend | libadwaita | ^1.3 | Moderne GNOME-Widgets |
| ui::theming_gtk | gtk4 | ^4.10 | Benutzeroberfläche |
| ui::portals | gtk4 | ^4.10 | Benutzeroberfläche |
| ui::portals | zbus | ^3.14 | D-Bus-Kommunikation |

## 6. Abhängigkeitsrichtlinien

### 6.1 Allgemeine Richtlinien

1. Abhängigkeiten MÜSSEN der Schichtenhierarchie folgen: Höhere Schichten dürfen von tieferen abhängen, aber nicht umgekehrt.
2. Zirkuläre Abhängigkeiten sind VERBOTEN.
3. Abhängigkeiten MÜSSEN explizit deklariert werden.
4. Versionsbeschränkungen MÜSSEN für alle Abhängigkeiten angegeben werden.
5. Abhängigkeiten SOLLTEN minimiert werden, um die Kopplung zu reduzieren.

### 6.2 Entkopplungsstrategien

1. **Dependency Inversion**: Höhere Schichten definieren Schnittstellen, die von tieferen Schichten implementiert werden.
2. **Event-basierte Kommunikation**: Komponenten kommunizieren über Events, ohne direkte Abhängigkeiten.
3. **Adapter-Muster**: Adapter werden verwendet, um Komponenten mit inkompatiblen Schnittstellen zu verbinden.
4. **Facade-Muster**: Komplexe Subsysteme werden hinter einfachen Schnittstellen verborgen.
5. **Service Locator**: Dienste werden über einen zentralen Service Locator gefunden, statt direkt referenziert zu werden.

### 6.3 Änderungsmanagement

1. Änderungen an Schnittstellen MÜSSEN die Versionsnummer gemäß semantischer Versionierung erhöhen.
2. Inkompatible Änderungen MÜSSEN die Hauptversionsnummer erhöhen.
3. Abwärtskompatible Funktionserweiterungen MÜSSEN die Nebenversionsnummer erhöhen.
4. Abwärtskompatible Fehlerbehebungen MÜSSEN die Patch-Versionsnummer erhöhen.
5. Änderungen an Abhängigkeiten MÜSSEN eine Auswirkungsanalyse durchlaufen.
6. Migrationsstrategien MÜSSEN für inkompatible Änderungen definiert werden.
