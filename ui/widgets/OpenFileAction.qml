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
    property string actionType: "openFile"
    property var actionData: ({
        path: "",
        application: "",
        mimeType: ""
    })
    
    // Signale
    signal executed(var result)
    
    // Methoden
    function execute(parameters) {
        console.log("Executing OpenFileAction:", actionId);
        console.log("Path:", actionData.path);
        console.log("Application:", actionData.application);
        
        // Kombiniere Pfad aus actionData und parameters
        var path = parameters && parameters.path ? parameters.path : actionData.path;
        var application = parameters && parameters.application ? parameters.application : actionData.application;
        
        // In einer vollständigen Implementierung würden wir hier die Datei öffnen
        // Beispiel:
        // SystemService.openFile(path, application);
        
        var result = {
            success: true,
            path: path,
            application: application
        };
        
        // Signalisiere Ausführung
        executed(result);
        
        return result;
    }
    
    function canExecute(parameters) {
        // Prüfe, ob die Aktion ausgeführt werden kann
        var path = parameters && parameters.path ? parameters.path : actionData.path;
        return path && path.length > 0;
    }
}
