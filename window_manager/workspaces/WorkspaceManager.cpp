#include "WorkspaceManager.h"
#include "Workspace.h"
#include "../windows/Window.h"
#include "../windows/WindowManager.h"

#include <QDebug>
#include <QUuid>

namespace VivoX::WindowManager {

WorkspaceManager::WorkspaceManager(QObject *parent)
    : QObject(parent)
    , m_windowManager(nullptr)
    , m_activeWorkspace(nullptr)
{
    qDebug() << "WorkspaceManager created";
}

WorkspaceManager::~WorkspaceManager()
{
    qDebug() << "WorkspaceManager destroyed";
}

bool WorkspaceManager::initialize(WindowManager *windowManager)
{
    if (!windowManager) {
        qWarning() << "Cannot initialize WorkspaceManager with null WindowManager";
        return false;
    }
    
    m_windowManager = windowManager;
    
    // Connect signals from the window manager
    connect(m_windowManager, &WindowManager::windowAdded, this, &WorkspaceManager::handleWindowAdded);
    connect(m_windowManager, &WindowManager::windowRemoved, this, &WorkspaceManager::handleWindowRemoved);
    
    // Create the default workspace
    Workspace *defaultWorkspace = createWorkspace("Default");
    if (!defaultWorkspace) {
        qWarning() << "Failed to create default workspace";
        return false;
    }
    
    // Activate the default workspace
    activateWorkspace(defaultWorkspace);
    
    qDebug() << "WorkspaceManager initialized";
    
    return true;
}

QList<Workspace*> WorkspaceManager::getAllWorkspaces() const
{
    return m_workspaces;
}

Workspace* WorkspaceManager::getActiveWorkspace() const
{
    return m_activeWorkspace;
}

Workspace* WorkspaceManager::getWorkspaceById(const QString &id) const
{
    return m_workspacesById.value(id, nullptr);
}

Workspace* WorkspaceManager::getWorkspaceByIndex(int index) const
{
    if (index < 0 || index >= m_workspaces.size()) {
        return nullptr;
    }
    
    return m_workspaces.at(index);
}

Workspace* WorkspaceManager::createWorkspace(const QString &name)
{
    // Create a new workspace
    Workspace *workspace = new Workspace(this);
    
    // Set workspace properties
    workspace->setId(QUuid::createUuid().toString(QUuid::WithoutBraces));
    workspace->setName(name);
    
    // Add to lists
    m_workspaces.append(workspace);
    m_workspacesById.insert(workspace->id(), workspace);
    
    // Connect signals
    connectWorkspaceSignals(workspace);
    
    // Emit signal
    emit workspaceAdded(workspace);
    
    qDebug() << "Created workspace:" << workspace->id() << name;
    
    return workspace;
}

bool WorkspaceManager::removeWorkspace(Workspace *workspace)
{
    if (!workspace) {
        qWarning() << "Cannot remove null workspace";
        return false;
    }
    
    if (!m_workspaces.contains(workspace)) {
        qWarning() << "Workspace not found in WorkspaceManager:" << workspace->id();
        return false;
    }
    
    // Cannot remove the last workspace
    if (m_workspaces.size() <= 1) {
        qWarning() << "Cannot remove the last workspace";
        return false;
    }
    
    // If this is the active workspace, activate another one
    if (m_activeWorkspace == workspace) {
        // Find the next workspace to activate
        int index = m_workspaces.indexOf(workspace);
        if (index >= 0 && index < m_workspaces.size() - 1) {
            // Activate the next workspace
            activateWorkspace(m_workspaces.at(index + 1));
        } else if (index > 0) {
            // Activate the previous workspace
            activateWorkspace(m_workspaces.at(index - 1));
        }
    }
    
    // Move all windows to the active workspace
    QList<Window*> windows;
    for (auto it = m_windowWorkspaceMap.begin(); it != m_windowWorkspaceMap.end(); ++it) {
        if (it.value() == workspace) {
            windows.append(it.key());
        }
    }
    
    for (Window *window : windows) {
        moveWindowToWorkspace(window, m_activeWorkspace);
    }
    
    // Disconnect signals
    disconnectWorkspaceSignals(workspace);
    
    // Remove from lists
    m_workspaces.removeOne(workspace);
    m_workspacesById.remove(workspace->id());
    
    // Emit signal
    emit workspaceRemoved(workspace);
    
    // Delete the workspace
    workspace->deleteLater();
    
    qDebug() << "Removed workspace:" << workspace->id();
    
    return true;
}

bool WorkspaceManager::activateWorkspace(Workspace *workspace)
{
    if (!workspace) {
        qWarning() << "Cannot activate null workspace";
        return false;
    }
    
    if (!m_workspaces.contains(workspace)) {
        qWarning() << "Workspace not found in WorkspaceManager:" << workspace->id();
        return false;
    }
    
    // If already active, do nothing
    if (m_activeWorkspace == workspace) {
        return true;
    }
    
    // Hide windows in the current workspace
    if (m_activeWorkspace) {
        for (auto it = m_windowWorkspaceMap.begin(); it != m_windowWorkspaceMap.end(); ++it) {
            if (it.value() == m_activeWorkspace) {
                it.key()->setVisible(false);
            }
        }
    }
    
    // Set new active workspace
    m_activeWorkspace = workspace;
    
    // Show windows in the new workspace
    for (auto it = m_windowWorkspaceMap.begin(); it != m_windowWorkspaceMap.end(); ++it) {
        if (it.value() == m_activeWorkspace) {
            it.key()->setVisible(true);
        }
    }
    
    // Emit signal
    emit workspaceActivated(workspace);
    
    qDebug() << "Activated workspace:" << workspace->id();
    
    return true;
}

bool WorkspaceManager::moveWindowToWorkspace(Window *window, Workspace *workspace)
{
    if (!window) {
        qWarning() << "Cannot move null window to workspace";
        return false;
    }
    
    if (!workspace) {
        qWarning() << "Cannot move window to null workspace";
        return false;
    }
    
    if (!m_windowManager->getAllWindows().contains(window)) {
        qWarning() << "Window not found in WindowManager:" << window->id();
        return false;
    }
    
    if (!m_workspaces.contains(workspace)) {
        qWarning() << "Workspace not found in WorkspaceManager:" << workspace->id();
        return false;
    }
    
    // Get the current workspace
    Workspace *currentWorkspace = m_windowWorkspaceMap.value(window, nullptr);
    
    // If already in the target workspace, do nothing
    if (currentWorkspace == workspace) {
        return true;
    }
    
    // Update the map
    m_windowWorkspaceMap[window] = workspace;
    
    // Update visibility based on active workspace
    window->setVisible(workspace == m_activeWorkspace);
    
    // Emit signal
    emit windowMovedToWorkspace(window, workspace);
    
    qDebug() << "Moved window" << window->id() << "to workspace:" << workspace->id();
    
    return true;
}

Workspace* WorkspaceManager::getWorkspaceForWindow(Window *window) const
{
    if (!window) {
        return nullptr;
    }
    
    return m_windowWorkspaceMap.value(window, nullptr);
}

void WorkspaceManager::connectWorkspaceSignals(Workspace *workspace)
{
    // No signals to connect for now
    Q_UNUSED(workspace);
}

void WorkspaceManager::disconnectWorkspaceSignals(Workspace *workspace)
{
    disconnect(workspace, nullptr, this, nullptr);
}

void WorkspaceManager::handleWindowAdded(Window *window)
{
    // Add the window to the active workspace
    if (m_activeWorkspace) {
        moveWindowToWorkspace(window, m_activeWorkspace);
    }
}

void WorkspaceManager::handleWindowRemoved(Window *window)
{
    // Remove the window from the workspace map
    m_windowWorkspaceMap.remove(window);
}

} // namespace VivoX::WindowManager
