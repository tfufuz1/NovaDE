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
    property string widgetType: "clock"
    property var widgetSettings: ({
        format: "HH:mm",
        showSeconds: false,
        showDate: false,
        dateFormat: "dd.MM.yyyy"
    })
    property int panelPosition: 0 // PanelPosition.Top
    
    // Größe
    property int minimumWidth: 60
    property int minimumHeight: 32
    property int preferredWidth: timeText.implicitWidth + (widgetSettings.showDate ? dateText.implicitWidth + 8 : 0) + 16
    property int preferredHeight: 48
    
    // Layout-Eigenschaften
    Layout.minimumWidth: minimumWidth
    Layout.minimumHeight: minimumHeight
    Layout.preferredWidth: preferredWidth
    Layout.preferredHeight: preferredHeight
    
    // Interne Eigenschaften
    property date currentTime: new Date()
    
    // Timer für die Aktualisierung der Uhrzeit
    Timer {
        interval: widgetSettings.showSeconds ? 1000 : 60000 // 1 Sekunde oder 1 Minute
        running: true
        repeat: true
        onTriggered: {
            currentTime = new Date();
        }
    }
    
    // Inhalt
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 4
        spacing: 0
        
        // Uhrzeit
        Text {
            id: timeText
            text: Qt.formatTime(currentTime, widgetSettings.showSeconds ? "HH:mm:ss" : widgetSettings.format)
            font.pixelSize: VxTheme.fontSizeLarge
            font.family: VxTheme.fontFamily
            color: VxTheme.colorTextPrimary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
        }
        
        // Datum (optional)
        Text {
            id: dateText
            text: Qt.formatDate(currentTime, widgetSettings.dateFormat)
            font.pixelSize: VxTheme.fontSizeSmall
            font.family: VxTheme.fontFamily
            color: VxTheme.colorTextSecondary
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            visible: widgetSettings.showDate
            
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
        }
    }
    
    // Mausinteraktion
    MouseArea {
        anchors.fill: parent
        
        onClicked: {
            // Hier könnte ein Kalender oder eine detaillierte Uhrzeitanzeige geöffnet werden
        }
    }
    
    // Methoden
    function openSettingsDialog() {
        // Hier würde ein Dialog zum Bearbeiten der Uhrzeit-Einstellungen geöffnet werden
    }
    
    // Initialisierung
    Component.onCompleted: {
        // Hier könnten wir die Uhrzeit-Einstellungen aus den Einstellungen laden
    }
}
