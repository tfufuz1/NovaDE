import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

Item {
    id: root
    
    // Eigenschaften
    property alias text: textField.text
    property alias placeholderText: textField.placeholderText
    property alias textField: textField
    property bool clearButton: true
    property bool searchOnType: false
    property int searchDelay: 300
    
    // Signale
    signal search(string query)
    signal cleared()
    
    // Visuelle Eigenschaften
    implicitWidth: 200
    implicitHeight: textField.height
    
    // Timer für verzögerte Suche
    Timer {
        id: searchTimer
        interval: root.searchDelay
        running: false
        repeat: false
        onTriggered: {
            if (textField.text.length > 0) {
                root.search(textField.text)
            }
        }
    }
    
    // Layout
    RowLayout {
        anchors.fill: parent
        spacing: 0
        
        // Suchicon
        VxIcon {
            source: "qrc:/icons/search.svg"
            size: 16
            color: VxTheme.colorTextSecondary
            Layout.leftMargin: 8
        }
        
        // Textfeld
        VxTextField {
            id: textField
            Layout.fillWidth: true
            Layout.leftMargin: 4
            Layout.rightMargin: clearButton && text.length > 0 ? 0 : 8
            
            background: Item {} // Transparenter Hintergrund
            
            onTextChanged: {
                if (root.searchOnType) {
                    searchTimer.restart()
                }
            }
            
            onAccepted: {
                if (text.length > 0) {
                    root.search(text)
                }
            }
        }
        
        // Löschbutton
        VxButton {
            visible: clearButton && textField.text.length > 0
            buttonType: "icon"
            size: 24
            icon.source: "qrc:/icons/x.svg"
            Layout.rightMargin: 4
            
            onClicked: {
                textField.clear()
                textField.forceActiveFocus()
                root.cleared()
            }
        }
    }
    
    // Hintergrund
    Rectangle {
        z: -1
        anchors.fill: parent
        color: VxTheme.colorBackgroundSecondary
        border.width: textField.activeFocus ? 1 : 0
        border.color: VxTheme.colorBorderFocus
        radius: VxTheme.cornerRadiusSmall
    }
}
