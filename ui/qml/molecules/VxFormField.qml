import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

Item {
    id: root
    
    // Eigenschaften
    property string label: ""
    property bool required: false
    property string helperText: ""
    property bool error: false
    property string errorText: ""
    property alias control: controlContainer.children
    property string layout: "vertical" // vertical, horizontal
    
    // Visuelle Eigenschaften
    implicitWidth: layout === "vertical" ? Math.max(labelItem.implicitWidth, controlContainer.implicitWidth) : labelItem.implicitWidth + controlContainer.implicitWidth + 16
    implicitHeight: layout === "vertical" ? labelItem.implicitHeight + controlContainer.implicitHeight + (label !== "" ? 8 : 0) + (helperTextItem.visible ? helperTextItem.implicitHeight + 4 : 0) : Math.max(labelItem.implicitHeight, controlContainer.implicitHeight) + (helperTextItem.visible ? helperTextItem.implicitHeight + 4 : 0)
    
    // Layout
    ColumnLayout {
        anchors.fill: parent
        spacing: 8
        visible: layout === "vertical"
        
        // Label
        RowLayout {
            Layout.fillWidth: true
            spacing: 4
            visible: label !== ""
            
            // Beschriftung
            VxLabel {
                id: labelItem
                text: root.label
                Layout.fillWidth: true
            }
            
            // Pflichtfeld-Markierung
            Text {
                text: "*"
                visible: root.required
                color: VxTheme.colorSemanticError
                font.pixelSize: VxTheme.fontSizeNormal
                font.family: VxTheme.fontFamily
            }
        }
        
        // Kontrollelement-Container
        Item {
            id: controlContainer
            Layout.fillWidth: true
            Layout.preferredHeight: childrenRect.height
            
            // Hier wird das Kontrollelement eingefügt
        }
        
        // Hilfe- oder Fehlertext
        Text {
            id: helperTextItem
            Layout.fillWidth: true
            text: root.error ? root.errorText : root.helperText
            visible: text !== ""
            color: root.error ? VxTheme.colorSemanticError : VxTheme.colorTextSecondary
            font.pixelSize: VxTheme.fontSizeSmall
            font.family: VxTheme.fontFamily
            wrapMode: Text.WordWrap
        }
    }
    
    // Horizontales Layout
    RowLayout {
        anchors.fill: parent
        spacing: 16
        visible: layout === "horizontal"
        
        // Label
        RowLayout {
            Layout.preferredWidth: label !== "" ? implicitWidth : 0
            spacing: 4
            visible: label !== ""
            
            // Beschriftung
            VxLabel {
                text: root.label
            }
            
            // Pflichtfeld-Markierung
            Text {
                text: "*"
                visible: root.required
                color: VxTheme.colorSemanticError
                font.pixelSize: VxTheme.fontSizeNormal
                font.family: VxTheme.fontFamily
            }
        }
        
        // Kontrollelement und Hilfetext
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            
            // Kontrollelement-Container (horizontal)
            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: childrenRect.height
                visible: layout === "horizontal"
                
                // Hier wird das Kontrollelement eingefügt (wird durch Bindung an controlContainer.children geteilt)
            }
            
            // Hilfe- oder Fehlertext (horizontal)
            Text {
                Layout.fillWidth: true
                text: root.error ? root.errorText : root.helperText
                visible: text !== ""
                color: root.error ? VxTheme.colorSemanticError : VxTheme.colorTextSecondary
                font.pixelSize: VxTheme.fontSizeSmall
                font.family: VxTheme.fontFamily
                wrapMode: Text.WordWrap
            }
        }
    }
}
