// src/ui/components/widgets/WindowGroup.qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../../theme"

Rectangle {
    id: root

    // Properties
    property string groupName: "Fenstergruppe"
    property color groupColor: "#3584e4"
    property int windowCount: 3
    property bool isActive: false

    color: ThemeManager.colorBackgroundSecondary
    radius: ThemeManager.cornerRadiusNormal

    // Farbige Markierung am oberen Rand
    Rectangle {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 4
        color: groupColor
        radius: ThemeManager.cornerRadiusNormal
    }

    // Schatten für die Gruppe
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
        anchors.topMargin: 16
        spacing: 8

        // Gruppenname und Aktionen
        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Rectangle {
                width: 8
                height: 8
                radius: 4
                color: groupColor
            }

            Label {
                text: groupName
                font.pixelSize: ThemeManager.fontSizeNormal
                font.bold: true
                color: ThemeManager.colorTextPrimary
                Layout.fillWidth: true
            }

            Label {
                text: windowCount + " Fenster"
                font.pixelSize: ThemeManager.fontSizeSmall
                color: ThemeManager.colorTextSecondary
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

        // Fenstervorschau im Stapel (würde in einer echten Implementierung Thumbnails der tatsächlichen Fenster zeigen)
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

            // Fensterstapel-Visualisierung
            Repeater {
                model: Math.min(windowCount, 3) // Zeige maximal 3 Fenster-Vorschaubilder

                Rectangle {
                    width: parent.width - index * 20
                    height: parent.height - index * 15
                    x: index * 10
                    y: index * 5
                    color: ThemeManager.colorBackgroundTertiary
                    radius: ThemeManager.cornerRadiusSmall
                    border.width: 1
                    border.color: Qt.rgba(groupColor.r, groupColor.g, groupColor.b, 0.5)

                    // Fenstertitel
                    Rectangle {
                        anchors.top: parent.top
                        anchors.left: parent.left
                        anchors.right: parent.right
                        height: 24
                        color: Qt.rgba(groupColor.r, groupColor.g, groupColor.b, 0.2)
                        radius: ThemeManager.cornerRadiusSmall

                        Label {
                            anchors.centerIn: parent
                            text: "Fenster " + (windowCount - index)
                            font.pixelSize: ThemeManager.fontSizeSmall
                            color: ThemeManager.colorTextSecondary
                        }
                    }
                }
            }

            // Falls mehr als 3 Fenster vorhanden sind, zeige einen Hinweis
            Label {
                anchors.bottom: parent.bottom
                anchors.right: parent.right
                anchors.margins: 8
                visible: windowCount > 3
                text: "+ " + (windowCount - 3) + " weitere"
                font.pixelSize: ThemeManager.fontSizeSmall
                color: ThemeManager.colorTextSecondary
            }
        }

        // Aktionsleiste
        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Button {
                text: "Aktivieren"
                icon.name: "window-new"
                flat: true
                Layout.fillWidth: true

                onClicked: {
                    isActive = true
                    // Würde in einer echten Implementierung die Fenstergruppe aktivieren
                }
            }

            Button {
                text: "Bearbeiten"
                icon.name: "configure"
                flat: true
                Layout.fillWidth: true

                onClicked: {
                    console.log("Gruppe bearbeiten: " + groupName)
                }
            }
        }
    }

    // Status der Gruppe anzeigen
    Rectangle {
        visible: isActive
        anchors.top: parent.top
        anchors.topMargin: 12
        anchors.right: parent.right
        anchors.rightMargin: 12
        width: 8
        height: 8
        radius: 4
        color: ThemeManager.colorSemanticSuccess
    }

    // Hover-Effekt
    states: [
        State {
            name: "hovered"
            when: mouseArea.containsMouse
            PropertyChanges {
                target: root
                color: Qt.rgba(ThemeManager.colorBackgroundSecondary.r,
                               ThemeManager.colorBackgroundSecondary.g,
                               ThemeManager.colorBackgroundSecondary.b,
                               1.2)
            }
        }
    ]

    transitions: [
        Transition {
            to: "*"
            ColorAnimation {
                duration: 150
                easing.type: Easing.InOutQuad
            }
        }
    ]

    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        onClicked: {
            isActive = !isActive
        }
    }
}
