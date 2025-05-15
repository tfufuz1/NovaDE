import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

Dialog {
    id: root
    
    // Eigenschaften
    property string dialogType: "custom" // alert, confirm, prompt, custom
    property string iconType: "" // info, success, warning, error
    property string promptText: ""
    property alias promptValue: promptField.text
    property bool showIcon: iconType !== ""
    
    // Signale
    signal confirmed()
    signal cancelled()
    
    // Visuelle Eigenschaften
    width: Math.min(parent.width * 0.9, 480)
    height: contentColumn.implicitHeight + footer.implicitHeight + 48
    
    // Zentrieren im Elternelement
    x: (parent.width - width) / 2
    y: (parent.height - height) / 2
    
    // Modal und mit Overlay
    modal: true
    closePolicy: Popup.CloseOnEscape
    
    // Hintergrund mit Schatten
    background: VxCard {
        cardType: "raised"
        elevation: 3
    }
    
    // Header, Inhalt und Eingabefeld
    contentItem: ColumnLayout {
        id: contentColumn
        spacing: 16
        anchors {
            left: parent.left
            right: parent.right
            top: parent.top
            margins: 24
        }
        
        // Icon (optional)
        VxIcon {
            visible: root.showIcon
            source: {
                switch(root.iconType) {
                    case "info": return "qrc:/icons/info.svg"
                    case "success": return "qrc:/icons/check-circle.svg"
                    case "warning": return "qrc:/icons/alert-triangle.svg"
                    case "error": return "qrc:/icons/alert-circle.svg"
                    default: return ""
                }
            }
            size: 48
            color: {
                switch(root.iconType) {
                    case "info": return VxTheme.colorSemanticInfo
                    case "success": return VxTheme.colorSemanticSuccess
                    case "warning": return VxTheme.colorSemanticWarning
                    case "error": return VxTheme.colorSemanticError
                    default: return VxTheme.colorTextPrimary
                }
            }
            Layout.alignment: Qt.AlignHCenter
        }
        
        // Titel
        VxLabel {
            text: root.title
            labelType: "header"
            horizontalAlignment: Text.AlignHCenter
            Layout.fillWidth: true
            visible: text !== ""
        }
        
        // Inhalt
        Text {
            text: root.dialogType === "prompt" ? root.promptText : root.text
            font.pixelSize: VxTheme.fontSizeNormal
            font.family: VxTheme.fontFamily
            color: VxTheme.colorTextPrimary
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignLeft
            Layout.fillWidth: true
        }
        
        // Eingabefeld für Prompt-Dialog
        VxTextField {
            id: promptField
            visible: root.dialogType === "prompt"
            Layout.fillWidth: true
            focus: visible
            onAccepted: {
                root.confirmed()
                root.accept()
            }
        }
    }
    
    // Buttons im Footer
    footer: DialogButtonBox {
        alignment: Qt.AlignRight
        background: Rectangle {
            color: "transparent"
        }
        
        // Abbrechen-Button (für confirm, prompt, custom)
        Button {
            id: cancelButton
            text: qsTr("Abbrechen")
            visible: root.dialogType !== "alert"
            DialogButtonBox.buttonRole: DialogButtonBox.RejectRole
            
            // Styling
            background: Rectangle {
                color: cancelButton.down ? VxTheme.colorBackgroundSelected : 
                       (cancelButton.hovered ? VxTheme.colorBackgroundHovered : "transparent")
                radius: VxTheme.cornerRadiusSmall
            }
            
            contentItem: Text {
                text: cancelButton.text
                font.pixelSize: VxTheme.fontSizeNormal
                font.family: VxTheme.fontFamily
                color: VxTheme.colorTextPrimary
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
            
            onClicked: {
                root.cancelled()
            }
        }
        
        // OK/Bestätigen-Button (für alle Typen)
        Button {
            id: confirmButton
            text: {
                switch(root.dialogType) {
                    case "alert": return qsTr("OK")
                    case "confirm": return qsTr("Bestätigen")
                    case "prompt": return qsTr("Bestätigen")
                    default: return qsTr("OK")
                }
            }
            DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
            
            // Styling
            background: Rectangle {
                color: confirmButton.down ? Qt.darker(VxTheme.colorAccentPrimary, 1.2) : 
                       (confirmButton.hovered ? Qt.darker(VxTheme.colorAccentPrimary, 1.1) : VxTheme.colorAccentPrimary)
                radius: VxTheme.cornerRadiusSmall
            }
            
            contentItem: Text {
                text: confirmButton.text
                font.pixelSize: VxTheme.fontSizeNormal
                font.family: VxTheme.fontFamily
                color: "white"
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
            }
            
            onClicked: {
                root.confirmed()
            }
        }
    }
    
    // Animationen
    enter: Transition {
        NumberAnimation { 
            property: "opacity"
            from: 0.0
            to: 1.0
            duration: VxTheme.animationDurationNormal
            easing.type: Easing.OutQuad
        }
        
        NumberAnimation { 
            property: "scale"
            from: 0.9
            to: 1.0
            duration: VxTheme.animationDurationNormal
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
    
    // Overlay
    Overlay.modal: Rectangle {
        color: "#80000000"
        
        // Animation für Overlay
        Behavior on opacity {
            NumberAnimation { 
                duration: VxTheme.animationDurationNormal
                easing.type: Easing.InOutQuad
            }
        }
    }
}
