import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

Switch {
    id: root
    
    // Eigenschaften
    property int size: 36 // Höhe des Switch
    
    // Visuelle Eigenschaften
    implicitWidth: contentItem.implicitWidth + indicator.width + spacing
    implicitHeight: Math.max(contentItem.implicitHeight, indicator.height)
    spacing: 8
    
    // Farben
    property color activeColor: VxTheme.colorAccentPrimary
    property color inactiveColor: VxTheme.colorBackgroundSecondary
    property color thumbColor: "white"
    
    // Switch-Indikator
    indicator: Rectangle {
        implicitWidth: root.size * 1.8
        implicitHeight: root.size
        x: root.text ? 0 : (root.width - width) / 2
        y: (parent.height - height) / 2
        radius: height / 2
        color: root.checked ? root.activeColor : root.inactiveColor
        border.color: root.checked ? root.activeColor : VxTheme.colorBorderNormal
        border.width: 1
        
        // Thumb (Schieberegler)
        Rectangle {
            id: thumb
            x: root.checked ? parent.width - width - 2 : 2
            y: 2
            width: root.size - 4
            height: root.size - 4
            radius: height / 2
            color: root.enabled ? root.thumbColor : "#CCCCCC"
            
            // Schatten
            layer.enabled: true
            layer.effect: DropShadow {
                transparentBorder: true
                horizontalOffset: 0
                verticalOffset: 1
                radius: 2
                samples: 5
                color: "#30000000"
            }
            
            // Animation
            Behavior on x {
                NumberAnimation { 
                    duration: VxTheme.animationDurationNormal
                    easing.type: Easing.OutCubic
                }
            }
        }
        
        // Hover-Effekt
        Rectangle {
            anchors.fill: parent
            radius: parent.radius
            color: root.hovered ? Qt.rgba(1, 1, 1, 0.1) : "transparent"
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
