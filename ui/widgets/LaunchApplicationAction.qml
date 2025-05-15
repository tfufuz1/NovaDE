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
    property string actionType: "launchApplication"
    property var actionData: ({
        applicationName: "",
        command: "",
        icon: "",
        arguments: []
    })
    
    // Signale
    signal executed(var result)
    
    // Methoden
    function execute(parameters) {
        console.log("Executing LaunchApplicationAction:", actionId);
        console.log("Application:", actionData.applicationName);
        console.log("Command:", actionData.command);
        
        // Kombiniere Argumente aus actionData und parameters
        var args = actionData.arguments || [];
        if (parameters && parameters.arguments) {
            args = args.concat(parameters.arguments);
        }
        
        // In einer vollständigen Implementierung würden wir hier die Anwendung starten
        // Beispiel:
        // SystemService.launchApplication(actionData.command, args);
        
        var result = {
            success: true,
            applicationName: actionData.applicationName,
            command: actionData.command,
            arguments: args
        };
        
        // Signalisiere Ausführung
        executed(result);
        
        return result;
    }
    
    function canExecute(parameters) {
        // Prüfe, ob die Aktion ausgeführt werden kann
        return actionData.command && actionData.command.length > 0;
    }
}
