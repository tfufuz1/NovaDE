import QtQuick 2.15
import QtQuick.Controls 2.15
import QtGraphicalEffects 1.15
import "../theme"

ToolTip {
    id: root
    
    // Eigenschaften
    property string tooltipPosition: "bottom" // top, right, bottom, left
    property int delay: 500
    
    // Visuelle Eigenschaften
    padding: 8
    delay: root.delay
    
    // Positionierung
    x: {
        if (tooltipPosition === "left") return -width
        if (tooltipPosition === "right") return parent.width
        return (parent.width - width) / 2
    }
    
    y: {
        if (tooltipPosition === "top") return -height
        if (tooltipPosition === "bottom") return parent.height
        return (parent.height - height) / 2
    }
    
    // Hintergrund
    background: Rectangle {
        color: VxTheme.colorBackgroundTertiary
        radius: VxTheme.cornerRadiusSmall
        
        // Schatten
        layer.enabled: true
        layer.effect: DropShadow {
            transparentBorder: true
            horizontalOffset: 0
            verticalOffset: 2
            radius: 4
            samples: 9
            color: "#40000000"
        }
    }
    
    // Text
    contentItem: Text {
        text: root.text
        font.pixelSize: VxTheme.fontSizeSmall
        font.family: VxTheme.fontFamily
        color: VxTheme.colorTextPrimary
        wrapMode: Text.Wrap
    }
    
    // Einblendanimation
    enter: Transition {
        NumberAnimation { 
            property: "opacity"
            from: 0.0
            to: 1.0
            duration: VxTheme.animationDurationFast
            easing.type: Easing.OutQuad
        }
    }
    
    // Ausblendanimation
    exit: Transition {
        NumberAnimation { 
            property: "opacity"
            from: 1.0
            to: 0.0
            duration: VxTheme.animationDurationFast
            easing.type: Easing.InQuad
        }
    }
}
