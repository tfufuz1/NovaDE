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
    property string widgetType: "systray"
    property var widgetSettings: ({
        showLabels: false,
        maxIcons: 5
    })
    property int panelPosition: 0 // PanelPosition.Top
    
    // Größe
    property int minimumWidth: 48
    property int minimumHeight: 32
    property int preferredWidth: Math.min(systemTrayModel.count, widgetSettings.maxIcons) * 32 + 16
    property int preferredHeight: 48
    
    // Interne Eigenschaften
    property bool isHorizontal: panelPosition === 0 || panelPosition === 1 // Top oder Bottom
    property bool isVertical: panelPosition === 2 || panelPosition === 3 // Left oder Right
    
    // Layout-Eigenschaften
    Layout.minimumWidth: minimumWidth
    Layout.minimumHeight: minimumHeight
    Layout.preferredWidth: preferredWidth
    Layout.preferredHeight: preferredHeight
    
    // Modell für System-Tray-Icons
    ListModel {
        id: systemTrayModel
        
        // Beispiel-Einträge
        ListElement {
            name: "Netzwerk"
            icon: "qrc:/icons/wifi.svg"
            status: "Verbunden"
        }
        
        ListElement {
            name: "Batterie"
            icon: "qrc:/icons/battery.svg"
            status: "85%"
        }
        
        ListElement {
            name: "Lautstärke"
            icon: "qrc:/icons/volume-2.svg"
            status: "60%"
        }
        
        ListElement {
            name: "Bluetooth"
            icon: "qrc:/icons/bluetooth.svg"
            status: "Ein"
        }
    }
    
    // Inhalt
    RowLayout {
        id: horizontalLayout
        anchors.fill: parent
        spacing: 2
        visible: isHorizontal
        
        Repeater {
            model: systemTrayModel
            
            VxButton {
                buttonType: "icon"
                icon.source: model.icon
                tooltip: model.name + ": " + model.status
                size: 32
                
                Layout.fillHeight: true
                
                onClicked: {
                    openSystemTrayMenu(model.name, index);
                }
            }
        }
    }
    
    ColumnLayout {
        id: verticalLayout
        anchors.fill: parent
        spacing: 2
        visible: isVertical
        
        Repeater {
            model: systemTrayModel
            
            VxButton {
                buttonType: "icon"
                icon.source: model.icon
                tooltip: model.name + ": " + model.status
                size: 32
                
                Layout.fillWidth: true
                
                onClicked: {
                    openSystemTrayMenu(model.name, index);
                }
            }
        }
    }
    
    // Methoden
    function openSystemTrayMenu(name, index) {
        console.log("Opening system tray menu for:", name, "at index:", index);
        // Hier würde ein Kontextmenü für das entsprechende System-Tray-Icon geöffnet werden
    }
    
    function addSystemTrayIcon(name, icon, status) {
        systemTrayModel.append({
            "name": name,
            "icon": icon,
            "status": status
        });
    }
    
    function removeSystemTrayIcon(index) {
        if (index >= 0 && index < systemTrayModel.count) {
            systemTrayModel.remove(index);
        }
    }
    
    function updateSystemTrayIcon(index, name, icon, status) {
        if (index >= 0 && index < systemTrayModel.count) {
            systemTrayModel.set(index, {
                "name": name,
                "icon": icon,
                "status": status
            });
        }
    }
    
    function openSettingsDialog() {
        // Hier würde ein Dialog zum Bearbeiten der System-Tray-Einstellungen geöffnet werden
    }
    
    // Initialisierung
    Component.onCompleted: {
        // Hier könnten wir die System-Tray-Einstellungen aus den Einstellungen laden
    }
}
