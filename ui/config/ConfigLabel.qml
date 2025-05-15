/**
 * @file ConfigLabel.qml
 * @brief Beschriftung für Konfigurationsdialoge
 * 
 * Diese Datei enthält ein Beschriftungs-Steuerelement für Konfigurationsdialoge.
 */

import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Label {
    id: root
    Layout.fillWidth: true
    font.pixelSize: 12
    color: "#ECEFF4"
    wrapMode: Text.WordWrap
    
    // Standardgestaltung
    padding: 8
    background: Rectangle {
        color: "#3B4252"
        radius: 4
        opacity: 0.5
    }
}
