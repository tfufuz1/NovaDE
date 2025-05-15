/**
 * @file WidgetConfigDialog.qml
 * @brief Dialog zur Konfiguration von Widgets
 * 
 * Diese Datei enthält einen Dialog zur Konfiguration von Widgets,
 * der dynamisch Konfigurationsoptionen basierend auf dem Widget-Typ anzeigt.
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import QtQuick.Dialogs 1.3
import VivoX.UI 1.0

Dialog {
    id: root
    
    // Eigenschaften
    property string widgetId: ""
    property string widgetName: ""
    property var widgetConfig: ({})
    
    // Dialog-Eigenschaften
    title: qsTr("Widget konfigurieren: ") + widgetName
    standardButtons: Dialog.Ok | Dialog.Cancel
    modal: true
    
    // Größe
    width: 500
    height: 600
    
    // Inhalt
    ColumnLayout {
        anchors.fill: parent
        spacing: 16
        
        // Titel
        Label {
            Layout.fillWidth: true
            text: qsTr("Konfiguration für ") + widgetName
            font.pixelSize: 16
            font.bold: true
        }
        
        // Trennlinie
        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: "#3B4252"
        }
        
        // Konfigurationsoptionen
        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            
            ColumnLayout {
                id: configOptionsLayout
                width: parent.width
                spacing: 12
                
                // Hier werden die Konfigurationsoptionen dynamisch hinzugefügt
            }
        }
    }
    
    // Komponente wird initialisiert
    Component.onCompleted: {
        // Konfigurationsoptionen laden
        loadConfigOptions()
    }
    
    // Funktionen
    
    /**
     * @brief Lädt die Konfigurationsoptionen basierend auf dem Widget-Typ
     */
    function loadConfigOptions() {
        // Bestehende Optionen entfernen
        for (var i = configOptionsLayout.children.length - 1; i >= 0; i--) {
            configOptionsLayout.children[i].destroy()
        }
        
        // Allgemeine Optionen hinzufügen
        addTextField("name", qsTr("Name"), widgetName)
        
        // Widget-spezifische Optionen hinzufügen
        if (widgetId.startsWith("clock")) {
            addComboBox("format", qsTr("Zeitformat"), ["24h", "12h"], widgetConfig.format || "24h")
            addCheckBox("showSeconds", qsTr("Sekunden anzeigen"), widgetConfig.showSeconds || false)
            addCheckBox("showDate", qsTr("Datum anzeigen"), widgetConfig.showDate || false)
        } 
        else if (widgetId.startsWith("weather")) {
            addTextField("location", qsTr("Ort"), widgetConfig.location || "")
            addComboBox("units", qsTr("Einheiten"), ["metric", "imperial"], widgetConfig.units || "metric")
            addComboBox("updateInterval", qsTr("Aktualisierungsintervall"), ["30min", "1h", "3h", "6h"], widgetConfig.updateInterval || "1h")
        }
        else if (widgetId.startsWith("system")) {
            addCheckBox("showCpu", qsTr("CPU anzeigen"), widgetConfig.showCpu || true)
            addCheckBox("showMemory", qsTr("Speicher anzeigen"), widgetConfig.showMemory || true)
            addCheckBox("showDisk", qsTr("Festplatte anzeigen"), widgetConfig.showDisk || true)
            addComboBox("updateInterval", qsTr("Aktualisierungsintervall"), ["1s", "5s", "10s", "30s"], widgetConfig.updateInterval || "5s")
        }
        else if (widgetId.startsWith("notes")) {
            addTextArea("content", qsTr("Inhalt"), widgetConfig.content || "")
            addColorPicker("backgroundColor", qsTr("Hintergrundfarbe"), widgetConfig.backgroundColor || "#FFEB3B")
        }
        else if (widgetId.startsWith("calendar")) {
            addCheckBox("showEvents", qsTr("Ereignisse anzeigen"), widgetConfig.showEvents || true)
            addCheckBox("showWeekNumbers", qsTr("Kalenderwoche anzeigen"), widgetConfig.showWeekNumbers || false)
            addComboBox("firstDayOfWeek", qsTr("Erster Tag der Woche"), ["Montag", "Sonntag"], widgetConfig.firstDayOfWeek || "Montag")
        }
        else if (widgetId.startsWith("media")) {
            addCheckBox("showControls", qsTr("Steuerelemente anzeigen"), widgetConfig.showControls || true)
            addCheckBox("showProgress", qsTr("Fortschritt anzeigen"), widgetConfig.showProgress || true)
            addCheckBox("showArtwork", qsTr("Albumcover anzeigen"), widgetConfig.showArtwork || true)
        }
        else {
            // Generische Optionen für unbekannte Widget-Typen
            addLabel(qsTr("Keine spezifischen Konfigurationsoptionen verfügbar"))
        }
    }
    
    /**
     * @brief Fügt ein Textfeld zur Konfiguration hinzu
     * @param key Schlüssel der Konfigurationsoption
     * @param label Beschriftung der Konfigurationsoption
     * @param value Wert der Konfigurationsoption
     */
    function addTextField(key, label, value) {
        var component = Qt.createComponent("../controls/ConfigTextField.qml")
        if (component.status === Component.Ready) {
            var textField = component.createObject(configOptionsLayout, {
                "configKey": key,
                "label": label,
                "text": value
            })
            
            textField.valueChanged.connect(function(key, value) {
                widgetConfig[key] = value
            })
        }
    }
    
    /**
     * @brief Fügt ein Textfeld mit mehreren Zeilen zur Konfiguration hinzu
     * @param key Schlüssel der Konfigurationsoption
     * @param label Beschriftung der Konfigurationsoption
     * @param value Wert der Konfigurationsoption
     */
    function addTextArea(key, label, value) {
        var component = Qt.createComponent("../controls/ConfigTextArea.qml")
        if (component.status === Component.Ready) {
            var textArea = component.createObject(configOptionsLayout, {
                "configKey": key,
                "label": label,
                "text": value
            })
            
            textArea.valueChanged.connect(function(key, value) {
                widgetConfig[key] = value
            })
        }
    }
    
    /**
     * @brief Fügt eine Checkbox zur Konfiguration hinzu
     * @param key Schlüssel der Konfigurationsoption
     * @param label Beschriftung der Konfigurationsoption
     * @param checked Status der Checkbox
     */
    function addCheckBox(key, label, checked) {
        var component = Qt.createComponent("../controls/ConfigCheckBox.qml")
        if (component.status === Component.Ready) {
            var checkBox = component.createObject(configOptionsLayout, {
                "configKey": key,
                "label": label,
                "checked": checked
            })
            
            checkBox.valueChanged.connect(function(key, value) {
                widgetConfig[key] = value
            })
        }
    }
    
    /**
     * @brief Fügt eine Combobox zur Konfiguration hinzu
     * @param key Schlüssel der Konfigurationsoption
     * @param label Beschriftung der Konfigurationsoption
     * @param options Optionen der Combobox
     * @param currentValue Aktueller Wert der Combobox
     */
    function addComboBox(key, label, options, currentValue) {
        var component = Qt.createComponent("../controls/ConfigComboBox.qml")
        if (component.status === Component.Ready) {
            var comboBox = component.createObject(configOptionsLayout, {
                "configKey": key,
                "label": label,
                "model": options,
                "currentValue": currentValue
            })
            
            comboBox.valueChanged.connect(function(key, value) {
                widgetConfig[key] = value
            })
        }
    }
    
    /**
     * @brief Fügt einen Farbwähler zur Konfiguration hinzu
     * @param key Schlüssel der Konfigurationsoption
     * @param label Beschriftung der Konfigurationsoption
     * @param color Farbe des Farbwählers
     */
    function addColorPicker(key, label, color) {
        var component = Qt.createComponent("../controls/ConfigColorPicker.qml")
        if (component.status === Component.Ready) {
            var colorPicker = component.createObject(configOptionsLayout, {
                "configKey": key,
                "label": label,
                "color": color
            })
            
            colorPicker.valueChanged.connect(function(key, value) {
                widgetConfig[key] = value
            })
        }
    }
    
    /**
     * @brief Fügt eine Beschriftung zur Konfiguration hinzu
     * @param text Text der Beschriftung
     */
    function addLabel(text) {
        var component = Qt.createComponent("../controls/ConfigLabel.qml")
        if (component.status === Component.Ready) {
            component.createObject(configOptionsLayout, {
                "text": text
            })
        }
    }
}
