#pragma once

#include <QObject>
#include <QHash>
#include <QTimer>
#include <QString>
#include <QKeySequence>

namespace VivoX::Input {

/**
 * @brief The ShortcutManager class handles global keyboard shortcuts and sequences.
 * 
 * It is responsible for detecting globally defined keyboard shortcuts and sequences
 * based on user configuration, and triggering the associated actions via the ActionManager.
 */
class ShortcutManager : public QObject {
    Q_OBJECT

public:
    explicit ShortcutManager(QObject *parent = nullptr);
    ~ShortcutManager();

    /**
     * @brief Initialize the shortcut manager
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Handle a key event from the InputManager
     * @param keyCode The key code
     * @param pressed Whether the key is pressed or released
     * @param modifiers The active keyboard modifiers
     * @return True if the event was handled, false otherwise
     */
    bool handleKeyEvent(quint32 keyCode, bool pressed, quint32 modifiers);

    /**
     * @brief Register a shortcut with an action
     * @param shortcut The keyboard shortcut
     * @param actionId The ID of the action to trigger
     * @return True if registration was successful
     */
    bool registerShortcut(const QKeySequence &shortcut, const QString &actionId);

    /**
     * @brief Unregister a shortcut
     * @param shortcut The keyboard shortcut to unregister
     * @return True if unregistration was successful
     */
    bool unregisterShortcut(const QKeySequence &shortcut);

    /**
     * @brief Load shortcuts from configuration
     * @return True if loading was successful
     */
    bool loadShortcuts();

    /**
     * @brief Save shortcuts to configuration
     * @return True if saving was successful
     */
    bool saveShortcuts();

signals:
    /**
     * @brief Signal emitted when a shortcut conflict is detected
     * @param shortcut The conflicting shortcut
     * @param existingAction The existing action
     * @param newAction The new action
     */
    void shortcutConflict(const QKeySequence &shortcut, const QString &existingAction, const QString &newAction);

private:
    // Shortcut sequence state
    struct SequenceState {
        QList<int> keys;
        int currentStep;
        bool active;
    };

    // Map of shortcuts to action IDs
    QHash<QKeySequence, QString> m_shortcuts;
    
    // Current sequence state
    SequenceState m_sequenceState;
    
    // Timer for sequence timeout
    QTimer m_sequenceTimer;
    
    // Sequence timeout in milliseconds
    int m_sequenceTimeout;
    
    // Reset the sequence state
    void resetSequenceState();
    
    // Check for shortcut conflicts
    bool checkConflicts(const QKeySequence &shortcut, const QString &actionId);
    
    // Convert key code and modifiers to Qt key
    int convertToQtKey(quint32 keyCode, quint32 modifiers);
};

} // namespace VivoX::Input
