import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

Image {
    id: root
    
    // Eigenschaften
    property int size: 24
    property alias color: colorOverlay.color
    
    // Visuelle Eigenschaften
    width: size
    height: size
    fillMode: Image.PreserveAspectFit
    sourceSize.width: width
    sourceSize.height: height
    
    // Farbe
    layer.enabled: true
    layer.effect: ColorOverlay {
        id: colorOverlay
        color: VxTheme.colorTextPrimary
    }
    
    // Barrierefreiheit
    Accessible.role: Accessible.Graphic
    Accessible.name: source.toString().split("/").pop().split(".")[0]
}
