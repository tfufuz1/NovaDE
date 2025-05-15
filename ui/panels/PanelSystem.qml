import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../ui/theme"
import "../ui/atoms"
import "../ui/molecules"

Item {
    id: root
    
    // Eigenschaften
    property var panelManager: null
    property var widgetRegistry: null
    
    // Signale
    signal panelAdded(string panelId)
    signal panelRemoved(string panelId)
    
    // Interne Eigenschaften
    property var panelContainers: ({})
    property var panelConfigurations: ({})
    
    // Methoden
    function initialize() {
        // Lade gespeicherte Panel-Konfigurationen
        loadPanelConfigurations();
        
        // Erstelle Standard-Panel, wenn keine vorhanden sind
        if (Object.keys(panelConfigurations).length === 0) {
            createDefaultPanels();
        } else {
            // Erstelle Panels aus Konfigurationen
            for (var panelId in panelConfigurations) {
                createPanelFromConfig(panelId, panelConfigurations[panelId]);
            }
        }
    }
    
    function createPanel(position, autoHide, locked) {
        // Generiere eine eindeutige Panel-ID
        var panelId = "panel_" + Math.random().toString(36).substr(2, 8);
        
        // Erstelle Panel-Konfiguration
        var panelConfig = {
            position: position || 1, // Standard: Bottom
            autoHide: autoHide || false,
            locked: locked || true,
            widgets: []
        };
        
        // Speichere Konfiguration
        panelConfigurations[panelId] = panelConfig;
        
        // Erstelle Panel-Container
        createPanelContainer(panelId, panelConfig);
        
        // Speichere Konfigurationen
        savePanelConfigurations();
        
        // Signalisiere Panel-Erstellung
        panelAdded(panelId);
        
        return panelId;
    }
    
    function removePanel(panelId) {
        if (!panelContainers[panelId]) {
            console.warn("Panel not found:", panelId);
            return false;
        }
        
        // Entferne Panel-Container
        panelContainers[panelId].destroy();
        delete panelContainers[panelId];
        
        // Entferne Konfiguration
        delete panelConfigurations[panelId];
        
        // Speichere Konfigurationen
        savePanelConfigurations();
        
        // Signalisiere Panel-Entfernung
        panelRemoved(panelId);
        
        return true;
    }
    
    function getPanelIds() {
        return Object.keys(panelContainers);
    }
    
    function getPanelContainer(panelId) {
        return panelContainers[panelId];
    }
    
    function getPanelConfiguration(panelId) {
        return panelConfigurations[panelId];
    }
    
    function updatePanelConfiguration(panelId, config) {
        if (!panelConfigurations[panelId]) {
            console.warn("Panel configuration not found:", panelId);
            return false;
        }
        
        // Aktualisiere Konfiguration
        panelConfigurations[panelId] = config;
        
        // Aktualisiere Panel-Container
        if (panelContainers[panelId]) {
            panelContainers[panelId].loadConfiguration(config);
        }
        
        // Speichere Konfigurationen
        savePanelConfigurations();
        
        return true;
    }
    
    function updatePanelPosition(panelId, position) {
        if (!panelConfigurations[panelId]) {
            return false;
        }
        
        panelConfigurations[panelId].position = position;
        savePanelConfigurations();
        return true;
    }
    
    function updatePanelAutoHide(panelId, autoHide) {
        if (!panelConfigurations[panelId]) {
            return false;
        }
        
        panelConfigurations[panelId].autoHide = autoHide;
        savePanelConfigurations();
        return true;
    }
    
    function updatePanelLocked(panelId, locked) {
        if (!panelConfigurations[panelId]) {
            return false;
        }
        
        panelConfigurations[panelId].locked = locked;
        savePanelConfigurations();
        return true;
    }
    
    function addWidgetToPanel(panelId, widgetId, position) {
        if (!panelContainers[panelId]) {
            console.warn("Panel not found:", panelId);
            return false;
        }
        
        var result = panelContainers[panelId].addWidget(widgetId, position);
        
        if (result) {
            // Aktualisiere Konfiguration
            if (!panelConfigurations[panelId].widgets.includes(widgetId)) {
                if (position >= 0 && position < panelConfigurations[panelId].widgets.length) {
                    panelConfigurations[panelId].widgets.splice(position, 0, widgetId);
                } else {
                    panelConfigurations[panelId].widgets.push(widgetId);
                }
                
                // Speichere Konfigurationen
                savePanelConfigurations();
            }
        }
        
        return result;
    }
    
    function removeWidgetFromPanel(panelId, widgetId) {
        if (!panelContainers[panelId]) {
            console.warn("Panel not found:", panelId);
            return false;
        }
        
        var result = panelContainers[panelId].removeWidget(widgetId);
        
        if (result) {
            // Aktualisiere Konfiguration
            var index = panelConfigurations[panelId].widgets.indexOf(widgetId);
            if (index !== -1) {
                panelConfigurations[panelId].widgets.splice(index, 1);
                
                // Speichere Konfigurationen
                savePanelConfigurations();
            }
        }
        
        return result;
    }
    
    function moveWidgetInPanel(panelId, widgetId, newPosition) {
        if (!panelContainers[panelId]) {
            console.warn("Panel not found:", panelId);
            return false;
        }
        
        var result = panelContainers[panelId].moveWidget(widgetId, newPosition);
        
        if (result) {
            // Aktualisiere Konfiguration
            var index = panelConfigurations[panelId].widgets.indexOf(widgetId);
            if (index !== -1 && index !== newPosition) {
                panelConfigurations[panelId].widgets.splice(index, 1);
                panelConfigurations[panelId].widgets.splice(newPosition, 0, widgetId);
                
                // Speichere Konfigurationen
                savePanelConfigurations();
            }
        }
        
        return result;
    }
    
    // Interne Methoden
    function createPanelContainer(panelId, config) {
        var component = Qt.createComponent("PanelContainer.qml");
        if (component.status === Component.Ready) {
            var container = component.createObject(root, {
                "panelId": panelId,
                "panelManager": root,
                "widgetRegistry": widgetRegistry,
                "panelConfig": config
            });
            
            panelContainers[panelId] = container;
            return container;
        } else {
            console.error("Error creating panel container:", component.errorString());
            return null;
        }
    }
    
    function createPanelFromConfig(panelId, config) {
        return createPanelContainer(panelId, config);
    }
    
    function createDefaultPanels() {
        // Erstelle Standard-Bottom-Panel
        var bottomPanelId = createPanel(1, false, true); // Bottom, nicht auto-hide, gesperrt
        
        // Füge Standard-Widgets hinzu
        if (bottomPanelId) {
            addWidgetToPanel(bottomPanelId, "launcher", 0);
            addWidgetToPanel(bottomPanelId, "taskbar", 1);
            addWidgetToPanel(bottomPanelId, "systray", 2);
            addWidgetToPanel(bottomPanelId, "clock", 3);
        }
    }
    
    function loadPanelConfigurations() {
        // In einer vollständigen Implementierung würden wir hier die Konfigurationen
        // aus dem ConfigManager laden
        
        // Beispiel für eine gespeicherte Konfiguration
        panelConfigurations = {
            "panel_default": {
                position: 1, // Bottom
                autoHide: false,
                locked: true,
                widgets: ["launcher", "taskbar", "systray", "clock"]
            }
        };
    }
    
    function savePanelConfigurations() {
        // In einer vollständigen Implementierung würden wir hier die Konfigurationen
        // im ConfigManager speichern
        
        // Beispiel für das Speichern der Konfigurationen
        console.log("Saving panel configurations:", JSON.stringify(panelConfigurations));
    }
    
    // Initialisierung
    Component.onCompleted: {
        initialize();
    }
}
