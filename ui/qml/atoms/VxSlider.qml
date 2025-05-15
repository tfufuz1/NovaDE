import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

Slider {
    id: root
    
    // Eigenschaften
    property string orientation: "horizontal" // horizontal, vertical
    property int size: 36 // Höhe des Sliders bei horizontaler Ausrichtung, Breite bei vertikaler
    property bool showValue: false
    property int decimals: 0
    property string valuePrefix: ""
    property string valueSuffix: ""
    
    // Visuelle Eigenschaften
    implicitWidth: orientation === "horizontal" ? 200 : size
    implicitHeight: orientation === "horizontal" ? size : 200
    
    // Farben
    property color activeColor: VxTheme.colorAccentPrimary
    property color inactiveColor: VxTheme.colorBackgroundSecondary
    property color handleColor: "white"
    
    // Ausrichtung
    orientation: orientation === "horizontal" ? Qt.Horizontal : Qt.Vertical
    
    // Hintergrund
    background: Rectangle {
        x: orientation === Qt.Horizontal ? 0 : (parent.width - width) / 2
        y: orientation === Qt.Horizontal ? (parent.height - height) / 2 : 0
        width: orientation === Qt.Horizontal ? parent.width : 4
        height: orientation === Qt.Horizontal ? 4 : parent.height
        radius: 2
        color: root.inactiveColor
        
        // Aktiver Bereich
        Rectangle {
            width: orientation === Qt.Horizontal ? 
                   root.visualPosition * parent.width : parent.width
            height: orientation === Qt.Horizontal ? 
                    parent.height : root.visualPosition * parent.height
            color: root.activeColor
            radius: 2
            
            // Bei vertikaler Ausrichtung muss der aktive Bereich unten beginnen
            x: 0
            y: orientation === Qt.Horizontal ? 
               0 : parent.height - height
        }
    }
    
    // Handle (Schieberegler)
    handle: Rectangle {
        x: orientation === Qt.Horizontal ? 
           root.leftPadding + root.visualPosition * (root.availableWidth - width) :
           root.leftPadding + (root.availableWidth - width) / 2
        y: orientation === Qt.Horizontal ? 
           root.topPadding + (root.availableHeight - height) / 2 :
           root.topPadding + root.visualPosition * (root.availableHeight - height)
        width: 16
        height: 16
        radius: 8
        color: root.enabled ? root.handleColor : "#CCCCCC"
        border.color: root.activeColor
        border.width: 1
        
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
        
        // Hover-Effekt
        Rectangle {
            anchors.centerIn: parent
            width: parent.width * 2
            height: parent.height * 2
            radius: width / 2
            color: root.hovered ? Qt.rgba(root.activeColor.r, root.activeColor.g, root.activeColor.b, 0.1) : "transparent"
            visible: root.enabled
        }
    }
    
    // Wertanzeige
    Text {
        visible: root.showValue
        text: root.valuePrefix + root.value.toFixed(root.decimals) + root.valueSuffix
        anchors {
            horizontalCenter: orientation === Qt.Horizontal ? root.handle.horizontalCenter : parent.horizontalCenter
            bottom: orientation === Qt.Horizontal ? root.handle.top : undefined
            right: orientation === Qt.Vertical ? root.handle.left : undefined
            verticalCenter: orientation === Qt.Vertical ? root.handle.verticalCenter : undefined
            margins: 4
        }
        font.pixelSize: VxTheme.fontSizeSmall
        font.family: VxTheme.fontFamily
        color: root.enabled ? VxTheme.colorTextPrimary : VxTheme.colorTextDisabled
    }
    
    // Tastaturnavigation
    Keys.onLeftPressed: if (orientation === Qt.Horizontal) decrease()
    Keys.onRightPressed: if (orientation === Qt.Horizontal) increase()
    Keys.onUpPressed: if (orientation === Qt.Vertical) increase()
    Keys.onDownPressed: if (orientation === Qt.Vertical) decrease()
    
    // Fokus-Highlight
    focusPolicy: Qt.StrongFocus
}
