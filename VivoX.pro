# Created by and for Qt Creator This file was created for editing the project sources only.
# You may attempt to use it for building too, by modifying this file here.

#TARGET = VivoX

QT = core gui widgets

HEADERS = \
   $$PWD/compositor/protocols/LinuxDmabufProtocol.h \
   $$PWD/compositor/protocols/PresentationTimeProtocol.h \
   $$PWD/compositor/protocols/ViewporterProtocol.h \
   $$PWD/compositor/protocols/WaylandLayerShell.h \
   $$PWD/compositor/protocols/WaylandProtocols.h \
   $$PWD/compositor/protocols/XWaylandIntegration.h \
   $$PWD/compositor/rendering/RenderEngine.h \
   $$PWD/compositor/rendering/RenderEngineInterface.h \
   $$PWD/compositor/wayland/OutputManager.h \
   $$PWD/compositor/wayland/WaylandCompositor.h \
   $$PWD/compositor/xwayland/XWaylandIntegration.h \
   $$PWD/compositor/CompositorInterface.h \
   $$PWD/core/actions/ActionInterface.h \
   $$PWD/core/actions/ActionManager.h \
   $$PWD/core/actions/ActionRegistry.h \
   $$PWD/core/configuration/ConfigManager.h \
   $$PWD/core/configuration/ConfigManagerInterface.h \
   $$PWD/core/events/EventManager.h \
   $$PWD/core/events/EventManagerInterface.h \
   $$PWD/core/logging/Logger.h \
   $$PWD/core/plugins/PluginInterface.h \
   $$PWD/core/plugins/PluginLoader.h \
   $$PWD/core/services/ServiceInterface.h \
   $$PWD/core/services/ServiceRegistry.h \
   $$PWD/core/testing/TestRunner.h \
   $$PWD/input/gestures/GestureEngine.h \
   $$PWD/input/gestures/GestureRecognizer.h \
   $$PWD/input/shortcuts/ShortcutManager.h \
   $$PWD/input/InputManager.h \
   $$PWD/input/InputManagerInterface.h \
   $$PWD/system/applications/ApplicationManager.h \
   $$PWD/system/media/MediaController.h \
   $$PWD/system/network/NetworkManager.h \
   $$PWD/system/network/NetworkManagerInterface.h \
   $$PWD/system/notifications/NotificationManager.h \
   $$PWD/system/notifications/NotificationManagerInterface.h \
   $$PWD/system/power/PowerManager.h \
   $$PWD/system/session/SessionManager.h \
   $$PWD/system/SystemService.h \
   $$PWD/system/SystemServiceInterface.h \
   $$PWD/ui/panels/PanelInterface.h \
   $$PWD/ui/panels/PanelManager.h \
   $$PWD/ui/qml/theme/ThemeManager.h \
   $$PWD/ui/widgets/Widget.h \
   $$PWD/ui/widgets/WidgetHost.h \
   $$PWD/ui/widgets/WidgetInterface.h \
   $$PWD/ui/widgets/WidgetManager.h \
   $$PWD/ui/widgets/WidgetRegistry.h \
   $$PWD/ui/UIManager.h \
   $$PWD/ui/UIManagerInterface.h \
   $$PWD/window_manager/layouts/LayoutEngine.h \
   $$PWD/window_manager/stage/StageManager.h \
   $$PWD/window_manager/tabbing/TabManager.h \
   $$PWD/window_manager/windows/WindowManager.h \
   $$PWD/window_manager/windows/WindowManagerInterface.h \
   $$PWD/window_manager/workspaces/Workspace.h \
   $$PWD/window_manager/workspaces/WorkspaceManager.h \
   $$PWD/InputManager.h \
   $$PWD/VivoXSystem.h

SOURCES = \
   $$PWD/build/Desktop-Debug/CMakeFiles/4.0.0/CompilerIdCXX/CMakeCXXCompilerId.cpp \
   $$PWD/compositor/protocols/LinuxDmabufProtocol.cpp \
   $$PWD/compositor/protocols/PresentationTimeProtocol.cpp \
   $$PWD/compositor/protocols/ViewporterProtocol.cpp \
   $$PWD/compositor/protocols/WaylandLayerShell.cpp \
   $$PWD/compositor/protocols/WaylandProtocols.cpp \
   $$PWD/compositor/protocols/XWaylandIntegration.cpp \
   $$PWD/compositor/rendering/RenderEngine.cpp \
   $$PWD/compositor/wayland/OutputManager.cpp \
   $$PWD/compositor/wayland/WaylandCompositor.cpp \
   $$PWD/compositor/xwayland/XWaylandIntegration.cpp \
   $$PWD/core/actions/ActionManager.cpp \
   $$PWD/core/actions/ActionRegistry.cpp \
   $$PWD/core/configuration/ConfigManager.cpp \
   $$PWD/core/events/EventManager.cpp \
   $$PWD/core/logging/Logger.cpp \
   $$PWD/core/plugins/PluginLoader.cpp \
   $$PWD/core/services/ServiceRegistry.cpp \
   $$PWD/input/gestures/GestureEngine.cpp \
   $$PWD/input/gestures/GestureRecognizer.cpp \
   $$PWD/input/shortcuts/ShortcutManager.cpp \
   $$PWD/input/InputManager.cpp \
   $$PWD/system/applications/ApplicationManager.cpp \
   $$PWD/system/media/MediaController.cpp \
   $$PWD/system/network/NetworkManager.cpp \
   $$PWD/system/notifications/NotificationManager.cpp \
   $$PWD/system/power/PowerManager.cpp \
   $$PWD/system/session/SessionManager.cpp \
   $$PWD/system/SystemService.cpp \
   $$PWD/tests/integration/core/CoreIntegrationTest.cpp \
   $$PWD/tests/unit/core/ActionManagerTest.cpp \
   $$PWD/tests/unit/core/ConfigManagerTest.cpp \
   $$PWD/tests/unit/core/EventManagerTest.cpp \
   $$PWD/tests/unit/core/LoggerTest.cpp \
   $$PWD/tests/unit/core/PluginLoaderTest.cpp \
   $$PWD/tests/unit/core/ServiceRegistryTest.cpp \
   $$PWD/ui/panels/PanelManager.cpp \
   $$PWD/ui/qml/theme/ThemeManager.cpp \
   $$PWD/ui/widgets/Widget.cpp \
   $$PWD/ui/widgets/WidgetHost.cpp \
   $$PWD/ui/widgets/WidgetManager.cpp \
   $$PWD/ui/widgets/WidgetRegistry.cpp \
   $$PWD/ui/UIManager.cpp \
   $$PWD/window_manager/layouts/LayoutEngine.cpp \
   $$PWD/window_manager/stage/StageManager.cpp \
   $$PWD/window_manager/tabbing/TabManager.cpp \
   $$PWD/window_manager/windows/WindowManager.cpp \
   $$PWD/window_manager/workspaces/Workspace.cpp \
   $$PWD/window_manager/workspaces/WorkspaceManager.cpp \
   $$PWD/InputManager.cpp \
   $$PWD/main.cpp \
   $$PWD/VivoXSystem.cpp

INCLUDEPATH = \
    $$PWD/. \
    $$PWD/compositor \
    $$PWD/compositor/protocols \
    $$PWD/compositor/rendering \
    $$PWD/compositor/wayland \
    $$PWD/compositor/xwayland \
    $$PWD/core/actions \
    $$PWD/core/configuration \
    $$PWD/core/events \
    $$PWD/core/logging \
    $$PWD/core/plugins \
    $$PWD/core/services \
    $$PWD/core/testing \
    $$PWD/input \
    $$PWD/input/gestures \
    $$PWD/input/shortcuts \
    $$PWD/system \
    $$PWD/system/applications \
    $$PWD/system/media \
    $$PWD/system/network \
    $$PWD/system/notifications \
    $$PWD/system/power \
    $$PWD/system/session \
    $$PWD/ui \
    $$PWD/ui/panels \
    $$PWD/ui/qml/theme \
    $$PWD/ui/widgets \
    $$PWD/window_manager/layouts \
    $$PWD/window_manager/stage \
    $$PWD/window_manager/tabbing \
    $$PWD/window_manager/windows \
    $$PWD/window_manager/workspaces

#DEFINES = 

