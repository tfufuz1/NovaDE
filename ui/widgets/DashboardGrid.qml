import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import VivoX.UI 1.0

/**
 * QML implementation of the DashboardGrid component
 * This component provides a grid layout for organizing widgets and applications
 */
Item {
    id: root
    
    // Properties
    property var gridModel: grid.items
    property int animationDuration: 250
    property bool editMode: false
    property bool showGridLines: editMode
    
    // Size
    width: grid.gridSize.width * (grid.cellSize.width + grid.spacing) - grid.spacing
    height: grid.gridSize.height * (grid.cellSize.height + grid.spacing) - grid.spacing
    
    // Background
    Rectangle {
        anchors.fill: parent
        color: "#f0f0f0"
        opacity: 0.6
        radius: 4
        
        // Grid lines (only visible in edit mode)
        Grid {
            anchors.fill: parent
            columns: grid.gridSize.width
            rows: grid.gridSize.height
            spacing: grid.spacing
            visible: showGridLines
            
            Repeater {
                model: grid.gridSize.width * grid.gridSize.height
                
                Rectangle {
                    width: grid.cellSize.width
                    height: grid.cellSize.height
                    color: "transparent"
                    border.color: "#80808080"
                    border.width: 1
                    radius: 2
                }
            }
        }
    }
    
    // Items container
    Item {
        id: itemsContainer
        anchors.fill: parent
        
        // Grid items
        Repeater {
            id: itemsRepeater
            model: gridModel
            
            // Grid cell delegate
            Loader {
                id: itemLoader
                
                // Get the model item
                required property var modelData
                
                // Position and size
                x: modelData.position.x * (grid.cellSize.width + grid.spacing)
                y: modelData.position.y * (grid.cellSize.height + grid.spacing)
                width: modelData.size.width * grid.cellSize.width + (modelData.size.width - 1) * grid.spacing
                height: modelData.size.height * grid.cellSize.height + (modelData.size.height - 1) * grid.spacing
                visible: modelData.visible
                
                // Determine source component based on cell type
                sourceComponent: {
                    switch (modelData.type) {
                        case 0: // EmptyCell
                            return emptyCellComponent;
                        case 1: // WidgetCell
                            return widgetCellComponent;
                        case 2: // AppCell
                            return appCellComponent;
                        case 3: // FolderCell
                            return folderCellComponent;
                        case 4: // CustomCell
                            return customCellComponent;
                        default:
                            return emptyCellComponent;
                    }
                }
                
                // Properties for the loaded component
                property var cellData: modelData
                property bool isEditing: editMode
                
                // Animation for position changes
                Behavior on x {
                    enabled: !dragArea.drag.active
                    NumberAnimation {
                        duration: animationDuration
                        easing.type: Easing.OutQuad
                    }
                }
                
                Behavior on y {
                    enabled: !dragArea.drag.active
                    NumberAnimation {
                        duration: animationDuration
                        easing.type: Easing.OutQuad
                    }
                }
                
                // Animation for size changes
                Behavior on width {
                    NumberAnimation {
                        duration: animationDuration
                        easing.type: Easing.OutQuad
                    }
                }
                
                Behavior on height {
                    NumberAnimation {
                        duration: animationDuration
                        easing.type: Easing.OutQuad
                    }
                }
                
                // Drag area (only active in edit mode)
                MouseArea {
                    id: dragArea
                    anchors.fill: parent
                    drag.target: editMode ? parent : undefined
                    drag.smoothed: true
                    drag.threshold: 10
                    
                    // Drag handling
                    property point startPosition
                    property bool isDragging: false
                    
                    onPressed: {
                        if (editMode) {
                            startPosition = Qt.point(parent.x, parent.y);
                            isDragging = true;
                        }
                    }
                    
                    onReleased: {
                        if (editMode && isDragging) {
                            // Calculate grid position
                            var newX = Math.round(parent.x / (grid.cellSize.width + grid.spacing));
                            var newY = Math.round(parent.y / (grid.cellSize.height + grid.spacing));
                            
                            // Try to move the item
                            if (!grid.moveItem(modelData, Qt.point(newX, newY))) {
                                // If move failed, revert to original position
                                parent.x = startPosition.x;
                                parent.y = startPosition.y;
                            }
                            
                            isDragging = false;
                        }
                    }
                    
                    // Context menu
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    onClicked: {
                        if (mouse.button === Qt.RightButton) {
                            contextMenu.popup();
                        }
                    }
                    
                    // Context menu
                    Menu {
                        id: contextMenu
                        
                        MenuItem {
                            text: "Edit"
                            onTriggered: {
                                // Open edit dialog
                                editDialog.open();
                            }
                        }
                        
                        MenuItem {
                            text: "Resize"
                            onTriggered: {
                                // Open resize dialog
                                resizeDialog.open();
                            }
                        }
                        
                        MenuItem {
                            text: "Remove"
                            onTriggered: {
                                // Remove item
                                grid.removeItem(modelData.id);
                            }
                        }
                    }
                    
                    // Edit dialog
                    Dialog {
                        id: editDialog
                        title: "Edit Item"
                        modal: true
                        standardButtons: Dialog.Ok | Dialog.Cancel
                        
                        onAccepted: {
                            // Update item properties
                            modelData.name = nameField.text;
                            modelData.iconName = iconField.text;
                        }
                        
                        contentItem: ColumnLayout {
                            spacing: 10
                            
                            Label {
                                text: "Name:"
                            }
                            
                            TextField {
                                id: nameField
                                Layout.fillWidth: true
                                text: modelData.name
                            }
                            
                            Label {
                                text: "Icon:"
                            }
                            
                            TextField {
                                id: iconField
                                Layout.fillWidth: true
                                text: modelData.iconName
                            }
                        }
                    }
                    
                    // Resize dialog
                    Dialog {
                        id: resizeDialog
                        title: "Resize Item"
                        modal: true
                        standardButtons: Dialog.Ok | Dialog.Cancel
                        
                        onAccepted: {
                            // Update item size
                            var newWidth = parseInt(widthField.text);
                            var newHeight = parseInt(heightField.text);
                            
                            if (newWidth > 0 && newHeight > 0) {
                                grid.resizeItem(modelData, Qt.size(newWidth, newHeight));
                            }
                        }
                        
                        contentItem: ColumnLayout {
                            spacing: 10
                            
                            Label {
                                text: "Width (cells):"
                            }
                            
                            SpinBox {
                                id: widthField
                                Layout.fillWidth: true
                                from: 1
                                to: grid.gridSize.width
                                value: modelData.size.width
                            }
                            
                            Label {
                                text: "Height (cells):"
                            }
                            
                            SpinBox {
                                id: heightField
                                Layout.fillWidth: true
                                from: 1
                                to: grid.gridSize.height
                                value: modelData.size.height
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Component for empty cells
    Component {
        id: emptyCellComponent
        
        Rectangle {
            color: "#f8f8f8"
            border.color: "#e0e0e0"
            border.width: 1
            radius: 4
            opacity: 0.6
            
            Text {
                anchors.centerIn: parent
                text: "Empty"
                color: "#a0a0a0"
                visible: isEditing
            }
        }
    }
    
    // Component for widget cells
    Component {
        id: widgetCellComponent
        
        Rectangle {
            color: "#ffffff"
            border.color: "#d0d0d0"
            border.width: 1
            radius: 6
            
            // Shadow effect
            layer.enabled: true
            layer.effect: DropShadow {
                horizontalOffset: 2
                verticalOffset: 2
                radius: 8.0
                samples: 17
                color: "#30000000"
            }
            
            // Widget content
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 8
                
                // Header
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8
                    
                    // Icon
                    Image {
                        Layout.preferredWidth: 24
                        Layout.preferredHeight: 24
                        source: cellData.iconName ? "image://icon/" + cellData.iconName : ""
                        fillMode: Image.PreserveAspectFit
                        visible: cellData.iconName !== ""
                    }
                    
                    // Title
                    Text {
                        Layout.fillWidth: true
                        text: cellData.name
                        font.bold: true
                        font.pixelSize: 14
                        elide: Text.ElideRight
                    }
                    
                    // Edit button (only visible in edit mode)
                    Button {
                        visible: isEditing
                        icon.name: "edit"
                        flat: true
                        onClicked: {
                            editDialog.open();
                        }
                    }
                }
                
                // Content placeholder
                Rectangle {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    color: "#f5f5f5"
                    radius: 4
                    
                    Text {
                        anchors.centerIn: parent
                        text: "Widget Content"
                        color: "#808080"
                    }
                }
            }
        }
    }
    
    // Component for app cells
    Component {
        id: appCellComponent
        
        Rectangle {
            color: "#f0f8ff"
            border.color: "#c0d0e0"
            border.width: 1
            radius: 6
            
            // Shadow effect
            layer.enabled: true
            layer.effect: DropShadow {
                horizontalOffset: 2
                verticalOffset: 2
                radius: 8.0
                samples: 17
                color: "#30000000"
            }
            
            // App content
            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 8
                
                // App icon
                Image {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.preferredWidth: 48
                    Layout.preferredHeight: 48
                    source: cellData.iconName ? "image://icon/" + cellData.iconName : ""
                    fillMode: Image.PreserveAspectFit
                    
                    // Placeholder if no icon
                    Rectangle {
                        anchors.fill: parent
                        color: "#e0e0e0"
                        radius: width / 4
                        visible: parent.status !== Image.Ready
                        
                        Text {
                            anchors.centerIn: parent
                            text: "App"
                            color: "#808080"
                        }
                    }
                }
                
                // App name
                Text {
                    Layout.fillWidth: true
                    text: cellData.name
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    elide: Text.ElideRight
                    wrapMode: Text.WordWrap
                    maximumLineCount: 2
                }
            }
            
            // Launch app on click
            MouseArea {
                anchors.fill: parent
                enabled: !isEditing
                onClick
(Content truncated due to size limit. Use line ranges to read in chunks)