// src/ui/components/widgets/SidebarWidget.qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../../theme"

Rectangle {
    id: root

    // Properties
    property string widgetTitle: "Widget"
    property string widgetContent: "Inhalt"
    property bool isResizing: false

    color: ThemeManager.colorBackgroundSecondary
    radius: ThemeManager.cornerRadiusNormal

    // Schatten für das Widget
    layer.enabled: true
    layer.effect: DropShadow {
        transparentBorder: true
        horizontalOffset: 0
        verticalOffset: 2
        radius: 6.0
        samples: 17
        color: "#30000000"
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        // Widget-Header mit Titel und Aktionsbuttons
        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Label {
                text: widgetTitle
                font.pixelSize: ThemeManager.fontSizeNormal
                font.bold: true
                color: ThemeManager.colorTextPrimary
                Layout.fillWidth: true
            }

            ToolButton {
                icon.name: "configure"
                icon.color: ThemeManager.colorTextSecondary
                icon.width: 16
                icon.height: 16
                implicitWidth: 24
                implicitHeight: 24
                flat: true

                onClicked: {
                    console.log("Widget konfigurieren: " + widgetTitle)
                }
            }

            ToolButton {
                icon.name: "window-close"
                icon.color: ThemeManager.colorTextSecondary
                icon.width: 16
                icon.height: 16
                implicitWidth: 24
                implicitHeight: 24
                flat: true

                onClicked: {
                    root.destroy()
                }
            }
        }

        // Widget-Inhalt
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: ThemeManager.colorBackgroundTertiary
            radius: ThemeManager.cornerRadiusSmall

            Label {
                anchors.centerIn: parent
                text: widgetContent
                color: ThemeManager.colorTextSecondary
                horizontalAlignment: Text.AlignHCenter
                wrapMode: Text.WordWrap
                width: parent.width - 16
            }
        }
    }

    // Resize-Handle in der unteren rechten Ecke
    Rectangle {
        id: resizeHandle
        width: 16
        height: 16
        color: "transparent"
        anchors.right: parent.right
        anchors.bottom: parent.bottom

        Rectangle {
            width: 8
            height: 2
            color: ThemeManager.colorTextSecondary
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.rightMargin: 3
            anchors.bottomMargin: 8
            rotation: 45
        }

        Rectangle {
            width: 14
            height: 2
            color: ThemeManager.colorTextSecondary
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.rightMargin: 3
            anchors.bottomMargin: 5
            rotation: 45
        }

        MouseArea {
            anchors.fill: parent
            cursorShape: Qt.SizeFDiagCursor

            property point startPoint
            property real startHeight

            onPressed: {
                startPoint = Qt.point(mouseX, mouseY)
                startHeight = root.height
                root.isResizing = true
            }

            onPositionChanged: {
                if (pressed) {
                    var deltaY = mouseY - startPoint.y
                    root.height = Math.max(100, startHeight + deltaY)
                }
            }

            onReleased: {
                root.isResizing = false
            }
        }
    }

    // Drag-Handle im Header
    MouseArea {
        id: dragArea
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 40

        property point startPoint
        property point originalPosition

        drag.target: root.parent instanceof Item ? null : root
        drag.axis: Drag.XAndYAxis

        onPressed: {
            startPoint = Qt.point(mouseX, mouseY)
            if (root.parent instanceof Item) {
                originalPosition = mapToItem(root.parent, startPoint.x, startPoint.y)
            }
        }

        onPositionChanged: {
            if (pressed && root.parent instanceof Item) {
                var currentPos = mapToItem(root.parent, mouseX, mouseY)
                var delta = Qt.point(currentPos.x - originalPosition.x, currentPos.y - originalPosition.y)
                root.x += delta.x
                root.y += delta.y
                originalPosition = currentPos
            }
        }
    }
}
