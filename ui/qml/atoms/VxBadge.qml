import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

Rectangle {
    id: root
    
    // Eigenschaften
    property string badgeType: "number" // dot, number, text
    property int value: 0
    property string text: ""
    property int maxValue: 99
    property bool small: false
    
    // Visuelle Eigenschaften
    width: {
        if (badgeType === "dot") return small ? 8 : 12
        if (badgeType === "number") {
            const textWidth = displayText.width + (small ? 8 : 12)
            return Math.max(small ? 16 : 20, textWidth)
        }
        if (badgeType === "text") {
            return displayText.width + (small ? 8 : 12)
        }
        return small ? 16 : 20
    }
    
    height: badgeType === "dot" ? width : (small ? 16 : 20)
    radius: height / 2
    color: VxTheme.colorSemanticError
    
    // Text für number und text Typen
    Text {
        id: displayText
        anchors.centerIn: parent
        visible: badgeType !== "dot"
        text: {
            if (badgeType === "number") {
                return value > maxValue ? maxValue + "+" : value.toString()
            }
            return root.text
        }
        font.pixelSize: small ? VxTheme.fontSizeSmall - 2 : VxTheme.fontSizeSmall
        font.family: VxTheme.fontFamily
        color: "white"
    }
}
