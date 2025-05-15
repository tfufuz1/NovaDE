/**
 * @file ConfigCheckBox.qml
 * @brief Checkbox für Konfigurationsdialoge
 * 
 * Diese Datei enthält ein Checkbox-Steuerelement für Konfigurationsdialoge.
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

RowLayout {
    id: root
    spacing: 8
    Layout.fillWidth: true
    
    // Eigenschaften
    property string configKey: ""
    property string label: ""
    property bool checked: false
    
    // Signale
    signal valueChanged(string key, bool value)
    
    // Checkbox
    CheckBox {
        id: checkBox
        checked: root.checked
        
        indicator: Rectangle {
            implicitWidth: 20
            implicitHeight: 20
            x: checkBox.leftPadding
            y: parent.height / 2 - height / 2
            radius: 3
            border.color: checkBox.down ? "#88C0D0" : "#4C566A"
            border.width: 1
            color: checkBox.checked ? "#88C0D0" : "#2E3440"
            
            Rectangle {
                width: 10
                height: 10
                x: 5
                y: 5
                radius: 2
                color: "#2E3440"
                visible: checkBox.checked
            }
        }
        
        contentItem: Text {
            text: ""
            leftPadding: checkBox.indicator.width + checkBox.spacing
            verticalAlignment: Text.AlignVCenter
            color: "#ECEFF4"
        }
        
        onCheckedChanged: {
            // Wert an übergeordnetes Element weitergeben
            root.valueChanged(root.configKey, checked)
        }
    }
    
    // Beschriftung
    Label {
        id: labelItem
        Layout.fillWidth: true
        text: root.label
        font.pixelSize: 12
        color: "#ECEFF4"
        
        // Klickbar machen, um die Checkbox umzuschalten
        MouseArea {
            anchors.fill: parent
            onClicked: {
                checkBox.checked = !checkBox.checked
            }
        }
    }
}
