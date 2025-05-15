#include "ActionManager.h"
#include <QDebug>
#include <QQmlEngine>
#include <QQmlContext>
#include "../core/Logger.h"
#include <QUuid>

namespace VivoX {
    namespace Action {

        ActionManager::ActionManager() : QObject(nullptr), m_nextCallbackId(1)
        {
            Core::Logger::instance().info("ActionManager initialized", "ActionManager");
        }

        ActionManager::~ActionManager()
        {
            Core::Logger::instance().info("ActionManager destroyed", "ActionManager");
        }

        ActionManager& ActionManager::instance()
        {
            static ActionManager instance;
            return instance;
        }

        void ActionManager::initialize()
        {
            // Registriere das Action-Framework bei QML
            qmlRegisterSingletonType<ActionManager>("VivoX.Action", 1, 0, "ActionManager", qmlInstance);

            Core::Logger::instance().info("ActionManager initialized with QML registration", "ActionManager");
        }

        QObject* ActionManager::qmlInstance(QQmlEngine* engine, QJSEngine* scriptEngine)
        {
            Q_UNUSED(scriptEngine)

            ActionManager* instance = &ActionManager::instance();
            QQmlEngine::setObjectOwnership(instance, QQmlEngine::CppOwnership);
            return instance;
        }

        bool ActionManager::registerAction(ActionInterface* action)
        {
            if (!action) {
                Core::Logger::instance().error("Attempt to register null action", "ActionManager");
                return false;
            }

            QString actionId = action->actionId();

            if (actionId.isEmpty()) {
                // Generiere eine neue ID, falls keine vorhanden ist
                actionId = QUuid::createUuid().toString(QUuid::WithoutBraces);
                action->setActionId(actionId);
            }

            if (m_actions.contains(actionId)) {
                Core::Logger::instance().warning("Action ID already registered: " + actionId, "ActionManager");
                return false;
            }

            m_actions[actionId] = action;

            // Verbinde Signale
            connect(action, &QObject::destroyed, this, [this, actionId]() {
                m_actions.remove(actionId);
                emit actionUnregistered(actionId);
            });

            emit actionRegistered(actionId);
            Core::Logger::instance().info("Action registered: " + actionId, "ActionManager");

            return true;
        }

        bool ActionManager::unregisterAction(const QString& actionId)
        {
            if (!m_actions.contains(actionId)) {
                return false;
            }

            ActionInterface* action = m_actions.take(actionId);

            // Trenne Verbindungen
            disconnect(action, nullptr, this, nullptr);

            emit actionUnregistered(actionId);
            Core::Logger::instance().info("Action unregistered: " + actionId, "ActionManager");

            return true;
        }

        ActionInterface* ActionManager::getAction(const QString& actionId) const
        {
            return m_actions.value(actionId, nullptr);
        }

        QStringList ActionManager::getActionIds() const
        {
            return m_actions.keys();
        }

        QList<ActionInterface*> ActionManager::getActions() const
        {
            return m_actions.values();
        }

        QList<ActionInterface*> ActionManager::getActionsByCategory(const QString& category) const
        {
            QList<ActionInterface*> result;

            for (ActionInterface* action : m_actions.values()) {
                if (action->category() == category) {
                    result.append(action);
                }
            }

            return result;
        }

        QList<ActionInterface*> ActionManager::getActionsByTag(const QString& tag) const
        {
            QList<ActionInterface*> result;

            for (ActionInterface* action : m_actions.values()) {
                if (action->tags().contains(tag)) {
                    result.append(action);
                }
            }

            return result;
        }

        QList<ActionInterface*> ActionManager::searchActions(const QString& query) const
        {
            QList<ActionInterface*> result;

            QString lowerQuery = query.toLower();

            for (ActionInterface* action : m_actions.values()) {
                // Suche in Namen, Beschreibung, Kategorie und ID
                if (action->name().toLower().contains(lowerQuery) ||
                    action->description().toLower().contains(lowerQuery) ||
                    action->category().toLower().contains(lowerQuery) ||
                    action->actionId().toLower().contains(lowerQuery)) {
                    result.append(action);
                continue;
                    }

                    // Suche in Tags
                    for (const QString& tag : action->tags()) {
                        if (tag.toLower().contains(lowerQuery)) {
                            result.append(action);
                            break;
                        }
                    }
            }

            return result;
        }

        // ActionManager - Ergänzungen für Undo/Redo-Integration
        // Diese Ergänzungen sollten in ActionManager.cpp hinzugefügt werden

        bool ActionManager::undo()
        {
            if (m_undoStack.isEmpty()) {
                Core::Logger::instance().warning("Undo stack is empty", "ActionManager");
                return false;
            }

            UndoEntry entry = m_undoStack.takeLast();

            // Finde die entsprechende Aktion
            ActionInterface* action = getAction(entry.actionId);
            if (!action) {
                Core::Logger::instance().error("Action not found for undo: " + entry.actionId, "ActionManager");
                // Stelle den Undo-Stack wieder her, da die Operation fehlgeschlagen ist
                m_undoStack.append(entry);
                return false;
            }

            // Prüfe, ob es sich um eine UndoableActionInterface handelt
            UndoableActionInterface* undoableAction = qobject_cast<UndoableActionInterface*>(action);

            bool success = false;
            QVariant result;

            try {
                if (undoableAction) {
                    // Verwende die spezifische Undo-Funktion des UndoableActionInterface
                    result = undoableAction->undo(entry.parameters);
                    success = result.toBool();
                } else {
                    // Fallback: Ausführung mit invertierten Parametern
                    QVariantMap undoParams = entry.parameters;
                    undoParams["__undo"] = true;
                    result = action->execute(undoParams);
                    success = result.toBool();
                }
            } catch (const std::exception& e) {
                Core::Logger::instance().error("Exception during undo operation: " + QString(e.what()), "ActionManager");
                // Stelle den Undo-Stack wieder her
                m_undoStack.append(entry);
                return false;
            }

            if (success) {
                // Zur Redo-Stack hinzufügen
                m_redoStack.append(entry);
                emit undoPerformed(entry.actionId);

                // Aktualisiere die Ausführungshistorie
                ExecutionHistoryEntry historyEntry;
                historyEntry.timestamp = QDateTime::currentDateTime();
                historyEntry.actionId = entry.actionId;
                historyEntry.parameters = entry.parameters;
                historyEntry.result = result;
                historyEntry.operationType = "undo";

                m_executionHistory.append(historyEntry);

                // Begrenze die Größe der Historie
                while (m_executionHistory.size() > m_maxHistorySize) {
                    m_executionHistory.removeFirst();
                }
            } else {
                // Bei Fehler wieder zur Undo-Stack hinzufügen
                m_undoStack.append(entry);
                Core::Logger::instance().warning("Undo operation failed for action: " + entry.actionId, "ActionManager");
            }

            return success;
        }

        bool ActionManager::redo()
        {
            if (m_redoStack.isEmpty()) {
                Core::Logger::instance().warning("Redo stack is empty", "ActionManager");
                return false;
            }

            UndoEntry entry = m_redoStack.takeLast();

            // Finde die entsprechende Aktion
            ActionInterface* action = getAction(entry.actionId);
            if (!action) {
                Core::Logger::instance().error("Action not found for redo: " + entry.actionId, "ActionManager");
                // Stelle den Redo-Stack wieder her, da die Operation fehlgeschlagen ist
                m_redoStack.append(entry);
                return false;
            }

            // Prüfe, ob es sich um eine UndoableActionInterface handelt
            UndoableActionInterface* undoableAction = qobject_cast<UndoableActionInterface*>(action);

            bool success = false;
            QVariant result;

            try {
                if (undoableAction) {
                    // Verwende die spezifische Redo-Funktion des UndoableActionInterface
                    result = undoableAction->redo(entry.parameters);
                    success = result.toBool();
                } else {
                    // Standard: Führe die Aktion erneut aus
                    result = action->execute(entry.parameters);
                    success = result.toBool();
                }
            } catch (const std::exception& e) {
                Core::Logger::instance().error("Exception during redo operation: " + QString(e.what()), "ActionManager");
                // Stelle den Redo-Stack wieder her
                m_redoStack.append(entry);
                return false;
            }

            if (success) {
                // Zur Undo-Stack hinzufügen
                m_undoStack.append(entry);
                emit redoPerformed(entry.actionId);

                // Aktualisiere die Ausführungshistorie
                ExecutionHistoryEntry historyEntry;
                historyEntry.timestamp = QDateTime::currentDateTime();
                historyEntry.actionId = entry.actionId;
                historyEntry.parameters = entry.parameters;
                historyEntry.result = result;
                historyEntry.operationType = "redo";

                m_executionHistory.append(historyEntry);

                // Begrenze die Größe der Historie
                while (m_executionHistory.size() > m_maxHistorySize) {
                    m_executionHistory.removeFirst();
                }
            } else {
                // Bei Fehler wieder zur Redo-Stack hinzufügen
                m_redoStack.append(entry);
                Core::Logger::instance().warning("Redo operation failed for action: " + entry.actionId, "ActionManager");
            }

            return success;
        }

        QVariant ActionManager::executeAction(const QString& actionId, const QVariantMap& parameters)
        {
            ActionInterface* action = getAction(actionId);

            if (!action) {
                Core::Logger::instance().error("Action not found: " + actionId, "ActionManager");
                return QVariant(false);
            }

            if (!action->isEnabled()) {
                Core::Logger::instance().warning("Action is disabled: " + actionId, "ActionManager");
                return QVariant(false);
            }

            // Führe die Aktion aus
            QVariant result = action->execute(parameters);

            // Aktualisiere die Ausführungshistorie
            ExecutionHistoryEntry entry;
            entry.timestamp = QDateTime::currentDateTime();
            entry.actionId = actionId;
            entry.parameters = parameters;
            entry.result = result;

            m_executionHistory.append(entry);

            // Begrenze die Größe der Historie
            while (m_executionHistory.size() > m_maxHistorySize) {
                m_executionHistory.removeFirst();
            }

            // Zur Undo-Stack hinzufügen, falls erfolgreich
            if (result.toBool()) {
                m_undoStack.append(UndoEntry{actionId, parameters});
                m_redoStack.clear();
            }

            // Benachrichtige Callbacks
            for (auto it = m_actionExecutedCallbacks.begin(); it != m_actionExecutedCallbacks.end(); ++it) {
                try {
                    it.value()(actionId, parameters, result);
                } catch (const std::exception& e) {
                    Core::Logger::instance().error("Exception in action executed callback: " + QString(e.what()),
                                                   "ActionManager");
                }
            }

            emit actionExecuted(actionId, parameters, result);

            return result;
        }

        int ActionManager::registerActionExecutedCallback(const std::function<void(const QString&, const QVariantMap&, const QVariant&)>& callback)
        {
            if (!callback) {
                return -1;
            }

            int id = m_nextCallbackId++;
            m_actionExecutedCallbacks[id] = callback;

            return id;
        }

        bool ActionManager::unregisterActionExecutedCallback(int callbackId)
        {
            return m_actionExecutedCallbacks.remove(callbackId) > 0;
        }

        QVariantList ActionManager::getExecutionHistory(int maxEntries) const
        {
            QVariantList result;

            int count = maxEntries > 0 ? qMin(maxEntries, m_executionHistory.size()) : m_executionHistory.size();
            int startIndex = m_executionHistory.size() - count;

            for (int i = startIndex; i < m_executionHistory.size(); ++i) {
                const ExecutionHistoryEntry& entry = m_executionHistory[i];

                QVariantMap historyEntry;
                historyEntry["timestamp"] = entry.timestamp;
                historyEntry["actionId"] = entry.actionId;
                historyEntry["parameters"] = entry.parameters;
                historyEntry["result"] = entry.result;

                result.append(historyEntry);
            }

            return result;
        }

        void ActionManager::clearExecutionHistory()
        {
            m_executionHistory.clear();
        }

        bool ActionManager::undo()
        {
            if (m_undoStack.isEmpty()) {
                return false;
            }

            UndoEntry entry = m_undoStack.takeLast();

            // Finde die entsprechende Aktion
            ActionInterface* action = getAction(entry.actionId);
            if (!action) {
                return false;
            }

            // Implementiere die Undo-Logik
            // Führe die Undo-Methode aus, falls vorhanden
            ActionInterface* undoableAction = qobject_cast<UndoableActionInterface*>(action);

            bool success = false;
            if (undoableAction) {
                UndoableActionInterface* undoable = static_cast<UndoableActionInterface*>(undoableAction);
                QVariant result = undoable->undo(entry.parameters);
                success = result.toBool();
            } else {
                // Fallback: Ausführung mit invertierten Parametern
                QVariantMap undoParams = entry.parameters;
                undoParams["__undo"] = true;
                QVariant result = action->execute(undoParams);
                success = result.toBool();
            }

            if (success) {
                // Zur Redo-Stack hinzufügen
                m_redoStack.append(entry);
                emit undoPerformed(entry.actionId);
            } else {
                // Bei Fehler wieder zur Undo-Stack hinzufügen
                m_undoStack.append(entry);
            }

            return success;
        }

        bool ActionManager::redo()
        {
            if (m_redoStack.isEmpty()) {
                return false;
            }

            UndoEntry entry = m_redoStack.takeLast();

            // Finde die entsprechende Aktion
            ActionInterface* action = getAction(entry.actionId);
            if (!action) {
                return false;
            }

            // Führe die Aktion erneut aus
            QVariant result = action->execute(entry.parameters);
            bool success = result.toBool();

            if (success) {
                // Zur Undo-Stack hinzufügen
                m_undoStack.append(entry);
                emit redoPerformed(entry.actionId);
            } else {
                // Bei Fehler wieder zur Redo-Stack hinzufügen
                m_redoStack.append(entry);
            }

            return success;
        }

        int ActionManager::undoStackSize() const
        {
            return m_undoStack.size();
        }

        int ActionManager::redoStackSize() const
        {
            return m_redoStack.size();
        }

        void ActionManager::clearUndoStack()
        {
            m_undoStack.clear();
        }

        void ActionManager::clearRedoStack()
        {
            m_redoStack.clear();
        }

        void ActionManager::setMaxHistorySize(int size)
        {
            m_maxHistorySize = qMax(10, size);  // Mindestens 10 Einträge

            // Begrenze die aktuelle Historie
            while (m_executionHistory.size() > m_maxHistorySize) {
                m_executionHistory.removeFirst();
            }
        }

        int ActionManager::maxHistorySize() const
        {
            return m_maxHistorySize;
        }

    } // namespace Action
} // namespace VivoX
