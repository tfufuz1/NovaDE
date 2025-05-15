import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

TextField {
    id: root
    
    // Eigenschaften
    property string textFieldType: "normal" // normal, password, number
    property bool error: false
    property string errorText: ""
    property string helperText: ""
    property int size: 36 // Höhe des Textfelds
    
    // Visuelle Eigenschaften
    implicitWidth: 200
    implicitHeight: size
    
    // Textfeld-Konfiguration
    echoMode: textFieldType === "password" ? TextInput.Password : TextInput.Normal
    inputMethodHints: textFieldType === "number" ? Qt.ImhDigitsOnly : Qt.ImhNone
    selectByMouse: true
    
    // Farben
    color: enabled ? VxTheme.colorTextPrimary : VxTheme.colorTextDisabled
    placeholderTextColor: VxTheme.colorTextSecondary
    
    // Hintergrund
    background: Rectangle {
        color: VxTheme.colorBackgroundSecondary
        border.width: 1
        border.color: {
            if (root.error) return VxTheme.colorSemanticError
            if (root.activeFocus) return VxTheme.colorBorderFocus
            return VxTheme.colorBorderNormal
        }
        radius: VxTheme.cornerRadiusSmall
    }
    
    // Hilfe- oder Fehlertext
    Text {
        id: helperTextItem
        anchors {
            top: parent.bottom
            left: parent.left
            topMargin: 4
        }
        text: root.error ? root.errorText : root.helperText
        visible: text !== ""
        color: root.error ? VxTheme.colorSemanticError : VxTheme.colorTextSecondary
        font.pixelSize: VxTheme.fontSizeSmall
        font.family: VxTheme.fontFamily
    }
    
    // Tastaturnavigation
    Keys.onReturnPressed: accepted()
    Keys.onEnterPressed: accepted()
    
    // Fokus-Highlight
    focusPolicy: Qt.StrongFocus
}
