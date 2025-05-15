#pragma once

#include <QObject>
#include <QList>
#include <QRect>

namespace VivoX::WindowManager {

class Window;
class Workspace;

/**
 * @brief The LayoutEngine class manages window layouts within workspaces.
 * 
 * It is responsible for arranging windows according to different layout algorithms
 * and ensuring proper window positioning and sizing.
 */
class LayoutEngine : public QObject {
    Q_OBJECT

public:
    /**
     * @brief Layout types supported by the engine
     */
    enum class LayoutType {
        Free,       ///< Free-form layout (windows can be placed anywhere)
        Tiling,     ///< Tiling layout (windows are arranged in a grid)
        Stacking,   ///< Stacking layout (windows are stacked on top of each other)
        Tabbed,     ///< Tabbed layout (windows are arranged as tabs)
        Floating    ///< Floating layout (windows float above other windows)
    };
    Q_ENUM(LayoutType)

    explicit LayoutEngine(QObject *parent = nullptr);
    ~LayoutEngine();

    /**
     * @brief Initialize the layout engine
     * @return True if initialization was successful
     */
    bool initialize();

    /**
     * @brief Get the current layout type
     * @return The current layout type
     */
    LayoutType layoutType() const;

    /**
     * @brief Set the layout type
     * @param type The layout type to set
     */
    void setLayoutType(LayoutType type);

    /**
     * @brief Get the available screen area for layout
     * @return The available screen area
     */
    QRect availableArea() const;

    /**
     * @brief Set the available screen area for layout
     * @param area The available screen area
     */
    void setAvailableArea(const QRect &area);

    /**
     * @brief Add a window to be managed by the layout engine
     * @param window The window to add
     */
    void addWindow(Window *window);

    /**
     * @brief Remove a window from the layout engine
     * @param window The window to remove
     */
    void removeWindow(Window *window);

    /**
     * @brief Relayout all windows according to the current layout type
     */
    void relayout();

    /**
     * @brief Apply the current layout to a specific workspace
     * @param workspace The workspace to apply the layout to
     */
    void applyToWorkspace(Workspace *workspace);

signals:
    /**
     * @brief Signal emitted when the layout type changes
     * @param type The new layout type
     */
    void layoutTypeChanged(LayoutType type);

    /**
     * @brief Signal emitted when the available area changes
     * @param area The new available area
     */
    void availableAreaChanged(const QRect &area);

    /**
     * @brief Signal emitted when a relayout is performed
     */
    void layoutChanged();

private:
    // Current layout type
    LayoutType m_layoutType;
    
    // Available screen area for layout
    QRect m_availableArea;
    
    // List of windows managed by the layout engine
    QList<Window*> m_windows;
    
    // Apply free layout
    void applyFreeLayout();
    
    // Apply tiling layout
    void applyTilingLayout();
    
    // Apply stacking layout
    void applyStackingLayout();
    
    // Apply tabbed layout
    void applyTabbedLayout();
    
    // Apply floating layout
    void applyFloatingLayout();
    
    // Connect window signals
    void connectWindowSignals(Window *window);
    
    // Disconnect window signals
    void disconnectWindowSignals(Window *window);
};

} // namespace VivoX::WindowManager
