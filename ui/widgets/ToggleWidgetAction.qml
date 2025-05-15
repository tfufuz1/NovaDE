import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import "../ui/theme"
import "../ui/atoms"
import "../ui/molecules"

Item {
    id: root
    
    // Eigenschaften
    property string actionId: ""
    property string actionType: "toggleWidget"
    property var actionData: ({
        widgetId: "",
        initialState: true,
        toggleProperty: "visible"
    })
    
    // Signale
    signal executed(var result)
    
    // Methoden
    function execute(parameters) {
        console.log("Executing ToggleWidgetAction:", actionId);
        console.log("Widget ID:", actionData.widgetId);
        
        // Kombiniere Widget-ID aus actionData und parameters
        var widgetId = parameters && parameters.widgetId ? parameters.widgetId : actionData.widgetId;
        var toggleProperty = parameters && parameters.toggleProperty ? parameters.toggleProperty : actionData.toggleProperty;
        var forcedState = parameters && parameters.state !== undefined ? parameters.state : undefined;
        
        // In einer vollständigen Implementierung würden wir hier das Widget umschalten
        // Beispiel:
        // var widget = WidgetManager.getWidget(widgetId);
        // if (widget) {
        //     if (forcedState !== undefined) {
        //         widget[toggleProperty] = forcedState;
        //     } else {
        //         widget[toggleProperty] = !widget[toggleProperty];
        //     }
        // }
        
        var newState = forcedState !== undefined ? forcedState : !actionData.initialState;
        
        // Aktualisiere den Zustand für zukünftige Umschaltungen
        actionData.initialState = newState;
        
        var result = {
            success: true,
            widgetId: widgetId,
            property: toggleProperty,
            newState: newState
        };
        
        // Signalisiere Ausführung
        executed(result);
        
        return result;
    }
    
    function canExecute(parameters) {
        // Prüfe, ob die Aktion ausgeführt werden kann
        var widgetId = parameters && parameters.widgetId ? parameters.widgetId : actionData.widgetId;
        return widgetId && widgetId.length > 0;
    }
}
