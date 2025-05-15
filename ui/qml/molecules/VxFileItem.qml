import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../atoms"
import "../theme"

Item {
    id: root
    
    // Eigenschaften
    property string fileName: ""
    property string filePath: ""
    property string fileType: ""
    property string fileSize: ""
    property string fileDate: ""
    property bool isDirectory: false
    property bool isSelected: false
    property bool isHidden: false
    property int viewMode: VxTheme.fileViewModeList // list, grid, compact
    
    // Signale
    signal clicked()
    signal doubleClicked()
    signal contextMenuRequested(point position)
    
    // Visuelle Eigenschaften basierend auf viewMode
    implicitWidth: viewMode === VxTheme.fileViewModeGrid ? 120 : (viewMode === VxTheme.fileViewModeCompact ? 240 : 400)
    implicitHeight: viewMode === VxTheme.fileViewModeGrid ? 120 : (viewMode === VxTheme.fileViewModeCompact ? 32 : 48)
    
    // Hintergrund
    Rectangle {
        anchors.fill: parent
        color: {
            if (root.isSelected) return VxTheme.colorBackgroundSelected
            if (mouseArea.containsMouse) return VxTheme.colorBackgroundHovered
            return "transparent"
        }
        radius: VxTheme.cornerRadiusSmall
    }
    
    // Inhalt basierend auf viewMode
    RowLayout {
        anchors.fill: parent
        anchors.margins: 4
        spacing: 8
        visible: viewMode !== VxTheme.fileViewModeGrid
        
        // Datei-Icon
        VxIcon {
            source: {
                if (root.isDirectory) return "qrc:/icons/folder.svg"
                
                // Dateityp-basierte Icons
                switch(root.fileType.toLowerCase()) {
                    case "pdf": return "qrc:/icons/file-pdf.svg"
                    case "doc":
                    case "docx": return "qrc:/icons/file-text.svg"
                    case "xls":
                    case "xlsx": return "qrc:/icons/file-spreadsheet.svg"
                    case "ppt":
                    case "pptx": return "qrc:/icons/file-presentation.svg"
                    case "jpg":
                    case "jpeg":
                    case "png":
                    case "gif": return "qrc:/icons/file-image.svg"
                    case "mp3":
                    case "wav":
                    case "ogg": return "qrc:/icons/file-audio.svg"
                    case "mp4":
                    case "avi":
                    case "mkv": return "qrc:/icons/file-video.svg"
                    case "zip":
                    case "rar":
                    case "tar":
                    case "gz": return "qrc:/icons/file-archive.svg"
                    default: return "qrc:/icons/file.svg"
                }
            }
            size: viewMode === VxTheme.fileViewModeCompact ? 16 : 24
            color: root.isHidden ? VxTheme.colorTextDisabled : VxTheme.colorTextPrimary
            opacity: root.isHidden ? 0.7 : 1.0
        }
        
        // Dateiname
        Text {
            text: root.fileName
            font.pixelSize: VxTheme.fontSizeNormal
            font.family: VxTheme.fontFamily
            color: root.isHidden ? VxTheme.colorTextDisabled : VxTheme.colorTextPrimary
            elide: Text.ElideMiddle
            Layout.fillWidth: true
            opacity: root.isHidden ? 0.7 : 1.0
        }
        
        // Dateigröße (nur im Listenmodus)
        Text {
            visible: viewMode === VxTheme.fileViewModeList
            text: root.fileSize
            font.pixelSize: VxTheme.fontSizeSmall
            font.family: VxTheme.fontFamily
            color: VxTheme.colorTextSecondary
            horizontalAlignment: Text.AlignRight
            Layout.preferredWidth: 80
            opacity: root.isHidden ? 0.7 : 1.0
        }
        
        // Datum (nur im Listenmodus)
        Text {
            visible: viewMode === VxTheme.fileViewModeList
            text: root.fileDate
            font.pixelSize: VxTheme.fontSizeSmall
            font.family: VxTheme.fontFamily
            color: VxTheme.colorTextSecondary
            horizontalAlignment: Text.AlignRight
            Layout.preferredWidth: 120
            opacity: root.isHidden ? 0.7 : 1.0
        }
    }
    
    // Grid-Ansicht
    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 4
        visible: viewMode === VxTheme.fileViewModeGrid
        
        // Icon
        VxIcon {
            source: {
                if (root.isDirectory) return "qrc:/icons/folder.svg"
                
                // Dateityp-basierte Icons
                switch(root.fileType.toLowerCase()) {
                    case "pdf": return "qrc:/icons/file-pdf.svg"
                    case "doc":
                    case "docx": return "qrc:/icons/file-text.svg"
                    case "xls":
                    case "xlsx": return "qrc:/icons/file-spreadsheet.svg"
                    case "ppt":
                    case "pptx": return "qrc:/icons/file-presentation.svg"
                    case "jpg":
                    case "jpeg":
                    case "png":
                    case "gif": return "qrc:/icons/file-image.svg"
                    case "mp3":
                    case "wav":
                    case "ogg": return "qrc:/icons/file-audio.svg"
                    case "mp4":
                    case "avi":
                    case "mkv": return "qrc:/icons/file-video.svg"
                    case "zip":
                    case "rar":
                    case "tar":
                    case "gz": return "qrc:/icons/file-archive.svg"
                    default: return "qrc:/icons/file.svg"
                }
            }
            size: 48
            color: root.isHidden ? VxTheme.colorTextDisabled : VxTheme.colorTextPrimary
            Layout.alignment: Qt.AlignHCenter
            opacity: root.isHidden ? 0.7 : 1.0
        }
        
        // Dateiname
        Text {
            text: root.fileName
            font.pixelSize: VxTheme.fontSizeSmall
            font.family: VxTheme.fontFamily
            color: root.isHidden ? VxTheme.colorTextDisabled : VxTheme.colorTextPrimary
            elide: Text.ElideMiddle
            horizontalAlignment: Text.AlignHCenter
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignHCenter
            opacity: root.isHidden ? 0.7 : 1.0
        }
    }
    
    // Mausinteraktion
    MouseArea {
        id: mouseArea
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        
        onClicked: {
            if (mouse.button === Qt.RightButton) {
                root.contextMenuRequested(Qt.point(mouse.x, mouse.y))
            } else {
                root.clicked()
            }
        }
        
        onDoubleClicked: {
            if (mouse.button === Qt.LeftButton) {
                root.doubleClicked()
            }
        }
    }
}
