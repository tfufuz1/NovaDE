#include "TabManager.h"
#include "../windows/Window.h"
#include "../windows/WindowManager.h"

#include <QDebug>
#include <QUuid>

namespace VivoX::WindowManager {

//
// TabGroup Implementation
//

TabGroup::TabGroup(QObject *parent)
    : QObject(parent)
    , m_activeWindow(nullptr)
{
    qDebug() << "TabGroup created";
}

TabGroup::~TabGroup()
{
    qDebug() << "TabGroup destroyed:" << m_id;
}

QString TabGroup::id() const
{
    return m_id;
}

void TabGroup::setId(const QString &id)
{
    m_id = id;
}

QList<Window*> TabGroup::getAllWindows() const
{
    return m_windows;
}

Window* TabGroup::getActiveWindow() const
{
    return m_activeWindow;
}

bool TabGroup::addWindow(Window *window)
{
    if (!window) {
        qWarning() << "Cannot add null window to TabGroup";
        return false;
    }
    
    if (m_windows.contains(window)) {
        qWarning() << "Window already added to TabGroup:" << window->id();
        return false;
    }
    
    // Add to list
    m_windows.append(window);
    
    // Emit signal
    emit windowAdded(window);
    
    // If this is the first window, activate it
    if (m_windows.size() == 1 && !m_activeWindow) {
        activateWindow(window);
    }
    
    qDebug() << "Added window to TabGroup:" << window->id();
    
    return true;
}

bool TabGroup::removeWindow(Window *window)
{
    if (!window) {
        qWarning() << "Cannot remove null window from TabGroup";
        return false;
    }
    
    if (!m_windows.contains(window)) {
        qWarning() << "Window not found in TabGroup:" << window->id();
        return false;
    }
    
    // If this is the active window, clear it
    if (m_activeWindow == window) {
        m_activeWindow = nullptr;
        
        // Activate another window if available
        if (!m_windows.isEmpty()) {
            // Find the next window to activate
            int index = m_windows.indexOf(window);
            if (index >= 0 && index < m_windows.size() - 1) {
                // Activate the next window
                activateWindow(m_windows.at(index + 1));
            } else if (!m_windows.isEmpty()) {
                // Activate the first window
                activateWindow(m_windows.first());
            }
        }
    }
    
    // Remove from list
    m_windows.removeOne(window);
    
    // Emit signal
    emit windowRemoved(window);
    
    qDebug() << "Removed window from TabGroup:" << window->id();
    
    // If the group is now empty, emit signal
    if (m_windows.isEmpty()) {
        emit groupEmpty(this);
    }
    
    return true;
}

bool TabGroup::activateWindow(Window *window)
{
    if (!window) {
        qWarning() << "Cannot activate null window";
        return false;
    }
    
    if (!m_windows.contains(window)) {
        qWarning() << "Window not found in TabGroup:" << window->id();
        return false;
    }
    
    // If already active, do nothing
    if (m_activeWindow == window) {
        return true;
    }
    
    // Set new active window
    m_activeWindow = window;
    
    // Show the active window and hide others
    for (Window *w : m_windows) {
        w->setVisible(w == m_activeWindow);
    }
    
    // Emit signal
    emit windowActivated(window);
    
    qDebug() << "Activated window in TabGroup:" << window->id();
    
    return true;
}

//
// TabManager Implementation
//

TabManager::TabManager(QObject *parent)
    : QObject(parent)
    , m_windowManager(nullptr)
{
    qDebug() << "TabManager created";
}

TabManager::~TabManager()
{
    qDebug() << "TabManager destroyed";
}

bool TabManager::initialize(WindowManager *windowManager)
{
    if (!windowManager) {
        qWarning() << "Cannot initialize TabManager with null WindowManager";
        return false;
    }
    
    m_windowManager = windowManager;
    
    // Connect signals from the window manager
    connect(m_windowManager, &WindowManager::windowAdded, this, &TabManager::handleWindowAdded);
    connect(m_windowManager, &WindowManager::windowRemoved, this, &TabManager::handleWindowRemoved);
    
    qDebug() << "TabManager initialized";
    
    return true;
}

QList<TabGroup*> TabManager::getAllGroups() const
{
    return m_groups;
}

TabGroup* TabManager::getGroupById(const QString &id) const
{
    return m_groupsById.value(id, nullptr);
}

TabGroup* TabManager::getGroupForWindow(Window *window) const
{
    if (!window) {
        return nullptr;
    }
    
    return m_windowGroupMap.value(window, nullptr);
}

TabGroup* TabManager::createGroup()
{
    // Create a new group
    TabGroup *group = new TabGroup(this);
    
    // Set group properties
    group->setId(QUuid::createUuid().toString(QUuid::WithoutBraces));
    
    // Add to lists
    m_groups.append(group);
    m_groupsById.insert(group->id(), group);
    
    // Connect signals
    connectGroupSignals(group);
    
    // Emit signal
    emit groupCreated(group);
    
    qDebug() << "Created tab group:" << group->id();
    
    return group;
}

bool TabManager::removeGroup(TabGroup *group)
{
    if (!group) {
        qWarning() << "Cannot remove null tab group";
        return false;
    }
    
    if (!m_groups.contains(group)) {
        qWarning() << "Tab group not found in TabManager:" << group->id();
        return false;
    }
    
    // Untab all windows in the group
    QList<Window*> windows = group->getAllWindows();
    for (Window *window : windows) {
        untabWindow(window);
    }
    
    // Disconnect signals
    disconnectGroupSignals(group);
    
    // Remove from lists
    m_groups.removeOne(group);
    m_groupsById.remove(group->id());
    
    // Emit signal
    emit groupRemoved(group);
    
    // Delete the group
    group->deleteLater();
    
    qDebug() << "Removed tab group:" << group->id();
    
    return true;
}

bool TabManager::tabWindowIntoGroup(Window *window, TabGroup *group)
{
    if (!window) {
        qWarning() << "Cannot tab null window into group";
        return false;
    }
    
    if (!group) {
        qWarning() << "Cannot tab window into null group";
        return false;
    }
    
    if (!m_windowManager->getAllWindows().contains(window)) {
        qWarning() << "Window not found in WindowManager:" << window->id();
        return false;
    }
    
    if (!m_groups.contains(group)) {
        qWarning() << "Tab group not found in TabManager:" << group->id();
        return false;
    }
    
    // If already in the group, do nothing
    if (m_windowGroupMap.value(window) == group) {
        return true;
    }
    
    // If in another group, untab first
    if (m_windowGroupMap.contains(window)) {
        untabWindow(window);
    }
    
    // Add to group
    if (!group->addWindow(window)) {
        return false;
    }
    
    // Update map
    m_windowGroupMap[window] = group;
    
    // Emit signal
    emit windowTabbed(window, group);
    
    qDebug() << "Tabbed window" << window->id() << "into group:" << group->id();
    
    return true;
}

bool TabManager::untabWindow(Window *window)
{
    if (!window) {
        qWarning() << "Cannot untab null window";
        return false;
    }
    
    // Get the group
    TabGroup *group = m_windowGroupMap.value(window, nullptr);
    if (!group) {
        qWarning() << "Window not found in any tab group:" << window->id();
        return false;
    }
    
    // Remove from group
    if (!group->removeWindow(window)) {
        return false;
    }
    
    // Update map
    m_windowGroupMap.remove(window);
    
    // Make window visible
    window->setVisible(true);
    
    // Emit signal
    emit windowUntabbed(window, group);
    
    qDebug() << "Untabbed window" << window->id() << "from group:" << group->id();
    
    return true;
}

TabGroup* TabManager::tabWindows(Window *window1, Window *window2)
{
    if (!window1 || !window2) {
        qWarning() << "Cannot tab null windows";
        return nullptr;
    }
    
    if (window1 == window2) {
        qWarning() << "Cannot tab a window with itself";
        return nullptr;
    }
    
    // Check if windows are in the window manager
    if (!m_windowManager->getAllWindows().contains(window1)) {
        qWarning() << "Window not found in WindowManager:" << window1->id();
        return nullptr;
    }
    
    if (!m_windowManager->getAllWindows().contains(window2)) {
        qWarning() << "Window not found in WindowManager:" << window2->id();
        return nullptr;
    }
    
    // Check if windows are already in groups
    TabGroup *group1 = m_windowGroupMap.value(window1, nullptr);
    TabGroup *group2 = m_windowGroupMap.value(window2, nullptr);
    
    // If both windows are in the same group, do nothing
    if (group1 && group1 == group2) {
        return group1;
    }
    
    // If window1 is in a group, use that group
    if (group1) {
        tabWindowIntoGroup(window2, group1);
        return group1;
    }
    
    // If window2 is in a group, use that group
    if (group2) {
        tabWindowIntoGroup(window1, group2);
        return group2;
    }
    
    // Create a new group
    TabGroup *newGroup = createGroup();
    if (!newGroup) {
        return nullptr;
    }
    
    // Tab both windows into the new group
    tabWindowIntoGroup(window1, newGroup);
    tabWindowIntoGroup(window2, newGroup);
    
    return newGroup;
}

void TabManager::connectGroupSignals(TabGroup *group)
{
    connect(group, &TabGroup::groupEmpty, this, &TabManager::handleGroupEmpty);
}

void TabManager::disconnectGroupSignals(TabGroup *group)
{
    disconnect(group, nullptr, this, nullptr);
}

void TabManager::handleWindowAdded(Window *window)
{
    // Nothing to do here
    Q_UNUSED(window);
}

void TabManager::handleWindowRemoved(Window *window)
{
    // If the window is in a group, untab it
    if (m_windowGroupMap.contains(window)) {
        untabWindow(window);
    }
}

void TabManager::handleGroupEmpty(TabGroup *group)
{
    // Remove the empty group
    removeGroup(group);
}

} // namespace VivoX::WindowManager
