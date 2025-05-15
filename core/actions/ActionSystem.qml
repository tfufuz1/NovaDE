import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../ui/theme"
import "../ui/atoms"
import "../ui/molecules"

Item {
    id: root
    
    // Eigenschaften
    property var actionManager: null
    property var actionRegistry: null
    
    // Signale
    signal actionAdded(string actionId, string actionType)
    signal actionRemoved(string actionId)
    signal actionExecuted(string actionId, var parameters, var result)
    
    // Interne Eigenschaften
    property var actionInstances: ({})
    property var actionComponents: ({})
    property var actionConfigurations: ({})
    
    // Methoden
    function initialize() {
        // Registriere Standard-Aktions-Typen
        registerStandardActionTypes();
        
        // Lade gespeicherte Aktions-Konfigurationen
        loadActionConfigurations();
        
        // Erstelle Aktionen aus Konfigurationen
        for (var actionId in actionConfigurations) {
            createActionFromConfig(actionId, actionConfigurations[actionId]);
        }
    }
    
    function registerActionType(actionType, qmlFile, metadata) {
        if (actionRegistry) {
            return actionRegistry.registerActionType(actionType, qmlFile, metadata);
        }
        
        // Fallback, wenn kein Registry vorhanden ist
        if (actionComponents[actionType]) {
            console.warn("Action type already registered:", actionType);
            return false;
        }
        
        var component = Qt.createComponent(qmlFile);
        if (component.status === Component.Ready) {
            actionComponents[actionType] = component;
            return true;
        } else {
            console.error("Error creating action component:", component.errorString());
            return false;
        }
    }
    
    function registerStandardActionTypes() {
        // Registriere Standard-Aktions-Typen
        registerActionType("launchApplication", "../actions/LaunchApplicationAction.qml", {
            name: "Anwendung starten",
            description: "Startet eine Anwendung",
            category: "System"
        });
        
        registerActionType("openFile", "../actions/OpenFileAction.qml", {
            name: "Datei öffnen",
            description: "Öffnet eine Datei",
            category: "System"
        });
        
        registerActionType("systemCommand", "../actions/SystemCommandAction.qml", {
            name: "Systembefehl ausführen",
            description: "Führt einen Systembefehl aus",
            category: "System"
        });
        
        registerActionType("toggleWidget", "../actions/ToggleWidgetAction.qml", {
            name: "Widget umschalten",
            description: "Aktiviert oder deaktiviert ein Widget",
            category: "UI"
        });
    }
    
    function createAction(actionType, actionId, settings) {
        // Generiere eine eindeutige Aktions-ID, wenn keine angegeben wurde
        var actualActionId = actionId || "action_" + Math.random().toString(36).substr(2, 8);
        
        // Prüfe, ob der Aktions-Typ registriert ist
        if (!actionRegistry && !actionComponents[actionType]) {
            console.warn("Action type not registered:", actionType);
            return null;
        }
        
        // Erstelle Aktions-Komponente
        var component;
        if (actionRegistry) {
            component = actionRegistry.createActionComponent(actionType);
        } else {
            component = actionComponents[actionType];
        }
        
        if (!component) {
            console.error("Failed to create action component for type:", actionType);
            return null;
        }
        
        // Erstelle Aktions-Instanz
        var action = component.createObject(root, {
            "actionId": actualActionId,
            "actionType": actionType,
            "actionData": settings || {}
        });
        
        if (!action) {
            console.error("Failed to create action instance for type:", actionType);
            return null;
        }
        
        // Speichere Aktions-Instanz
        actionInstances[actualActionId] = action;
        
        // Speichere Aktions-Konfiguration
        actionConfigurations[actualActionId] = {
            type: actionType,
            settings: settings || {}
        };
        
        // Speichere Konfigurationen
        saveActionConfigurations();
        
        // Signalisiere Aktions-Erstellung
        actionAdded(actualActionId, actionType);
        
        return action;
    }
    
    function removeAction(actionId) {
        if (!actionInstances[actionId]) {
            console.warn("Action not found:", actionId);
            return false;
        }
        
        // Entferne Aktions-Instanz
        actionInstances[actionId].destroy();
        delete actionInstances[actionId];
        
        // Entferne Konfiguration
        delete actionConfigurations[actionId];
        
        // Speichere Konfigurationen
        saveActionConfigurations();
        
        // Signalisiere Aktions-Entfernung
        actionRemoved(actionId);
        
        return true;
    }
    
    function getAction(actionId) {
        return actionInstances[actionId];
    }
    
    function getActionIds() {
        return Object.keys(actionInstances);
    }
    
    function executeAction(actionId, parameters) {
        if (!actionInstances[actionId]) {
            console.warn("Action not found:", actionId);
            return null;
        }
        
        var action = actionInstances[actionId];
        
        // Prüfe, ob die Aktion ausgeführt werden kann
        if (!action.canExecute(parameters || {})) {
            console.warn("Action cannot be executed:", actionId);
            return null;
        }
        
        // Führe die Aktion aus
        var result = action.execute(parameters || {});
        
        // Signalisiere Aktions-Ausführung
        actionExecuted(actionId, parameters || {}, result);
        
        return result;
    }
    
    // Interne Methoden
    function createActionFromConfig(actionId, config) {
        return createAction(config.type, actionId, config.settings);
    }
    
    function loadActionConfigurations() {
        // In einer vollständigen Implementierung würden wir hier die Konfigurationen
        // aus dem ConfigManager laden
        
        // Beispiel für eine gespeicherte Konfiguration
        actionConfigurations = {
            "action_launch_terminal": {
                type: "launchApplication",
                settings: {
                    applicationName: "Terminal",
                    command: "xterm",
                    icon: "qrc:/icons/terminal.svg"
                }
            },
            "action_open_home": {
                type: "openFile",
                settings: {
                    path: "/home/user",
                    application: "fileManager"
                }
            },
            "action_toggle_clock": {
                type: "toggleWidget",
                settings: {
                    widgetId: "widget_clock",
                    initialState: true
                }
            }
        };
    }
    
    function saveActionConfigurations() {
        // In einer vollständigen Implementierung würden wir hier die Konfigurationen
        // im ConfigManager speichern
        
        // Beispiel für das Speichern der Konfigurationen
        console.log("Saving action configurations:", JSON.stringify(actionConfigurations));
    }
    
    // Initialisierung
    Component.onCompleted: {
        initialize();
    }
}
