import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

ProgressBar {
    id: root
    
    // Eigenschaften
    property string progressType: "determinate" // determinate, indeterminate
    property int size: 8 // Höhe der ProgressBar
    property string progressState: "normal" // normal, paused, error
    property bool showValue: false
    property int decimals: 0
    property string valuePrefix: ""
    property string valueSuffix: "%"
    
    // Visuelle Eigenschaften
    implicitWidth: 200
    implicitHeight: size
    
    // Farben basierend auf progressState
    property color progressColor: {
        if (progressState === "error") return VxTheme.colorSemanticError
        if (progressState === "paused") return VxTheme.colorSemanticWarning
        return VxTheme.colorAccentPrimary
    }
    
    // Indeterminate-Animation
    indeterminate: progressType === "indeterminate"
    
    // Hintergrund
    background: Rectangle {
        implicitWidth: root.implicitWidth
        implicitHeight: root.implicitHeight
        color: VxTheme.colorBackgroundSecondary
        radius: height / 2
    }
    
    // Fortschrittsanzeige
    contentItem: Item {
        implicitWidth: root.implicitWidth
        implicitHeight: root.implicitHeight
        
        // Fortschrittsbalken
        Rectangle {
            width: root.indeterminate ? parent.width : root.visualPosition * parent.width
            height: parent.height
            radius: height / 2
            color: root.progressColor
            
            // Indeterminate-Animation
            NumberAnimation on x {
                from: -parent.width
                to: parent.width
                running: root.indeterminate
                loops: Animation.Infinite
                duration: 1500
            }
        }
    }
    
    // Wertanzeige
    Text {
        visible: root.showValue && !root.indeterminate
        text: root.valuePrefix + (root.value * 100).toFixed(root.decimals) + root.valueSuffix
        anchors {
            horizontalCenter: parent.horizontalCenter
            top: parent.bottom
            topMargin: 4
        }
        font.pixelSize: VxTheme.fontSizeSmall
        font.family: VxTheme.fontFamily
        color: root.enabled ? VxTheme.colorTextPrimary : VxTheme.colorTextDisabled
    }
}
