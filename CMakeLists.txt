cmake_minimum_required(VERSION 3.16)
project(VivoX VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

# Find required Qt packages
find_package(Qt6 COMPONENTS Core Gui Widgets Quick QuickControls2 WaylandCompositor REQUIRED)

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/src
    ${CMAKE_CURRENT_SOURCE_DIR}/src/core
    ${CMAKE_CURRENT_SOURCE_DIR}/src/compositor
    ${CMAKE_CURRENT_SOURCE_DIR}/src/window_manager
    ${CMAKE_CURRENT_SOURCE_DIR}/src/ui
    ${CMAKE_CURRENT_SOURCE_DIR}/src/input
    ${CMAKE_CURRENT_SOURCE_DIR}/src/system
)

# Add subdirectories
add_subdirectory(src/core)
add_subdirectory(src/compositor)
add_subdirectory(src/window_manager)
add_subdirectory(src/ui)
add_subdirectory(src/input)
add_subdirectory(src/system)

# Main executable
add_executable(vivox
    src/main.cpp
    src/VivoXSystem.cpp
    src/VivoXSystem.h
)

# Link libraries
target_link_libraries(vivox
    PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Widgets
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::WaylandCompositor
    vivox_core
    vivox_compositor
    vivox_window_manager
    vivox_ui
    vivox_input
    vivox_system
)

# Install
install(TARGETS vivox DESTINATION bin)
install(DIRECTORY src/ui/qml/ DESTINATION share/vivox/qml)
