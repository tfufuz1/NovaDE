import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

TabButton {
    id: root
    
    // Eigenschaften
    property string iconSource: ""
    property bool showIcon: iconSource !== ""
    property bool showText: true
    property bool showBadge: false
    property int badgeValue: 0
    property string badgeText: ""
    property bool badgeDot: false
    
    // Visuelle Eigenschaften
    implicitWidth: showText ? 120 : 48
    implicitHeight: 48
    
    // Farben
    property color textColor: checked ? VxTheme.colorAccentPrimary : VxTheme.colorTextPrimary
    property color iconColor: textColor
    
    // Inhalt
    contentItem: RowLayout {
        spacing: 8
        anchors.centerIn: parent
        
        // Icon
        VxIcon {
            visible: root.showIcon
            source: root.iconSource
            size: 20
            color: root.iconColor
            Layout.alignment: Qt.AlignVCenter
        }
        
        // Text
        Text {
            visible: root.showText
            text: root.text
            font.pixelSize: VxTheme.fontSizeNormal
            font.family: VxTheme.fontFamily
            color: root.textColor
            elide: Text.ElideRight
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            Layout.alignment: Qt.AlignVCenter
        }
        
        // Badge
        VxBadge {
            visible: root.showBadge && (root.badgeValue > 0 || root.badgeText !== "" || root.badgeDot)
            badgeType: root.badgeDot ? "dot" : (root.badgeText !== "" ? "text" : "number")
            value: root.badgeValue
            text: root.badgeText
            Layout.alignment: Qt.AlignVCenter
        }
    }
    
    // Hintergrund
    background: Rectangle {
        color: {
            if (root.checked) return Qt.rgba(VxTheme.colorAccentPrimary.r, VxTheme.colorAccentPrimary.g, VxTheme.colorAccentPrimary.b, 0.1)
            if (root.hovered) return VxTheme.colorBackgroundHovered
            return "transparent"
        }
    }
}
