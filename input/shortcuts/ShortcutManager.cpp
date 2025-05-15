#include "ShortcutManager.h"

#include <QDebug>
#include <QKeySequence>
#include <QJsonDocument>
#include <QJsonObject>
#include <QJsonArray>
#include <QFile>

namespace VivoX::Input {

ShortcutManager::ShortcutManager(QObject *parent)
    : QObject(parent)
    , m_sequenceTimeout(1000) // 1 second timeout for key sequences
{
    qDebug() << "ShortcutManager created";
    
    // Initialize sequence state
    resetSequenceState();
    
    // Connect sequence timer
    connect(&m_sequenceTimer, &QTimer::timeout, this, &ShortcutManager::resetSequenceState);
}

ShortcutManager::~ShortcutManager()
{
    qDebug() << "ShortcutManager destroyed";
}

bool ShortcutManager::initialize()
{
    // Load shortcuts from configuration
    loadShortcuts();
    
    qDebug() << "ShortcutManager initialized";
    return true;
}

bool ShortcutManager::handleKeyEvent(quint32 keyCode, bool pressed, quint32 modifiers)
{
    // Only handle key press events for shortcuts
    if (!pressed) {
        return false;
    }
    
    // Convert to Qt key
    int qtKey = convertToQtKey(keyCode, modifiers);
    
    // Add to current sequence
    if (pressed) {
        m_sequenceState.keys.append(qtKey);
        m_sequenceState.active = true;
        
        // Restart sequence timer
        m_sequenceTimer.start(m_sequenceTimeout);
    }
    
    // Check if this matches any registered shortcuts
    for (auto it = m_shortcuts.begin(); it != m_shortcuts.end(); ++it) {
        QKeySequence shortcut = it.key();
        QString actionId = it.value();
        
        // Check if the current sequence matches this shortcut
        if (shortcut.count() == m_sequenceState.keys.size()) {
            bool match = true;
            for (int i = 0; i < shortcut.count(); ++i) {
                if (shortcut[i] != m_sequenceState.keys[i]) {
                    match = false;
                    break;
                }
            }
            
            if (match) {
                qDebug() << "Shortcut triggered:" << shortcut.toString() << "Action:" << actionId;
                
                // Reset sequence state
                resetSequenceState();
                
                // TODO: Trigger action via ActionManager
                // For now, just return true to indicate the event was handled
                return true;
            }
        }
    }
    
    // If we have a partial match for a multi-key sequence, return true to indicate handling
    for (auto it = m_shortcuts.begin(); it != m_shortcuts.end(); ++it) {
        QKeySequence shortcut = it.key();
        
        if (shortcut.count() > m_sequenceState.keys.size()) {
            bool partialMatch = true;
            for (int i = 0; i < m_sequenceState.keys.size(); ++i) {
                if (shortcut[i] != m_sequenceState.keys[i]) {
                    partialMatch = false;
                    break;
                }
            }
            
            if (partialMatch) {
                return true; // Partial match, continue collecting keys
            }
        }
    }
    
    // If no match and no partial match, reset sequence state
    resetSequenceState();
    
    return false; // Not handled
}

bool ShortcutManager::registerShortcut(const QKeySequence &shortcut, const QString &actionId)
{
    if (shortcut.isEmpty()) {
        qWarning() << "Cannot register empty shortcut";
        return false;
    }
    
    if (actionId.isEmpty()) {
        qWarning() << "Cannot register shortcut with empty action ID";
        return false;
    }
    
    // Check for conflicts
    if (!checkConflicts(shortcut, actionId)) {
        return false;
    }
    
    // Register the shortcut
    m_shortcuts[shortcut] = actionId;
    
    qDebug() << "Registered shortcut:" << shortcut.toString() << "Action:" << actionId;
    
    return true;
}

bool ShortcutManager::unregisterShortcut(const QKeySequence &shortcut)
{
    if (!m_shortcuts.contains(shortcut)) {
        qWarning() << "Shortcut not found:" << shortcut.toString();
        return false;
    }
    
    m_shortcuts.remove(shortcut);
    
    qDebug() << "Unregistered shortcut:" << shortcut.toString();
    
    return true;
}

bool ShortcutManager::loadShortcuts()
{
    // Clear existing shortcuts
    m_shortcuts.clear();
    
    // Load from configuration file
    QFile file(":/config/shortcuts.json");
    
    if (!file.open(QIODevice::ReadOnly)) {
        qWarning() << "Failed to open shortcuts configuration file";
        
        // Load default shortcuts
        registerShortcut(QKeySequence("Alt+F1"), "app.launcher.show");
        registerShortcut(QKeySequence("Alt+Tab"), "window.switcher.next");
        registerShortcut(QKeySequence("Alt+Shift+Tab"), "window.switcher.previous");
        registerShortcut(QKeySequence("Super+D"), "desktop.show");
        registerShortcut(QKeySequence("Super+E"), "app.filemanager.show");
        registerShortcut(QKeySequence("Super+L"), "system.lock");
        registerShortcut(QKeySequence("Ctrl+Alt+T"), "app.terminal.show");
        registerShortcut(QKeySequence("Ctrl+Alt+Delete"), "system.logout");
        
        return true;
    }
    
    QByteArray data = file.readAll();
    file.close();
    
    QJsonDocument doc = QJsonDocument::fromJson(data);
    
    if (doc.isNull() || !doc.isObject()) {
        qWarning() << "Invalid shortcuts configuration format";
        return false;
    }
    
    QJsonObject obj = doc.object();
    QJsonArray shortcuts = obj["shortcuts"].toArray();
    
    for (int i = 0; i < shortcuts.size(); ++i) {
        QJsonObject shortcutObj = shortcuts[i].toObject();
        QString shortcutStr = shortcutObj["shortcut"].toString();
        QString actionId = shortcutObj["action"].toString();
        
        if (!shortcutStr.isEmpty() && !actionId.isEmpty()) {
            QKeySequence shortcut(shortcutStr);
            registerShortcut(shortcut, actionId);
        }
    }
    
    qDebug() << "Loaded" << m_shortcuts.size() << "shortcuts from configuration";
    
    return true;
}

bool ShortcutManager::saveShortcuts()
{
    QJsonObject obj;
    QJsonArray shortcuts;
    
    for (auto it = m_shortcuts.begin(); it != m_shortcuts.end(); ++it) {
        QJsonObject shortcutObj;
        shortcutObj["shortcut"] = it.key().toString();
        shortcutObj["action"] = it.value();
        shortcuts.append(shortcutObj);
    }
    
    obj["shortcuts"] = shortcuts;
    
    QJsonDocument doc(obj);
    
    QFile file(":/config/shortcuts.json");
    
    if (!file.open(QIODevice::WriteOnly)) {
        qWarning() << "Failed to open shortcuts configuration file for writing";
        return false;
    }
    
    file.write(doc.toJson());
    file.close();
    
    qDebug() << "Saved" << m_shortcuts.size() << "shortcuts to configuration";
    
    return true;
}

void ShortcutManager::resetSequenceState()
{
    m_sequenceState.keys.clear();
    m_sequenceState.currentStep = 0;
    m_sequenceState.active = false;
    
    m_sequenceTimer.stop();
}

bool ShortcutManager::checkConflicts(const QKeySequence &shortcut, const QString &actionId)
{
    if (m_shortcuts.contains(shortcut)) {
        QString existingAction = m_shortcuts[shortcut];
        
        if (existingAction != actionId) {
            qWarning() << "Shortcut conflict:" << shortcut.toString()
                      << "already assigned to" << existingAction;
            
            emit shortcutConflict(shortcut, existingAction, actionId);
            
            return false;
        }
    }
    
    return true;
}

int ShortcutManager::convertToQtKey(quint32 keyCode, quint32 modifiers)
{
    // This is a simplified conversion that assumes keyCode is already a Qt key code
    // In a real implementation, this would convert from the Wayland/Linux key code to Qt key code
    
    int qtKey = keyCode;
    
    // Add modifiers
    if (modifiers & Qt::ShiftModifier) {
        qtKey |= Qt::ShiftModifier;
    }
    if (modifiers & Qt::ControlModifier) {
        qtKey |= Qt::ControlModifier;
    }
    if (modifiers & Qt::AltModifier) {
        qtKey |= Qt::AltModifier;
    }
    if (modifiers & Qt::MetaModifier) {
        qtKey |= Qt::MetaModifier;
    }
    
    return qtKey;
}

} // namespace VivoX::Input
