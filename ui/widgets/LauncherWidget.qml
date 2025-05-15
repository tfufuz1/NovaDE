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
    property string widgetType: "launcher"
    property var widgetSettings: ({
        applications: [
            { name: "Terminal", icon: "qrc:/icons/terminal.svg", command: "xterm" },
            { name: "Browser", icon: "qrc:/icons/globe.svg", command: "firefox" },
            { name: "Dateien", icon: "qrc:/icons/folder.svg", command: "nautilus" }
        ]
    })
    property int panelPosition: 0 // PanelPosition.Top
    
    // Größe
    property int minimumWidth: 48
    property int minimumHeight: 48
    property int preferredWidth: isHorizontal ? 48 * widgetSettings.applications.length : 48
    property int preferredHeight: isHorizontal ? 48 : 48 * widgetSettings.applications.length
    
    // Interne Eigenschaften
    property bool isHorizontal: panelPosition === 0 || panelPosition === 1 // Top oder Bottom
    property bool isVertical: panelPosition === 2 || panelPosition === 3 // Left oder Right
    
    // Layout-Eigenschaften
    Layout.minimumWidth: minimumWidth
    Layout.minimumHeight: minimumHeight
    Layout.preferredWidth: preferredWidth
    Layout.preferredHeight: preferredHeight
    
    // Inhalt
    RowLayout {
        id: horizontalLayout
        anchors.fill: parent
        spacing: 2
        visible: isHorizontal
        
        Repeater {
            model: widgetSettings.applications
            
            VxButton {
                buttonType: "icon"
                icon.source: modelData.icon
                tooltip: modelData.name
                size: 48
                
                Layout.fillHeight: true
                
                onClicked: {
                    launchApplication(modelData.command);
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
            model: widgetSettings.applications
            
            VxButton {
                buttonType: "icon"
                icon.source: modelData.icon
                tooltip: modelData.name
                size: 48
                
                Layout.fillWidth: true
                
                onClicked: {
                    launchApplication(modelData.command);
                }
            }
        }
    }
    
    // Methoden
    function launchApplication(command) {
        console.log("Launching application:", command);
        // In einer vollständigen Implementierung würden wir hier den SystemService verwenden,
        // um die Anwendung zu starten
    }
    
    function openSettingsDialog() {
        // Hier würde ein Dialog zum Bearbeiten der Anwendungsliste geöffnet werden
    }
    
    // Initialisierung
    Component.onCompleted: {
        // Hier könnten wir die Anwendungsliste aus den Einstellungen laden
    }
}
