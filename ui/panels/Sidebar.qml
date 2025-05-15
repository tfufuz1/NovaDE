// src/ui/components/panels/Sidebar.qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../../theme"
import "../widgets"

Rectangle {
    id: sidebar
    color: ThemeManager.colorBackgroundPrimary

    // Schatten für die Sidebar
    layer.enabled: true
    layer.effect: DropShadow {
        transparentBorder: true
        horizontalOffset: 2
        verticalOffset: 0
        radius: 8.0
        samples: 17
        color: "#40000000"
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Tabs für verschiedene Seitenleistenfunktionen
        TabBar {
            id: sidebarTabs
            Layout.fillWidth: true
            position: TabBar.Header

            TabButton {
                text: "Widgets"
                icon.name: "dashboard"
            }

            TabButton {
                text: "Fenster"
                icon.name: "window"
            }

            TabButton {
                text: "Apps"
                icon.name: "applications-all"
            }
        }

        // Stacklayout für die verschiedenen Tabs
        StackLayout {
            currentIndex: sidebarTabs.currentIndex
            Layout.fillWidth: true
            Layout.fillHeight: true

            // Widget-Board
            ScrollView {
                id: widgetBoardView
                clip: true

                ColumnLayout {
                    width: sidebar.width - 20
                    spacing: 12
                    padding: 16

                    Label {
                        text: "Widgets"
                        font.pixelSize: ThemeManager.fontSizeHeader
                        color: ThemeManager.colorTextPrimary
                    }

                    // Platzhalter für Widgets (in der echten Implementierung dynamisch)
                    Repeater {
                        model: 5

                        SidebarWidget {
                            Layout.fillWidth: true
                            height: 120
                            widgetTitle: "Widget " + (index + 1)
                            widgetContent: "Inhalt für Widget " + (index + 1)
                        }
                    }

                    Button {
                        text: "Widget hinzufügen"
                        icon.name: "list-add"
                        Layout.fillWidth: true
                        flat: true
                    }
                }
            }

            // Stage Manager / Fensterverwaltung
            ScrollView {
                id: stageManagerView
                clip: true

                ColumnLayout {
                    width: sidebar.width - 20
                    spacing: 12
                    padding: 16

                    Label {
                        text: "Fenstergruppen"
                        font.pixelSize: ThemeManager.fontSizeHeader
                        color: ThemeManager.colorTextPrimary
                    }

                    // Platzhalter für Fenstergruppen (in der echten Implementierung dynamisch vom WindowManager)
                    Repeater {
                        model: 3

                        WindowGroup {
                            Layout.fillWidth: true
                            height: 160
                            groupName: "Gruppe " + (index + 1)
                            groupColor: index === 0 ? "#3584e4" : index === 1 ? "#e01b24" : "#2ec27e"
                            windowCount: index + 2
                        }
                    }

                    Button {
                        text: "Neue Fenstergruppe"
                        icon.name: "window-new"
                        Layout.fillWidth: true
                        flat: true
                    }
                }
            }

            // App-Launcher
            GridView {
                id: appGridView
                clip: true
                cellWidth: sidebar.width / 3
                cellHeight: 90

                model: 15

                delegate: Item {
                    width: appGridView.cellWidth
                    height: appGridView.cellHeight

                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 4

                        Rectangle {
                            width: 48
                            height: 48
                            radius: 8
                            color: Qt.rgba(Math.random(), Math.random(), Math.random(), 0.7)
                            Layout.alignment: Qt.AlignHCenter
                        }

                        Label {
                            text: "App " + (index + 1)
                            color: ThemeManager.colorTextPrimary
                            font.pixelSize: ThemeManager.fontSizeSmall
                            horizontalAlignment: Text.AlignHCenter
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            console.log("App " + (index + 1) + " geklickt")
                        }
                    }
                }
            }
        }
    }
}
