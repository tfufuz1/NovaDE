import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

CheckBox {
    id: root
    
    // Eigenschaften
    property int size: 20 // Größe der Checkbox
    property bool indeterminate: false
    
    // Visuelle Eigenschaften
    implicitWidth: contentItem.implicitWidth + indicator.width + spacing
    implicitHeight: Math.max(contentItem.implicitHeight, indicator.height)
    spacing: 8
    
    // Farben
    property color checkColor: VxTheme.colorAccentPrimary
    property color borderColor: enabled ? VxTheme.colorBorderNormal : VxTheme.colorBorderLight
    
    // Checkbox-Indikator
    indicator: Rectangle {
        implicitWidth: root.size
        implicitHeight: root.size
        x: 0
        y: parent.height / 2 - height / 2
        radius: VxTheme.cornerRadiusSmall
        color: root.checked || root.indeterminate ? root.checkColor : "transparent"
        border.color: root.checked || root.indeterminate ? root.checkColor : root.borderColor
        border.width: 1
        
        // Checkmark oder Indeterminate-Indikator
        Rectangle {
            visible: root.indeterminate
            anchors.centerIn: parent
            width: parent.width * 0.6
            height: 2
            color: "white"
        }
        
        Image {
            visible: root.checked && !root.indeterminate
            anchors.centerIn: parent
            width: parent.width * 0.7
            height: parent.height * 0.7
            source: "qrc:/icons/check.svg"
            sourceSize.width: width
            sourceSize.height: height
            fillMode: Image.PreserveAspectFit
            antialiasing: true
            
            // Fallback für fehlende SVG
            Rectangle {
                visible: parent.status !== Image.Ready
                anchors.centerIn: parent
                width: parent.width * 0.8
                height: parent.height * 0.4
                rotation: -45
                antialiasing: true
                color: "white"
                
                Rectangle {
                    width: parent.width * 0.6
                    height: parent.height * 2
                    anchors.bottom: parent.bottom
                    anchors.right: parent.right
                    color: "white"
                    antialiasing: true
                }
            }
        }
        
        // Hover-Effekt
        Rectangle {
            anchors.fill: parent
            radius: parent.radius
            color: root.hovered ? Qt.rgba(root.checkColor.r, root.checkColor.g, root.checkColor.b, 0.1) : "transparent"
            visible: root.enabled
        }
    }
    
    // Text
    contentItem: Text {
        text: root.text
        font.pixelSize: VxTheme.fontSizeNormal
        font.family: VxTheme.fontFamily
        color: root.enabled ? VxTheme.colorTextPrimary : VxTheme.colorTextDisabled
        verticalAlignment: Text.AlignVCenter
        leftPadding: root.indicator.width + root.spacing
        wrapMode: Text.WordWrap
    }
    
    // Tastaturnavigation
    Keys.onReturnPressed: toggle()
    Keys.onEnterPressed: toggle()
    Keys.onSpacePressed: toggle()
    
    // Fokus-Highlight
    focusPolicy: Qt.StrongFocus
    
    // Zustandsänderung
    onToggled: {
        if (indeterminate) {
            indeterminate = false
        }
    }
}
