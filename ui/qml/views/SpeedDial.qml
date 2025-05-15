// src/ui/components/SpeedDial.qml
import QtQuick 2.15
import QtQuick.Layouts 1.15
import QtQuick.Controls 2.15
import "../theme"
import "../controls"

Rectangle {
    id: root

    // Properties
    property string wallpaperSource: "/assets/wallpapers/default.jpg"
    property bool showWidgetArea: true
    property int gridColumns: 6
    property int gridRows: 4
    property int iconSize: 64

    // Background
    color: "transparent"

    // Wallpaper
    Image {
        id: wallpaper
        anchors.fill: parent
        source: root.wallpaperSource
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        cache: true

        // Darkening overlay for better readability
        Rectangle {
            anchors.fill: parent
            color: "#40000000"
            opacity: 0.3
        }
    }

    // Main layout
    Item {
        anchors.fill: parent

        // App grid area
        Rectangle {
            id: appGridContainer
            anchors {
                top: parent.top
                left: parent.left
                right: widgetArea.visible ? widgetArea.left : parent.right
                bottom: parent.bottom
                margins: ThemeManager.spacingUnit * 2
            }
            color: "transparent"

            // App grid
            GridView {
                id: appGrid
                anchors.fill: parent
                cellWidth: appGridContainer.width / root.gridColumns
                cellHeight: appGridContainer.height / root.gridRows
                model: AppModel

                delegate: Item {
                    width: appGrid.cellWidth
                    height: appGrid.cellHeight

                    Column {
                        anchors.centerIn: parent
                        spacing: ThemeManager.spacingUnit

                        // App icon
                        Rectangle {
                            id: iconBackground
                            width: root.iconSize
                            height: root.iconSize
                            radius: ThemeManager.cornerRadiusNormal
                            color: ThemeManager.colorBackgroundSecondary
                            anchors.horizontalCenter: parent.horizontalCenter

                            Image {
                                id: appIcon
                                anchors.centerIn: parent
                                width: parent.width * 0.7
                                height: parent.height * 0.7
                                source: model.iconPath
                                sourceSize: Qt.size(width, height)
                                asynchronous: true
                                cache: true
                            }

                            // Selection/hover effect
                            Rectangle {
                                id: selectionIndicator
                                anchors.fill: parent
                                radius: parent.radius
                                color: ThemeManager.colorAccentPrimary
                                opacity: 0
                            }

                            // Click animation
                            ParallelAnimation {
                                id: clickAnimation

                                NumberAnimation {
                                    target: iconBackground
                                    property: "scale"
                                    from: 1.0
                                    to: 0.9
                                    duration: ThemeManager.animationDurationFast
                                    easing.type: Easing.OutQuad
                                }

                                NumberAnimation {
                                    target: iconBackground
                                    property: "scale"
                                    from: 0.9
                                    to: 1.0
                                    duration: ThemeManager.animationDurationFast
                                    easing.type: Easing.OutBounce
                                    startDelay: ThemeManager.animationDurationFast
                                }
                            }
                        }

                        // App label
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: model.name
                            color: ThemeManager.colorTextPrimary
                            font.pixelSize: ThemeManager.fontSizeSmall
                            elide: Text.ElideRight
                            horizontalAlignment: Text.AlignHCenter
                            width: appGrid.cellWidth - ThemeManager.spacingUnit * 2
                        }
                    }

                    // Interaction
                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true

                        onEntered: {
                            selectionIndicator.opacity = 0.2
                        }

                        onExited: {
                            selectionIndicator.opacity = 0
                        }

                        onPressed: {
                            selectionIndicator.opacity = 0.4
                        }

                        onClicked: {
                            clickAnimation.start()
                            ApplicationLauncher.launchApp(model.appId)
                        }
                    }
                }

                // Scroll indicator
                ScrollIndicator.vertical: ScrollIndicator { }
            }

            // Search field at the top
            Rectangle {
                id: searchContainer
                anchors {
                    top: parent.top
                    left: parent.left
                    right: parent.right
                }
                height: 48
                color: ThemeManager.colorBackgroundPrimary
                radius: height / 2
                opacity: 0.8

                SearchField {
                    id: searchField
                    anchors.fill: parent
                    anchors.margins: ThemeManager.spacingUnit
                    placeholderText: "Search applications..."

                    onTextChanged: {
                        AppModel.filterByName(text)
                    }
                }
            }
        }

        // Widgets area
        Rectangle {
            id: widgetArea
            visible: root.showWidgetArea
            anchors {
                top: parent.top
                right: parent.right
                bottom: parent.bottom
                margins: ThemeManager.spacingUnit * 2
            }
            width: parent.width * 0.3
            color: Qt.rgba(
                ThemeManager.colorBackgroundPrimary.r,
                ThemeManager.colorBackgroundPrimary.g,
                ThemeManager.colorBackgroundPrimary.b,
                0.7
            )
            radius: ThemeManager.cornerRadiusLarge

            // Widget area container
            Flickable {
                id: widgetFlickable
                anchors.fill: parent
                anchors.margins: ThemeManager.spacingUnit
                contentHeight: widgetColumn.height
                clip: true

                // Column layout for widgets
                Column {
                    id: widgetColumn
                    width: parent.width
                    spacing: ThemeManager.spacingUnit * 2

                    // Dynamic widgets will be added here
                    Repeater {
                        id: widgetRepeater
                        model: WidgetModel

                        delegate: Loader {
                            width: widgetColumn.width
                            height: item ? item.height : 100
                            source: model.widgetSource

                            // Pass properties to loaded widget
                            onLoaded: {
                                if (model.properties) {
                                    for (var prop in model.properties) {
                                        item[prop] = model.properties[prop]
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Scroll indicator
            ScrollIndicator {
                anchors.right: widgetFlickable.right
                anchors.top: widgetFlickable.top
                anchors.bottom: widgetFlickable.bottom
            }

            // Add widget button
            RoundButton {
                anchors {
                    right: parent.right
                    bottom: parent.bottom
                    margins: ThemeManager.spacingUnit * 2
                }
                width: 48
                height: 48
                icon.source: "/assets/icons/add.svg"
                icon.color: ThemeManager.colorTextPrimary

                background: Rectangle {
                    radius: width / 2
                    color: ThemeManager.colorBackgroundSecondary
                }

                onClicked: {
                    widgetPickerDialog.open()
                }
            }
        }
    }

    // Widget picker dialog (simplified)
    Dialog {
        id: widgetPickerDialog
        title: "Add Widget"
        modal: true
        anchors.centerIn: parent
        width: parent.width * 0.6
        height: parent.height * 0.7

        // Implementation would continue here...
    }

    // Methods for managing widgets
    function addWidget(widgetUrl, properties) {
        WidgetModel.addWidget({
            widgetSource: widgetUrl,
            properties: properties || {}
        })
    }

    function removeWidget(index) {
        WidgetModel.removeWidget(index)
    }
}
