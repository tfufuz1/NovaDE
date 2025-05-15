#include "StageManager.h"
#include "../windows/Window.h"
#include "../windows/WindowManager.h"

#include <QDebug>
#include <QList>
#include <QRect>
#include <cmath>

namespace VivoX::WindowManager {

StageManager::StageManager(QObject *parent)
    : QObject(parent)
    , m_windowManager(nullptr)
    , m_active(false)
{
    qDebug() << "StageManager created";
}

StageManager::~StageManager()
{
    qDebug() << "StageManager destroyed";
}

bool StageManager::initialize(WindowManager *windowManager)
{
    if (!windowManager) {
        qWarning() << "Cannot initialize StageManager with null WindowManager";
        return false;
    }
    
    m_windowManager = windowManager;
    
    // Connect signals from the window manager
    connect(m_windowManager, &WindowManager::windowAdded, this, &StageManager::handleWindowAdded);
    connect(m_windowManager, &WindowManager::windowRemoved, this, &StageManager::handleWindowRemoved);
    
    qDebug() << "StageManager initialized";
    
    return true;
}

bool StageManager::isActive() const
{
    return m_active;
}

void StageManager::setActive(bool active)
{
    if (m_active != active) {
        m_active = active;
        
        if (m_active) {
            // Save original window positions
            for (Window *window : m_windowManager->getAllWindows()) {
                m_originalWindowPositions[window] = window->geometry();
            }
            
            // Arrange windows in stage mode
            arrangeWindows();
        } else {
            // Restore original window positions
            restoreWindowPositions();
        }
        
        emit activeChanged(m_active);
        
        qDebug() << "Stage mode active state changed to:" << m_active;
    }
}

bool StageManager::toggleActive()
{
    setActive(!m_active);
    return m_active;
}

QRect StageManager::stageArea() const
{
    return m_stageArea;
}

void StageManager::setStageArea(const QRect &area)
{
    if (m_stageArea != area) {
        m_stageArea = area;
        
        // Recalculate window positions if stage mode is active
        if (m_active) {
            arrangeWindows();
        }
        
        emit stageAreaChanged(m_stageArea);
        
        qDebug() << "Stage area changed to:" << m_stageArea;
    }
}

QRect StageManager::getWindowStagePosition(Window *window) const
{
    if (!window) {
        return QRect();
    }
    
    return m_windowStagePositions.value(window, window->geometry());
}

bool StageManager::selectWindow(Window *window)
{
    if (!m_active) {
        qWarning() << "Cannot select window when stage mode is not active";
        return false;
    }
    
    if (!window) {
        qWarning() << "Cannot select null window";
        return false;
    }
    
    if (!m_windowManager->getAllWindows().contains(window)) {
        qWarning() << "Window not found in WindowManager:" << window->id();
        return false;
    }
    
    // Activate the window
    m_windowManager->activateWindow(window);
    
    // Exit stage mode
    setActive(false);
    
    // Emit signal
    emit windowSelected(window);
    
    qDebug() << "Selected window in stage mode:" << window->id();
    
    return true;
}

void StageManager::arrangeWindows()
{
    if (!m_active || m_stageArea.isEmpty()) {
        return;
    }
    
    // Calculate window positions
    calculateWindowPositions();
    
    // Apply positions to windows
    for (auto it = m_windowStagePositions.begin(); it != m_windowStagePositions.end(); ++it) {
        Window *window = it.key();
        QRect geometry = it.value();
        
        window->setGeometry(geometry);
    }
    
    qDebug() << "Arranged windows in stage mode";
}

void StageManager::restoreWindowPositions()
{
    // Restore original window positions
    for (auto it = m_originalWindowPositions.begin(); it != m_originalWindowPositions.end(); ++it) {
        Window *window = it.key();
        QRect geometry = it.value();
        
        window->setGeometry(geometry);
    }
    
    // Clear saved positions
    m_originalWindowPositions.clear();
    
    qDebug() << "Restored window positions after exiting stage mode";
}

void StageManager::handleWindowAdded(Window *window)
{
    if (m_active) {
        // Save original position
        m_originalWindowPositions[window] = window->geometry();
        
        // Recalculate window positions
        calculateWindowPositions();
        
        // Apply position to the new window
        window->setGeometry(m_windowStagePositions.value(window, window->geometry()));
    }
}

void StageManager::handleWindowRemoved(Window *window)
{
    // Remove from maps
    m_windowStagePositions.remove(window);
    m_originalWindowPositions.remove(window);
    
    if (m_active) {
        // Recalculate window positions
        calculateWindowPositions();
    }
}

void StageManager::calculateWindowPositions()
{
    QList<Window*> windows = m_windowManager->getAllWindows();
    int count = windows.size();
    
    if (count == 0 || m_stageArea.isEmpty()) {
        return;
    }
    
    // Clear previous positions
    m_windowStagePositions.clear();
    
    // Calculate grid dimensions based on the number of windows
    int cols = qMax(1, static_cast<int>(std::sqrt(count)));
    int rows = (count + cols - 1) / cols; // Ceiling division
    
    // Calculate cell size with padding
    int padding = 10;
    int cellWidth = (m_stageArea.width() - (cols + 1) * padding) / cols;
    int cellHeight = (m_stageArea.height() - (rows + 1) * padding) / rows;
    
    // Position windows in a grid
    for (int i = 0; i < count; ++i) {
        int row = i / cols;
        int col = i % cols;
        
        QRect geometry(
            m_stageArea.x() + col * (cellWidth + padding) + padding,
            m_stageArea.y() + row * (cellHeight + padding) + padding,
            cellWidth,
            cellHeight
        );
        
        // Store position
        m_windowStagePositions[windows.at(i)] = geometry;
    }
    
    qDebug() << "Calculated window positions for stage mode with" << cols << "columns and" << rows << "rows";
}

} // namespace VivoX::WindowManager
