#pragma once

#include <QObject>
#include <QList>
#include <QHash>
#include <QString>

namespace VivoX::WindowManager {

class Window;
class WindowManager;

/**
 * @brief The WorkspaceManager class manages workspaces in the system.
 * 
 * It is responsible for creating, removing, and switching between workspaces,
 * as well as managing the assignment of windows to workspaces.
 */
class WorkspaceManager : public QObject {
    Q_OBJECT

public:
    explicit WorkspaceManager(QObject *parent = nullptr);
    ~WorkspaceManager();

    /**
     * @brief Initialize the workspace manager
     * @param windowManager The window manager instance
     * @return True if initialization was successful
     */
    bool initialize(WindowManager *windowManager);

    /**
     * @brief Get all workspaces
     * @return List of all workspaces
     */
    QList<Workspace*> getAllWorkspaces() const;

    /**
     * @brief Get the active workspace
     * @return The active workspace, or nullptr if none
     */
    Workspace* getActiveWorkspace() const;

    /**
     * @brief Get a workspace by ID
     * @param id The workspace ID
     * @return The workspace, or nullptr if not found
     */
    Workspace* getWorkspaceById(const QString &id) const;

    /**
     * @brief Get a workspace by index
     * @param index The workspace index (0-based)
     * @return The workspace, or nullptr if not found
     */
    Workspace* getWorkspaceByIndex(int index) const;

    /**
     * @brief Create a new workspace
     * @param name The workspace name
     * @return The created workspace, or nullptr if creation failed
     */
    Workspace* createWorkspace(const QString &name);

    /**
     * @brief Remove a workspace
     * @param workspace The workspace to remove
     * @return True if successful
     */
    bool removeWorkspace(Workspace *workspace);

    /**
     * @brief Activate a workspace
     * @param workspace The workspace to activate
     * @return True if successful
     */
    bool activateWorkspace(Workspace *workspace);

    /**
     * @brief Move a window to a workspace
     * @param window The window to move
     * @param workspace The target workspace
     * @return True if successful
     */
    bool moveWindowToWorkspace(Window *window, Workspace *workspace);

    /**
     * @brief Get the workspace containing a window
     * @param window The window
     * @return The workspace containing the window, or nullptr if not found
     */
    Workspace* getWorkspaceForWindow(Window *window) const;

signals:
    /**
     * @brief Signal emitted when a workspace is added
     * @param workspace The added workspace
     */
    void workspaceAdded(Workspace *workspace);

    /**
     * @brief Signal emitted when a workspace is removed
     * @param workspace The removed workspace
     */
    void workspaceRemoved(Workspace *workspace);

    /**
     * @brief Signal emitted when a workspace is activated
     * @param workspace The activated workspace
     */
    void workspaceActivated(Workspace *workspace);

    /**
     * @brief Signal emitted when a window is moved to a workspace
     * @param window The window
     * @param workspace The workspace
     */
    void windowMovedToWorkspace(Window *window, Workspace *workspace);

private:
    // The window manager
    WindowManager *m_windowManager;
    
    // List of all workspaces
    QList<Workspace*> m_workspaces;
    
    // Map of workspace ID to workspace
    QHash<QString, Workspace*> m_workspacesById;
    
    // Active workspace
    Workspace *m_activeWorkspace;
    
    // Map of window to workspace
    QHash<Window*, Workspace*> m_windowWorkspaceMap;
    
    // Connect signals from a workspace
    void connectWorkspaceSignals(Workspace *workspace);
    
    // Disconnect signals from a workspace
    void disconnectWorkspaceSignals(Workspace *workspace);
    
    // Handle window added to the window manager
    void handleWindowAdded(Window *window);
    
    // Handle window removed from the window manager
    void handleWindowRemoved(Window *window);
};

} // namespace VivoX::WindowManager
