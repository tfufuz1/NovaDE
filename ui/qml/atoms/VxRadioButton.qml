import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

RadioButton {
    id: root
    
    // Eigenschaften
    property int size: 20 // Größe des RadioButtons
    
    // Visuelle Eigenschaften
    implicitWidth: contentItem.implicitWidth + indicator.width + spacing
    implicitHeight: Math.max(contentItem.implicitHeight, indicator.height)
    spacing: 8
    
    // Farben
    property color checkColor: VxTheme.colorAccentPrimary
    property color borderColor: enabled ? VxTheme.colorBorderNormal : VxTheme.colorBorderLight
    
    // RadioButton-Indikator
    indicator: Rectangle {
        implicitWidth: root.size
        implicitHeight: root.size
        x: 0
        y: parent.height / 2 - height / 2
        radius: width / 2
        color: "transparent"
        border.color: root.checked ? root.checkColor : root.borderColor
        border.width: 1
        
        // Innerer Kreis (Checked-Indikator)
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 0.6
            height: parent.height * 0.6
            radius: width / 2
            color: root.checkColor
            visible: root.checked
            
            // Animation beim Aktivieren/Deaktivieren
            Behavior on width {
                NumberAnimation { 
                    duration: VxTheme.animationDurationFast
                    easing.type: Easing.OutCubic
                }
            }
            
            Behavior on height {
                NumberAnimation { 
                    duration: VxTheme.animationDurationFast
                    easing.type: Easing.OutCubic
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
}
