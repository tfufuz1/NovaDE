/**
 * @file StandardWidget.qml
 * @brief Standardimplementierung eines Widgets
 * 
 * Diese Datei enthält eine standardmäßige Widget-Implementierung, die von BaseWidget erbt
 * und häufig benötigte Funktionen und Layouts bereitstellt.
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import VivoX.UI 1.0
import VivoX.Animation 1.0

BaseWidget {
    id: root
    
    // Standardwerte
    widgetName: "Standard Widget"
    widgetDescription: "Ein Standard-Widget mit häufig verwendeten Funktionen"
    widgetIconName: "widget"
    widgetCategory: "standard"
    widgetSize: WidgetSize.Medium
    widgetResizable: true
    widgetConfigurable: true
    
    // Eigenschaften
    property alias contentItem: contentLoader.item
    property url contentSource: ""
    property var contentData: ({})
    
    // Signale
    signal contentLoaded()
    signal refreshRequested()
    
    // Inhaltsloader
    Loader {
        id: contentLoader
        anchors.fill: contentArea
        source: contentSource
        onLoaded: {
            // Daten an den Inhalt weitergeben
            if (item && typeof item.setData === "function") {
                item.setData(contentData)
            }
            contentLoaded()
        }
    }
    
    // Aktualisierungsbutton
    Button {
        id: refreshButton
        anchors {
            right: parent.right
            bottom: parent.bottom
            margins: 8
        }
        width: 32
        height: 32
        visible: false // Standardmäßig ausgeblendet, kann von abgeleiteten Widgets aktiviert werden
        
        icon.source: "qrc:/icons/refresh.svg"
        icon.color: "#D8DEE9"
        
        background: Rectangle {
            color: refreshButton.pressed ? "#4C566A" : "#3B4252"
            radius: width / 2
            
            // Animation beim Drücken
            Behavior on color {
                ColorAnimation {
                    duration: 150
                    easing.type: Easing.InOutQuad
                }
            }
        }
        
        onClicked: {
            // Rotationsanimation
            refreshAnimation.start()
            refreshRequested()
        }
        
        // Rotationsanimation
        RotationAnimation {
            id: refreshAnimation
            target: refreshButton
            from: 0
            to: 360
            duration: 500
            easing.type: Easing.OutCubic
        }
    }
    
    // Ladeindikator
    BusyIndicator {
        id: busyIndicator
        anchors.centerIn: contentArea
        width: 48
        height: 48
        running: false // Standardmäßig ausgeblendet, kann von abgeleiteten Widgets aktiviert werden
    }
    
    // Fehleranzeige
    Rectangle {
        id: errorDisplay
        anchors.fill: contentArea
        color: "#4C566A"
        radius: 4
        visible: false // Standardmäßig ausgeblendet
        
        ColumnLayout {
            anchors.centerIn: parent
            width: parent.width - 32
            spacing: 16
            
            Image {
                Layout.alignment: Qt.AlignHCenter
                width: 48
                height: 48
                source: "qrc:/icons/error.svg"
                fillMode: Image.PreserveAspectFit
            }
            
            Text {
                id: errorText
                Layout.alignment: Qt.AlignHCenter
                Layout.fillWidth: true
                text: "Ein Fehler ist aufgetreten"
                color: "#ECEFF4"
                font.pixelSize: 14
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WordWrap
            }
            
            Button {
                Layout.alignment: Qt.AlignHCenter
                text: "Erneut versuchen"
                onClicked: {
                    errorDisplay.visible = false
                    refreshRequested()
                }
            }
        }
    }
    
    // Funktionen
    
    /**
     * @brief Setzt den Inhalt des Widgets
     * @param source URL der Quelle
     * @param data Daten für den Inhalt
     */
    function setContent(source, data) {
        contentSource = source
        contentData = data || {}
        contentLoader.source = ""  // Zurücksetzen
        contentLoader.source = source
    }
    
    /**
     * @brief Zeigt den Ladeindikator an
     */
    function showBusy() {
        busyIndicator.running = true
        errorDisplay.visible = false
    }
    
    /**
     * @brief Versteckt den Ladeindikator
     */
    function hideBusy() {
        busyIndicator.running = false
    }
    
    /**
     * @brief Zeigt eine Fehlermeldung an
     * @param message Fehlermeldung
     */
    function showError(message) {
        errorText.text = message || "Ein Fehler ist aufgetreten"
        errorDisplay.visible = true
        busyIndicator.running = false
    }
    
    /**
     * @brief Versteckt die Fehlermeldung
     */
    function hideError() {
        errorDisplay.visible = false
    }
    
    /**
     * @brief Zeigt oder versteckt den Aktualisierungsbutton
     * @param visible true, um den Button anzuzeigen, false, um ihn zu verstecken
     */
    function setRefreshButtonVisible(visible) {
        refreshButton.visible = visible
    }
    
    // Konfigurationsanfrage verarbeiten
    onConfigureRequested: {
        // Öffne den Konfigurationsdialog
        var configDialog = Qt.createComponent("qrc:/qml/ui/dialogs/WidgetConfigDialog.qml")
        if (configDialog.status === Component.Ready) {
            var dialog = configDialog.createObject(root, {
                "widgetId": widgetId,
                "widgetName": widgetName,
                "widgetConfig": widgetConfig
            })
            
            dialog.accepted.connect(function() {
                // Konfiguration aktualisieren
                widgetConfig = dialog.widgetConfig
                
                // Widget aktualisieren
                refreshRequested()
            })
            
            dialog.open()
        }
    }
}
