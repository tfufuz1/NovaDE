import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

Button {
    id: root
    
    // Eigenschaften
    property string buttonType: "secondary" // primary, secondary, text, icon
    property int size: 36 // Höhe des Buttons
    property bool loading: false
    property alias icon: contentIcon
    
    // Visuelle Eigenschaften basierend auf buttonType
    property color backgroundColor: {
        if (buttonType === "primary") return VxTheme.colorAccentPrimary
        if (buttonType === "secondary") return VxTheme.colorBackgroundSecondary
        if (buttonType === "text" || buttonType === "icon") return "transparent"
        return VxTheme.colorBackgroundSecondary
    }
    
    property color textColor: {
        if (buttonType === "primary") return "white"
        return VxTheme.colorTextPrimary
    }
    
    property color borderColor: {
        if (buttonType === "primary") return VxTheme.colorAccentPrimary
        if (buttonType === "secondary") return VxTheme.colorBorderNormal
        return "transparent"
    }
    
    property color hoverColor: {
        if (buttonType === "primary") return Qt.darker(VxTheme.colorAccentPrimary, 1.1)
        if (buttonType === "secondary") return VxTheme.colorBackgroundHovered
        if (buttonType === "text" || buttonType === "icon") return VxTheme.colorBackgroundHovered
        return VxTheme.colorBackgroundHovered
    }
    
    property color pressedColor: {
        if (buttonType === "primary") return Qt.darker(VxTheme.colorAccentPrimary, 1.2)
        if (buttonType === "secondary") return Qt.darker(VxTheme.colorBackgroundSecondary, 1.2)
        if (buttonType === "text" || buttonType === "icon") return VxTheme.colorBackgroundSelected
        return Qt.darker(VxTheme.colorBackgroundSecondary, 1.2)
    }
    
    // Größe basierend auf buttonType und size
    implicitWidth: {
        if (buttonType === "icon") return size
        return contentText.implicitWidth + (contentIcon.visible ? contentIcon.width + spacing : 0) + leftPadding + rightPadding
    }
    implicitHeight: size
    
    // Abstände
    padding: buttonType === "icon" ? 0 : 12
    spacing: 8
    
    // Visuelle Eigenschaften
    background: Rectangle {
        color: {
            if (!root.enabled) {
                return buttonType === "primary" ? Qt.rgba(VxTheme.colorAccentPrimary.r, VxTheme.colorAccentPrimary.g, VxTheme.colorAccentPrimary.b, 0.5) : VxTheme.colorBackgroundSecondary
            }
            if (root.pressed) return root.pressedColor
            if (root.hovered) return root.hoverColor
            return root.backgroundColor
        }
        border.width: buttonType === "text" || buttonType === "icon" ? 0 : 1
        border.color: {
            if (!root.enabled) return Qt.rgba(root.borderColor.r, root.borderColor.g, root.borderColor.b, 0.5)
            if (root.activeFocus) return VxTheme.colorBorderFocus
            return root.borderColor
        }
        radius: VxTheme.cornerRadiusSmall
        
        // Ripple-Effekt
        Rectangle {
            id: ripple
            width: 0
            height: 0
            radius: width / 2
            opacity: 0
            color: "white"
            anchors.centerIn: parent
            
            NumberAnimation {
                id: rippleAnimation
                target: ripple
                properties: "width, height"
                from: 0
                to: root.width * 2
                duration: VxTheme.animationDurationNormal
                easing.type: Easing.OutQuad
            }
            
            NumberAnimation {
                id: rippleOpacityAnimation
                target: ripple
                property: "opacity"
                from: 0.3
                to: 0
                duration: VxTheme.animationDurationNormal
                easing.type: Easing.OutQuad
            }
        }
    }
    
    // Inhalt
    contentItem: Item {
        implicitWidth: row.implicitWidth
        implicitHeight: row.implicitHeight
        
        Row {
            id: row
            anchors.centerIn: parent
            spacing: root.spacing
            
            // Ladeindikator
            BusyIndicator {
                id: loadingIndicator
                visible: root.loading
                width: contentText.height
                height: contentText.height
                running: visible
                palette.dark: root.textColor
            }
            
            // Icon
            Image {
                id: contentIcon
                visible: source.toString() !== "" && !root.loading
                width: visible ? height : 0
                height: contentText.height
                fillMode: Image.PreserveAspectFit
                sourceSize.width: width
                sourceSize.height: height
                opacity: root.enabled ? 1.0 : 0.5
            }
            
            // Text
            Text {
                id: contentText
                visible: text !== "" && buttonType !== "icon"
                text: root.text
                font.pixelSize: VxTheme.fontSizeNormal
                font.family: VxTheme.fontFamily
                color: root.enabled ? root.textColor : VxTheme.colorTextDisabled
                horizontalAlignment: Text.AlignHCenter
                verticalAlignment: Text.AlignVCenter
                elide: Text.ElideRight
            }
        }
    }
    
    // Ripple-Effekt beim Klicken
    onPressed: {
        if (buttonType !== "text" && buttonType !== "icon") {
            rippleAnimation.start()
            rippleOpacityAnimation.start()
        }
    }
    
    // Tastaturnavigation
    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()
    Keys.onSpacePressed: clicked()
    
    // Fokus-Highlight
    focusPolicy: Qt.StrongFocus
}
