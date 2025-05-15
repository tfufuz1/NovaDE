import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

Popup {
    id: root
    
    // Eigenschaften
    property string notificationType: "info" // info, success, warning, error
    property string message: ""
    property string actionText: ""
    property int displayTime: 5000 // ms, 0 für unbegrenzt
    property bool showIcon: true
    property string position: "bottom" // top, bottom
    
    // Signale
    signal actionClicked()
    signal closed()
    
    // Visuelle Eigenschaften
    width: Math.min(parent.width * 0.9, 400)
    height: contentLayout.implicitHeight + 32
    
    // Positionierung
    x: (parent.width - width) / 2
    y: position === "top" ? 20 : (parent.height - height - 20)
    
    // Automatisches Schließen
    Timer {
        id: closeTimer
        interval: root.displayTime
        running: root.displayTime > 0 && root.visible
        repeat: false
        onTriggered: {
            closeAnimation.start()
        }
    }
    
    // Hintergrund
    background: Rectangle {
        color: VxTheme.colorBackgroundSecondary
        radius: VxTheme.cornerRadiusNormal
        
        // Farbiger Indikator am linken Rand
        Rectangle {
            width: 4
            height: parent.height
            anchors.left: parent.left
            color: {
                switch(root.notificationType) {
                    case "info": return VxTheme.colorSemanticInfo
                    case "success": return VxTheme.colorSemanticSuccess
                    case "warning": return VxTheme.colorSemanticWarning
                    case "error": return VxTheme.colorSemanticError
                    default: return VxTheme.colorSemanticInfo
                }
            }
            radius: VxTheme.cornerRadiusNormal
            // Nur die linke Seite soll abgerundet sein
            Rectangle {
                width: parent.width / 2
                height: parent.height
                anchors.right: parent.right
                color: parent.color
            }
        }
        
        // Schatten
        layer.enabled: true
        layer.effect: DropShadow {
            horizontalOffset: 0
            verticalOffset: 2
            radius: 8
            samples: 17
            color: "#40000000"
        }
    }
    
    // Inhalt
    contentItem: RowLayout {
        id: contentLayout
        spacing: 12
        anchors {
            fill: parent
            leftMargin: 16
            rightMargin: 16
            topMargin: 16
            bottomMargin: 16
        }
        
        // Icon
        VxIcon {
            visible: root.showIcon
            source: {
                switch(root.notificationType) {
                    case "info": return "qrc:/icons/info.svg"
                    case "success": return "qrc:/icons/check-circle.svg"
                    case "warning": return "qrc:/icons/alert-triangle.svg"
                    case "error": return "qrc:/icons/alert-circle.svg"
                    default: return "qrc:/icons/info.svg"
                }
            }
            size: 24
            color: {
                switch(root.notificationType) {
                    case "info": return VxTheme.colorSemanticInfo
                    case "success": return VxTheme.colorSemanticSuccess
                    case "warning": return VxTheme.colorSemanticWarning
                    case "error": return VxTheme.colorSemanticError
                    default: return VxTheme.colorSemanticInfo
                }
            }
        }
        
        // Nachricht
        Text {
            text: root.message
            font.pixelSize: VxTheme.fontSizeNormal
            font.family: VxTheme.fontFamily
            color: VxTheme.colorTextPrimary
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }
        
        // Aktionsbutton
        VxButton {
            visible: root.actionText !== ""
            text: root.actionText
            buttonType: "text"
            onClicked: {
                root.actionClicked()
            }
        }
        
        // Schließen-Button
        VxButton {
            buttonType: "icon"
            icon.source: "qrc:/icons/x.svg"
            size: 32
            onClicked: {
                closeAnimation.start()
            }
        }
    }
    
    // Einblendanimation
    enter: Transition {
        NumberAnimation { 
            property: "opacity"
            from: 0.0
            to: 1.0
            duration: VxTheme.animationDurationNormal
            easing.type: Easing.OutQuad
        }
        
        NumberAnimation { 
            property: "y"
            from: position === "top" ? -root.height : parent.height
            to: position === "top" ? 20 : (parent.height - root.height - 20)
            duration: VxTheme.animationDurationNormal
            easing.type: Easing.OutQuad
        }
    }
    
    // Ausblendanimation
    exit: Transition {
        NumberAnimation { 
            property: "opacity"
            from: 1.0
            to: 0.0
            duration: VxTheme.animationDurationFast
            easing.type: Easing.InQuad
        }
    }
    
    // Benutzerdefinierte Ausblendanimation
    SequentialAnimation {
        id: closeAnimation
        
        NumberAnimation { 
            target: root
            property: "opacity"
            from: 1.0
            to: 0.0
            duration: VxTheme.animationDurationFast
            easing.type: Easing.InQuad
        }
        
        ScriptAction {
            script: {
                root.close()
                root.closed()
            }
        }
    }
}
