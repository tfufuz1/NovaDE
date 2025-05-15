/**
 * @file ConfigTextField.qml
 * @brief Textfeld für Konfigurationsdialoge
 * 
 * Diese Datei enthält ein Textfeld-Steuerelement für Konfigurationsdialoge.
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

ColumnLayout {
    id: root
    spacing: 4
    Layout.fillWidth: true
    
    // Eigenschaften
    property string configKey: ""
    property string label: ""
    property string text: ""
    property string placeholderText: ""
    property bool readOnly: false
    property int maximumLength: 1000
    property string validationRegExp: ""
    property bool isValid: true
    
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
    
    // Textfeld
    TextField {
        id: textField
        Layout.fillWidth: true
        text: root.text
        placeholderText: root.placeholderText
        readOnly: root.readOnly
        maximumLength: root.maximumLength
        color: "#ECEFF4"
        selectionColor: "#88C0D0"
        selectedTextColor: "#2E3440"
        
        background: Rectangle {
            color: "#2E3440"
            border.color: textField.focus ? "#88C0D0" : "#4C566A"
            border.width: 1
            radius: 4
        }
        
        onTextChanged: {
            // Validierung
            if (root.validationRegExp) {
                var regex = new RegExp(root.validationRegExp)
                root.isValid = regex.test(text)
            } else {
                root.isValid = true
            }
            
            // Wert an übergeordnetes Element weitergeben
            root.valueChanged(root.configKey, text)
        }
    }
    
    // Validierungsmeldung
    Label {
        id: validationMessage
        Layout.fillWidth: true
        text: qsTr("Ungültige Eingabe")
        font.pixelSize: 10
        color: "#BF616A"
        visible: !root.isValid
    }
}
