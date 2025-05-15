#include "VivoXSystem.h"

#include <QDebug>
#include <QQuickWindow>
#include <QQuickStyle>

namespace VivoX {

VivoXSystem::VivoXSystem(QObject *parent)
    : QObject(parent)
    , m_app(nullptr)
    , m_qmlEngine(nullptr)
    , m_logger(nullptr)
    , m_configManager(nullptr)
    , m_eventManager(nullptr)
    , m_pluginManager(nullptr)
    , m_actionManager(nullptr)
    , m_waylandCompositor(nullptr)
    , m_waylandProtocols(nullptr)
    , m_renderEngine(nullptr)
    , m_xwaylandIntegration(nullptr)
    , m_windowManager(nullptr)
    , m_workspaceManager(nullptr)
    , m_layoutEngine(nullptr)
    , m_uiManager(nullptr)
    , m_themeManager(nullptr)
    , m_panelManager(nullptr)
    , m_widgetManager(nullptr)
    , m_inputManager(nullptr)
    , m_shortcutManager(nullptr)
    , m_gestureEngine(nullptr)
    , m_systemService(nullptr)
    , m_applicationManager(nullptr)
    , m_notificationManager(nullptr)
    , m_networkManager(nullptr)
    , m_powerManager(nullptr)
    , m_mediaController(nullptr)
    , m_sessionManager(nullptr)
{
    qDebug() << "VivoXSystem created";
}

VivoXSystem::~VivoXSystem()
{
    shutdown();
    qDebug() << "VivoXSystem destroyed";
}

bool VivoXSystem::initialize(int argc, char *argv[])
{
    qDebug() << "Initializing VivoXSystem...";

    // Create application instance
    m_app = new QGuiApplication(argc, argv);
    m_app->setApplicationName("VivoX");
    m_app->setApplicationVersion("1.0.0");
    m_app->setOrganizationName("VivoX");
    m_app->setOrganizationDomain("vivox.org");

    // Set up QML application engine
    m_qmlEngine = new QQmlApplicationEngine();

    // Set Qt Quick style
    QQuickStyle::setStyle("Material");

    // Initialize components in the correct order
    if (!initializeCoreComponents()) {
        qCritical() << "Failed to initialize core components";
        return false;
    }

    if (!initializeCompositorComponents()) {
        qCritical() << "Failed to initialize compositor components";
        return false;
    }

    if (!initializeWindowManagerComponents()) {
        qCritical() << "Failed to initialize window manager components";
        return false;
    }

    if (!initializeInputComponents()) {
        qCritical() << "Failed to initialize input components";
        return false;
    }

    if (!initializeSystemComponents()) {
        qCritical() << "Failed to initialize system components";
        return false;
    }

    if (!initializeUIComponents()) {
        qCritical() << "Failed to initialize UI components";
        return false;
    }

    // Register all services in the service registry
    registerServices();

    // Connect signals between components
    connectSignals();

    qDebug() << "VivoXSystem initialized successfully";
    return true;
}

int VivoXSystem::run()
{
    qDebug() << "Running VivoXSystem...";

    // Load main QML file
    m_qmlEngine->load(QUrl("qrc:/qml/main.qml"));

    // Run the application
    return m_app->exec();
}

void VivoXSystem::shutdown()
{
    qDebug() << "Shutting down VivoXSystem...";

    // Clean up in reverse order of initialization

    // UI components
    delete m_widgetManager;
    m_widgetManager = nullptr;

    delete m_panelManager;
    m_panelManager = nullptr;

    delete m_themeManager;
    m_themeManager = nullptr;

    delete m_uiManager;
    m_uiManager = nullptr;

    // System components
    delete m_sessionManager;
    m_sessionManager = nullptr;

    delete m_mediaController;
    m_mediaController = nullptr;

    delete m_powerManager;
    m_powerManager = nullptr;

    delete m_networkManager;
    m_networkManager = nullptr;

    delete m_notificationManager;
    m_notificationManager = nullptr;

    delete m_applicationManager;
    m_applicationManager = nullptr;

    delete m_systemService;
    m_systemService = nullptr;

    // Input components
    delete m_gestureEngine;
    m_gestureEngine = nullptr;

    delete m_shortcutManager;
    m_shortcutManager = nullptr;

    delete m_inputManager;
    m_inputManager = nullptr;

    // Window manager components
    delete m_layoutEngine;
    m_layoutEngine = nullptr;

    delete m_workspaceManager;
    m_workspaceManager = nullptr;

    delete m_windowManager;
    m_windowManager = nullptr;

    // Compositor components
    delete m_xwaylandIntegration;
    m_xwaylandIntegration = nullptr;

    delete m_waylandProtocols;
    m_waylandProtocols = nullptr;

    delete m_renderEngine;
    m_renderEngine = nullptr;

    delete m_waylandCompositor;
    m_waylandCompositor = nullptr;

    // Core components
    delete m_actionManager;
    m_actionManager = nullptr;

    delete m_pluginManager;
    m_pluginManager = nullptr;

    delete m_eventManager;
    m_eventManager = nullptr;

    delete m_configManager;
    m_configManager = nullptr;

    delete m_logger;
    m_logger = nullptr;

    // QML engine
    delete m_qmlEngine;
    m_qmlEngine = nullptr;

    // Application
    delete m_app;
    m_app = nullptr;

    qDebug() << "VivoXSystem shutdown complete";
}

bool VivoXSystem::initializeCoreComponents()
{
    qDebug() << "Initializing core components...";

    // Initialize logger
    m_logger = new Core::Logger(this);
    if (!m_logger->initialize()) {
        qCritical() << "Failed to initialize logger";
        return false;
    }

    // Initialize config manager
    m_configManager = new Core::ConfigManager(this);
    if (!m_configManager->initialize()) {
        qCritical() << "Failed to initialize config manager";
        return false;
    }

    // Initialize event manager
    m_eventManager = new Core::EventManager(this);
    if (!m_eventManager->initialize()) {
        qCritical() << "Failed to initialize event manager";
        return false;
    }

    // Initialize plugin manager
    m_pluginManager = new Core::PluginManager(this);
    if (!m_pluginManager->initialize()) {
        qCritical() << "Failed to initialize plugin manager";
        return false;
    }

    // Initialize action manager
    m_actionManager = new Core::ActionManager(this);
    if (!m_actionManager->initialize()) {
        qCritical() << "Failed to initialize action manager";
        return false;
    }

    qDebug() << "Core components initialized successfully";
    return true;
}

bool VivoXSystem::initializeCompositorComponents()
{
    qDebug() << "Initializing compositor components...";

    // Initialize render engine
    m_renderEngine = new Compositor::RenderEngine(this);
    if (!m_renderEngine->initialize()) {
        qCritical() << "Failed to initialize render engine";
        return false;
    }

    // Initialize Wayland compositor
    m_waylandCompositor = new Compositor::WaylandCompositor(this);
    if (!m_waylandCompositor->initialize(m_renderEngine)) {
        qCritical() << "Failed to initialize Wayland compositor";
        return false;
    }

    // Initialize Wayland protocols
    m_waylandProtocols = new Compositor::WaylandProtocols(m_waylandCompositor->compositor(), this);
    if (!m_waylandProtocols->initialize()) {
        qCritical() << "Failed to initialize Wayland protocols";
        return false;
    }

    // Initialize XWayland integration
    m_xwaylandIntegration = new Compositor::XWaylandIntegration(m_waylandCompositor, this);
    if (!m_xwaylandIntegration->initialize()) {
        qCritical() << "Failed to initialize XWayland integration";
        return false;
    }

    qDebug() << "Compositor components initialized successfully";
    return true;
}

bool VivoXSystem::initializeWindowManagerComponents()
{
    qDebug() << "Initializing window manager components...";

    // Initialize window manager
    m_windowManager = new WindowManager::WindowManager(this);
    if (!m_windowManager->initialize()) {
        qCritical() << "Failed to initialize window manager";
        return false;
    }

    // Initialize layout engine
    m_layoutEngine = new WindowManager::LayoutEngine(this);
    if (!m_layoutEngine->initialize()) {
        qCritical() << "Failed to initialize layout engine";
        return false;
    }

    // Set layout engine in window manager
    m_windowManager->setLayoutEngine(m_layoutEngine);

    // Initialize workspace manager
    m_workspaceManager = new WindowManager::WorkspaceManager(this);
    if (!m_workspaceManager->initialize(m_windowManager)) {
        qCritical() << "Failed to initialize workspace manager";
        return false;
    }

    qDebug() << "Window manager components initialized successfully";
    return true;
}

bool VivoXSystem::initializeUIComponents()
{
    qDebug() << "Initializing UI components...";

    // Initialize UI manager
    m_uiManager = new UI::UIManager(this);
    if (!m_uiManager->initialize()) {
        qCritical() << "Failed to initialize UI manager";
        return false;
    }

    // Initialize theme manager
    m_themeManager = new UI::ThemeManager(this);
    if (!m_themeManager->initialize()) {
        qCritical() << "Failed to initialize theme manager";
        return false;
    }

    // Initialize panel manager
    m_panelManager = new UI::PanelManager(this);
    if (!m_panelManager->initialize()) {
        qCritical() << "Failed to initialize panel manager";
        return false;
    }

    // Initialize widget manager
    m_widgetManager = new UI::WidgetManager(this);
    if (!m_widgetManager->initialize()) {
        qCritical() << "Failed to initialize widget manager";
        return false;
    }

    // Register UI components with QML engine
    m_uiManager->registerContextProperty("themeManager", m_themeManager);
    m_uiManager->registerContextProperty("panelManager", m_panelManager);
    m_uiManager->registerContextProperty("widgetManager", m_widgetManager);

    qDebug() << "UI components initialized successfully";
    return true;
}

bool VivoXSystem::initializeInputComponents()
{
    qDebug() << "Initializing input components...";

    // Initialize input manager
    m_inputManager = new Input::InputManager(this);
    if (!m_inputManager->initialize()) {
        qCritical() << "Failed to initialize input manager";
        return false;
    }

    // Initialize shortcut manager
    m_shortcutManager = new Input::ShortcutManager(this);
    if (!m_shortcutManager->initialize()) {
        qCritical() << "Failed to initialize shortcut manager";
        return false;
    }

    // Initialize gesture engine
    m_gestureEngine = new Input::GestureEngine(this);
    if (!m_gestureEngine->initialize()) {
        qCritical() << "Failed to initialize gesture engine";
        return false;
    }

    qDebug() << "Input components initialized successfully";
    return true;
}

bool VivoXSystem::initializeSystemComponents()
{
    qDebug() << "Initializing system components...";

    // Initialize system service
    m_systemService = new System::SystemService(this);
    if (!m_systemService->initialize()) {
        qCritical() << "Failed to initialize system service";
        return false;
    }

    // Initialize application manager
    m_applicationManager = new System::ApplicationManager(this);
    if (!m_applicationManager->initialize()) {
        qCritical() << "Failed to initialize application manager";
        return false;
    }

    // Initialize notification manager
    m_notificationManager = new System::NotificationManager(this);
    if (!m_notificationManager->initialize()) {
        qCritical() << "Failed to initialize notification manager";
        return false;
    }

    // Initialize network manager
    m_networkManager = new System::NetworkManager(this);
    if (!m_networkManager->initialize()) {
        qCritical() << "Failed to initialize network manager";
        return false;
    }

    // Initialize power manager
    m_powerManager = new System::PowerManager(this);
    if (!m_powerManager->initialize()) {
        qCritical() << "Failed to initialize power manager";
        return false;
    }

    // Initialize media controller
    m_mediaController = new System::MediaController(this);
    if (!m_mediaController->initialize()) {
        qCritical() << "Failed to initialize media controller";
        return false;
    }

    // Initialize session manager
    m_sessionManager = new System::SessionManager(this);
    if (!m_sessionManager->initialize()) {
        qCritical() << "Failed to initialize session manager";
        return false;
    }

    qDebug() << "System components initialized successfully";
    return true;
}

void VivoXSystem::registerServices()
{
    qDebug() << "Registering services in service registry...";

    ServiceRegistry *registry = ServiceRegistry::instance();

    // Register core services
    registry->registerService(m_logger);
    registry->registerService(m_configManager);
    registry->registerService(m_eventManager);
    registry->registerService(m_pluginManager);
    registry->registerService(m_actionManager);

    // Register compositor services
    registry->registerService(m_waylandCompositor);
    registry->registerService(m_waylandProtocols);
    registry->registerService(m_renderEngine);
    registry->registerService(m_xwaylandIntegration);

    // Register window manager services
    registry->registerService(m_windowManager);
    registry->registerService(m_workspaceManager);
    registry->registerService(m_layoutEngine);

    // Register UI services
    registry->registerService(m_uiManager);
    registry->registerService(m_themeManager);
    registry->registerService(m_panelManager);
    registry->registerService(m_widgetManager);

    // Register input services
    registry->registerService(m_inputManager);
    registry->registerService(m_shortcutManager);
    registry->registerService(m_gestureEngine);

    // Register system services
    registry->registerService(m_systemService);
    registry->registerService(m_applicationManager);
    registry->registerService(m_notificationManager);
    registry->registerService(m_networkManager);
    registry->registerService(m_powerManager);
    registry->registerService(m_mediaController);
    registry->registerService(m_sessionManager);

    qDebug() << "Services registered successfully";
}

void VivoXSystem::connectSignals()
{
    qDebug() << "Connecting signals between components...";

    // Connect window manager signals to compositor
    connect(m_waylandCompositor, &Compositor::WaylandCompositor::xdgToplevelCreated,
            m_windowManager, [this](QWaylandXdgToplevel *toplevel, QWaylandSurface *surface) {
                // Create a new window for this toplevel
                WindowManager::Window *window = new WindowManager::Window(toplevel, surface);
                m_windowManager->addWindow(window);
            });

    // Connect input manager to window manager
    connect(m_inputManager, &Input::InputManager::keyPressed,
            m_shortcutManager, &Input::ShortcutManager::handleKeyPress);

    connect(m_inputManager, &Input::InputManager::pointerMoved,
            m_gestureEngine, &Input::GestureEngine::handlePointerMotion);

    // Connect window manager to UI
    connect(m_windowManager, &WindowManager::WindowManager::windowAdded,
            m_uiManager, [this](WindowManager::Window *window) {
                // Update UI when a window is added
                m_uiManager->addWindowToUI(window);
            });

    connect(m_windowManager, &WindowManager::WindowManager::windowRemoved,
            m_uiManager, [this](WindowManager::Window *window) {
                // Update UI when a window is removed
                m_uiManager->removeWindowFromUI(window);
            });

    connect(m_windowManager, &WindowManager::WindowManager::windowActivated,
            m_uiManager, [this](WindowManager::Window *window) {
                // Update UI when a window is activated
                m_uiManager->activateWindowInUI(window);
            });

    // Connect workspace manager to UI
    connect(m_workspaceManager, &WindowManager::WorkspaceManager::workspaceAdded,
            m_uiManager, [this](WindowManager::Workspace *workspace) {
                // Update UI when a workspace is added
                m_uiManager->addWorkspaceToUI(workspace);
            });

    connect(m_workspaceManager, &WindowManager::WorkspaceManager::workspaceRemoved,
            m_uiManager, [this](WindowManager::Workspace *workspace) {
                // Update UI when a workspace is removed
                m_uiManager->removeWorkspaceFromUI(workspace);
            });

    connect(m_workspaceManager, &WindowManager::WorkspaceManager::workspaceActivated,
            m_uiManager, [this](WindowManager::Workspace *workspace) {
                // Update UI when a workspace is activated
                m_uiManager->activateWorkspaceInUI(workspace);
            });

    // Connect system services to UI
    connect(m_notificationManager, &System::NotificationManager::notificationCreated,
            m_uiManager, [this](const System::NotificationInfo &info) {
                // Display notification in UI
                m_uiManager->showNotification(info);
            });

    connect(m_notificationManager, &System::NotificationManager::notificationClosed,
            m_uiManager, [this](uint32_t id) {
                // Remove notification from UI
                m_uiManager->closeNotification(id);
            });

    // Connect power manager to system
    connect(m_powerManager, &System::PowerManager::powerStateChanged,
            this, [this](System::PowerManager::PowerState state) {
                // Handle power state changes
                switch (state) {
                    case System::PowerManager::OnAC:
                        // Switch to performance mode
                        m_uiManager->updatePowerIndicator("ac");
                        break;
                    case System::PowerManager::OnBattery:
                        // Switch to balanced mode
                        m_uiManager->updatePowerIndicator("battery");
                        break;
                    case System::PowerManager::LowBattery:
                        // Show low battery notification
                        {
                            System::NotificationInfo notification;
                            notification.id = 0; // Auto-assign ID
                            notification.appName = "VivoX";
                            notification.summary = "Low Battery";
                            notification.body = "Battery level is low. Please connect to a power source.";
                            notification.icon = "battery-low";
                            notification.timeout = 10000; // 10 seconds
                            m_notificationManager->showNotification(notification);
                            m_uiManager->updatePowerIndicator("battery-low");
                        }
                        break;
                    case System::PowerManager::CriticalBattery:
                        // Show critical battery notification and prepare for hibernation
                        {
                            System::NotificationInfo notification;
                            notification.id = 0; // Auto-assign ID
                            notification.appName = "VivoX";
                            notification.summary = "Critical Battery";
                            notification.body = "Battery level is critical. The system will hibernate soon.";
                            notification.icon = "battery-critical";
                            notification.timeout = 0; // No timeout
                            m_notificationManager->showNotification(notification);
                            m_uiManager->updatePowerIndicator("battery-critical");
                        }
                        break;
                }
            });

    connect(m_powerManager, &System::PowerManager::batteryLevelChanged,
            m_uiManager, [this](int level) {
                // Update battery level indicator in UI
                m_uiManager->updateBatteryLevel(level);
            });

    // Connect media controller to UI
    connect(m_mediaController, &System::MediaController::mediaInfoChanged,
            m_uiManager, [this](const System::MediaInfo &info) {
                // Update media controls in UI
                m_uiManager->updateMediaInfo(info);
            });

    // Connect network manager to UI
    connect(m_networkManager, &System::NetworkManager::networkStatusChanged,
            m_uiManager, [this](const System::NetworkStatus &status) {
                // Update network indicator in UI
                m_uiManager->updateNetworkStatus(status);
            });

    // Connect session manager to system
    connect(m_sessionManager, &System::SessionManager::sessionEnding,
            this, [this](const QString &reason) {
                // Save session state before ending
                m_sessionManager->saveSession();
                
                // Notify all components about session ending
                m_eventManager->emitEvent("sessionEnding", reason);
                
                // If reason is "shutdown" or "reboot", initiate system shutdown
                if (reason == "shutdown" || reason == "reboot") {
                    // Give components time to save state
                    QTimer::singleShot(1000, [this, reason]() {
                        // Shutdown the system
                        this->shutdown();
                        
                        // Exit the application
                        m_app->exit(reason == "reboot" ? 1 : 0);
                    });
                }
            });

    // Connect shortcut manager to action manager
    connect(m_shortcutManager, &Input::ShortcutManager::shortcutTriggered,
            m_actionManager, &Core::ActionManager::executeAction);

    // Connect gesture engine to action manager
    connect(m_gestureEngine, &Input::GestureEngine::gestureDetected,
            m_actionManager, &Core::ActionManager::executeAction);

    // Connect application manager to window manager
    connect(m_applicationManager, &System::ApplicationManager::applicationLaunched,
            m_windowManager, [this](const QString &appId, pid_t pid) {
                // Associate the application with its windows when they appear
                m_windowManager->setApplicationForPid(pid, appId);
            });

    qDebug() << "Signals connected successfully";
}

} // namespace VivoX
