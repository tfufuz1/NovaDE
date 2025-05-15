import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../ui/theme"
import "../ui/atoms"
import "../ui/molecules"

Item {
    id: root
    
    // Eigenschaften
    property string panelId: ""
    property var panelManager: null
    property var widgetRegistry: null
    
    // Panel-Konfiguration
    property var panelConfig: ({
        position: 1, // Bottom
        autoHide: false,
        locked: true,
        widgets: ["launcher", "taskbar", "systray", "clock"]
    })
    
    // Panel-Instanz
    Panel {
        id: panel
        anchors.fill: parent
        panelId: root.panelId
        position: panelConfig.position
        autoHide: panelConfig.autoHide
        locked: panelConfig.locked
        
        // Signale verbinden
        onPanelNameChanged: {
            if (panelManager) {
                panelManager.updatePanelName(panelId, name);
            }
        }
        
        onPositionChanged: {
            if (panelManager) {
                panelManager.updatePanelPosition(panelId, position);
            }
            panelConfig.position = position;
        }
        
        onAutoHideChanged: {
            panelConfig.autoHide = autoHide;
        }
        
        onLockedChanged: {
            panelConfig.locked = locked;
        }
        
        onWidgetAdded: {
            if (!panelConfig.widgets.includes(widgetId)) {
                panelConfig.widgets.splice(position, 0, widgetId);
            }
        }
        
        onWidgetRemoved: {
            var index = panelConfig.widgets.indexOf(widgetId);
            if (index !== -1) {
                panelConfig.widgets.splice(index, 1);
            }
        }
        
        onWidgetMoved: {
            var index = panelConfig.widgets.indexOf(widgetId);
            if (index !== -1 && index !== newPosition) {
                panelConfig.widgets.splice(index, 1);
                panelConfig.widgets.splice(newPosition, 0, widgetId);
            }
        }
    }
    
    // Methoden
    function initialize() {
        // Widgets hinzufügen
        for (var i = 0; i < panelConfig.widgets.length; i++) {
            var widgetId = panelConfig.widgets[i];
            panel.addWidget(widgetId, i);
        }
    }
    
    function addWidget(widgetId, position) {
        return panel.addWidget(widgetId, position);
    }
    
    function removeWidget(widgetId) {
        return panel.removeWidget(widgetId);
    }
    
    function moveWidget(widgetId, newPosition) {
        return panel.moveWidget(widgetId, newPosition);
    }
    
    function getWidgets() {
        return panel.widgets();
    }
    
    function setPanelPosition(position) {
        panel.setPosition(position);
    }
    
    function setPanelAutoHide(autoHide) {
        panel.setAutoHide(autoHide);
    }
    
    function setPanelLocked(locked) {
        panel.setLocked(locked);
    }
    
    function showPanel() {
        panel.show();
    }
    
    function hidePanel() {
        panel.hide();
    }
    
    function saveConfiguration() {
        if (panelManager) {
            panelManager.savePanelConfiguration(panelId, panelConfig);
        }
    }
    
    function loadConfiguration(config) {
        if (config) {
            panelConfig = config;
            panel.setPosition(config.position);
            panel.setAutoHide(config.autoHide);
            panel.setLocked(config.locked);
            
            // Widgets neu laden
            var currentWidgets = panel.widgets();
            for (var i = 0; i < currentWidgets.length; i++) {
                panel.removeWidget(currentWidgets[i]);
            }
            
            for (var j = 0; j < config.widgets.length; j++) {
                panel.addWidget(config.widgets[j], j);
            }
        }
    }
    
    // Initialisierung
    Component.onCompleted: {
        initialize();
    }
}
