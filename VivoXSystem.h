#pragma once

#include "core/Services/ServiceRegistry.h"
#include "core/Configuration/ConfigManager.h"
#include "core/Logging/Logger.h"
#include "core/Events/EventManager.h"
#include "core/Plugins/PluginManager.h"
#include "core/Actions/ActionManager.h"

#include "compositor/wayland/WaylandCompositor.h"
#include "compositor/wayland/WaylandProtocols.h"
#include "compositor/rendering/RenderEngine.h"
#include "compositor/xwayland/XWaylandIntegration.h"

#include "window_manager/windows/WindowManager.h"
#include "window_manager/workspaces/WorkspaceManager.h"
#include "window_manager/layouts/LayoutEngine.h"

#include "ui/UIManager.h"
#include "ui/qml/theme/ThemeManager.h"
#include "ui/panels/PanelManager.h"
#include "ui/widgets/WidgetManager.h"

#include "input/InputManager.h"
#include "input/shortcuts/ShortcutManager.h"
#include "input/gestures/GestureEngine.h"

#include "system/SystemService.h"
#include "system/applications/ApplicationManager.h"
#include "system/notifications/NotificationManager.h"
#include "system/network/NetworkManager.h"
#include "system/power/PowerManager.h"
#include "system/media/MediaController.h"
#include "system/session/SessionManager.h"

#include <QObject>
#include <QCoreApplication>
#include <QGuiApplication>
#include <QQmlApplicationEngine>

namespace VivoX {

/**
 * @brief The VivoXSystem class is the main system class that integrates all components.
 * 
 * It initializes and manages all the core components of the VivoX desktop environment,
 * ensuring proper dependencies and interactions between different subsystems.
 */
class VivoXSystem : public QObject {
    Q_OBJECT

public:
    explicit VivoXSystem(QObject *parent = nullptr);
    ~VivoXSystem();

    /**
     * @brief Initialize the system
     * @param argc Command line argument count
     * @param argv Command line arguments
     * @return True if initialization was successful
     */
    bool initialize(int argc, char *argv[]);

    /**
     * @brief Run the system
     * @return The application exit code
     */
    int run();

    /**
     * @brief Shutdown the system
     */
    void shutdown();

private:
    // Application instances
    QGuiApplication *m_app;
    QQmlApplicationEngine *m_qmlEngine;

    // Core components
    Core::Logger *m_logger;
    Core::ConfigManager *m_configManager;
    Core::EventManager *m_eventManager;
    Core::PluginManager *m_pluginManager;
    Core::ActionManager *m_actionManager;

    // Compositor components
    Compositor::WaylandCompositor *m_waylandCompositor;
    Compositor::WaylandProtocols *m_waylandProtocols;
    Compositor::RenderEngine *m_renderEngine;
    Compositor::XWaylandIntegration *m_xwaylandIntegration;

    // Window manager components
    WindowManager::WindowManager *m_windowManager;
    WindowManager::WorkspaceManager *m_workspaceManager;
    WindowManager::LayoutEngine *m_layoutEngine;

    // UI components
    UI::UIManager *m_uiManager;
    UI::ThemeManager *m_themeManager;
    UI::PanelManager *m_panelManager;
    UI::WidgetManager *m_widgetManager;

    // Input components
    Input::InputManager *m_inputManager;
    Input::ShortcutManager *m_shortcutManager;
    Input::GestureEngine *m_gestureEngine;

    // System components
    System::SystemService *m_systemService;
    System::ApplicationManager *m_applicationManager;
    System::NotificationManager *m_notificationManager;
    System::NetworkManager *m_networkManager;
    System::PowerManager *m_powerManager;
    System::MediaController *m_mediaController;
    System::SessionManager *m_sessionManager;

    // Initialize core components
    bool initializeCoreComponents();

    // Initialize compositor components
    bool initializeCompositorComponents();

    // Initialize window manager components
    bool initializeWindowManagerComponents();

    // Initialize UI components
    bool initializeUIComponents();

    // Initialize input components
    bool initializeInputComponents();

    // Initialize system components
    bool initializeSystemComponents();

    // Register all services in the service registry
    void registerServices();

    // Connect signals between components
    void connectSignals();
};

} // namespace VivoX
