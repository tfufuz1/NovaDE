import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../ui/theme"
import "../ui/atoms"
import "../ui/molecules"

Item {
    id: root
    
    // Eigenschaften
    property string widgetId: ""
    property string widgetType: "desktop"
    property var widgetSettings: ({
        showIcons: true,
        iconSize: 64,
        iconSpacing: 10,
        showLabels: true,
        sortBy: "name" // name, type, date
    })
    
    // Größe
    property int minimumWidth: 200
    property int minimumHeight: 200
    
    // Interne Eigenschaften
    property var desktopItems: []
    
    // Modell für Desktop-Elemente
    ListModel {
        id: desktopModel
        
        // Beispiel-Einträge
        ListElement {
            name: "Dokumente"
            icon: "qrc:/icons/folder.svg"
            type: "folder"
            path: "/home/user/Documents"
        }
        
        ListElement {
            name: "Bilder"
            icon: "qrc:/icons/image.svg"
            type: "folder"
            path: "/home/user/Pictures"
        }
        
        ListElement {
            name: "Musik"
            icon: "qrc:/icons/music.svg"
            type: "folder"
            path: "/home/user/Music"
        }
        
        ListElement {
            name: "Terminal"
            icon: "qrc:/icons/terminal.svg"
            type: "application"
            command: "xterm"
        }
    }
    
    // Inhalt
    ScrollView {
        id: scrollView
        anchors.fill: parent
        clip: true
        
        GridView {
            id: gridView
            anchors.fill: parent
            cellWidth: widgetSettings.iconSize + widgetSettings.iconSpacing
            cellHeight: widgetSettings.iconSize + (widgetSettings.showLabels ? 20 : 0) + widgetSettings.iconSpacing
            model: desktopModel
            
            delegate: Item {
                width: gridView.cellWidth
                height: gridView.cellHeight
                
                Column {
                    anchors.centerIn: parent
                    spacing: 5
                    
                    VxIcon {
                        source: model.icon
                        size: widgetSettings.iconSize
                        anchors.horizontalCenter: parent.horizontalCenter
                    }
                    
                    Text {
                        text: model.name
                        font.pixelSize: VxTheme.fontSizeSmall
                        font.family: VxTheme.fontFamily
                        color: VxTheme.colorTextPrimary
                        horizontalAlignment: Text.AlignHCenter
                        visible: widgetSettings.showLabels
                        width: widgetSettings.iconSize
                        elide: Text.ElideRight
                        anchors.horizontalCenter: parent.horizontalCenter
                    }
                }
                
                MouseArea {
                    anchors.fill: parent
                    acceptedButtons: Qt.LeftButton | Qt.RightButton
                    
                    onClicked: {
                        if (mouse.button === Qt.LeftButton) {
                            openItem(model.type, model.path || model.command);
                        } else if (mouse.button === Qt.RightButton) {
                            showContextMenu(model.index, mouse.x, mouse.y);
                        }
                    }
                    
                    onDoubleClicked: {
                        if (mouse.button === Qt.LeftButton) {
                            openItem(model.type, model.path || model.command);
                        }
                    }
                }
            }
        }
    }
    
    // Kontextmenü
    VxPopupMenu {
        id: contextMenu
        property int currentIndex: -1
        
        VxMenuItem {
            text: qsTr("Öffnen")
            onTriggered: {
                var item = desktopModel.get(contextMenu.currentIndex);
                openItem(item.type, item.path || item.command);
            }
        }
        
        VxMenuItem {
            text: qsTr("Umbenennen")
            onTriggered: {
                var item = desktopModel.get(contextMenu.currentIndex);
                renameItem(contextMenu.currentIndex, item.name);
            }
        }
        
        VxMenuItem {
            text: qsTr("Löschen")
            onTriggered: {
                removeItem(contextMenu.currentIndex);
            }
        }
    }
    
    // Methoden
    function openItem(type, pathOrCommand) {
        console.log("Opening item of type:", type, "with path/command:", pathOrCommand);
        
        if (type === "folder") {
            // Öffne Ordner
            // In einer vollständigen Implementierung würden wir hier den Dateimanager öffnen
        } else if (type === "application") {
            // Starte Anwendung
            // In einer vollständigen Implementierung würden wir hier die Anwendung starten
        } else if (type === "file") {
            // Öffne Datei
            // In einer vollständigen Implementierung würden wir hier die Datei öffnen
        }
    }
    
    function showContextMenu(index, x, y) {
        contextMenu.currentIndex = index;
        contextMenu.popup(mapToGlobal(x, y));
    }
    
    function addItem(name, icon, type, pathOrCommand) {
        desktopModel.append({
            "name": name,
            "icon": icon,
            "type": type,
            "path": type === "application" ? "" : pathOrCommand,
            "command": type === "application" ? pathOrCommand : ""
        });
        
        saveDesktopItems();
    }
    
    function removeItem(index) {
        if (index >= 0 && index < desktopModel.count) {
            desktopModel.remove(index);
            saveDesktopItems();
        }
    }
    
    function renameItem(index, newName) {
        if (index >= 0 && index < desktopModel.count) {
            desktopModel.setProperty(index, "name", newName);
            saveDesktopItems();
        }
    }
    
    function sortItems() {
        // Sortiere Desktop-Elemente
        var items = [];
        
        // Sammle alle Elemente
        for (var i = 0; i < desktopModel.count; i++) {
            items.push(desktopModel.get(i));
        }
        
        // Sortiere Elemente
        if (widgetSettings.sortBy === "name") {
            items.sort(function(a, b) {
                return a.name.localeCompare(b.name);
            });
        } else if (widgetSettings.sortBy === "type") {
            items.sort(function(a, b) {
                return a.type.localeCompare(b.type) || a.name.localeCompare(b.name);
            });
        }
        
        // Aktualisiere Modell
        desktopModel.clear();
        for (var j = 0; j < items.length; j++) {
            desktopModel.append(items[j]);
        }
        
        saveDesktopItems();
    }
    
    function loadDesktopItems() {
        // In einer vollständigen Implementierung würden wir hier die Desktop-Elemente
        // aus den Einstellungen laden
        
        // Beispiel für das Laden der Desktop-Elemente
        desktopItems = [
            { name: "Dokumente", icon: "qrc:/icons/folder.svg", type: "folder", path: "/home/user/Documents" },
            { name: "Bilder", icon: "qrc:/icons/image.svg", type: "folder", path: "/home/user/Pictures" },
            { name: "Musik", icon: "qrc:/icons/music.svg", type: "folder", path: "/home/user/Music" },
            { name: "Terminal", icon: "qrc:/icons/terminal.svg", type: "application", command: "xterm" }
        ];
        
        // Aktualisiere Modell
        desktopModel.clear();
        for (var i = 0; i < desktopItems.length; i++) {
            desktopModel.append(desktopItems[i]);
        }
    }
    
    function saveDesktopItems() {
        // In einer vollständigen Implementierung würden wir hier die Desktop-Elemente
        // in den Einstellungen speichern
        
        // Sammle alle Elemente
        desktopItems = [];
        for (var i = 0; i < desktopModel.count; i++) {
            var item = desktopModel.get(i);
            desktopItems.push({
                name: item.name,
                icon: item.icon,
                type: item.type,
                path: item.path,
                command: item.command
            });
        }
        
        // Beispiel für das Speichern der Desktop-Elemente
        console.log("Saving desktop items:", JSON.stringify(desktopItems));
    }
    
    // Initialisierung
    Component.onCompleted: {
        loadDesktopItems();
    }
}
