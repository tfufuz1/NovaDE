import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

ComboBox {
    id: root
    
    // Eigenschaften
    property bool showIcon: false
    property bool showClearButton: false
    property bool error: false
    property string errorText: ""
    property string helperText: ""
    
    // Visuelle Eigenschaften
    implicitWidth: 200
    implicitHeight: 36
    
    // Farben
    property color textColor: enabled ? VxTheme.colorTextPrimary : VxTheme.colorTextDisabled
    property color backgroundColor: VxTheme.colorBackgroundSecondary
    property color borderColor: {
        if (error) return VxTheme.colorSemanticError
        if (root.activeFocus) return VxTheme.colorBorderFocus
        return VxTheme.colorBorderNormal
    }
    
    // Dropdown-Indikator
    indicator: VxIcon {
        x: root.width - width - 8
        y: (root.height - height) / 2
        source: "qrc:/icons/chevron-down.svg"
        size: 16
        color: root.textColor
    }
    
    // Textinhalt
    contentItem: RowLayout {
        spacing: 8
        anchors.fill: parent
        anchors.leftMargin: 12
        anchors.rightMargin: 32
        
        // Icon (optional)
        VxIcon {
            visible: root.showIcon && currentIndex >= 0 && model[currentIndex].icon !== undefined
            source: visible ? model[currentIndex].icon : ""
            size: 16
            color: root.textColor
        }
        
        // Text
        Text {
            Layout.fillWidth: true
            text: root.displayText
            font.pixelSize: VxTheme.fontSizeNormal
            font.family: VxTheme.fontFamily
            color: root.textColor
            elide: Text.ElideRight
            verticalAlignment: Text.AlignVCenter
        }
        
        // Clear-Button (optional)
        VxButton {
            visible: root.showClearButton && root.currentIndex >= 0
            buttonType: "icon"
            size: 24
            icon.source: "qrc:/icons/x.svg"
            
            onClicked: {
                root.currentIndex = -1
            }
        }
    }
    
    // Hintergrund
    background: Rectangle {
        implicitWidth: root.implicitWidth
        implicitHeight: root.implicitHeight
        color: root.backgroundColor
        border.width: 1
        border.color: root.borderColor
        radius: VxTheme.cornerRadiusSmall
    }
    
    // Dropdown-Popup
    popup: Popup {
        y: root.height
        width: root.width
        implicitHeight: contentItem.implicitHeight + 2
        padding: 1
        
        // Animation beim Öffnen/Schließen
        enter: Transition {
            NumberAnimation { 
                property: "opacity"
                from: 0.0
                to: 1.0
                duration: VxTheme.animationDurationFast
                easing.type: Easing.OutQuad
            }
            
            NumberAnimation { 
                property: "y"
                from: root.height - 10
                to: root.height
                duration: VxTheme.animationDurationFast
                easing.type: Easing.OutQuad
            }
        }
        
        exit: Transition {
            NumberAnimation { 
                property: "opacity"
                from: 1.0
                to: 0.0
                duration: VxTheme.animationDurationFast
                easing.type: Easing.InQuad
            }
        }
        
        // Hintergrund
        background: Rectangle {
            color: VxTheme.colorBackgroundPrimary
            radius: VxTheme.cornerRadiusSmall
            border.width: 1
            border.color: VxTheme.colorBorderNormal
            
            // Schatten
            layer.enabled: true
            layer.effect: DropShadow {
                horizontalOffset: 0
                verticalOffset: 3
                radius: 8
                samples: 17
                color: "#30000000"
            }
        }
        
        // Inhalt (ListView für Elemente)
        contentItem: ListView {
            implicitHeight: contentHeight
            model: root.popup.visible ? root.delegateModel : null
            currentIndex: root.highlightedIndex
            clip: true
            
            ScrollIndicator.vertical: ScrollIndicator {}
        }
    }
    
    // Delegat für Listenelemente
    delegate: ItemDelegate {
        width: root.width
        height: 36
        
        // Hintergrund
        background: Rectangle {
            color: highlighted ? VxTheme.colorBackgroundHovered : "transparent"
            radius: VxTheme.cornerRadiusSmall
        }
        
        // Inhalt
        contentItem: RowLayout {
            spacing: 8
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 12
            
            // Icon (optional)
            VxIcon {
                visible: root.showIcon && modelData.icon !== undefined
                source: visible ? modelData.icon : ""
                size: 16
                color: VxTheme.colorTextPrimary
            }
            
            // Text
            Text {
                Layout.fillWidth: true
                text: modelData.text !== undefined ? modelData.text : modelData
                font.pixelSize: VxTheme.fontSizeNormal
                font.family: VxTheme.fontFamily
                color: VxTheme.colorTextPrimary
                elide: Text.ElideRight
                verticalAlignment: Text.AlignVCenter
            }
        }
    }
    
    // Hilfe- oder Fehlertext
    Text {
        id: helperTextItem
        anchors {
            top: parent.bottom
            left: parent.left
            topMargin: 4
        }
        text: root.error ? root.errorText : root.helperText
        visible: text !== ""
        color: root.error ? VxTheme.colorSemanticError : VxTheme.colorTextSecondary
        font.pixelSize: VxTheme.fontSizeSmall
        font.family: VxTheme.fontFamily
    }
}
