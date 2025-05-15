#pragma once

#include <QObject>
#include <QString>
#include <QVariant>
#include <QMap>
#include <QVector>
#include "PanelInterface.h"

namespace VivoX {
namespace Panel {

/**
 * @brief Die PanelManager-Klasse verwaltet alle Panels im VivoX Desktop Environment.
 * 
 * Diese Klasse ist verantwortlich für die Erstellung, Verwaltung und Konfiguration
 * von Panels. Sie folgt dem Singleton-Muster.
 */
class PanelManager : public QObject
{
    Q_OBJECT

public:
    /**
     * @brief Gibt die Singleton-Instanz des PanelManagers zurück.
     * @return Referenz auf die PanelManager-Instanz
     */
    static PanelManager& instance();

    /**
     * @brief Initialisiert den PanelManager.
     */
    void initialize();

    /**
     * @brief Erstellt ein neues Panel.
     * @param panelType Typ des zu erstellenden Panels
     * @param position Position des Panels
     * @param name Name des Panels (optional)
     * @return ID des erstellten Panels oder leerer String bei Fehler
     */
    QString createPanel(const QString& panelType, PanelPosition position, const QString& name = QString());

    /**
     * @brief Entfernt ein Panel.
     * @param panelId ID des zu entfernenden Panels
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool removePanel(const QString& panelId);

    /**
     * @brief Gibt ein Panel anhand seiner ID zurück.
     * @param panelId ID des abzufragenden Panels
     * @return Zeiger auf das Panel oder nullptr, wenn nicht gefunden
     */
    PanelInterface* getPanel(const QString& panelId) const;

    /**
     * @brief Gibt alle Panels zurück.
     * @return Map mit Panel-IDs und Zeigern auf die Panels
     */
    QMap<QString, PanelInterface*> getPanels() const;

    /**
     * @brief Gibt die IDs aller Panels zurück.
     * @return Liste der Panel-IDs
     */
    QStringList getPanelIds() const;

    /**
     * @brief Gibt die Panels an einer bestimmten Position zurück.
     * @param position Position der abzufragenden Panels
     * @return Liste der Panel-IDs an der angegebenen Position
     */
    QStringList getPanelsByPosition(PanelPosition position) const;

    /**
     * @brief Registriert einen Panel-Typ.
     * @param panelType Typ des zu registrierenden Panels
     * @param factory Factory-Funktion zur Erstellung von Panels dieses Typs
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool registerPanelType(const QString& panelType, std::function<PanelInterface*()> factory);

    /**
     * @brief Gibt die verfügbaren Panel-Typen zurück.
     * @return Liste der verfügbaren Panel-Typen
     */
    QStringList getAvailablePanelTypes() const;

    /**
     * @brief Speichert die Panel-Konfiguration.
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool savePanelConfiguration();

    /**
     * @brief Lädt die Panel-Konfiguration.
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool loadPanelConfiguration();

    /**
     * @brief Setzt die Panel-Konfiguration zurück.
     * @return true wenn erfolgreich, false bei Fehler
     */
    bool resetPanelConfiguration();

signals:
    /**
     * @brief Signal, das ausgelöst wird, wenn ein Panel erstellt wurde.
     * @param panelId ID des erstellten Panels
     */
    void panelCreated(const QString& panelId);

    /**
     * @brief Signal, das ausgelöst wird, wenn ein Panel entfernt wurde.
     * @param panelId ID des entfernten Panels
     */
    void panelRemoved(const QString& panelId);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Position eines Panels ändert.
     * @param panelId ID des Panels
     * @param position Neue Position
     */
    void panelPositionChanged(const QString& panelId, PanelPosition position);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Geometrie eines Panels ändert.
     * @param panelId ID des Panels
     * @param geometry Neue Geometrie
     */
    void panelGeometryChanged(const QString& panelId, const QRect& geometry);

    /**
     * @brief Signal, das ausgelöst wird, wenn sich die Sichtbarkeit eines Panels ändert.
     * @param panelId ID des Panels
     * @param visible Neue Sichtbarkeit
     */
    void panelVisibilityChanged(const QString& panelId, bool visible);

private:
    // Private Konstruktoren für Singleton-Muster
    PanelManager();
    ~PanelManager();
    PanelManager(const PanelManager&) = delete;
    PanelManager& operator=(const PanelManager&) = delete;

    // Interne Methoden
    QString generatePanelId() const;
    void setupPanelConnections(PanelInterface* panel);

    // Interne Variablen
    QMap<QString, PanelInterface*> m_panels;
    QMap<QString, std::function<PanelInterface*()>> m_panelFactories;
    int m_nextPanelId;
};

} // namespace Panel
} // namespace VivoX
