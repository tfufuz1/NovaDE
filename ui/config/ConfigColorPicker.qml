/**
 * @file ConfigColorPicker.qml
 * @brief Farbwähler für Konfigurationsdialoge
 * 
 * Diese Datei enthält ein Farbwähler-Steuerelement für Konfigurationsdialoge.
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Dialogs 1.3

ColumnLayout {
    id: root
    spacing: 4
    Layout.fillWidth: true
    
    // Eigenschaften
    property string configKey: ""
    property string label: ""
    property string color: "#88C0D0"
    
    // Signale
    signal valueChanged(string key, string value)
    
    // Beschriftung
    Label {
        id: labelItem
        Layout.fillWidth: true
        text: root.label
        font.pixelSize: 12
        color: "#ECEFF4"
    }
    
    // Farbwähler
    RowLayout {
        Layout.fillWidth: true
        spacing: 8
        
        // Farbvorschau
        Rectangle {
            id: colorPreview
            width: 32
            height: 32
            color: root.color
            border.color: "#4C566A"
            border.width: 1
            radius: 4
            
            // Klickbar machen, um den Farbwähler zu öffnen
            MouseArea {
                anchors.fill: parent
                onClicked: {
                    colorDialog.open()
                }
            }
        }
        
        // Farbwert als Text
        TextField {
            id: colorTextField
            Layout.fillWidth: true
            text: root.color
            placeholderText: "#RRGGBB"
            color: "#ECEFF4"
            selectionColor: "#88C0D0"
            selectedTextColor: "#2E3440"
            
            background: Rectangle {
                color: "#2E3440"
                border.color: colorTextField.focus ? "#88C0D0" : "#4C566A"
                border.width: 1
                radius: 4
            }
            
            // Validierung für Hex-Farbwerte
            validator: RegExpValidator {
                regExp: /#([0-9A-Fa-f]{6}|[0-9A-Fa-f]{8})/
            }
            
            onTextChanged: {
                if (text.match(/#([0-9A-Fa-f]{6}|[0-9A-Fa-f]{8})/)) {
                    root.color = text
                    root.valueChanged(root.configKey, text)
                }
            }
        }
        
        // Button zum Öffnen des Farbwählers
        Button {
            id: colorButton
            text: qsTr("Wählen")
            
            onClicked: {
                colorDialog.open()
            }
        }
    }
    
    // Farbwähler-Dialog
    ColorDialog {
        id: colorDialog
        title: qsTr("Farbe wählen")
        color: root.color
        showAlphaChannel: true
        
        onAccepted: {
            root.color = colorDialog.color
            colorTextField.text = colorDialog.color
            root.valueChanged(root.configKey, colorDialog.color)
        }
    }
}
