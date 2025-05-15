import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

TabBar {
    id: root
    
    // Eigenschaften
    property string tabPosition: "top" // top, bottom, left, right
    property bool showIcons: false
    property bool showText: true
    property bool showBadges: false
    property bool fillWidth: true
    
    // Visuelle Eigenschaften
    implicitWidth: tabPosition === "left" || tabPosition === "right" ? 120 : contentWidth
    implicitHeight: tabPosition === "left" || tabPosition === "right" ? contentHeight : 48
    
    // Ausrichtung
    position: {
        switch(tabPosition) {
            case "top": return TabBar.Header
            case "bottom": return TabBar.Footer
            case "left": return TabBar.Left
            case "right": return TabBar.Right
            default: return TabBar.Header
        }
    }
    
    // Hintergrund
    background: Rectangle {
        color: VxTheme.colorBackgroundPrimary
        
        // Trennlinie
        Rectangle {
            color: VxTheme.colorBorderLight
            
            // Position und Größe basierend auf tabPosition
            width: tabPosition === "left" || tabPosition === "right" ? 1 : parent.width
            height: tabPosition === "left" || tabPosition === "right" ? parent.height : 1
            
            // Positionierung
            anchors {
                bottom: tabPosition === "top" ? parent.bottom : undefined
                top: tabPosition === "bottom" ? parent.top : undefined
                right: tabPosition === "left" ? parent.right : undefined
                left: tabPosition === "right" ? parent.left : undefined
            }
        }
    }
    
    // Standarddelegat für TabButtons
    contentItem: ListView {
        model: root.contentModel
        currentIndex: root.currentIndex
        
        spacing: 0
        orientation: tabPosition === "left" || tabPosition === "right" ? ListView.Vertical : ListView.Horizontal
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.AutoFlickIfNeeded
        snapMode: ListView.SnapToItem
        
        highlightMoveDuration: 250
        highlightResizeDuration: 0
        
        highlight: Rectangle {
            color: "transparent"
            
            // Indikator für aktiven Tab
            Rectangle {
                color: VxTheme.colorAccentPrimary
                
                // Position und Größe basierend auf tabPosition
                width: tabPosition === "left" || tabPosition === "right" ? 3 : parent.width
                height: tabPosition === "left" || tabPosition === "right" ? parent.height : 3
                
                // Positionierung
                anchors {
                    bottom: tabPosition === "top" ? parent.bottom : undefined
                    top: tabPosition === "bottom" ? parent.top : undefined
                    right: tabPosition === "left" ? parent.right : undefined
                    left: tabPosition === "right" ? parent.left : undefined
                }
            }
        }
    }
}
