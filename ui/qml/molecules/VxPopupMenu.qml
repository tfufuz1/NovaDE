import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

Menu {
    id: root
    
    // Eigenschaften
    property bool showIcons: true
    
    // Visuelle Eigenschaften
    width: 200
    
    // Hintergrund
    background: Rectangle {
        implicitWidth: 200
        color: VxTheme.colorBackgroundPrimary
        border.color: VxTheme.colorBorderNormal
        border.width: 1
        radius: VxTheme.cornerRadiusSmall
        
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
    
    // Animationen
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
            from: parent ? parent.mapFromItem(parent, 0, 0).y - 10 : -10
            to: parent ? parent.mapFromItem(parent, 0, 0).y : 0
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
    
    // Standarddelegat für Menüelemente
    delegate: MenuItem {
        id: menuItem
        
        // Eigenschaften
        property string iconSource: ""
        property bool isCheckable: false
        property bool isChecked: false
        property bool isSeparator: false
        
        // Visuelle Eigenschaften
        implicitWidth: 200
        implicitHeight: isSeparator ? 8 : 36
        
        // Hintergrund
        background: Rectangle {
            color: menuItem.highlighted ? VxTheme.colorBackgroundHovered : "transparent"
            visible: !menuItem.isSeparator
        }
        
        // Inhalt
        contentItem: RowLayout {
            spacing: 8
            visible: !menuItem.isSeparator
            
            // Icon
            VxIcon {
                visible: root.showIcons && menuItem.iconSource !== ""
                source: menuItem.iconSource
                size: 16
                color: menuItem.enabled ? VxTheme.colorTextPrimary : VxTheme.colorTextDisabled
                Layout.leftMargin: 8
            }
            
            // Checkbox/Radio-Indikator
            Rectangle {
                visible: menuItem.isCheckable
                width: 16
                height: 16
                radius: menuItem.checkable ? 8 : 3
                color: "transparent"
                border.color: menuItem.enabled ? VxTheme.colorBorderNormal : VxTheme.colorBorderLight
                border.width: 1
                
                // Checked-Indikator
                Rectangle {
                    visible: menuItem.isChecked
                    anchors.centerIn: parent
                    width: parent.radius === 8 ? 8 : 10
                    height: parent.radius === 8 ? 8 : 10
                    radius: parent.radius === 8 ? 4 : 2
                    color: VxTheme.colorAccentPrimary
                }
            }
            
            // Text
            Text {
                text: menuItem.text
                font.pixelSize: VxTheme.fontSizeNormal
                font.family: VxTheme.fontFamily
                color: menuItem.enabled ? VxTheme.colorTextPrimary : VxTheme.colorTextDisabled
                elide: Text.ElideRight
                Layout.fillWidth: true
            }
            
            // Shortcut-Text
            Text {
                text: menuItem.shortcut || ""
                visible: menuItem.shortcut
                font.pixelSize: VxTheme.fontSizeSmall
                font.family: VxTheme.fontFamily
                color: VxTheme.colorTextSecondary
                horizontalAlignment: Text.AlignRight
                Layout.rightMargin: 8
            }
        }
        
        // Separator
        Rectangle {
            visible: menuItem.isSeparator
            anchors {
                left: parent.left
                right: parent.right
                verticalCenter: parent.verticalCenter
            }
            height: 1
            color: VxTheme.colorBorderLight
        }
    }
}
