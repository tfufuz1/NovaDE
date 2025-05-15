import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../ui/theme"
import "../ui/atoms"
import "../ui/molecules"

Item {
    id: root
    
    // Implementierung der PanelInterface-Eigenschaften
    property string panelId: ""
    property string panelName: "Default Panel"
    property int position: 0 // PanelPosition.Top
    property bool visible: true
    property bool locked: false
    property bool autoHide: false
    
    // Interne Eigenschaften
    property int panelSize: 48
    property int panelSpacing: 4
    property bool isHorizontal: position === 0 || position === 1 // Top oder Bottom
    property bool isVertical: position === 2 || position === 3 // Left oder Right
    
    // Geometrie
    property rect geometry: Qt.rect(x, y, width, height)
    
    // Widget-Verwaltung
    property var widgetList: []
    property var widgetComponents: ({})
    
    // Signale
    signal panelNameChanged(string name)
    signal positionChanged(int position)
    signal geometryChanged(rect geometry)
    signal visibleChanged(bool visible)
    signal lockedChanged(bool locked)
    signal autoHideChanged(bool autoHide)
    signal widgetAdded(string widgetId, int position)
    signal widgetRemoved(string widgetId)
    signal widgetMoved(string widgetId, int newPosition)
    
    // Größe und Position basierend auf Panel-Position
    width: isHorizontal ? parent.width : panelSize
    height: isVertical ? parent.height : panelSize
    
    x: position === 3 ? parent.width - width : 0 // Rechts
    y: position === 1 ? parent.height - height : 0 // Unten
    
    // Hintergrund
    Rectangle {
        anchors.fill: parent
        color: VxTheme.colorBackgroundPrimary
        opacity: 0.9
        
        // Schatten
        layer.enabled: true
        layer.effect: DropShadow {
            horizontalOffset: 0
            verticalOffset: position === 0 ? 2 : (position === 1 ? -2 : 0) // Top: nach unten, Bottom: nach oben
            radius: 8
            samples: 17
            color: "#30000000"
        }
    }
    
    // Container für Widgets
    RowLayout {
        id: horizontalLayout
        anchors.fill: parent
        anchors.margins: panelSpacing
        spacing: panelSpacing
        visible: isHorizontal
    }
    
    ColumnLayout {
        id: verticalLayout
        anchors.fill: parent
        anchors.margins: panelSpacing
        spacing: panelSpacing
        visible: isVertical
    }
    
    // Methoden
    function setPanelName(name) {
        if (panelName !== name) {
            panelName = name;
            panelNameChanged(name);
        }
    }
    
    function setPosition(pos) {
        if (position !== pos) {
            position = pos;
            positionChanged(pos);
            
            // Aktualisiere Ausrichtung
            isHorizontal = position === 0 || position === 1;
            isVertical = position === 2 || position === 3;
            
            // Aktualisiere Geometrie
            updateGeometry();
        }
    }
    
    function setGeometry(geo) {
        x = geo.x;
        y = geo.y;
        width = geo.width;
        height = geo.height;
        geometry = geo;
        geometryChanged(geo);
    }
    
    function setVisible(vis) {
        if (visible !== vis) {
            visible = vis;
            visibleChanged(vis);
        }
    }
    
    function setLocked(lock) {
        if (locked !== lock) {
            locked = lock;
            lockedChanged(lock);
        }
    }
    
    function setAutoHide(auto) {
        if (autoHide !== auto) {
            autoHide = auto;
            autoHideChanged(auto);
        }
    }
    
    function addWidget(widgetId, pos) {
        // Prüfe, ob das Widget bereits existiert
        if (widgetList.indexOf(widgetId) !== -1) {
            console.warn("Widget already exists in panel:", widgetId);
            return false;
        }
        
        // Prüfe, ob die Komponente für dieses Widget existiert
        if (!widgetComponents[widgetId]) {
            console.warn("Widget component not found:", widgetId);
            return false;
        }
        
        // Füge das Widget an der angegebenen Position ein
        if (pos >= 0 && pos < widgetList.length) {
            widgetList.splice(pos, 0, widgetId);
        } else {
            widgetList.push(widgetId);
            pos = widgetList.length - 1;
        }
        
        // Erstelle die Widget-Instanz
        var container = isHorizontal ? horizontalLayout : verticalLayout;
        var widget = widgetComponents[widgetId].createObject(container, {
            "panelPosition": position
        });
        
        // Speichere die Widget-Instanz
        widgetInstances[widgetId] = widget;
        
        // Aktualisiere die Reihenfolge der Widgets
        updateWidgetOrder();
        
        widgetAdded(widgetId, pos);
        return true;
    }
    
    function removeWidget(widgetId) {
        var index = widgetList.indexOf(widgetId);
        if (index === -1) {
            console.warn("Widget not found in panel:", widgetId);
            return false;
        }
        
        // Entferne das Widget aus der Liste
        widgetList.splice(index, 1);
        
        // Zerstöre die Widget-Instanz
        if (widgetInstances[widgetId]) {
            widgetInstances[widgetId].destroy();
            delete widgetInstances[widgetId];
        }
        
        widgetRemoved(widgetId);
        return true;
    }
    
    function widgets() {
        return widgetList;
    }
    
    function moveWidget(widgetId, newPos) {
        var index = widgetList.indexOf(widgetId);
        if (index === -1) {
            console.warn("Widget not found in panel:", widgetId);
            return false;
        }
        
        if (newPos < 0 || newPos >= widgetList.length) {
            console.warn("Invalid position:", newPos);
            return false;
        }
        
        // Entferne das Widget aus der aktuellen Position
        widgetList.splice(index, 1);
        
        // Füge es an der neuen Position ein
        widgetList.splice(newPos, 0, widgetId);
        
        // Aktualisiere die Reihenfolge der Widgets
        updateWidgetOrder();
        
        widgetMoved(widgetId, newPos);
        return true;
    }
    
    function show() {
        setVisible(true);
    }
    
    function hide() {
        setVisible(false);
    }
    
    function update() {
        updateGeometry();
        updateWidgetOrder();
    }
    
    // Interne Methoden
    function updateGeometry() {
        // Aktualisiere Größe und Position basierend auf Panel-Position
        width = isHorizontal ? parent.width : panelSize;
        height = isVertical ? parent.height : panelSize;
        
        x = position === 3 ? parent.width - width : 0; // Rechts
        y = position === 1 ? parent.height - height : 0; // Unten
        
        geometry = Qt.rect(x, y, width, height);
        geometryChanged(geometry);
    }
    
    function updateWidgetOrder() {
        // Aktualisiere die Reihenfolge der Widgets im Layout
        var container = isHorizontal ? horizontalLayout : verticalLayout;
        
        for (var i = 0; i < widgetList.length; i++) {
            var widgetId = widgetList[i];
            if (widgetInstances[widgetId]) {
                widgetInstances[widgetId].parent = null;
                widgetInstances[widgetId].parent = container;
            }
        }
    }
    
    // Initialisierung
    Component.onCompleted: {
        // Initialisiere Widget-Instanzen
        widgetInstances = {};
        
        // Registriere Standard-Widget-Komponenten
        widgetComponents["launcher"] = Qt.createComponent("../widgets/LauncherWidget.qml");
        widgetComponents["clock"] = Qt.createComponent("../widgets/ClockWidget.qml");
        widgetComponents["systray"] = Qt.createComponent("../widgets/SystemTrayWidget.qml");
        widgetComponents["taskbar"] = Qt.createComponent("../widgets/TaskbarWidget.qml");
        
        // Aktualisiere Geometrie
        updateGeometry();
    }
    
    // Auto-Hide-Verhalten
    MouseArea {
        id: autoHideArea
        anchors.fill: parent
        hoverEnabled: true
        visible: autoHide
        
        property bool isHidden: false
        property int hideOffset: isHorizontal ? 
                                (position === 0 ? -height + 5 : height - 5) : 
                                (position === 2 ? -width + 5 : width - 5)
        
        onEntered: {
            if (isHidden) {
                showAnimation.start();
                isHidden = false;
            }
        }
        
        onExited: {
            if (!isHidden && !containsMouse && autoHide) {
                hideAnimation.start();
                isHidden = true;
            }
        }
        
        NumberAnimation {
            id: hideAnimation
            target: root
            property: isHorizontal ? "y" : "x"
            to: isHorizontal ? 
                (position === 0 ? -root.height + 5 : parent.height - 5) : 
                (position === 2 ? -root.width + 5 : parent.width - 5)
            duration: 300
            easing.type: Easing.OutQuad
        }
        
        NumberAnimation {
            id: showAnimation
            target: root
            property: isHorizontal ? "y" : "x"
            to: isHorizontal ? 
                (position === 0 ? 0 : parent.height - root.height) : 
                (position === 2 ? 0 : parent.width - root.width)
            duration: 300
            easing.type: Easing.OutQuad
        }
    }
    
    // Drag-Bereich für Verschieben des Panels (wenn nicht gesperrt)
    MouseArea {
        id: dragArea
        anchors.fill: parent
        visible: !locked
        drag.target: !locked ? root : undefined
        drag.axis: Drag.XAndYAxis
        drag.minimumX: 0
        drag.maximumX: parent.width - width
        drag.minimumY: 0
        drag.maximumY: parent.height - height
        
        onReleased: {
            // Bestimme die neue Position basierend auf der aktuellen Position
            var centerX = root.x + root.width / 2;
            var centerY = root.y + root.height / 2;
            var parentWidth = parent.width;
            var parentHeight = parent.height;
            
            var newPosition;
            
            if (centerY < parentHeight * 0.25) {
                newPosition = 0; // Top
            } else if (centerY > parentHeight * 0.75) {
                newPosition = 1; // Bottom
            } else if (centerX < parentWidth * 0.25) {
                newPosition = 2; // Left
            } else if (centerX > parentWidth * 0.75) {
                newPosition = 3; // Right
            } else {
                newPosition = 4; // Floating
            }
            
            if (newPosition !== position) {
                setPosition(newPosition);
            }
            
            // Aktualisiere die Geometrie
            updateGeometry();
        }
    }
}
