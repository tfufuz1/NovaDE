#pragma once

#include <QObject>
#include <QList>
#include <QHash>
#include <QString>
#include <QRect>

namespace VivoX::WindowManager {

class Window;
class WindowManager;

/**
 * @brief The StageManager class manages window stages for overview mode.
 * 
 * It is responsible for arranging windows in an overview mode similar to
 * macOS Mission Control or GNOME Activities, allowing users to see all
 * windows at once and select one to focus.
 */
class StageManager : public QObject {
    Q_OBJECT
    Q_PROPERTY(bool active READ isActive WRITE setActive NOTIFY activeChanged)

public:
    explicit StageManager(QObject *parent = nullptr);
    ~StageManager();

    /**
     * @brief Initialize the stage manager
     * @param windowManager The window manager instance
     * @return True if initialization was successful
     */
    bool initialize(WindowManager *windowManager);

    /**
     * @brief Check if stage mode is active
     * @return True if stage mode is active
     */
    bool isActive() const;

    /**
     * @brief Set stage mode active state
     * @param active The active state
     */
    void setActive(bool active);

    /**
     * @brief Toggle stage mode
     * @return The new active state
     */
    bool toggleActive();

    /**
     * @brief Get the stage area
     * @return The stage area
     */
    QRect stageArea() const;

    /**
     * @brief Set the stage area
     * @param area The stage area
     */
    void setStageArea(const QRect &area);

    /**
     * @brief Get the position of a window in stage mode
     * @param window The window
     * @return The window position in stage mode
     */
    QRect getWindowStagePosition(Window *window) const;

    /**
     * @brief Select a window in stage mode
     * @param window The window to select
     * @return True if successful
     */
    bool selectWindow(Window *window);

signals:
    /**
     * @brief Signal emitted when stage mode active state changes
     * @param active The new active state
     */
    void activeChanged(bool active);

    /**
     * @brief Signal emitted when stage area changes
     * @param area The new stage area
     */
    void stageAreaChanged(const QRect &area);

    /**
     * @brief Signal emitted when a window is selected in stage mode
     * @param window The selected window
     */
    void windowSelected(Window *window);

private:
    // The window manager
    WindowManager *m_windowManager;
    
    // Whether stage mode is active
    bool m_active;
    
    // The stage area
    QRect m_stageArea;
    
    // Map of window to stage position
    QHash<Window*, QRect> m_windowStagePositions;
    
    // Original window positions before entering stage mode
    QHash<Window*, QRect> m_originalWindowPositions;
    
    // Arrange windows in stage mode
    void arrangeWindows();
    
    // Restore window positions after exiting stage mode
    void restoreWindowPositions();
    
    // Handle window added to the window manager
    void handleWindowAdded(Window *window);
    
    // Handle window removed from the window manager
    void handleWindowRemoved(Window *window);
    
    // Calculate window positions in stage mode
    void calculateWindowPositions();
};

} // namespace VivoX::WindowManager
