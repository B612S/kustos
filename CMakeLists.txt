# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

cmake_minimum_required(VERSION 3.20)

# Declared before project(): the generator reads CMAKE_EXPORT_COMPILE_COMMANDS
# during the first configure too.
option(AION_CLANG_TIDY "Add a 'lint' target that runs clang-tidy" OFF)
if(AION_CLANG_TIDY)
    set(CMAKE_EXPORT_COMPILE_COMMANDS ON CACHE BOOL
        "Write compile_commands.json for the lint target and IDE tooling" FORCE)
endif()

file(STRINGS "${CMAKE_CURRENT_SOURCE_DIR}/VERSION" AION_VERSION LIMIT_COUNT 1)
string(STRIP "${AION_VERSION}" AION_VERSION)
if(NOT AION_VERSION MATCHES "^[0-9]+\\.[0-9]+\\.[0-9]+$")
    message(FATAL_ERROR "VERSION must be MAJOR.MINOR.PATCH (got '${AION_VERSION}')")
endif()
project(aion VERSION ${AION_VERSION} LANGUAGES CXX)

# Revision shown in the About dialog as "<version>-<git hash>". The hash is
# optional (source tarballs have no repository).
find_package(Git QUIET)
set(AION_GIT_HASH "" CACHE STRING
    "Short git revision shown in About; detected automatically when empty")
if(NOT AION_GIT_HASH AND GIT_FOUND AND EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/.git")
    execute_process(
        COMMAND "${GIT_EXECUTABLE}" -C "${CMAKE_CURRENT_SOURCE_DIR}" rev-parse --short=7 HEAD
        OUTPUT_VARIABLE AION_GIT_HASH
        OUTPUT_STRIP_TRAILING_WHITESPACE
        ERROR_QUIET)
endif()

include(GNUInstallDirs)
if(NOT CMAKE_INSTALL_DATADIR)
    set(CMAKE_INSTALL_DATADIR "share")
endif()

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)

find_package(Qt6 6.5 REQUIRED COMPONENTS
    Core Gui Quick QuickControls2 Widgets DBus Network LinguistTools
)

qt_standard_project_setup(REQUIRES 6.5)

qt_add_executable(aion
    main.cpp
    core/app/ConfigManager.cpp
    core/app/ConfigManager.hpp
    core/app/Logger.cpp
    core/app/Logger.hpp
    core/backup/BackupRunner.cpp
    core/backup/BackupRunner.hpp
    core/backup/Retention.cpp
    core/backup/Retention.hpp
    core/backup/Snapshot.cpp
    core/backup/Snapshot.hpp
    core/backup/RestoreRunner.cpp
    core/backup/RestoreRunner.hpp
    core/backup/SnapshotStore.cpp
    core/backup/SnapshotStore.hpp
    core/backup/SnapshotModel.cpp
    core/backup/SnapshotModel.hpp
    core/backup/DirectoryModel.cpp
    core/backup/DirectoryModel.hpp
    core/schedule/Autostart.cpp
    core/schedule/Autostart.hpp
    core/ui/AppController.cpp
    core/ui/AppController.hpp
    core/ui/Notifier.cpp
    core/ui/Notifier.hpp
    core/util/Format.cpp
    core/util/Format.hpp
    core/util/Loc.cpp
    core/util/Loc.hpp
)

qt_add_qml_module(aion
    URI aion
    VERSION 1.0
    # Kirigami and qqc2-desktop-style are runtime-only QML imports, not build
    # dependencies. Without this, qmlimportscanner tries to link their static
    # plugin targets, which do not exist here.
    NO_IMPORT_SCAN
    QML_FILES
        qml/Main.qml
        qml/TimelinePanel.qml
        qml/SnapshotBrowser.qml
        qml/SettingsDialog.qml
        qml/RestoreDialog.qml
        qml/AboutDialog.qml
        qml/AionMenu.qml
        qml/AionMenuButton.qml
)

qt_add_resources(aion "assets"
    PREFIX "/"
    FILES
        assets/aion.svg
        assets/aion.png
)

qt_add_translations(aion
    TS_FILES
        locales/en.ts
        locales/ru.ts
)

install(FILES
    ${CMAKE_CURRENT_BINARY_DIR}/en.qm
    ${CMAKE_CURRENT_BINARY_DIR}/ru.qm
    DESTINATION ${CMAKE_INSTALL_DATADIR}/aion/translations
)

install(FILES aion.desktop DESTINATION ${CMAKE_INSTALL_DATADIR}/applications)
install(FILES assets/aion.svg DESTINATION ${CMAKE_INSTALL_DATADIR}/icons/hicolor/scalable/apps)
install(FILES assets/aion.png DESTINATION ${CMAKE_INSTALL_DATADIR}/icons/hicolor/256x256/apps)

target_include_directories(aion PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/core)

target_link_libraries(aion PRIVATE
    Qt6::Core
    Qt6::Gui
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::Widgets
    Qt6::DBus
    Qt6::Network
)

target_compile_definitions(aion PRIVATE
    AION_VERSION="${PROJECT_VERSION}"
    AION_GIT_HASH="${AION_GIT_HASH}"
)

install(TARGETS aion
    BUNDLE DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)

install(IMPORTED_RUNTIME_ARTIFACTS aion DESTINATION ${CMAKE_INSTALL_LIBDIR}/qt6/qml/aion)
install(DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}/aion/ DESTINATION ${CMAKE_INSTALL_LIBDIR}/qt6/qml/aion)

# ---- Tests (off by default, never ship in a package) ----
option(AION_BUILD_TESTS "Build autotests (Qt Test)" OFF)
if(AION_BUILD_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()

# ---- Optional clang-tidy linting ----
if(AION_CLANG_TIDY)
    find_program(AION_CLANG_TIDY_EXECUTABLE
        NAMES clang-tidy clang-tidy-20 clang-tidy-19 clang-tidy-18 clang-tidy-17)
    find_program(AION_RUN_CLANG_TIDY_EXECUTABLE
        NAMES run-clang-tidy run-clang-tidy-20 run-clang-tidy-19 run-clang-tidy-18)
    if(NOT AION_CLANG_TIDY_EXECUTABLE OR NOT AION_RUN_CLANG_TIDY_EXECUTABLE)
        message(FATAL_ERROR "AION_CLANG_TIDY=ON requires clang-tidy and run-clang-tidy")
    endif()
    add_custom_target(lint
        COMMAND ${AION_RUN_CLANG_TIDY_EXECUTABLE}
                -p ${CMAKE_BINARY_DIR}
                -quiet
                -hide-progress
                -clang-tidy-binary ${AION_CLANG_TIDY_EXECUTABLE}
                -removed-arg=-mno-direct-extern-access
                "^${CMAKE_SOURCE_DIR}/(main\\.cpp|core/)"
        COMMENT "Running clang-tidy on main.cpp and core/"
        USES_TERMINAL
        VERBATIM)
    add_dependencies(lint aion)
endif()
