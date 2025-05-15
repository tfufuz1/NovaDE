import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../ui/theme"
import "../ui/atoms"
import "../ui/molecules"

Item {
    id: root
    
    // Eigenschaften
    property string widgetId: ""
    property string widgetType: "taskbar"
    property var widgetSettings: ({
        showLabels: true,
        groupSimilarWindows: true,
        maxWidth: 300
    })
    property int panelPosition: 0 // PanelPosition.Top
    
    // Größe
    property int minimumWidth: 100
    property int minimumHeight: 32
    property int preferredWidth: Math.min(taskbarModel.count * (widgetSettings.showLabels ? 150 : 48) + 16, widgetSettings.maxWidth)
    property int preferredHeight: 48
    
    // Interne Eigenschaften
    property bool isHorizontal: panelPosition === 0 || panelPosition === 1 // Top oder Bottom
    property bool isVertical: panelPosition === 2 || panelPosition === 3 // Left oder Right
    
    // Layout-Eigenschaften
    Layout.minimumWidth: minimumWidth
    Layout.minimumHeight: minimumHeight
    Layout.preferredWidth: preferredWidth
    Layout.preferredHeight: preferredHeight
    Layout.fillWidth: isHorizontal
    Layout.fillHeight: isVertical
    
    // Modell für Taskbar-Einträge
    ListModel {
        id: taskbarModel
        
        // Beispiel-Einträge
        ListElement {
            name: "Terminal"
            icon: "qrc:/icons/terminal.svg"
            windowId: "window1"
            active: true
        }
        
        ListElement {
            name: "Browser"
            icon: "qrc:/icons/globe.svg"
            windowId: "window2"
            active: false
        }
        
        ListElement {
            name: "Dateien"
            icon: "qrc:/icons/folder.svg"
            windowId: "window3"
            active: false
        }
    }
    
    // Inhalt
    ScrollView {
        anchors.fill: parent
        clip: true
        
        RowLayout {
            id: horizontalLayout
            width: isHorizontal ? implicitWidth : parent.width
            height: isHorizontal ? parent.height : implicitHeight
            spacing: 2
            visible: isHorizontal
            
            Repeater {
                model: taskbarModel
                
                VxButton {
                    buttonType: widgetSettings.showLabels ? "iconText" : "icon"
                    icon.source: model.icon
                    text: widgetSettings.showLabels ? model.name : ""
                    tooltip: model.name
                    highlighted: model.active
                    
                    Layout.fillHeight: true
                    Layout.preferredWidth: widgetSettings.showLabels ? 150 : 48
                    
                    onClicked: {
                        activateWindow(model.windowId);
                    }
                }
            }
        }
        
        ColumnLayout {
            id: verticalLayout
            width: isVertical ? parent.width : implicitWidth
            height: isVertical ? implicitHeight : parent.height
            spacing: 2
            visible: isVertical
            
            Repeater {
                model: taskbarModel
                
                VxButton {
                    buttonType: widgetSettings.showLabels ? "iconText" : "icon"
                    icon.source: model.icon
                    text: widgetSettings.showLabels ? model.name : ""
                    tooltip: model.name
                    highlighted: model.active
                    
                    Layout.fillWidth: true
                    Layout.preferredHeight: 48
                    
                    onClicked: {
                        activateWindow(model.windowId);
                    }
                }
            }
        }
    }
    
    // Methoden
    function activateWindow(windowId) {
        console.log("Activating window:", windowId);
        
        // Setze alle Fenster auf inaktiv
        for (var i = 0; i < taskbarModel.count; i++) {
            taskbarModel.setProperty(i, "active", false);
        }
        
        // Setze das ausgewählte Fenster auf aktiv
        for (var j = 0; j < taskbarModel.count; j++) {
            if (taskbarModel.get(j).windowId === windowId) {
                taskbarModel.setProperty(j, "active", true);
                break;
            }
        }
        
        // In einer vollständigen Implementierung würden wir hier den WindowManager verwenden,
        // um das Fenster zu aktivieren
    }
    
    function addWindow(windowId, name, icon) {
        // Prüfe, ob das Fenster bereits in der Taskbar ist
        for (var i = 0; i < taskbarModel.count; i++) {
            if (taskbarModel.get(i).windowId === windowId) {
                return;
            }
        }
        
        // Füge das Fenster zur Taskbar hinzu
        taskbarModel.append({
            "name": name,
            "icon": icon,
            "windowId": windowId,
            "active": false
        });
    }
    
    function removeWindow(windowId) {
        // Entferne das Fenster aus der Taskbar
        for (var i = 0; i < taskbarModel.count; i++) {
            if (taskbarModel.get(i).windowId === windowId) {
                taskbarModel.remove(i);
                break;
            }
        }
    }
    
    function updateWindow(windowId, name, icon, active) {
        // Aktualisiere die Fensterdaten
        for (var i = 0; i < taskbarModel.count; i++) {
            if (taskbarModel.get(i).windowId === windowId) {
                taskbarModel.setProperty(i, "name", name);
                taskbarModel.setProperty(i, "icon", icon);
                taskbarModel.setProperty(i, "active", active);
                break;
            }
        }
    }
    
    function openSettingsDialog() {
        // Hier würde ein Dialog zum Bearbeiten der Taskbar-Einstellungen geöffnet werden
    }
    
    // Initialisierung
    Component.onCompleted: {
        // Hier könnten wir die Taskbar-Einstellungen aus den Einstellungen laden
        // und die aktuell geöffneten Fenster zur Taskbar hinzufügen
    }
}
