#include "ActionInterface.h"
#include <QVariant>
#include <QDebug>
#include "../core/Logger.h"

namespace VivoX {
    namespace Action {

        // Basisimplementierung des ActionInterface
        ActionInterface::ActionInterface(QObject* parent)
        : QObject(parent),
        m_enabled(true),
        m_actionId(QString()),
        m_name(QString()),
        m_description(QString()),
        m_category(QString("Default")),
        m_iconName(QString("action-default"))
        {
        }

        ActionInterface::~ActionInterface()
        {
            Core::Logger::instance().debug("Action destroyed: " + m_actionId, "ActionInterface");
        }

        QString ActionInterface::actionId() const
        {
            return m_actionId;
        }

        void ActionInterface::setActionId(const QString& actionId)
        {
            if (m_actionId != actionId) {
                m_actionId = actionId;
                emit actionIdChanged();
            }
        }

        QString ActionInterface::name() const
        {
            return m_name;
        }

        void ActionInterface::setName(const QString& name)
        {
            if (m_name != name) {
                m_name = name;
                emit nameChanged();
            }
        }

        QString ActionInterface::description() const
        {
            return m_description;
        }

        void ActionInterface::setDescription(const QString& description)
        {
            if (m_description != description) {
                m_description = description;
                emit descriptionChanged();
            }
        }

        QString ActionInterface::category() const
        {
            return m_category;
        }

        void ActionInterface::setCategory(const QString& category)
        {
            if (m_category != category) {
                m_category = category;
                emit categoryChanged();
            }
        }

        bool ActionInterface::isEnabled() const
        {
            return m_enabled;
        }

        void ActionInterface::setEnabled(bool enabled)
        {
            if (m_enabled != enabled) {
                m_enabled = enabled;
                emit enabledChanged();
            }
        }

        QVariantList ActionInterface::parameters() const
        {
            return m_parameters;
        }

        void ActionInterface::setParameters(const QVariantList& parameters)
        {
            if (m_parameters != parameters) {
                m_parameters = parameters;
                emit parametersChanged();
            }
        }

        void ActionInterface::addParameter(const QString& name, const QString& description,
                                           const QString& type, bool required, const QVariant& defaultValue)
        {
            QVariantMap parameter;
            parameter["name"] = name;
            parameter["description"] = description;
            parameter["type"] = type;
            parameter["required"] = required;
            parameter["defaultValue"] = defaultValue;

            m_parameters.append(parameter);
            emit parametersChanged();
        }

        QStringList ActionInterface::tags() const
        {
            return m_tags;
        }

        void ActionInterface::setTags(const QStringList& tags)
        {
            if (m_tags != tags) {
                m_tags = tags;
                emit tagsChanged();
            }
        }

        void ActionInterface::addTag(const QString& tag)
        {
            if (!m_tags.contains(tag)) {
                m_tags.append(tag);
                emit tagsChanged();
            }
        }

        QString ActionInterface::iconName() const
        {
            return m_iconName;
        }

        void ActionInterface::setIconName(const QString& iconName)
        {
            if (m_iconName != iconName) {
                m_iconName = iconName;
                emit iconNameChanged();
            }
        }

        bool ActionInterface::canExecute(const QVariantMap& parameters) const
        {
            if (!m_enabled) {
                return false;
            }

            // Prüfe, ob alle erforderlichen Parameter vorhanden sind
            for (const QVariant& paramVar : m_parameters) {
                QVariantMap param = paramVar.toMap();
                if (param["required"].toBool() && !parameters.contains(param["name"].toString())) {
                    Core::Logger::instance().warning("Missing required parameter: " + param["name"].toString(),
                                                     "ActionInterface");
                    return false;
                }
            }

            return true;
        }

        QVariant ActionInterface::execute(const QVariantMap& parameters)
        {
            if (!canExecute(parameters)) {
                Core::Logger::instance().warning("Cannot execute action: " + m_actionId, "ActionInterface");
                return QVariant(false);
            }

            // Parameter validieren und Standardwerte einfügen
            QVariantMap validatedParameters = validateAndCompleteParameters(parameters);

            // Von der abgeleiteten Klasse implementiert
            QVariant result = executeImpl(validatedParameters);

            // Ausführungshistorie aktualisieren
            m_executionHistory.append({QDateTime::currentDateTime(), validatedParameters, result});
            if (m_executionHistory.size() > 100) { // Begrenze die Größe der Historie
                m_executionHistory.removeFirst();
            }

            return result;
        }

        QVariantMap ActionInterface::validateAndCompleteParameters(const QVariantMap& parameters) const
        {
            QVariantMap result = parameters;

            // Füge Standardwerte für fehlende Parameter ein
            for (const QVariant& paramVar : m_parameters) {
                QVariantMap param = paramVar.toMap();
                QString name = param["name"].toString();

                if (!result.contains(name) && param.contains("defaultValue")) {
                    result[name] = param["defaultValue"];
                }
            }

            return result;
        }

        QVariantList ActionInterface::executionHistory() const
        {
            QVariantList result;

            for (const auto& entry : m_executionHistory) {
                QVariantMap historyEntry;
                historyEntry["timestamp"] = entry.timestamp;
                historyEntry["parameters"] = entry.parameters;
                historyEntry["result"] = entry.result;
                result.append(historyEntry);
            }

            return result;
        }

        void ActionInterface::clearExecutionHistory()
        {
            m_executionHistory.clear();
        }

        QVariant ActionInterface::lastExecutionResult() const
        {
            if (m_executionHistory.isEmpty()) {
                return QVariant();
            }

            return m_executionHistory.last().result;
        }

        QDateTime ActionInterface::lastExecutionTime() const
        {
            if (m_executionHistory.isEmpty()) {
                return QDateTime();
            }

            return m_executionHistory.last().timestamp;
        }

        QVariant ActionInterface::executeImpl(const QVariantMap& parameters)
        {
            // Diese Methode sollte von abgeleiteten Klassen überschrieben werden
            Q_UNUSED(parameters)
            Core::Logger::instance().warning("executeImpl not implemented for action: " + m_actionId, "ActionInterface");
            return QVariant(false);
        }

    } // namespace Action
} // namespace VivoX
