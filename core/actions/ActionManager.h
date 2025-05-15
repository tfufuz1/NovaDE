#ifndef VIVOX_ACTIONMANAGER_H
#define VIVOX_ACTIONMANAGER_H

#include <QObject>
#include <QMap>
#include <QVariant>
#include <QDateTime>
#include <QList>
#include <functional>
#include <QQmlEngine>
#include <QJSEngine>
#include "ActionInterface.h"
#include "UndoableActionInterface.h"

namespace VivoX {
    namespace Action {

        /**
         * @brief Manager für alle verfügbaren Aktionen
         *
         * Diese Klasse verwaltet alle registrierten Aktionen und bietet
         * zentrale Funktionalitäten für deren Ausführung, Undo/Redo und
         * die Verwaltung der Ausführungshistorie.
         */
        class ActionManager : public QObject {
            Q_OBJECT

        public:
            /**
             * @brief Gibt die Singleton-Instanz zurück
             * @return Referenz auf die Singleton-Instanz
             */
            static ActionManager& instance();

            /**
             * @brief Initialisiert den ActionManager
             */
            static void initialize();

            /**
             * @brief Factory-Methode für QML
             * @param engine QML-Engine
             * @param scriptEngine JavaScript-Engine
             * @return Singleton-Instanz als QObject
             */
            static QObject* qmlInstance(QQmlEngine* engine, QJSEngine* scriptEngine);

            /**
             * @brief Registriert eine Aktion
             * @param action Zeiger auf die Aktion
             * @return true bei Erfolg, sonst false
             */
            bool registerAction(ActionInterface* action);

            /**
             * @brief Entfernt eine Aktion
             * @param actionId ID der zu entfernenden Aktion
             * @return true bei Erfolg, sonst false
             */
            bool unregisterAction(const QString& actionId);

            /**
             * @brief Gibt eine Aktion anhand ihrer ID zurück
             * @param actionId ID der Aktion
             * @return Zeiger auf die Aktion oder nullptr
             */
            ActionInterface* getAction(const QString& actionId) const;

            /**
             * @brief Gibt alle verfügbaren Aktions-IDs zurück
             * @return Liste von Aktions-IDs
             */
            QStringList getActionIds() const;

            /**
             * @brief Gibt alle verfügbaren Aktionen zurück
             * @return Liste von Aktionen
             */
            QList<ActionInterface*> getActions() const;

            /**
             * @brief Gibt Aktionen nach Kategorie gefiltert zurück
             * @param category Kategorie
             * @return Liste von Aktionen
             */
            QList<ActionInterface*> getActionsByCategory(const QString& category) const;

            /**
             * @brief Gibt Aktionen nach Tag gefiltert zurück
             * @param tag Tag
             * @return Liste von Aktionen
             */
            QList<ActionInterface*> getActionsByTag(const QString& tag) const;

            /**
             * @brief Sucht Aktionen nach Suchbegriff
             * @param query Suchbegriff
             * @return Liste von gefundenen Aktionen
             */
            QList<ActionInterface*> searchActions(const QString& query) const;

            /**
             * @brief Führt eine Aktion aus
             * @param actionId ID der Aktion
             * @param parameters Parameter für die Ausführung
             * @return Ergebnis der Ausführung
             */
            Q_INVOKABLE QVariant executeAction(const QString& actionId, const QVariantMap& parameters = QVariantMap());

            /**
             * @brief Registriert einen Callback für Aktionsausführungen
             * @param callback Callback-Funktion
             * @return ID des Callbacks
             */
            int registerActionExecutedCallback(const std::function<void(const QString&, const QVariantMap&, const QVariant&)>& callback);

            /**
             * @brief Entfernt einen Callback
             * @param callbackId ID des Callbacks
             * @return true bei Erfolg, sonst false
             */
            bool unregisterActionExecutedCallback(int callbackId);

            /**
             * @brief Gibt die Ausführungshistorie zurück
             * @param maxEntries Maximale Anzahl der Einträge (0 für alle)
             * @return Liste von Historieneinträgen
             */
            Q_INVOKABLE QVariantList getExecutionHistory(int maxEntries = 0) const;

            /**
             * @brief Leert die Ausführungshistorie
             */
            Q_INVOKABLE void clearExecutionHistory();

            /**
             * @brief Führt Undo durch
             * @return true bei Erfolg, sonst false
             */
            Q_INVOKABLE bool undo();

            /**
             * @brief Führt Redo durch
             * @return true bei Erfolg, sonst false
             */
            Q_INVOKABLE bool redo();

            /**
             * @brief Gibt die Größe des Undo-Stacks zurück
             * @return Anzahl der Undo-Einträge
             */
            Q_INVOKABLE int undoStackSize() const;

            /**
             * @brief Gibt die Größe des Redo-Stacks zurück
             * @return Anzahl der Redo-Einträge
             */
            Q_INVOKABLE int redoStackSize() const;

            /**
             * @brief Leert den Undo-Stack
             */
            Q_INVOKABLE void clearUndoStack();

            /**
             * @brief Leert den Redo-Stack
             */
            Q_INVOKABLE void clearRedoStack();

            /**
             * @brief Setzt die maximale Größe der Historie
             * @param size Maximale Anzahl der Einträge
             */
            void setMaxHistorySize(int size);

            /**
             * @brief Gibt die maximale Größe der Historie zurück
             * @return Maximale Anzahl der Einträge
             */
            int maxHistorySize() const;

        signals:
            /**
             * @brief Signal, wenn eine Aktion registriert wurde
             * @param actionId ID der registrierten Aktion
             */
            void actionRegistered(const QString& actionId);

            /**
             * @brief Signal, wenn eine Aktion entfernt wurde
             * @param actionId ID der entfernten Aktion
             */
            void actionUnregistered(const QString& actionId);

            /**
             * @brief Signal, wenn eine Aktion ausgeführt wurde
             * @param actionId ID der ausgeführten Aktion
             * @param parameters Parameter der Ausführung
             * @param result Ergebnis der Ausführung
             */
            void actionExecuted(const QString& actionId, const QVariantMap& parameters, const QVariant& result);

            /**
             * @brief Signal, wenn Undo durchgeführt wurde
             * @param actionId ID der betroffenen Aktion
             */
            void undoPerformed(const QString& actionId);

            /**
             * @brief Signal, wenn Redo durchgeführt wurde
             * @param actionId ID der betroffenen Aktion
             */
            void redoPerformed(const QString& actionId);

        private:
            /**
             * @brief Konstruktor (privat für Singleton)
             */
            ActionManager();

            /**
             * @brief Destruktor
             */
            ~ActionManager();

            // Verhindert Kopieren und Zuweisen
            ActionManager(const ActionManager&) = delete;
            ActionManager& operator=(const ActionManager&) = delete;

            /**
             * @brief Struktur für Einträge in der Undo/Redo-Historie
             */
            struct UndoEntry {
                QString actionId;        ///< ID der Aktion
                QVariantMap parameters;  ///< Parameter der Aktion
            };

            /**
             * @brief Struktur für Einträge in der Ausführungshistorie
             */
            struct ExecutionHistoryEntry {
                QDateTime timestamp;
                QString actionId;
                QVariantMap parameters;
                QVariant result;
                QString operationType = "execute"; // "execute", "undo", "redo"
            };

            QMap<QString, ActionInterface*> m_actions;                      ///< Registrierte Aktionen
            QList<ExecutionHistoryEntry> m_executionHistory;                ///< Ausführungshistorie
            QList<UndoEntry> m_undoStack;                                   ///< Undo-Stack
            QList<UndoEntry> m_redoStack;                                   ///< Redo-Stack
            QMap<int, std::function<void(const QString&, const QVariantMap&, const QVariant&)>> m_actionExecutedCallbacks; ///< Callbacks für Aktionsausführungen
            int m_nextCallbackId;                                            ///< Nächste Callback-ID
            int m_maxHistorySize = 100;                                     ///< Maximale Größe der Historie
        };

    } // namespace Action
} // namespace VivoX

#endif // VIVOX_ACTIONMANAGER_H
