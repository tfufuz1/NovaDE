#include "ActionExecutor.h"
#include "ActionManager.h"
#include "../core/Logger.h"

namespace VivoX {
    namespace Action {

        ActionExecutor::ActionExecutor(QObject* parent) : QObject(parent)
        {
        }

        ActionExecutor::~ActionExecutor()
        {
        }

        QString ActionExecutor::actionId() const
        {
            return m_actionId;
        }

        void ActionExecutor::setActionId(const QString& actionId)
        {
            if (m_actionId != actionId) {
                m_actionId = actionId;
                emit actionIdChanged();
            }
        }

        QVariantMap ActionExecutor::parameters() const
        {
            return m_parameters;
        }

        void ActionExecutor::setParameters(const QVariantMap& parameters)
        {
            if (m_parameters != parameters) {
                m_parameters = parameters;
                emit parametersChanged();
            }
        }

        QVariant ActionExecutor::result() const
        {
            return m_result;
        }

        bool ActionExecutor::isExecuting() const
        {
            return m_executing;
        }

        void ActionExecutor::setParameter(const QString& name, const QVariant& value)
        {
            if (m_parameters[name] != value) {
                m_parameters[name] = value;
                emit parametersChanged();
            }
        }

        QVariant ActionExecutor::getParameter(const QString& name) const
        {
            return m_parameters.value(name);
        }

        void ActionExecutor::removeParameter(const QString& name)
        {
            if (m_parameters.contains(name)) {
                m_parameters.remove(name);
                emit parametersChanged();
            }
        }

        void ActionExecutor::clearParameters()
        {
            if (!m_parameters.isEmpty()) {
                m_parameters.clear();
                emit parametersChanged();
            }
        }

        bool ActionExecutor::canExecute() const
        {
            ActionInterface* action = ActionManager::instance().getAction(m_actionId);

            if (!action) {
                return false;
            }

            return action->canExecute(m_parameters);
        }

        void ActionExecutor::execute()
        {
            if (m_executing) {
                Core::Logger::instance().warning("Action is already executing: " + m_actionId, "ActionExecutor");
                return;
            }

            if (m_actionId.isEmpty()) {
                Core::Logger::instance().error("No action ID specified", "ActionExecutor");
                emit executionFailed("No action ID specified");
                return;
            }

            m_executing = true;
            emit executingChanged();

            // Führe die Aktion asynchron aus
            QFuture<QVariant> future = QtConcurrent::run([this]() {
                return ActionManager::instance().executeAction(m_actionId, m_parameters);
            });

            // Verbinde mit dem Watcher
            m_watcher.setFuture(future);
        }

        void ActionExecutor::abort()
        {
            if (!m_executing) {
                return;
            }

            // Versuche, die Ausführung abzubrechen
            m_watcher.cancel();

            // Setze den Status zurück
            m_executing = false;
            emit executingChanged();
            emit executionAborted();
        }

        void ActionExecutor::handleExecutionFinished()
        {
            m_executing = false;
            emit executingChanged();

            if (m_watcher.isCanceled()) {
                emit executionAborted();
                return;
            }

            m_result = m_watcher.result();
            emit resultChanged();

            if (m_result.type() == QVariant::Bool && !m_result.toBool()) {
                emit executionFailed("Action execution failed");
            } else {
                emit executionSucceeded();
            }

            emit executionFinished();
        }

        void ActionExecutor::connectWatcher()
        {
            connect(&m_watcher, &QFutureWatcher<QVariant>::finished,
                    this, &ActionExecutor::handleExecutionFinished);
        }

    } // namespace Action
} // namespace VivoX
