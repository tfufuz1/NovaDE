#pragma once

#include <QObject>
#include <QList>
#include <QHash>
#include <QString>

namespace VivoX::WindowManager {

class Window;
class WindowManager;

/**
 * @brief The TabGroup class represents a group of tabbed windows.
 */
class TabGroup : public QObject {
    Q_OBJECT

public:
    explicit TabGroup(QObject *parent = nullptr);
    ~TabGroup();

    /**
     * @brief Get the group ID
     * @return The group ID
     */
    QString id() const;

    /**
     * @brief Set the group ID
     * @param id The group ID
     */
    void setId(const QString &id);

    /**
     * @brief Get all windows in the group
     * @return List of all windows in the group
     */
    QList<Window*> getAllWindows() const;

    /**
     * @brief Get the active window in the group
     * @return The active window, or nullptr if none
     */
    Window* getActiveWindow() const;

    /**
     * @brief Add a window to the group
     * @param window The window to add
     * @return True if successful
     */
    bool addWindow(Window *window);

    /**
     * @brief Remove a window from the group
     * @param window The window to remove
     * @return True if successful
     */
    bool removeWindow(Window *window);

    /**
     * @brief Activate a window in the group
     * @param window The window to activate
     * @return True if successful
     */
    bool activateWindow(Window *window);

signals:
    /**
     * @brief Signal emitted when a window is added to the group
     * @param window The added window
     */
    void windowAdded(Window *window);

    /**
     * @brief Signal emitted when a window is removed from the group
     * @param window The removed window
     */
    void windowRemoved(Window *window);

    /**
     * @brief Signal emitted when a window is activated in the group
     * @param window The activated window
     */
    void windowActivated(Window *window);

    /**
     * @brief Signal emitted when the group becomes empty
     * @param group The group that became empty
     */
    void groupEmpty(TabGroup *group);

private:
    // Group ID
    QString m_id;
    
    // List of windows in the group
    QList<Window*> m_windows;
    
    // Active window in the group
    Window *m_activeWindow;
};

/**
 * @brief The TabManager class manages tabbed window groups.
 * 
 * It is responsible for creating, removing, and managing tab groups,
 * as well as handling the tabbing and untabbing of windows.
 */
class TabManager : public QObject {
    Q_OBJECT

public:
    explicit TabManager(QObject *parent = nullptr);
    ~TabManager();

    /**
     * @brief Initialize the tab manager
     * @param windowManager The window manager instance
     * @return True if initialization was successful
     */
    bool initialize(WindowManager *windowManager);

    /**
     * @brief Get all tab groups
     * @return List of all tab groups
     */
    QList<TabGroup*> getAllGroups() const;

    /**
     * @brief Get a tab group by ID
     * @param id The group ID
     * @return The group, or nullptr if not found
     */
    TabGroup* getGroupById(const QString &id) const;

    /**
     * @brief Get the tab group containing a window
     * @param window The window
     * @return The group containing the window, or nullptr if not found
     */
    TabGroup* getGroupForWindow(Window *window) const;

    /**
     * @brief Create a new tab group
     * @return The created group, or nullptr if creation failed
     */
    TabGroup* createGroup();

    /**
     * @brief Remove a tab group
     * @param group The group to remove
     * @return True if successful
     */
    bool removeGroup(TabGroup *group);

    /**
     * @brief Tab a window into a group
     * @param window The window to tab
     * @param group The target group
     * @return True if successful
     */
    bool tabWindowIntoGroup(Window *window, TabGroup *group);

    /**
     * @brief Untab a window from its group
     * @param window The window to untab
     * @return True if successful
     */
    bool untabWindow(Window *window);

    /**
     * @brief Tab two windows together
     * @param window1 The first window
     * @param window2 The second window
     * @return The created or modified group, or nullptr if failed
     */
    TabGroup* tabWindows(Window *window1, Window *window2);

signals:
    /**
     * @brief Signal emitted when a tab group is created
     * @param group The created group
     */
    void groupCreated(TabGroup *group);

    /**
     * @brief Signal emitted when a tab group is removed
     * @param group The removed group
     */
    void groupRemoved(TabGroup *group);

    /**
     * @brief Signal emitted when a window is tabbed into a group
     * @param window The tabbed window
     * @param group The target group
     */
    void windowTabbed(Window *window, TabGroup *group);

    /**
     * @brief Signal emitted when a window is untabbed from its group
     * @param window The untabbed window
     * @param group The source group
     */
    void windowUntabbed(Window *window, TabGroup *group);

private:
    // The window manager
    WindowManager *m_windowManager;
    
    // List of all tab groups
    QList<TabGroup*> m_groups;
    
    // Map of group ID to group
    QHash<QString, TabGroup*> m_groupsById;
    
    // Map of window to group
    QHash<Window*, TabGroup*> m_windowGroupMap;
    
    // Connect signals from a group
    void connectGroupSignals(TabGroup *group);
    
    // Disconnect signals from a group
    void disconnectGroupSignals(TabGroup *group);
    
    // Handle window added to the window manager
    void handleWindowAdded(Window *window);
    
    // Handle window removed from the window manager
    void handleWindowRemoved(Window *window);
    
    // Handle group becoming empty
    void handleGroupEmpty(TabGroup *group);
};

} // namespace VivoX::WindowManager
