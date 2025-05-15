// src/ui/components/widgets/ControlToggle.qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../../theme"

Item {
    id: root

    // Properties
    property alias text: label.text
    property alias icon: iconImage
    property alias checked: toggle.checked

    signal toggled(bool checked)

    implicitWidth: 90
    implicitHeight: 90

    Rectangle {
        anchors.fill: parent
        radius: ThemeManager.cornerRadiusSmall
        color: toggle.checked ? Qt.rgba(ThemeManager.colorAccentPrimary.r,
                                        ThemeManager.colorAccentPrimary.g,
                                        ThemeManager.colorAccentPrimary.b, 0.2) :
                                        Qt.rgba(ThemeManager.colorBackgroundTertiary.r,
                                                ThemeManager.colorBackgroundTertiary.g,
                                                ThemeManager.colorBackgroundTertiary.b, 0.5)
    }

    ColumnLayout {
        anchors.centerIn: parent
        spacing: 8

        Image {
            id: iconImage
            source: ""
            width: 32
            height: 32
            sourceSize: Qt.size(32, 32)
            Layout.alignment: Qt.AlignHCenter

            ColorOverlay {
                anchors.fill: parent
                source: parent
                color: toggle.checked ? ThemeManager.colorAccentPrimary : ThemeManager.colorTextSecondary
            }
        }

        Label {
            id: label
            text: "Toggle"
            color: toggle.checked ? ThemeManager.colorTextPrimary : ThemeManager.colorTextSecondary
            font.pixelSize: ThemeManager.fontSizeSmall
            Layout.alignment: Qt.AlignHCenter
        }

        Switch {
            id: toggle
            Layout.alignment: Qt.AlignHCenter

            onToggled: {
                root.toggled(checked)
            }
        }
    }

    MouseArea {
        anchors.fill: parent
        onClicked: toggle.toggle()
    }
}
