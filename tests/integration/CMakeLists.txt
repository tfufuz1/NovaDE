cmake_minimum_required(VERSION 3.10)

# Include directories
include_directories(
  ${CMAKE_SOURCE_DIR}/src
)

# Integration tests for core components
add_executable(core_integration_test
  core/CoreIntegrationTest.cpp
)
target_link_libraries(core_integration_test
  gtest_main
  vivox_core
)
add_test(NAME core_integration_test COMMAND core_integration_test)

# Integration tests for compositor and window manager
add_executable(compositor_window_integration_test
  compositor_window/CompositorWindowIntegrationTest.cpp
)
target_link_libraries(compositor_window_integration_test
  gtest_main
  vivox_compositor
  vivox_window_manager
)
add_test(NAME compositor_window_integration_test COMMAND compositor_window_integration_test)

# Integration tests for UI and input systems
add_executable(ui_input_integration_test
  ui_input/UIInputIntegrationTest.cpp
)
target_link_libraries(ui_input_integration_test
  gtest_main
  vivox_ui
  vivox_input
)
add_test(NAME ui_input_integration_test COMMAND ui_input_integration_test)

# Integration tests for system services
add_executable(system_integration_test
  system/SystemIntegrationTest.cpp
)
target_link_libraries(system_integration_test
  gtest_main
  vivox_system
)
add_test(NAME system_integration_test COMMAND system_integration_test)

# Full system integration test
add_executable(full_system_integration_test
  full_system/FullSystemIntegrationTest.cpp
)
target_link_libraries(full_system_integration_test
  gtest_main
  vivox_core
  vivox_compositor
  vivox_window_manager
  vivox_ui
  vivox_input
  vivox_system
)
add_test(NAME full_system_integration_test COMMAND full_system_integration_test)
