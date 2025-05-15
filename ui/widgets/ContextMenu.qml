import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import VivoX.UI 1.0

/**
 * QML implementation of the ContextMenu component
 * This component provides a context menu that can be shown at a specific position
 */
Menu {
    id: root
    
    // Properties
    property var menuModel: menu.items
    property bool showIcons: true
    property int maxHeight: 400
    property int maxWidth: 300
    property int animationDuration: 150
    
    // Position
    x: 0
    y: 0
    
    // Appearance
    width: Math.min(implicitWidth, maxWidth)
    height: Math.min(contentHeight, maxHeight)
    
    // Animation
    enter: Transition {
        NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: animationDuration; easing.type: Easing.OutQuad }
        NumberAnimation { property: "scale"; from: 0.9; to: 1.0; duration: animationDuration; easing.type: Easing.OutQuad }
    }
    
    exit: Transition {
        NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: animationDuration; easing.type: Easing.InQuad }
        NumberAnimation { property: "scale"; from: 1.0; to: 0.9; duration: animationDuration; easing.type: Easing.InQuad }
    }
    
    // Background
    background: Rectangle {
        color: "#f5f5f5"
        border.color: "#d0d0d0"
        border.width: 1
        radius: 4
        
        // Shadow
        layer.enabled: true
        layer.effect: DropShadow {
            horizontalOffset: 2
            verticalOffset: 2
            radius: 8.0
            samples: 17
            color: "#30000000"
        }
    }
    
    // Scrollbar for long menus
    ScrollBar.vertical: ScrollBar {
        policy: root.contentHeight > root.maxHeight ? ScrollBar.AlwaysOn : ScrollBar.AsNeeded
        width: 8
        contentItem: Rectangle {
            implicitWidth: 8
            implicitHeight: 100
            radius: width / 2
            color: parent.pressed ? "#606060" : "#909090"
        }
    }
    
    // Delegate for menu items
    delegate: MenuItem {
        id: menuItem
        
        // Get the model item
        required property var modelData
        
        // Item properties
        text: modelData.text
        enabled: modelData.enabled
        checkable: modelData.checkable
        checked: modelData.checked
        
        // Icon
        icon.name: showIcons ? modelData.iconName : ""
        
        // Submenu
        Component.onCompleted: {
            // Create submenu if needed
            if (modelData.subItems && modelData.subItems.length > 0) {
                var subMenu = subMenuComponent.createObject(menuItem, {
                    "menuModel": modelData.subItems
                });
                menuItem.subMenu = subMenu;
            }
        }
        
        // Handle click
        onTriggered: {
            modelData.triggered();
        }
        
        // Appearance
        contentItem: RowLayout {
            spacing: 8
            
            // Icon
            Image {
                Layout.preferredWidth: 16
                Layout.preferredHeight: 16
                source: menuItem.icon.name ? "image://icon/" + menuItem.icon.name : ""
                visible: showIcons && menuItem.icon.name
                fillMode: Image.PreserveAspectFit
                opacity: menuItem.enabled ? 1.0 : 0.5
            }
            
            // Text
            Text {
                Layout.fillWidth: true
                text: menuItem.text
                font: menuItem.font
                opacity: menuItem.enabled ? 1.0 : 0.5
                color: menuItem.highlighted ? "#ffffff" : "#202020"
                horizontalAlignment: Text.AlignLeft
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
            }
            
            // Checkmark
            Image {
                Layout.preferredWidth: 16
                Layout.preferredHeight: 16
                visible: menuItem.checkable && menuItem.checked
                source: "qrc:/icons/check.svg"
                fillMode: Image.PreserveAspectFit
                opacity: menuItem.enabled ? 1.0 : 0.5
            }
            
            // Arrow for submenu
            Image {
                Layout.preferredWidth: 12
                Layout.preferredHeight: 12
                visible: menuItem.subMenu
                source: "qrc:/icons/arrow-right.svg"
                fillMode: Image.PreserveAspectFit
                opacity: menuItem.enabled ? 1.0 : 0.5
            }
        }
        
        // Background
        background: Rectangle {
            implicitWidth: 200
            implicitHeight: 40
            opacity: menuItem.enabled ? 1 : 0.5
            color: menuItem.highlighted ? "#3080e8" : "transparent"
        }
    }
    
    // Component for creating submenus
    Component {
        id: subMenuComponent
        
        Menu {
            id: subMenu
            
            // Properties
            property var menuModel
            
            // Appearance
            width: Math.min(implicitWidth, root.maxWidth)
            
            // Animation
            enter: Transition {
                NumberAnimation { property: "opacity"; from: 0.0; to: 1.0; duration: root.animationDuration; easing.type: Easing.OutQuad }
                NumberAnimation { property: "x"; from: -10; to: 0; duration: root.animationDuration; easing.type: Easing.OutQuad }
            }
            
            exit: Transition {
                NumberAnimation { property: "opacity"; from: 1.0; to: 0.0; duration: root.animationDuration; easing.type: Easing.InQuad }
                NumberAnimation { property: "x"; from: 0; to: -10; duration: root.animationDuration; easing.type: Easing.InQuad }
            }
            
            // Background
            background: Rectangle {
                color: "#f5f5f5"
                border.color: "#d0d0d0"
                border.width: 1
                radius: 4
                
                // Shadow
                layer.enabled: true
                layer.effect: DropShadow {
                    horizontalOffset: 2
                    verticalOffset: 2
                    radius: 8.0
                    samples: 17
                    color: "#30000000"
                }
            }
            
            // Scrollbar for long menus
            ScrollBar.vertical: ScrollBar {
                policy: subMenu.contentHeight > root.maxHeight ? ScrollBar.AlwaysOn : ScrollBar.AsNeeded
                width: 8
                contentItem: Rectangle {
                    implicitWidth: 8
                    implicitHeight: 100
                    radius: width / 2
                    color: parent.pressed ? "#606060" : "#909090"
                }
            }
            
            // Set up model
            model: menuModel
            
            // Use the same delegate as the parent menu
            delegate: root.delegate
        }
    }
    
    // Set up model
    model: menuModel
    
    // Connect signals
    Component.onCompleted: {
        // Connect to menu's opened signal
        menu.opened.connect(function() {
            root.open();
        });
        
        // Connect to menu's closed signal
        menu.closed.connect(function() {
            root.close();
        });
    }
}
