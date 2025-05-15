import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../ui/theme"
import "../ui/atoms"
import "../ui/molecules"

Item {
    id: root
    
    // Eigenschaften
    property var widgetManager: null
    property var widgetRegistry: null
    
    // Signale
    signal widgetAdded(string widgetId, string widgetType)
    signal widgetRemoved(string widgetId)
    
    // Interne Eigenschaften
    property var widgetInstances: ({})
    property var widgetComponents: ({})
    property var widgetConfigurations: ({})
    
    // Methoden
    function initialize() {
        // Registriere Standard-Widget-Typen
        registerStandardWidgetTypes();
        
        // Lade gespeicherte Widget-Konfigurationen
        loadWidgetConfigurations();
        
        // Erstelle Widgets aus Konfigurationen
        for (var widgetId in widgetConfigurations) {
            createWidgetFromConfig(widgetId, widgetConfigurations[widgetId]);
        }
    }
    
    function registerWidgetType(widgetType, qmlFile, metadata) {
        if (widgetRegistry) {
            return widgetRegistry.registerWidgetType(widgetType, qmlFile, metadata);
        }
        
        // Fallback, wenn kein Registry vorhanden ist
        if (widgetComponents[widgetType]) {
            console.warn("Widget type already registered:", widgetType);
            return false;
        }
        
        var component = Qt.createComponent(qmlFile);
        if (component.status === Component.Ready) {
            widgetComponents[widgetType] = component;
            return true;
        } else {
            console.error("Error creating widget component:", component.errorString());
            return false;
        }
    }
    
    function registerStandardWidgetTypes() {
        // Registriere Standard-Widget-Typen
        registerWidgetType("launcher", "../widgets/LauncherWidget.qml", {
            name: "Launcher",
            description: "Startet Anwendungen",
            category: "System"
        });
        
        registerWidgetType("clock", "../widgets/ClockWidget.qml", {
            name: "Uhr",
            description: "Zeigt die aktuelle Uhrzeit an",
            category: "System"
        });
        
        registerWidgetType("systray", "../widgets/SystemTrayWidget.qml", {
            name: "System-Tray",
            description: "Zeigt Systembenachrichtigungen an",
            category: "System"
        });
        
        registerWidgetType("taskbar", "../widgets/TaskbarWidget.qml", {
            name: "Taskleiste",
            description: "Zeigt geöffnete Fenster an",
            category: "System"
        });
    }
    
    function createWidget(widgetType, widgetId, settings) {
        // Generiere eine eindeutige Widget-ID, wenn keine angegeben wurde
        var actualWidgetId = widgetId || "widget_" + Math.random().toString(36).substr(2, 8);
        
        // Prüfe, ob der Widget-Typ registriert ist
        if (!widgetRegistry && !widgetComponents[widgetType]) {
            console.warn("Widget type not registered:", widgetType);
            return null;
        }
        
        // Erstelle Widget-Komponente
        var component;
        if (widgetRegistry) {
            component = widgetRegistry.createWidgetComponent(widgetType);
        } else {
            component = widgetComponents[widgetType];
        }
        
        if (!component) {
            console.error("Failed to create widget component for type:", widgetType);
            return null;
        }
        
        // Erstelle Widget-Instanz
        var widget = component.createObject(root, {
            "widgetId": actualWidgetId,
            "widgetType": widgetType,
            "widgetSettings": settings || {}
        });
        
        if (!widget) {
            console.error("Failed to create widget instance for type:", widgetType);
            return null;
        }
        
        // Speichere Widget-Instanz
        widgetInstances[actualWidgetId] = widget;
        
        // Speichere Widget-Konfiguration
        widgetConfigurations[actualWidgetId] = {
            type: widgetType,
            settings: settings || {}
        };
        
        // Speichere Konfigurationen
        saveWidgetConfigurations();
        
        // Signalisiere Widget-Erstellung
        widgetAdded(actualWidgetId, widgetType);
        
        return widget;
    }
    
    function removeWidget(widgetId) {
        if (!widgetInstances[widgetId]) {
            console.warn("Widget not found:", widgetId);
            return false;
        }
        
        // Entferne Widget-Instanz
        widgetInstances[widgetId].destroy();
        delete widgetInstances[widgetId];
        
        // Entferne Konfiguration
        delete widgetConfigurations[widgetId];
        
        // Speichere Konfigurationen
        saveWidgetConfigurations();
        
        // Signalisiere Widget-Entfernung
        widgetRemoved(widgetId);
        
        return true;
    }
    
    function getWidget(widgetId) {
        return widgetInstances[widgetId];
    }
    
    function getWidgetIds() {
        return Object.keys(widgetInstances);
    }
    
    function getWidgetsByType(widgetType) {
        var result = [];
        
        for (var widgetId in widgetInstances) {
            if (widgetInstances[widgetId].widgetType === widgetType) {
                result.push(widgetId);
            }
        }
        
        return result;
    }
    
    function updateWidgetSettings(widgetId, settings) {
        if (!widgetInstances[widgetId]) {
            console.warn("Widget not found:", widgetId);
            return false;
        }
        
        // Aktualisiere Widget-Einstellungen
        widgetInstances[widgetId].widgetSettings = settings;
        
        // Aktualisiere Konfiguration
        widgetConfigurations[widgetId].settings = settings;
        
        // Speichere Konfigurationen
        saveWidgetConfigurations();
        
        return true;
    }
    
    // Interne Methoden
    function createWidgetFromConfig(widgetId, config) {
        return createWidget(config.type, widgetId, config.settings);
    }
    
    function loadWidgetConfigurations() {
        // In einer vollständigen Implementierung würden wir hier die Konfigurationen
        // aus dem ConfigManager laden
        
        // Beispiel für eine gespeicherte Konfiguration
        widgetConfigurations = {
            "widget_launcher": {
                type: "launcher",
                settings: {
                    applications: [
                        { name: "Terminal", icon: "qrc:/icons/terminal.svg", command: "xterm" },
                        { name: "Browser", icon: "qrc:/icons/globe.svg", command: "firefox" },
                        { name: "Dateien", icon: "qrc:/icons/folder.svg", command: "nautilus" }
                    ]
                }
            },
            "widget_clock": {
                type: "clock",
                settings: {
                    format: "HH:mm",
                    showSeconds: false,
                    showDate: true,
                    dateFormat: "dd.MM.yyyy"
                }
            },
            "widget_systray": {
                type: "systray",
                settings: {
                    showLabels: false,
                    maxIcons: 5
                }
            },
            "widget_taskbar": {
                type: "taskbar",
                settings: {
                    showLabels: true,
                    groupSimilarWindows: true,
                    maxWidth: 300
                }
            }
        };
    }
    
    function saveWidgetConfigurations() {
        // In einer vollständigen Implementierung würden wir hier die Konfigurationen
        // im ConfigManager speichern
        
        // Beispiel für das Speichern der Konfigurationen
        console.log("Saving widget configurations:", JSON.stringify(widgetConfigurations));
    }
    
    // Initialisierung
    Component.onCompleted: {
        initialize();
    }
}
