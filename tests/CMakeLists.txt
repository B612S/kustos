# SPDX-License-Identifier: GPL-3.0-or-later
# SPDX-FileCopyrightText: 2026 B612S <sem.alimov@inbox.ru>

find_package(Qt6 REQUIRED COMPONENTS Test)

set(AION_TEST_CORE
    ${CMAKE_SOURCE_DIR}/core/app/ConfigManager.cpp
    ${CMAKE_SOURCE_DIR}/core/backup/Retention.cpp
    ${CMAKE_SOURCE_DIR}/core/backup/Snapshot.cpp
    ${CMAKE_SOURCE_DIR}/core/backup/SnapshotStore.cpp
    ${CMAKE_SOURCE_DIR}/core/util/Format.cpp
)

foreach(test_name tst_retention tst_configmanager tst_snapshotstore)
    add_executable(${test_name} ${test_name}.cpp ${AION_TEST_CORE})
    target_include_directories(${test_name} PRIVATE ${CMAKE_SOURCE_DIR}/core)
    target_link_libraries(${test_name} PRIVATE Qt6::Core Qt6::Test)
    add_test(NAME ${test_name} COMMAND ${test_name})
endforeach()
