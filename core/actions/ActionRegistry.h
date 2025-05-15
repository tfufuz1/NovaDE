#pragma once

#include <QObject>
#include <QString>
#include <QVariant>
#include <QMap>
#include <QQmlEngine>
#include <QJSEngine>
#include "ActionInterface.h"

namespace VivoX {
namespace Action {

/**
 * @brief Die ActionRegistry-Klasse registriert Aktionen im QML-System.
 * 
 * Diese Klasse ist verantwortlich für die Registrierung von Aktions-Typen im QML-System
 * und stellt eine Brücke zwischen dem C++-Action-Framework und dem QML-Action-Framework dar.
 * Sie folgt dem Singleton-Muster.
 */
class ActionRegistry : public QObject
{
    Q_OBJECT

public:
    /**
     * @brief Gibt die Singleton-Instanz der ActionRegistry zurück.
     * @return Referenz auf die ActionRegistry-Instanz
     */
    static ActionRegistry& instance();

    /**
     * @brief Initialisiert die ActionRegistry.
     */
    void initialize();

    /**
     * @brief Registriert einen Aktions-Typ im QML-System.
     * @param actionType Typ der zu registrierenden Aktion
     * @param qmlFile Pfad zur QML-Datei der Aktion
     * @param metadata Metadaten des Aktions-Typs
     * @return true wenn erfolgreich, false bei Fehler
     */
    Q_INVOKABLE bool registerActionType(const QString& actionType, 
                                       const QString& qmlFile,
                                       const QVariantMap& metadata = QVariantMap());

    /**
     * @brief Gibt die verfügbaren Aktions-Typen zurück.
     * @return Liste der verfügbaren Aktions-Typen
     */
    Q_INVOKABLE QStringList getAvailableActionTypes() const;

    /**
     * @brief Gibt die Metadaten eines Aktions-Typs zurück.
     * @param actionType Typ der abzufragenden Aktion
     * @return Metadaten des Aktions-Typs
     */
    Q_INVOKABLE QVariantMap getActionTypeMetadata(const QString& actionType) const;

    /**
     * @brief Gibt den QML-Pfad eines Aktions-Typs zurück.
     * @param actionType Typ der abzufragenden Aktion
     * @return QML-Pfad des Aktions-Typs
     */
    Q_INVOKABLE QString getActionQmlPath(const QString& actionType) const;

    /**
     * @brief Erstellt ein QML-Action-Objekt.
     * @param actionType Typ der zu erstellenden Aktion
     * @param parent Elternobjekt
     * @return Erstelltes QML-Objekt oder nullptr bei Fehler
     */
    Q_INVOKABLE QObject* createActionObject(const QString& actionType, QObject* parent = nullptr);

    /**
     * @brief Erstellt eine QML-Action-Komponente.
     * @param actionType Typ der zu erstellenden Aktion
     * @return Erstellte QML-Komponente oder nullptr bei Fehler
     */
    Q_INVOKABLE QObject* createActionComponent(const QString& actionType);

    /**
     * @brief Führt eine Aktion aus.
     * @param actionId ID der auszuführenden Aktion
     * @param parameters Parameter für die Ausführung
     * @return Ergebnis der Ausführung
     */
    Q_INVOKABLE QVariant executeAction(const QString& actionId, const QVariantMap& parameters = QVariantMap());

    /**
     * @brief Registriert die ActionRegistry als QML-Singleton.
     * @param engine QML-Engine
     */
    static QObject* qmlInstance(QQmlEngine* engine, QJSEngine* scriptEngine);

signals:
    /**
     * @brief Signal, das ausgelöst wird, wenn ein Aktions-Typ registriert wurde.
     * @param actionType Typ der registrierten Aktion
     */
    void actionTypeRegistered(const QString& actionType);

    /**
     * @brief Signal, das ausgelöst wird, wenn eine Aktion ausgeführt wurde.
     * @param actionId ID der ausgeführten Aktion
     * @param parameters Parameter der Ausführung
     * @param result Ergebnis der Ausführung
     */
    void actionExecuted(const QString& actionId, const QVariantMap& parameters, const QVariant& result);

private:
    // Private Konstruktoren für Singleton-Muster
    ActionRegistry();
    ~ActionRegistry();
    ActionRegistry(const ActionRegistry&) = delete;
    ActionRegistry& operator=(const ActionRegistry&) = delete;

    // Interne Variablen
    QMap<QString, QString> m_actionQmlPaths;
    QMap<QString, QVariantMap> m_actionTypeMetadata;
};

} // namespace Action
} // namespace VivoX
