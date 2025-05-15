import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

Text {
    id: root
    
    // Eigenschaften
    property string labelType: "body" // title, header, subtitle, body, caption
    
    // Visuelle Eigenschaften basierend auf labelType
    font.pixelSize: {
        switch(labelType) {
            case "title": return VxTheme.fontSizeTitle
            case "header": return VxTheme.fontSizeHeader
            case "subtitle": return VxTheme.fontSizeLarge
            case "caption": return VxTheme.fontSizeSmall
            default: return VxTheme.fontSizeNormal // body
        }
    }
    
    font.family: VxTheme.fontFamily
    font.weight: {
        switch(labelType) {
            case "title":
            case "header": 
            case "subtitle": return Font.Medium
            default: return Font.Normal
        }
    }
    
    // Farbe
    color: enabled ? VxTheme.colorTextPrimary : VxTheme.colorTextDisabled
    
    // Standardeinstellungen
    elide: Text.ElideRight
    verticalAlignment: Text.AlignVCenter
    
    // Barrierefreiheit
    Accessible.role: Accessible.StaticText
    Accessible.name: text
}
