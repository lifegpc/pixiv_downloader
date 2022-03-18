cmake_minimum_required(VERSION 3.9)

project(avdict)

if (MSVC)
    add_compile_options(/utf-8)
endif()

include(GNUInstallDirs)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../cmake")
include(GetLinkLibraries)
find_package(AVUTIL REQUIRED)

add_library(avdict STATIC avdict.h src/avdict.c)
target_link_libraries(avdict AVUTIL::AVUTIL)
target_compile_definitions(avdict PRIVATE -DBUILD_AVDICT)

get_link_libraries(OUT avdict)
file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/avdict_dep.txt" "${OUT}")

install(TARGETS avdict)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/avdict_dep.txt" DESTINATION ${CMAKE_INSTALL_PREFIX})
