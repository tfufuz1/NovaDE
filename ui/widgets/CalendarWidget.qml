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
    property string widgetType: "calendar"
    property var widgetSettings: ({
        showWeekNumbers: true,
        firstDayOfWeek: 1, // Montag
        showEvents: true
    })
    
    // Größe
    property int minimumWidth: 250
    property int minimumHeight: 250
    
    // Interne Eigenschaften
    property var calendarEvents: []
    property date currentDate: new Date()
    
    // Inhalt
    ColumnLayout {
        anchors.fill: parent
        spacing: 10
        
        // Kopfzeile mit Monat/Jahr und Navigation
        RowLayout {
            Layout.fillWidth: true
            spacing: 10
            
            VxButton {
                buttonType: "icon"
                icon.source: "qrc:/icons/chevron-left.svg"
                size: 32
                onClicked: {
                    // Vorheriger Monat
                    var newDate = new Date(currentDate);
                    newDate.setMonth(newDate.getMonth() - 1);
                    currentDate = newDate;
                }
            }
            
            Text {
                text: Qt.formatDate(currentDate, "MMMM yyyy")
                font.pixelSize: VxTheme.fontSizeLarge
                font.family: VxTheme.fontFamily
                font.weight: Font.Medium
                color: VxTheme.colorTextPrimary
                horizontalAlignment: Text.AlignHCenter
                Layout.fillWidth: true
            }
            
            VxButton {
                buttonType: "icon"
                icon.source: "qrc:/icons/chevron-right.svg"
                size: 32
                onClicked: {
                    // Nächster Monat
                    var newDate = new Date(currentDate);
                    newDate.setMonth(newDate.getMonth() + 1);
                    currentDate = newDate;
                }
            }
        }
        
        // Kalender
        GridLayout {
            id: calendarGrid
            columns: 7 + (widgetSettings.showWeekNumbers ? 1 : 0)
            Layout.fillWidth: true
            Layout.fillHeight: true
            
            // Wochentage
            Repeater {
                model: widgetSettings.showWeekNumbers ? 1 : 0
                
                Text {
                    text: ""
                    font.pixelSize: VxTheme.fontSizeSmall
                    font.family: VxTheme.fontFamily
                    font.weight: Font.Bold
                    color: VxTheme.colorTextSecondary
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                }
            }
            
            Repeater {
                model: 7
                
                Text {
                    text: getDayName(index)
                    font.pixelSize: VxTheme.fontSizeSmall
                    font.family: VxTheme.fontFamily
                    font.weight: Font.Bold
                    color: VxTheme.colorTextSecondary
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                }
            }
            
            // Kalendertage
            Repeater {
                model: getDaysModel()
                
                Rectangle {
                    color: model.isCurrentMonth ? 
                           (model.isToday ? VxTheme.colorBackgroundHighlighted : "transparent") : 
                           VxTheme.colorBackgroundDisabled
                    radius: VxTheme.cornerRadiusSmall
                    border.width: model.hasEvents ? 1 : 0
                    border.color: VxTheme.colorBorderPrimary
                    
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    
                    ColumnLayout {
                        anchors.fill: parent
                        anchors.margins: 2
                        spacing: 2
                        
                        Text {
                            text: model.text
                            font.pixelSize: VxTheme.fontSizeSmall
                            font.family: VxTheme.fontFamily
                            font.weight: model.isToday ? Font.Bold : Font.Normal
                            color: model.isCurrentMonth ? 
                                   (model.isToday ? VxTheme.colorTextOnHighlighted : VxTheme.colorTextPrimary) : 
                                   VxTheme.colorTextDisabled
                            horizontalAlignment: Text.AlignHCenter
                            verticalAlignment: Text.AlignVCenter
                            Layout.fillWidth: true
                        }
                        
                        // Ereignisanzeige (optional)
                        Rectangle {
                            visible: model.hasEvents && widgetSettings.showEvents
                            color: VxTheme.colorBackgroundAccent
                            radius: height / 2
                            Layout.preferredHeight: 4
                            Layout.preferredWidth: 4
                            Layout.alignment: Qt.AlignHCenter
                        }
                    }
                    
                    MouseArea {
                        anchors.fill: parent
                        onClicked: {
                            if (model.isCurrentMonth) {
                                selectDate(model.date);
                            }
                        }
                    }
                }
            }
        }
        
        // Aktuelle Ereignisse (optional)
        ListView {
            id: eventsList
            visible: widgetSettings.showEvents && getEventsForSelectedDate().length > 0
            Layout.fillWidth: true
            Layout.preferredHeight: visible ? Math.min(contentHeight, 100) : 0
            clip: true
            model: getEventsForSelectedDate()
            
            delegate: Rectangle {
                width: eventsList.width
                height: 30
                color: VxTheme.colorBackgroundSecondary
                radius: VxTheme.cornerRadiusSmall
                
                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 5
                    spacing: 5
                    
                    Rectangle {
                        width: 10
                        height: 10
                        radius: 5
                        color: modelData.color || VxTheme.colorBackgroundAccent
                        Layout.alignment: Qt.AlignVCenter
                    }
                    
                    Text {
                        text: modelData.title
                        font.pixelSize: VxTheme.fontSizeSmall
                        font.family: VxTheme.fontFamily
                        color: VxTheme.colorTextPrimary
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter
                    }
                    
                    Text {
                        text: modelData.time || ""
                        font.pixelSize: VxTheme.fontSizeSmall
                        font.family: VxTheme.fontFamily
                        color: VxTheme.colorTextSecondary
                        Layout.alignment: Qt.AlignVCenter
                    }
                }
            }
        }
    }
    
    // Methoden
    function getDayName(index) {
        var days = ["So", "Mo", "Di", "Mi", "Do", "Fr", "Sa"];
        var adjustedIndex = (index + widgetSettings.firstDayOfWeek) % 7;
        return days[adjustedIndex];
    }
    
    function getDaysModel() {
        var result = [];
        var today = new Date();
        var year = currentDate.getFullYear();
        var month = currentDate.getMonth();
        
        // Erster Tag des Monats
        var firstDay = new Date(year, month, 1);
        
        // Letzter Tag des Monats
        var lastDay = new Date(year, month + 1, 0);
        
        // Erster Tag in der Kalenderansicht (kann im vorherigen Monat liegen)
        var firstDayInView = new Date(firstDay);
        var dayOfWeek = firstDay.getDay();
        var diff = dayOfWeek - widgetSettings.firstDayOfWeek;
        if (diff < 0) diff += 7;
        firstDayInView.setDate(firstDayInView.getDate() - diff);
        
        // Füge Wochennummern und Tage hinzu
        var currentDay = new Date(firstDayInView);
        
        // 6 Wochen anzeigen (42 Tage)
        for (var i = 0; i < 42; i++) {
            // Wochennummer am Anfang jeder Woche
            if (i % 7 === 0 && widgetSettings.showWeekNumbers) {
                var weekNumber = getWeekNumber(currentDay);
                result.push({
                    text: weekNumber,
                    isHeader: true,
                    isWeekNumber: true
                });
            }
            
            // Tag
            var isCurrentMonth = currentDay.getMonth() === month;
            var isToday = currentDay.getFullYear() === today.getFullYear() &&
                          currentDay.getMonth() === today.getMonth() &&
                          currentDay.getDate() === today.getDate();
            
            result.push({
                text: currentDay.getDate(),
                date: new Date(currentDay),
                isCurrentMonth: isCurrentMonth,
                isToday: isToday,
                hasEvents: hasEventsOnDate(currentDay)
            });
            
            // Nächster Tag
            currentDay.setDate(currentDay.getDate() + 1);
        }
        
        return result;
    }
    
    function getWeekNumber(date) {
        var d = new Date(Date.UTC(date.getFullYear(), date.getMonth(), date.getDate()));
        var dayNum = d.getUTCDay() || 7;
        d.setUTCDate(d.getUTCDate() + 4 - dayNum);
        var yearStart = new Date(Date.UTC(d.getUTCFullYear(), 0, 1));
        return Math.ceil((((d - yearStart) / 86400000) + 1) / 7);
    }
    
    function hasEventsOnDate(date) {
        if (!widgetSettings.showEvents) return false;
        
        for (var i = 0; i < calendarEvents.length; i++) {
            var eventDate = new Date(calendarEvents[i].date);
            if (eventDate.getFullYear() === date.getFullYear() &&
                eventDate.getMonth() === date.getMonth() &&
                eventDate.getDate() === date.getDate()) {
                return true;
            }
        }
        
        return false;
    }
    
    function getEventsForSelectedDate() {
        var result = [];
        
        for (var i = 0; i < calendarEvents.length; i++) {
            var eventDate = new Date(calendarEvents[i].date);
            if (eventDate.getFullYear() === currentDate.getFullYear() &&
                eventDate.getMonth() === currentDate.getMonth() &&
                eventDate.getDate() === currentDate.getDate()) {
                result.push(calendarEvents[i]);
            }
        }
        
        return result;
    }
    
    function selectDate(date) {
        currentDate = date;
    }
    
    function addEvent(title, date, time, color) {
        calendarEvents.push({
            title: title,
            date: date,
            time: time,
            color: color
        });
        
        saveEvents();
    }
    
    function removeEvent(index) {
        if (index >= 0 && index < calendarEvents.length) {
            calendarEvents.splice(index, 1);
            saveEvents();
        }
    }
    
    function saveEvents() {
        // In einer vollständigen Implementierung würden wir hier die Ereignisse
        // in den Einstellungen speichern
        
        // Beispiel für das Speichern der Ereignisse
        console.log("Saving calendar events:", JSON.stringify(calendarEvents));
    }
    
    function loadEvents() {
        // In einer vollständigen Implementierung würden wir hier die Ereignisse
        // aus den Einstellungen laden
        
        // Beispiel für das Laden der Ereignisse
        calendarEvents = [
            { title: "Meeting", date: new Date(), time: "10:00", color: "#FF5722" },
            { title: "Mittagessen", date: new Date(), time: "12:30", color: "#4CAF50" },
            { title: "Präsentation", date: new Date(new Date().setDate(new Date().getDate() + 2)), time: "14:00", color: "#2196F3" }
        ];
    }
    
    // Initialisierung
    Component.onCompleted: {
        loadEvents();
    }
}
