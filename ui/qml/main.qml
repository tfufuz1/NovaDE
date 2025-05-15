import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Window 2.15
import VivoX.Widget 1.0
import VivoX.Action 1.0
import "ui/theme"
import "ui/atoms"
import "ui/molecules"
import "panel"
import "widget"
import "action"

Window {
    id: root
    visible: true
    width: 1280
    height: 720
    title: qsTr("VivoX Desktop Environment")
    
    // Eigenschaften
    property bool initialized: false
    
    // Hintergrund
    Rectangle {
        anchors.fill: parent
        color: VxTheme.colorBackgroundPrimary
    }
    
    // Hauptlayout
    ColumnLayout {
        anchors.fill: parent
        spacing: 0
        
        // Desktop-Bereich
        Item {
            id: desktopArea
            Layout.fillWidth: true
            Layout.fillHeight: true
            
            // Desktop-Widget
            Loader {
                id: desktopWidgetLoader
                anchors.fill: parent
                source: "widgets/DesktopWidget.qml"
                
                onLoaded: {
                    item.widgetId = "desktop_main"
                    item.widgetType = "desktop"
                    item.widgetSettings = {
                        showIcons: true,
                        iconSize: 64,
                        iconSpacing: 10,
                        showLabels: true,
                        sortBy: "name"
                    }
                }
            }
        }
    }
    
    // Panel-System
    PanelSystem {
        id: panelSystem
        anchors.fill: parent
        
        Component.onCompleted: {
            // Erstelle Standard-Panels
            createDefaultPanels();
        }
        
        function createDefaultPanels() {
            // Oberes Panel
            var topPanel = createPanel("panel_top", 0); // PanelPosition.Top
            
            // Unteres Panel
            var bottomPanel = createPanel("panel_bottom", 1); // PanelPosition.Bottom
            
            // Füge Widgets zum oberen Panel hinzu
            addWidgetToPanel(topPanel, "launcher", "widget_launcher", {
                applications: [
                    { name: "Terminal", icon: "qrc:/icons/terminal.svg", command: "xterm" },
                    { name: "Browser", icon: "qrc:/icons/globe.svg", command: "firefox" },
                    { name: "Dateien", icon: "qrc:/icons/folder.svg", command: "nautilus" }
                ]
            });
            
            addWidgetToPanel(topPanel, "clock", "widget_clock", {
                format: "HH:mm",
                showSeconds: false,
                showDate: true,
                dateFormat: "dd.MM.yyyy"
            });
            
            // Füge Widgets zum unteren Panel hinzu
            addWidgetToPanel(bottomPanel, "taskbar", "widget_taskbar", {
                showLabels: true,
                groupSimilarWindows: true,
                maxWidth: 300
            });
            
            addWidgetToPanel(bottomPanel, "systray", "widget_systray", {
                showLabels: false,
                maxIcons: 5
            });
        }
        
        function createPanel(panelId, position) {
            var panel = panelSystem.createPanel(panelId, position);
            panel.height = 48;
            panel.autoHide = false;
            return panel;
        }
        
        function addWidgetToPanel(panel, widgetType, widgetId, settings) {
            if (panel) {
                panel.addWidget(widgetType, widgetId, settings);
            }
        }
    }
    
    // Widget-System
    WidgetSystem {
        id: widgetSystem
        widgetRegistry: WidgetRegistry
        
        Component.onCompleted: {
            // Initialisiere Widget-System
            initialize();
        }
    }
    
    // Action-System
    ActionSystem {
        id: actionSystem
        actionRegistry: ActionRegistry
        
        Component.onCompleted: {
            // Initialisiere Action-System
            initialize();
            
            // Erstelle Standard-Aktionen
            createDefaultActions();
        }
        
        function createDefaultActions() {
            // Aktion zum Starten des Terminals
            createAction("launchApplication", "action_launch_terminal", {
                applicationName: "Terminal",
                command: "xterm",
                icon: "qrc:/icons/terminal.svg"
            });
            
            // Aktion zum Öffnen des Home-Verzeichnisses
            createAction("openFile", "action_open_home", {
                path: "/home/user",
                application: "fileManager"
            });
            
            // Aktion zum Umschalten der Uhr
            createAction("toggleWidget", "action_toggle_clock", {
                widgetId: "widget_clock",
                initialState: true
            });
        }
    }
    
    // Initialisierung
    Component.onCompleted: {
        // Initialisiere Theme
        VxTheme.initialize();
        
        // Markiere als initialisiert
        initialized = true;
        
        console.log("VivoX Desktop Environment initialized");
    }
}
