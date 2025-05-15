/**
 * @file BaseWidget.qml
 * @brief Basiskomponente für alle Widgets
 * 
 * Diese Datei enthält die Basiskomponente für alle Widgets im VivoX-System.
 * Sie definiert die grundlegende Struktur und Funktionalität, die alle Widgets gemeinsam haben.
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import VivoX.UI 1.0
import VivoX.Animation 1.0

Item {
    id: root

    // Eigenschaften
    property string widgetId: ""
    property string widgetName: ""
    property string widgetDescription: ""
    property string widgetIconName: "widget"
    property string widgetCategory: "misc"
    property int widgetSize: WidgetSize.Small
    property bool widgetResizable: false
    property bool widgetConfigurable: false
    property var widgetConfig: ({})

    // Signale
    signal configureRequested()
    signal closeRequested()
    signal moveRequested(var dragEvent)
    signal resizeRequested(var resizeEvent)
    signal widgetClicked()
    signal widgetDoubleClicked()
    signal widgetRightClicked(var mouseEvent)

    // Standardgröße basierend auf widgetSize
    property var sizeMap: {
        "0": Qt.size(100, 100),   // Small
        "1": Qt.size(200, 200),   // Medium
        "2": Qt.size(400, 200),   // Large
        "3": Qt.size(400, 400),   // ExtraLarge
        "4": Qt.size(width, height) // Custom
    }

    // Standardgröße setzen
    width: sizeMap[widgetSize].width
    height: sizeMap[widgetSize].height

    // Minimale und maximale Größe
    property int minimumWidth: 100
    property int minimumHeight: 100
    property int maximumWidth: 800
    property int maximumHeight: 800

    // Zustände
    property bool isSelected: false
    property bool isHovered: false
    property bool isDragging: false
    property bool isResizing: false
    property bool isConfiguring: false

    // Animation-Eigenschaften
    property int animationDuration: 200
    property int hoverAnimationDuration: 100

    // Hauptrechteck des Widgets
    Rectangle {
        id: background
        anchors.fill: parent
        color: "#2E3440"
        radius: 8
        border.width: isSelected ? 2 : 1
        border.color: isSelected ? "#88C0D0" : "#434C5E"

        // Schatten
        layer.enabled: true
        layer.effect: DropShadow {
            horizontalOffset: 0
            verticalOffset: 2
            radius: 8.0
            samples: 17
            color: "#30000000"
        }

        // Hover-Effekt
        states: [
            State {
                name: "hovered"
                when: isHovered && !isDragging && !isResizing
                PropertyChanges {
                    target: background
                    color: "#353B49"
                }
            },
            State {
                name: "selected"
                when: isSelected
                PropertyChanges {
                    target: background
                    color: "#3B4252"
                    border.color: "#88C0D0"
                    border.width: 2
                }
            },
            State {
                name: "dragging"
                when: isDragging
                PropertyChanges {
                    target: background
                    color: "#3B4252"
                    opacity: 0.8
                }
            }
        ]

        transitions: [
            Transition {
                from: "*"
                to: "*"
                ColorAnimation {
                    duration: hoverAnimationDuration
                    easing.type: Easing.OutQuad
                }
                PropertyAnimation {
                    properties: "opacity"
                    duration: animationDuration
                    easing.type: Easing.OutQuad
                }
            }
        ]
    }

    // Titel des Widgets
    Rectangle {
        id: titleBar
        anchors {
            top: parent.top
            left: parent.left
            right: parent.right
        }
        height: 30
        color: "#3B4252"
        radius: 8
        
        // Nur die oberen Ecken abrunden
        Rectangle {
            anchors {
                left: parent.left
                right: parent.right
                bottom: parent.bottom
            }
            height: parent.radius
            color: parent.color
        }

        // Icon
        Image {
            id: titleIcon
            anchors {
                left: parent.left
                leftMargin: 8
                verticalCenter: parent.verticalCenter
            }
            width: 16
            height: 16
            source: "qrc:/icons/" + widgetIconName + ".svg"
            fillMode: Image.PreserveAspectFit
        }

        // Titel
        Text {
            id: titleText
            anchors {
                left: titleIcon.right
                leftMargin: 8
                right: configButton.left
                rightMargin: 8
                verticalCenter: parent.verticalCenter
            }
            text: widgetName
            color: "#ECEFF4"
            font.pixelSize: 12
            font.bold: true
            elide: Text.ElideRight
        }

        // Konfigurationsbutton
        Button {
            id: configButton
            anchors {
                right: closeButton.left
                rightMargin: 4
                verticalCenter: parent.verticalCenter
            }
            width: 24
            height: 24
            visible: widgetConfigurable
            icon.source: "qrc:/icons/settings.svg"
            icon.color: "#D8DEE9"
            background: Rectangle {
                color: "transparent"
                radius: 4
            }
            
            onClicked: {
                configureRequested()
            }
        }

        // Schließen-Button
        Button {
            id: closeButton
            anchors {
                right: parent.right
                rightMargin: 4
                verticalCenter: parent.verticalCenter
            }
            width: 24
            height: 24
            icon.source: "qrc:/icons/close.svg"
            icon.color: "#D8DEE9"
            background: Rectangle {
                color: "transparent"
                radius: 4
            }
            
            onClicked: {
                closeRequested()
            }
        }

        // Drag-Bereich
        MouseArea {
            id: dragArea
            anchors.fill: parent
            anchors.rightMargin: configButton.width + closeButton.width + 8
            drag.target: root.parent ? null : root // Nur ziehen, wenn kein Elternelement vorhanden ist
            drag.axis: Drag.XAndYAxis
            drag.minimumX: 0
            drag.minimumY: 0
            drag.maximumX: root.parent ? root.parent.width - root.width : 0
            drag.maximumY: root.parent ? root.parent.height - root.height : 0
            
            onPressed: {
                isSelected = true
                isDragging = true
                moveRequested(mouse)
            }
            
            onReleased: {
                isDragging = false
            }
            
            onPositionChanged: {
                if (pressed) {
                    moveRequested(mouse)
                }
            }
        }
    }

    // Inhaltsbereich des Widgets
    Item {
        id: contentArea
        anchors {
            top: titleBar.bottom
            left: parent.left
            right: parent.right
            bottom: parent.bottom
            margins: 8
        }
        
        // Hier wird der eigentliche Inhalt des Widgets platziert
        // Unterklassen sollten hier ihre eigenen Elemente hinzufügen
    }

    // Resize-Handle (nur sichtbar, wenn das Widget größenveränderbar ist)
    Rectangle {
        id: resizeHandle
        anchors {
            right: parent.right
            bottom: parent.bottom
        }
        width: 16
        height: 16
        color: "transparent"
        visible: widgetResizable
        
        Image {
            anchors.fill: parent
            source: "qrc:/icons/resize.svg"
            fillMode: Image.PreserveAspectFit
        }
        
        MouseArea {
            id: resizeArea
            anchors.fill: parent
            
            onPressed: {
                isResizing = true
            }
            
            onReleased: {
                isResizing = false
            }
            
            onPositionChanged: {
                if (pressed) {
                    var newWidth = Math.max(minimumWidth, Math.min(maximumWidth, root.width + mouse.x))
                    var newHeight = Math.max(minimumHeight, Math.min(maximumHeight, root.height + mouse.y))
                    
                    root.width = newWidth
                    root.height = newHeight
                    
                    resizeRequested({
                        width: newWidth,
                        height: newHeight
                    })
                }
            }
        }
    }

    // Mausbereich für Hover-Effekte und Klick-Events
    MouseArea {
        id: mouseArea
        anchors.fill: parent
        anchors.topMargin: titleBar.height // Nicht mit dem Titel-Bereich überlappen
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        
        onEntered: {
            isHovered = true
        }
        
        onExited: {
            isHovered = false
        }
        
        onClicked: {
            isSelected = true
            if (mouse.button === Qt.RightButton) {
                widgetRightClicked(mouse)
            } else {
                widgetClicked()
            }
        }
        
        onDoubleClicked: {
            widgetDoubleClicked()
        }
    }

    // Komponente wird initialisiert
    Component.onCompleted: {
        // Standardwerte setzen, falls nicht definiert
        if (!widgetName) {
            widgetName = "Widget"
        }
    }
}
