/**
 * @file ConfigTextArea.qml
 * @brief Mehrzeiliges Textfeld für Konfigurationsdialoge
 * 
 * Diese Datei enthält ein mehrzeiliges Textfeld-Steuerelement für Konfigurationsdialoge.
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
    property int maximumLength: 10000
    property int minimumHeight: 100
    
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
    ScrollView {
        id: scrollView
        Layout.fillWidth: true
        Layout.minimumHeight: root.minimumHeight
        Layout.preferredHeight: Math.min(300, Math.max(root.minimumHeight, textArea.contentHeight + 20))
        
        TextArea {
            id: textArea
            text: root.text
            placeholderText: root.placeholderText
            readOnly: root.readOnly
            wrapMode: TextEdit.Wrap
            color: "#ECEFF4"
            selectionColor: "#88C0D0"
            selectedTextColor: "#2E3440"
            
            background: Rectangle {
                color: "#2E3440"
                border.color: textArea.focus ? "#88C0D0" : "#4C566A"
                border.width: 1
                radius: 4
            }
            
            onTextChanged: {
                // Maximale Länge prüfen
                if (text.length > root.maximumLength) {
                    text = text.substring(0, root.maximumLength)
                }
                
                // Wert an übergeordnetes Element weitergeben
                root.valueChanged(root.configKey, text)
            }
        }
    }
    
    // Zeichenzähler
    Label {
        id: characterCounter
        Layout.alignment: Qt.AlignRight
        text: textArea.text.length + " / " + root.maximumLength
        font.pixelSize: 10
        color: textArea.text.length > root.maximumLength * 0.9 ? "#BF616A" : "#4C566A"
    }
}
