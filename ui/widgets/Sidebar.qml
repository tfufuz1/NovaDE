/**
 * @file Sidebar.qml
 * @brief Seitenleiste für Schnellzugriff auf Anwendungen und Widgets
 * 
 * Diese Datei enthält die Seitenleiste, die Schnellzugriff auf Anwendungen,
 * Widgets und andere Funktionen bietet.
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import VivoX.UI 1.0
import VivoX.System 1.0

Item {
    id: root
    
    // Eigenschaften
    property bool expanded: false
    property int collapsedWidth: 60
    property int expandedWidth: 300
    property string activeCategory: "favorites"
    
    // Signale
    signal appLaunched(string appId)
    
    // Animation für Breitenänderung
    Behavior on width {
        NumberAnimation {
            duration: 300
            easing.type: Easing.OutCubic
        }
    }
    
    // Hintergrund
    Rectangle {
        id: background
        anchors.fill: parent
        color: "#2E3440"
        opacity: 0.95
        
        // Schatten
        layer.enabled: true
        layer.effect: DropShadow {
            horizontalOffset: 2
            verticalOffset: 0
            radius: 12.0
            samples: 25
            color: "#40000000"
        }
    }
    
    // Hauptlayout
    RowLayout {
        anchors.fill: parent
        spacing: 0
        
        // Kategorie-Leiste (immer sichtbar)
        Rectangle {
            id: categoryBar
            Layout.fillHeight: true
            Layout.preferredWidth: root.collapsedWidth
            color: "#3B4252"
            
            // Kategorien
            ColumnLayout {
                anchors.fill: parent
                spacing: 0
                
                // Logo
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: root.collapsedWidth
                    color: "#4C566A"
                    
                    Image {
                        anchors.centerIn: parent
                        width: 32
                        height: 32
                        source: "qrc:/icons/logo.svg"
                        fillMode: Image.PreserveAspectFit
                    }
                    
                    // Erweiterungsbutton
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            root.expanded = !root.expanded
                        }
                    }
                }
                
                // Favoriten
                CategoryButton {
                    icon.source: "qrc:/icons/star.svg"
                    active: root.activeCategory === "favorites"
                    
                    onClicked: {
                        root.activeCategory = "favorites"
                        if (!root.expanded) {
                            root.expanded = true
                        }
                    }
                }
                
                // Alle Apps
                CategoryButton {
                    icon.source: "qrc:/icons/apps.svg"
                    active: root.activeCategory === "apps"
                    
                    onClicked: {
                        root.activeCategory = "apps"
                        if (!root.expanded) {
                            root.expanded = true
                        }
                    }
                }
                
                // Widgets
                CategoryButton {
                    icon.source: "qrc:/icons/widgets.svg"
                    active: root.activeCategory === "widgets"
                    
                    onClicked: {
                        root.activeCategory = "widgets"
                        if (!root.expanded) {
                            root.expanded = true
                        }
                    }
                }
                
                // Dateien
                CategoryButton {
                    icon.source: "qrc:/icons/folder.svg"
                    active: root.activeCategory === "files"
                    
                    onClicked: {
                        root.activeCategory = "files"
                        if (!root.expanded) {
                            root.expanded = true
                        }
                    }
                }
                
                // Einstellungen
                CategoryButton {
                    icon.source: "qrc:/icons/settings.svg"
                    active: root.activeCategory === "settings"
                    
                    onClicked: {
                        root.activeCategory = "settings"
                        if (!root.expanded) {
                            root.expanded = true
                        }
                    }
                }
                
                // Spacer
                Item {
                    Layout.fillHeight: true
                }
                
                // Benutzer
                CategoryButton {
                    icon.source: "qrc:/icons/user.svg"
                    active: root.activeCategory === "user"
                    
                    onClicked: {
                        root.activeCategory = "user"
                        if (!root.expanded) {
                            root.expanded = true
                        }
                    }
                }
                
                // Power
                CategoryButton {
                    icon.source: "qrc:/icons/power.svg"
                    active: root.activeCategory === "power"
                    
                    onClicked: {
                        root.activeCategory = "power"
                        if (!root.expanded) {
                            root.expanded = true
                        }
                    }
                }
            }
        }
        
        // Inhaltsbereich (nur sichtbar, wenn erweitert)
        Rectangle {
            id: contentArea
            Layout.fillHeight: true
            Layout.preferredWidth: root.expandedWidth - root.collapsedWidth
            visible: root.expanded
            color: "transparent"
            
            // Suchleiste
            Rectangle {
                id: searchBar
                anchors {
                    top: parent.top
                    left: parent.left
                    right: parent.right
                    margins: 8
                }
                height: 40
                color: "#3B4252"
                radius: 4
                
                RowLayout {
                    anchors {
                        fill: parent
                        margins: 8
                    }
                    spacing: 8
                    
                    Image {
                        source: "qrc:/icons/search.svg"
                        Layout.preferredWidth: 16
                        Layout.preferredHeight: 16
                        
                        ColorOverlay {
                            anchors.fill: parent
                            source: parent
                            color: "#D8DEE9"
                        }
                    }
                    
                    TextField {
                        id: searchField
                        Layout.fillWidth: true
                        placeholderText: qsTr("Suchen...")
                        color: "#ECEFF4"
                        font.pixelSize: 14
                        
                        background: Rectangle {
                            color: "transparent"
                        }
                        
                        onTextChanged: {
                            // Suchfunktion implementieren
                            if (text.length > 0) {
                                searchResults.visible = true
                                categoryContent.visible = false
                            } else {
                                searchResults.visible = false
                                categoryContent.visible = true
                            }
                        }
                    }
                    
                    Button {
                        visible: searchField.text.length > 0
                        icon.source: "qrc:/icons/close.svg"
                        icon.color: "#D8DEE9"
                        background: Rectangle {
                            color: "transparent"
                        }
                        
                        onClicked: {
                            searchField.text = ""
                        }
                    }
                }
            }
            
            // Kategorietitel
            Label {
                id: categoryTitle
                anchors {
                    top: searchBar.bottom
                    left: parent.left
                    right: parent.right
                    margins: 16
                }
                text: {
                    switch (root.activeCategory) {
                    case "favorites":
                        return qsTr("Favoriten")
                    case "apps":
                        return qsTr("Alle Apps")
                    case "widgets":
                        return qsTr("Widgets")
                    case "files":
                        return qsTr("Dateien")
                    case "settings":
                        return qsTr("Einstellungen")
                    case "user":
                        return qsTr("Benutzer")
                    case "power":
                        return qsTr("System")
                    default:
                        return ""
                    }
                }
                font.pixelSize: 16
                font.bold: true
                color: "#ECEFF4"
            }
            
            // Kategorie-Inhalt
            Item {
                id: categoryContent
                anchors {
                    top: categoryTitle.bottom
                    left: parent.left
                    right: parent.right
                    bottom: parent.bottom
                    margins: 8
                }
                
                // Favoriten
                GridView {
                    id: favoritesGrid
                    anchors.fill: parent
                    visible: root.activeCategory === "favorites"
                    cellWidth: 80
                    cellHeight: 80
                    model: ApplicationManager.favoriteApps
                    
                    delegate: AppItem {
                        appName: modelData.name
                        appIcon: modelData.icon
                        
                        onClicked: {
                            root.appLaunched(modelData.id)
                        }
                    }
                }
                
                // Alle Apps
                ListView {
                    id: appsListView
                    anchors.fill: parent
                    visible: root.activeCategory === "apps"
                    model: ApplicationManager.allApps
                    
                    section.property: "category"
                    section.criteria: ViewSection.FullString
                    section.delegate: Rectangle {
                        width: appsListView.width
                        height: 30
                        color: "#3B4252"
                        
                        Text {
                            anchors {
                                left: parent.left
                                leftMargin: 8
                                verticalCenter: parent.verticalCenter
                            }
                            text: section
                            font.bold: true
                            color: "#ECEFF4"
                        }
                    }
                    
                    delegate: AppListItem {
                        width: appsListView.width
                        appName: model.name
                        appIcon: model.icon
                        appDescription: model.description
                        
                        onClicked: {
                            root.appLaunched(model.id)
                        }
                    }
                }
                
                // Widgets
                GridView {
                    id: widgetsGrid
                    anchors.fill: parent
                    visible: root.activeCategory === "widgets"
                    cellWidth: 120
                    cellHeight: 120
                    model: WidgetManager.availableWidgets
                    
                    delegate: WidgetItem {
                        widgetName: modelData.name
                        widgetIcon: modelData.icon
                        widgetType: modelData.type
                        
                        onClicked: {
                            widgetDialog.widgetType = widgetType
                            widgetDialog.widgetName = widgetName
                            widgetDialog.open()
                        }
                    }
                }
                
                // Dateien
                ListView {
                    id: filesListView
                    anchors.fill: parent
                    visible: root.activeCategory === "files"
                    model: FileManager.recentFiles
                    
                    section.property: "type"
                    section.criteria: ViewSection.FullString
                    section.delegate: Rectangle {
                        width: filesListView.width
                        height: 30
                        color: "#3B4252"
                        
                        Text {
                            anchors {
                                left: parent.left
                                leftMargin: 8
                                verticalCenter: parent.verticalCenter
                            }
                            text: section
                            font.bold: true
                            color: "#ECEFF4"
                        }
                    }
                    
                    delegate: FileItem {
                        width: filesListView.width
                        fileName: model.name
                        filePath: model.path
                        fileIcon: model.icon
                        fileSize: model.size
                        fileDate: model.date
                        
                        onClicked: {
                            FileManager.openFile(model.path)
                        }
                    }
                }
                
                // Einstellungen
                ListView {
                    id: settingsListView
                    anchors.fill: parent
                    visible: root.activeCategory === "settings"
                    model: [
                        { name: qsTr("Erscheinungsbild"), icon: "qrc:/icons/theme.svg", id: "appearance-settings" },
                        { name: qsTr("Anzeige"), icon: "qrc:/icons/display.svg", id: "display-settings" },
                        { name: qsTr("Sound"), icon: "qrc:/icons/sound.svg", id: "sound-settings" },
                        { name: qsTr("Netzwerk"), icon: "qrc:/icons/network.svg", id: "network-settings" },
                        { name: qsTr("Bluetooth"), icon: "qrc:/icons/bluetooth.svg", id: "bluetooth-settings" },
                        { name: qsTr("Benachrichtigungen"), icon: "qrc:/icons/notifications.svg", id: "notification-settings" },
                        { name: qsTr("Energieverwaltung"), icon: "qrc:/icons/power.svg", id: "power-settings" },
                        { name: qsTr("Benutzer"), icon: "qrc:/icons/user.svg", id: "user-settings" },
                        { name: qsTr("Sicherheit"), icon: "qrc:/icons/security.svg", id: "secur
(Content truncated due to size limit. Use line ranges to read in chunks)