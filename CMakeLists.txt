cmake_minimum_required(VERSION 3.21)
project(waycast LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Use Ninja if available (optional)
if(NOT CMAKE_GENERATOR)
  set(CMAKE_GENERATOR "Ninja" CACHE INTERNAL "" FORCE)
endif()

find_package(PkgConfig REQUIRED)
find_package(Qt6 REQUIRED COMPONENTS
  Core
  Gui
  Qml
  Quick
  QuickControls2
)
find_package(LayerShellQt REQUIRED)

pkg_check_modules(GIO REQUIRED IMPORTED_TARGET gio-2.0)

qt_standard_project_setup()

qt_add_executable(waycast)

include_directories("${CMAKE_SOURCE_DIR}/lib")
include_directories("${CMAKE_SOURCE_DIR}/lib/ui")
file(GLOB_RECURSE SRC_FILES CONFIGURE_DEPENDS "${CMAKE_SOURCE_DIR}/src/*.cpp")
file(GLOB_RECURSE LIB_UI_FILES CONFIGURE_DEPENDS "${CMAKE_SOURCE_DIR}/lib/ui/*.cpp")
target_sources(waycast PRIVATE ${SRC_FILES} ${LIB_UI_FILES} src/main.cpp)
target_include_directories(waycast PRIVATE ${CMAKE_SOURCE_DIR}/lib)
# target_include_directories(waycast PRIVATE ${CMAKE_SOURCE_DIR}/lib ${CMAKE_CURRENT_BINARY_DIR}) 

# Bundle QML into the app (no runtime QML path headaches)
qt_add_qml_module(waycast
  URI WayCast 
  VERSION 1.0
  QML_FILES
    ui/Main.qml
)

target_link_libraries(waycast PRIVATE
  Qt6::Core
  Qt6::Gui
  Qt6::Qml
  Qt6::Quick
  PkgConfig::GIO
  LayerShellQt::Interface
)

# On Linux, ensure plugins/libs are found at runtime if needed
# (usually fine when running inside nix shell)
if(UNIX AND NOT APPLE)
  set_target_properties(waycast PROPERTIES
    INSTALL_RPATH "$ORIGIN"
    BUILD_WITH_INSTALL_RPATH TRUE
  )
endif()

# Add -fmodules-ts where needed
if (CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
  target_compile_options(waycast PRIVATE -fmodules-ts)
elseif (CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
  target_compile_options(waycast PRIVATE -fmodules-ts)
endif()