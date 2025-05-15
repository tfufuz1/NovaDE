#include "LayoutEngine.h"
#include "../windows/Window.h"
#include "../workspaces/Workspace.h"

#include <QDebug>
#include <QList>
#include <QRect>
#include <cmath>

namespace VivoX::WindowManager {

LayoutEngine::LayoutEngine(QObject *parent)
    : QObject(parent)
    , m_layoutType(LayoutType::Free)
{
    qDebug() << "LayoutEngine created";
}

LayoutEngine::~LayoutEngine()
{
    qDebug() << "LayoutEngine destroyed";
}

bool LayoutEngine::initialize()
{
    qDebug() << "LayoutEngine initialized";
    return true;
}

LayoutEngine::LayoutType LayoutEngine::layoutType() const
{
    return m_layoutType;
}

void LayoutEngine::setLayoutType(LayoutType type)
{
    if (m_layoutType != type) {
        m_layoutType = type;
        emit layoutTypeChanged(m_layoutType);
        
        // Apply the new layout
        relayout();
        
        qDebug() << "Layout type changed to:" << static_cast<int>(m_layoutType);
    }
}

QRect LayoutEngine::availableArea() const
{
    return m_availableArea;
}

void LayoutEngine::setAvailableArea(const QRect &area)
{
    if (m_availableArea != area) {
        m_availableArea = area;
        emit availableAreaChanged(m_availableArea);
        
        // Apply the layout with the new area
        relayout();
        
        qDebug() << "Available area changed to:" << m_availableArea;
    }
}

void LayoutEngine::addWindow(Window *window)
{
    if (!window) {
        qWarning() << "Cannot add null window to LayoutEngine";
        return;
    }
    
    if (m_windows.contains(window)) {
        qWarning() << "Window already added to LayoutEngine:" << window->id();
        return;
    }
    
    // Add to list
    m_windows.append(window);
    
    // Connect signals
    connectWindowSignals(window);
    
    // Apply layout
    relayout();
    
    qDebug() << "Added window to LayoutEngine:" << window->id();
}

void LayoutEngine::removeWindow(Window *window)
{
    if (!window) {
        qWarning() << "Cannot remove null window from LayoutEngine";
        return;
    }
    
    if (!m_windows.contains(window)) {
        qWarning() << "Window not found in LayoutEngine:" << window->id();
        return;
    }
    
    // Disconnect signals
    disconnectWindowSignals(window);
    
    // Remove from list
    m_windows.removeOne(window);
    
    // Apply layout
    relayout();
    
    qDebug() << "Removed window from LayoutEngine:" << window->id();
}

void LayoutEngine::relayout()
{
    if (m_windows.isEmpty() || m_availableArea.isEmpty()) {
        return;
    }
    
    // Apply the appropriate layout based on the current type
    switch (m_layoutType) {
        case LayoutType::Free:
            applyFreeLayout();
            break;
        case LayoutType::Tiling:
            applyTilingLayout();
            break;
        case LayoutType::Stacking:
            applyStackingLayout();
            break;
        case LayoutType::Tabbed:
            applyTabbedLayout();
            break;
        case LayoutType::Floating:
            applyFloatingLayout();
            break;
    }
    
    emit layoutChanged();
}

void LayoutEngine::applyToWorkspace(Workspace *workspace)
{
    if (!workspace) {
        qWarning() << "Cannot apply layout to null workspace";
        return;
    }
    
    // Set the available area to the workspace geometry
    setAvailableArea(workspace->geometry());
    
    // Apply the layout
    relayout();
}

void LayoutEngine::applyFreeLayout()
{
    // In free layout, we don't change window positions or sizes
    qDebug() << "Applied free layout";
}

void LayoutEngine::applyTilingLayout()
{
    if (m_windows.isEmpty() || m_availableArea.isEmpty()) {
        return;
    }
    
    // Calculate the grid dimensions based on the number of windows
    int count = m_windows.size();
    int cols = qMax(1, static_cast<int>(std::sqrt(count)));
    int rows = (count + cols - 1) / cols; // Ceiling division
    
    // Calculate cell size
    int cellWidth = m_availableArea.width() / cols;
    int cellHeight = m_availableArea.height() / rows;
    
    // Position windows in a grid
    for (int i = 0; i < count; ++i) {
        int row = i / cols;
        int col = i % cols;
        
        QRect geometry(
            m_availableArea.x() + col * cellWidth,
            m_availableArea.y() + row * cellHeight,
            cellWidth,
            cellHeight
        );
        
        // Apply geometry to window
        m_windows.at(i)->setGeometry(geometry);
    }
    
    qDebug() << "Applied tiling layout with" << cols << "columns and" << rows << "rows";
}

void LayoutEngine::applyStackingLayout()
{
    if (m_windows.isEmpty() || m_availableArea.isEmpty()) {
        return;
    }
    
    // In stacking layout, all windows have the same size and position
    for (Window *window : m_windows) {
        window->setGeometry(m_availableArea);
    }
    
    // Ensure the active window is on top
    // This is handled by the window manager's z-order
    
    qDebug() << "Applied stacking layout";
}

void LayoutEngine::applyTabbedLayout()
{
    if (m_windows.isEmpty() || m_availableArea.isEmpty()) {
        return;
    }
    
    // In tabbed layout, all windows have the same size and position
    // but only the active one is visible
    for (Window *window : m_windows) {
        window->setGeometry(m_availableArea);
        
        // Only the active window is visible
        // This is handled by the window manager's visibility control
    }
    
    qDebug() << "Applied tabbed layout";
}

void LayoutEngine::applyFloatingLayout()
{
    // In floating layout, windows can be placed anywhere
    // We don't change their positions or sizes
    qDebug() << "Applied floating layout";
}

void LayoutEngine::connectWindowSignals(Window *window)
{
    // Connect to window state changes
    connect(window, &Window::stateChanged, this, [this]() {
        relayout();
    });
}

void LayoutEngine::disconnectWindowSignals(Window *window)
{
    disconnect(window, nullptr, this, nullptr);
}

} // namespace VivoX::WindowManager
