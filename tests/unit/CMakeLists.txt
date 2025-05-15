cmake_minimum_required(VERSION 3.10)

# Include directories
include_directories(
  ${CMAKE_SOURCE_DIR}/src
)

# Core unit tests
add_executable(core_logging_test
  core/LoggerTest.cpp
)
target_link_libraries(core_logging_test
  gtest_main
  vivox_core
)
add_test(NAME core_logging_test COMMAND core_logging_test)

add_executable(core_config_test
  core/ConfigManagerTest.cpp
)
target_link_libraries(core_config_test
  gtest_main
  vivox_core
)
add_test(NAME core_config_test COMMAND core_config_test)

add_executable(core_events_test
  core/EventManagerTest.cpp
)
target_link_libraries(core_events_test
  gtest_main
  vivox_core
)
add_test(NAME core_events_test COMMAND core_events_test)

add_executable(core_plugins_test
  core/PluginLoaderTest.cpp
)
target_link_libraries(core_plugins_test
  gtest_main
  vivox_core
)
add_test(NAME core_plugins_test COMMAND core_plugins_test)

add_executable(core_services_test
  core/ServiceRegistryTest.cpp
)
target_link_libraries(core_services_test
  gtest_main
  vivox_core
)
add_test(NAME core_services_test COMMAND core_services_test)

add_executable(core_actions_test
  core/ActionManagerTest.cpp
)
target_link_libraries(core_actions_test
  gtest_main
  vivox_core
)
add_test(NAME core_actions_test COMMAND core_actions_test)

# Compositor unit tests
add_executable(compositor_wayland_test
  compositor/WaylandCompositorTest.cpp
)
target_link_libraries(compositor_wayland_test
  gtest_main
  vivox_compositor
)
add_test(NAME compositor_wayland_test COMMAND compositor_wayland_test)

add_executable(compositor_rendering_test
  compositor/RenderEngineTest.cpp
)
target_link_libraries(compositor_rendering_test
  gtest_main
  vivox_compositor
)
add_test(NAME compositor_rendering_test COMMAND compositor_rendering_test)

# Window manager unit tests
add_executable(window_manager_test
  window_manager/WindowManagerTest.cpp
)
target_link_libraries(window_manager_test
  gtest_main
  vivox_window_manager
)
add_test(NAME window_manager_test COMMAND window_manager_test)

# UI unit tests
add_executable(ui_manager_test
  ui/UIManagerTest.cpp
)
target_link_libraries(ui_manager_test
  gtest_main
  vivox_ui
)
add_test(NAME ui_manager_test COMMAND ui_manager_test)

# Input unit tests
add_executable(input_manager_test
  input/InputManagerTest.cpp
)
target_link_libraries(input_manager_test
  gtest_main
  vivox_input
)
add_test(NAME input_manager_test COMMAND input_manager_test)

add_executable(input_shortcuts_test
  input/ShortcutManagerTest.cpp
)
target_link_libraries(input_shortcuts_test
  gtest_main
  vivox_input
)
add_test(NAME input_shortcuts_test COMMAND input_shortcuts_test)

add_executable(input_gestures_test
  input/GestureEngineTest.cpp
)
target_link_libraries(input_gestures_test
  gtest_main
  vivox_input
)
add_test(NAME input_gestures_test COMMAND input_gestures_test)
