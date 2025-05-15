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
    property string actionType: "systemCommand"
    property var actionData: ({
        command: "",
        workingDirectory: "",
        runAsAdmin: false,
        environment: {}
    })
    
    // Signale
    signal executed(var result)
    
    // Methoden
    function execute(parameters) {
        console.log("Executing SystemCommandAction:", actionId);
        console.log("Command:", actionData.command);
        console.log("Working Directory:", actionData.workingDirectory);
        
        // Kombiniere Befehl aus actionData und parameters
        var command = parameters && parameters.command ? parameters.command : actionData.command;
        var workingDirectory = parameters && parameters.workingDirectory ? parameters.workingDirectory : actionData.workingDirectory;
        var runAsAdmin = parameters && parameters.runAsAdmin !== undefined ? parameters.runAsAdmin : actionData.runAsAdmin;
        
        // In einer vollständigen Implementierung würden wir hier den Systembefehl ausführen
        // Beispiel:
        // SystemService.executeCommand(command, workingDirectory, runAsAdmin);
        
        var result = {
            success: true,
            command: command,
            workingDirectory: workingDirectory,
            runAsAdmin: runAsAdmin
        };
        
        // Signalisiere Ausführung
        executed(result);
        
        return result;
    }
    
    function canExecute(parameters) {
        // Prüfe, ob die Aktion ausgeführt werden kann
        var command = parameters && parameters.command ? parameters.command : actionData.command;
        return command && command.length > 0;
    }
}
