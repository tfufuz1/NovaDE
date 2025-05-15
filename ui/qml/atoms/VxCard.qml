import QtQuick 2.15
import QtQuick.Controls 2.15
import QtGraphicalEffects 1.15
import "../theme"

Rectangle {
    id: root
    
    // Eigenschaften
    property string cardType: "raised" // flat, raised, outlined
    property int elevation: {
        if (cardType === "raised") return 2
        return 0
    }
    
    // Visuelle Eigenschaften
    implicitWidth: 200
    implicitHeight: 200
    radius: VxTheme.cornerRadiusNormal
    color: VxTheme.colorBackgroundPrimary
    
    // Rahmen für outlined-Typ
    border.width: cardType === "outlined" ? 1 : 0
    border.color: VxTheme.colorBorderNormal
    
    // Schatten für raised-Typ
    layer.enabled: cardType === "raised"
    layer.effect: DropShadow {
        transparentBorder: true
        horizontalOffset: 0
        verticalOffset: elevation
        radius: elevation * 4
        samples: radius * 2 + 1
        color: "#30000000"
    }
}
