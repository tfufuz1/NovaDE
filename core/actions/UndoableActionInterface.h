// UndoableActionInterface.h - Vollständige Definition
#ifndef UNDOABLEACTIONINTERFACE_H
#define UNDOABLEACTIONINTERFACE_H

#include "ActionInterface.h"
#include <QVariant>
#include <QList>
#include <QDateTime>

namespace VivoX {
    namespace Action {

        /**
         * @brief UndoableActionInterface erweitert ActionInterface um Undo/Redo-Funktionalität
         *
         * Diese Schnittstelle ermöglicht es Aktionen, ihre Auswirkungen rückgängig zu machen und wiederherzustellen.
         * Abgeleitete Klassen müssen die undoImpl und redoImpl Methoden implementieren sowie captureCurrentState,
         * um den aktuellen Zustand für späteres Undo/Redo zu speichern.
         */
        class UndoableActionInterface : public ActionInterface {
            Q_OBJECT
            Q_PROPERTY(bool canUndo READ canUndo NOTIFY canUndoChanged)
            Q_PROPERTY(bool canRedo READ canRedo NOTIFY canRedoChanged)

        public:
            /**
             * @brief Konstruktor
             * @param parent Parent-QObject
             */
            explicit UndoableActionInterface(QObject* parent = nullptr);

            /**
             * @brief Destruktor
             */
            virtual ~UndoableActionInterface();

            /**
             * @brief Macht die letzte Aktion rückgängig
             * @param parameters Parameter für die Undo-Operation
             * @return Ergebnis der Undo-Operation
             */
            virtual QVariant undo(const QVariantMap& parameters = QVariantMap());

            /**
             * @brief Stellt die letzte rückgängig gemachte Aktion wieder her
             * @param parameters Parameter für die Redo-Operation
             * @return Ergebnis der Redo-Operation
             */
            virtual QVariant redo(const QVariantMap& parameters = QVariantMap());

            /**
             * @brief Prüft, ob Undo möglich ist
             * @return true, wenn Undo möglich ist, sonst false
             */
            bool canUndo() const;

            /**
             * @brief Prüft, ob Redo möglich ist
             * @return true, wenn Redo möglich ist, sonst false
             */
            bool canRedo() const;

            /**
             * @brief Löscht den Undo-Stapel
             */
            void clearUndoStack();

            /**
             * @brief Löscht den Redo-Stapel
             */
            void clearRedoStack();

            /**
             * @brief Überschreibt die execute-Methode von ActionInterface
             * @param parameters Parameter für die Ausführung
             * @return Ergebnis der Ausführung
             */
            virtual QVariant execute(const QVariantMap& parameters) override;

            /**
             * @brief Gibt die Anzahl der Aktionen im Undo-Stapel zurück
             * @return Anzahl der Undo-Aktionen
             */
            int undoStackSize() const { return m_undoStack.size(); }

            /**
             * @brief Gibt die Anzahl der Aktionen im Redo-Stapel zurück
             * @return Anzahl der Redo-Aktionen
             */
            int redoStackSize() const { return m_redoStack.size(); }

        signals:
            /**
             * @brief Signal, das emittiert wird, wenn sich der canUndo-Status ändert
             */
            void canUndoChanged();

            /**
             * @brief Signal, das emittiert wird, wenn sich der canRedo-Status ändert
             */
            void canRedoChanged();

            /**
             * @brief Signal, das emittiert wird, wenn eine Undo-Operation erfolgreich war
             * @param previousState Der vorherige Zustand
             */
            void undoPerformed(const QVariant& previousState);

            /**
             * @brief Signal, das emittiert wird, wenn eine Redo-Operation erfolgreich war
             * @param newState Der neue Zustand
             */
            void redoPerformed(const QVariant& newState);

        protected:
            /**
             * @brief Struktur zur Speicherung von Zuständen für Undo/Redo
             */
            struct UndoState {
                QVariantMap parameters;  ///< Parameter der Aktion
                QVariant previousState;  ///< Vorheriger Zustand
                QDateTime timestamp;     ///< Zeitstempel der Operation
            };

            /**
             * @brief Erfasst den aktuellen Zustand für Undo/Redo
             * @return Zustandsbeschreibung als QVariant
             */
            virtual QVariant captureCurrentState() const;

            /**
             * @brief Implementierung der Undo-Operation
             * @param parameters Parameter der ursprünglichen Aktion
             * @param previousState Gespeicherter Zustand vor der Aktion
             * @return Ergebnis der Undo-Operation
             */
            virtual QVariant undoImpl(const QVariantMap& parameters, const QVariant& previousState);

            /**
             * @brief Implementierung der Redo-Operation
             * @param parameters Parameter der ursprünglichen Aktion
             * @param previousState Gespeicherter Zustand nach der Undo-Operation
             * @return Ergebnis der Redo-Operation
             */
            virtual QVariant redoImpl(const QVariantMap& parameters, const QVariant& previousState);

            /**
             * @brief Validiert und optimiert den zu speichernden Zustand
             * @param state Der zu validierende Zustand
             * @return Der validierte und ggf. optimierte Zustand
             */
            virtual QVariant validateState(const QVariant& state) const;

            QList<UndoState> m_undoStack;  ///< Stapel für Undo-Operationen
            QList<UndoState> m_redoStack;  ///< Stapel für Redo-Operationen

            bool m_lastUndoSuccessful;  ///< Flag, ob die letzte Undo-Operation erfolgreich war
            bool m_lastRedoSuccessful;  ///< Flag, ob die letzte Redo-Operation erfolgreich war

            static const int MAX_STACK_SIZE;  ///< Maximale Größe der Stapel
        };

    } // namespace Action
} // namespace VivoX

#endif // UNDOABLEACTIONINTERFACE_H
