import QtQuick 2.15
import QtQuick.Controls 2.15
import "../theme"

Rectangle {
    id: root
    
    // Eigenschaften
    property string separatorType: "horizontal" // horizontal, vertical
    property int thickness: 1
    
    // Visuelle Eigenschaften
    implicitWidth: separatorType === "horizontal" ? 200 : thickness
    implicitHeight: separatorType === "horizontal" ? thickness : 200
    color: VxTheme.colorBorderLight
    
    // Barrierefreiheit
    Accessible.role: Accessible.Separator
}
