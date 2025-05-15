/**
 * @file ConfigComboBox.qml
 * @brief Dropdown-Liste für Konfigurationsdialoge
 * 
 * Diese Datei enthält ein Dropdown-Steuerelement für Konfigurationsdialoge.
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
    property var model: []
    property string currentValue: ""
    property int currentIndex: -1
    
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
    
    // Dropdown-Liste
    ComboBox {
        id: comboBox
        Layout.fillWidth: true
        model: root.model
        
        // Initialen Wert setzen
        Component.onCompleted: {
            if (root.currentValue) {
                for (var i = 0; i < model.length; i++) {
                    if (model[i] === root.currentValue) {
                        currentIndex = i
                        break
                    }
                }
            }
        }
        
        // Aussehen anpassen
        background: Rectangle {
            color: "#2E3440"
            border.color: comboBox.focus ? "#88C0D0" : "#4C566A"
            border.width: 1
            radius: 4
        }
        
        contentItem: Text {
            leftPadding: 8
            rightPadding: comboBox.indicator.width + 8
            text: comboBox.displayText
            font: comboBox.font
            color: "#ECEFF4"
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        
        indicator: Canvas {
            id: canvas
            x: comboBox.width - width - 8
            y: comboBox.topPadding + (comboBox.availableHeight - height) / 2
            width: 12
            height: 8
            contextType: "2d"
            
            onPaint: {
                context.reset()
                context.moveTo(0, 0)
                context.lineTo(width, 0)
                context.lineTo(width / 2, height)
                context.closePath()
                context.fillStyle = "#88C0D0"
                context.fill()
            }
        }
        
        delegate: ItemDelegate {
            width: comboBox.width
            contentItem: Text {
                text: modelData
                color: "#ECEFF4"
                font: comboBox.font
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
            }
            highlighted: comboBox.highlightedIndex === index
            
            background: Rectangle {
                color: highlighted ? "#4C566A" : "#2E3440"
            }
        }
        
        popup: Popup {
            y: comboBox.height
            width: comboBox.width
            implicitHeight: contentItem.implicitHeight
            padding: 1
            
            contentItem: ListView {
                clip: true
                implicitHeight: contentHeight
                model: comboBox.popup.visible ? comboBox.delegateModel : null
                currentIndex: comboBox.highlightedIndex
                
                ScrollIndicator.vertical: ScrollIndicator { }
            }
            
            background: Rectangle {
                color: "#2E3440"
                border.color: "#4C566A"
                border.width: 1
                radius: 4
            }
        }
        
        onActivated: {
            // Wert an übergeordnetes Element weitergeben
            root.valueChanged(root.configKey, model[index])
        }
    }
}
