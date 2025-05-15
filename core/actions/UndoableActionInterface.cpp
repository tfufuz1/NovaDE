// UndoableActionInterface.cpp - Vollständige Implementierung
#include "UndoableActionInterface.h"
#include "../core/Logger.h"

namespace VivoX {
    namespace Action {

        // Definiere die maximale Stapelgröße
        const int UndoableActionInterface::MAX_STACK_SIZE = 100;

        UndoableActionInterface::UndoableActionInterface(QObject* parent)
        : ActionInterface(parent),
        m_lastUndoSuccessful(false),
        m_lastRedoSuccessful(false)
        {
            // Füge spezifische Tags für undoable Actions hinzu
            addTag("undoable");
        }

        UndoableActionInterface::~UndoableActionInterface()
        {
            Core::Logger::instance().debug("UndoableAction destroyed: " + actionId(), "UndoableActionInterface");
        }

        QVariant UndoableActionInterface::undo(const QVariantMap& parameters)
        {
            // Speichere die ursprünglichen Parameter
            QVariantMap undoParams = parameters;

            // Standardimplementierung
            if (m_undoStack.isEmpty()) {
                Core::Logger::instance().warning("Undo stack is empty for action: " + actionId(), "UndoableActionInterface");
                m_lastUndoSuccessful = false;
                emit canUndoChanged();
                return QVariant(false);
            }

            // Nehme den letzten Zustand aus dem Undo-Stack
            UndoState state = m_undoStack.takeLast();

            // Aktualisiere die Parameter falls keine angegeben wurden
            if (undoParams.isEmpty()) {
                undoParams = state.parameters;
            }

            // Führe die Undo-Operation durch
            QVariant currentState = captureCurrentState();
            QVariant result;

            try {
                result = undoImpl(state.parameters, state.previousState);

                // Überprüfe das Ergebnis
                if (!result.isValid() || (result.type() == QVariant::Bool && !result.toBool())) {
                    // Undo-Operation fehlgeschlagen, stelle den Undo-Stack wieder her
                    m_undoStack.append(state);
                    Core::Logger::instance().warning("Undo operation failed for action: " + actionId(), "UndoableActionInterface");
                    m_lastUndoSuccessful = false;
                } else {
                    // Undo-Operation erfolgreich, speichere den aktuellen Zustand im Redo-Stack
                    UndoState redoState;
                    redoState.parameters = state.parameters;
                    redoState.previousState = currentState;
                    redoState.timestamp = QDateTime::currentDateTime();

                    m_redoStack.append(redoState);

                    // Begrenze die Größe des Redo-Stapels
                    while (m_redoStack.size() > MAX_STACK_SIZE) {
                        m_redoStack.removeFirst();
                    }

                    m_lastUndoSuccessful = true;
                    emit undoPerformed(state.previousState);
                }
            } catch (const std::exception& e) {
                // Fehlerbehandlung
                Core::Logger::instance().error("Exception during undo operation: " + QString(e.what()), "UndoableActionInterface");
                m_undoStack.append(state);
                m_lastUndoSuccessful = false;
                result = QVariant(false);
            }

            // Emitiere Signale für Zustandsänderungen
            emit canUndoChanged();
            emit canRedoChanged();

            return result;
        }

        QVariant UndoableActionInterface::redo(const QVariantMap& parameters)
        {
            // Speichere die ursprünglichen Parameter
            QVariantMap redoParams = parameters;

            // Standardimplementierung
            if (m_redoStack.isEmpty()) {
                Core::Logger::instance().warning("Redo stack is empty for action: " + actionId(), "UndoableActionInterface");
                m_lastRedoSuccessful = false;
                emit canRedoChanged();
                return QVariant(false);
            }

            // Nehme den letzten Zustand aus dem Redo-Stack
            UndoState state = m_redoStack.takeLast();

            // Aktualisiere die Parameter falls keine angegeben wurden
            if (redoParams.isEmpty()) {
                redoParams = state.parameters;
            }

            // Führe die Redo-Operation durch
            QVariant currentState = captureCurrentState();
            QVariant result;

            try {
                result = redoImpl(state.parameters, state.previousState);

                // Überprüfe das Ergebnis
                if (!result.isValid() || (result.type() == QVariant::Bool && !result.toBool())) {
                    // Redo-Operation fehlgeschlagen, stelle den Redo-Stack wieder her
                    m_redoStack.append(state);
                    Core::Logger::instance().warning("Redo operation failed for action: " + actionId(), "UndoableActionInterface");
                    m_lastRedoSuccessful = false;
                } else {
                    // Redo-Operation erfolgreich, speichere den aktuellen Zustand im Undo-Stack
                    UndoState undoState;
                    undoState.parameters = state.parameters;
                    undoState.previousState = currentState;
                    undoState.timestamp = QDateTime::currentDateTime();

                    m_undoStack.append(undoState);

                    // Begrenze die Größe des Undo-Stapels
                    while (m_undoStack.size() > MAX_STACK_SIZE) {
                        m_undoStack.removeFirst();
                    }

                    m_lastRedoSuccessful = true;
                    emit redoPerformed(currentState);
                }
            } catch (const std::exception& e) {
                // Fehlerbehandlung
                Core::Logger::instance().error("Exception during redo operation: " + QString(e.what()), "UndoableActionInterface");
                m_redoStack.append(state);
                m_lastRedoSuccessful = false;
                result = QVariant(false);
            }

            // Emitiere Signale für Zustandsänderungen
            emit canUndoChanged();
            emit canRedoChanged();

            return result;
        }

        bool UndoableActionInterface::canUndo() const
        {
            return !m_undoStack.isEmpty();
        }

        bool UndoableActionInterface::canRedo() const
        {
            return !m_redoStack.isEmpty();
        }

        void UndoableActionInterface::clearUndoStack()
        {
            if (!m_undoStack.isEmpty()) {
                m_undoStack.clear();
                emit canUndoChanged();
            }
        }

        void UndoableActionInterface::clearRedoStack()
        {
            if (!m_redoStack.isEmpty()) {
                m_redoStack.clear();
                emit canRedoChanged();
            }
        }

        QVariant UndoableActionInterface::execute(const QVariantMap& parameters)
        {
            // Speichere den aktuellen Zustand für Undo
            QVariant currentState = captureCurrentState();

            // Validiere den Zustand
            currentState = validateState(currentState);

            // Führe die Aktion aus
            QVariant result = ActionInterface::execute(parameters);

            // Füge den Zustand zum Undo-Stack hinzu, wenn die Ausführung erfolgreich war
            if (result.toBool()) {
                UndoState state;
                state.parameters = parameters;
                state.previousState = currentState;
                state.timestamp = QDateTime::currentDateTime();

                m_undoStack.append(state);

                // Begrenze die Größe des Undo-Stapels
                while (m_undoStack.size() > MAX_STACK_SIZE) {
                    m_undoStack.removeFirst();
                }

                // Lösche den Redo-Stack, da eine neue Aktion ausgeführt wurde
                if (!m_redoStack.isEmpty()) {
                    m_redoStack.clear();
                    emit canRedoChanged();
                }

                emit canUndoChanged();
            }

            return result;
        }

        QVariant UndoableActionInterface::captureCurrentState() const
        {
            // Standardimplementierung: Leerer Zustand
            // Diese Methode sollte in abgeleiteten Klassen überschrieben werden
            Core::Logger::instance().debug("captureCurrentState called but not implemented for: " + actionId(), "UndoableActionInterface");
            return QVariant();
        }

        QVariant UndoableActionInterface::undoImpl(const QVariantMap& parameters, const QVariant& previousState)
        {
            // Standardimplementierung: Nicht unterstützt
            // Diese Methode sollte in abgeleiteten Klassen überschrieben werden
            Q_UNUSED(parameters)
            Q_UNUSED(previousState)

            Core::Logger::instance().warning("undoImpl not implemented for action: " + actionId(), "UndoableActionInterface");
            return QVariant(false);
        }

        QVariant UndoableActionInterface::redoImpl(const QVariantMap& parameters, const QVariant& previousState)
        {
            // Standardimplementierung: Einfach erneut ausführen
            // Diese Methode kann in abgeleiteten Klassen überschrieben werden
            Q_UNUSED(previousState)

            return ActionInterface::execute(parameters);
        }

        QVariant UndoableActionInterface::validateState(const QVariant& state) const
        {
            // Standardimplementierung: Validiere und optimiere den Zustand
            // Diese Methode kann in abgeleiteten Klassen überschrieben werden

            // Wenn der Zustand ungültig ist, geben wir einen leeren Zustand zurück
            if (!state.isValid()) {
                return QVariant();
            }

            return state;
        }

    } // namespace Action
} // namespace VivoX
