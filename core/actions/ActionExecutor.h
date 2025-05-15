#ifndef VIVOX_ACTIONEXECUTOR_H
#define VIVOX_ACTIONEXECUTOR_H

#include <QObject>
#include <QVariant>
#include <QVariantMap>
#include <QFutureWatcher>
#include <QtConcurrent>

namespace VivoX {
    namespace Action {

        /**
         * @brief Ausführungshelfer für Aktionen
         *
         * Diese Klasse dient als Adapter zwischen QML und dem ActionManager.
         * Sie ermöglicht die asynchrone Ausführung von Aktionen und bietet
         * eine einfache API für QML.
         */
        class ActionExecutor : public QObject {
            Q_OBJECT

            /// ID der auszuführenden Aktion
            Q_PROPERTY(QString actionId READ actionId WRITE setActionId NOTIFY actionIdChanged)

            /// Parameter für die Ausführung
            Q_PROPERTY(QVariantMap parameters READ parameters WRITE setParameters NOTIFY parametersChanged)

            /// Ergebnis der letzten Ausführung
            Q_PROPERTY(QVariant result READ result NOTIFY resultChanged)

            /// Flag, ob die Aktion gerade ausgeführt wird
            Q_PROPERTY(bool isExecuting READ isExecuting NOTIFY executingChanged)

        public:
            /**
             * @brief Konstruktor
             * @param parent Elternobjekt
             */
            explicit ActionExecutor(QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            virtual ~ActionExecutor();

            /**
             * @brief Gibt die ID der Aktion zurück
             * @return ID der Aktion
             */
            QString actionId() const;

            /**
             * @brief Setzt die ID der Aktion
             * @param actionId ID der Aktion
             */
            void setActionId(const QString& actionId);

            /**
             * @brief Gibt die Parameter zurück
             * @return Parameter der Aktion
             */
            QVariantMap parameters() const;

            /**
             * @brief Setzt die Parameter
             * @param parameters Parameter der Aktion
             */
            void setParameters(const QVariantMap& parameters);

            /**
             * @brief Gibt das Ergebnis zurück
             * @return Ergebnis der letzten Ausführung
             */
            QVariant result() const;

            /**
             * @brief Gibt zurück, ob die Aktion gerade ausgeführt wird
             * @return true, wenn die Aktion ausgeführt wird, sonst false
             */
            bool isExecuting() const;

        public slots:
            /**
             * @brief Setzt einen einzelnen Parameter
             * @param name Name des Parameters
             * @param value Wert des Parameters
             */
            Q_INVOKABLE void setParameter(const QString& name, const QVariant& value);

            /**
             * @brief Gibt den Wert eines Parameters zurück
             * @param name Name des Parameters
             * @return Wert des Parameters
             */
            Q_INVOKABLE QVariant getParameter(const QString& name) const;

            /**
             * @brief Entfernt einen Parameter
             * @param name Name des Parameters
             */
            Q_INVOKABLE void removeParameter(const QString& name);

            /**
             * @brief Leert alle Parameter
             */
            Q_INVOKABLE void clearParameters();

            /**
             * @brief Prüft, ob die Aktion ausgeführt werden kann
             * @return true, wenn die Aktion ausgeführt werden kann, sonst false
             */
            Q_INVOKABLE bool canExecute() const;

            /**
             * @brief Führt die Aktion asynchron aus
             */
            Q_INVOKABLE void execute();

            /**
             * @brief Bricht die Ausführung ab
             */
            Q_INVOKABLE void abort();

        signals:
            /**
             * @brief Signal, wenn sich die Aktions-ID ändert
             */
            void actionIdChanged();

            /**
             * @brief Signal, wenn sich die Parameter ändern
             */
            void parametersChanged();

            /**
             * @brief Signal, wenn sich das Ergebnis ändert
             */
            void resultChanged();

            /**
             * @brief Signal, wenn sich der Ausführungsstatus ändert
             */
            void executingChanged();

            /**
             * @brief Signal, wenn die Ausführung erfolgreich beendet wurde
             */
            void executionSucceeded();

            /**
             * @brief Signal, wenn die Ausführung fehlgeschlagen ist
             * @param errorMessage Fehlermeldung
             */
            void executionFailed(const QString& errorMessage);

            /**
             * @brief Signal, wenn die Ausführung abgebrochen wurde
             */
            void executionAborted();

            /**
             * @brief Signal, wenn die Ausführung beendet wurde (erfolgreich oder nicht)
             */
            void executionFinished();

        private slots:
            /**
             * @brief Behandelt das Ende der asynchronen Ausführung
             */
            void handleExecutionFinished();

        private:
            /**
             * @brief Verbindet den FutureWatcher
             */
            void connectWatcher();

            QString m_actionId;                      ///< ID der Aktion
            QVariantMap m_parameters;                ///< Parameter der Aktion
            QVariant m_result;                       ///< Ergebnis der Ausführung
            bool m_executing = false;                ///< Ausführungsstatus
            QFutureWatcher<QVariant> m_watcher;      ///< Watcher für asynchrone Ausführung
        };

    } // namespace Action
} // namespace VivoX

#endif // VIVOX_ACTIONEXECUTOR_H
