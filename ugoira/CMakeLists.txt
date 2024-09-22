cmake_minimum_required(VERSION 3.9)

project(ugoira)

if (MSVC)
    add_compile_options(/utf-8)
endif()

include(CheckIncludeFiles)
include(GNUInstallDirs)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../cmake")
include(GetLinkLibraries)
find_package(AVUTIL REQUIRED)
find_package(AVFORMAT REQUIRED)
find_package(AVCODEC REQUIRED)
find_package(SWSCALE REQUIRED)
find_package(LIBZIP REQUIRED)
find_package(RapidJSON REQUIRED)

option(UTILS_LIBRARY "The path of utils of library." "")
if (UTILS_LIBRARY)
    find_library(UTILS_LIB utils PATHS "${UTILS_LIBRARY}")
    add_library(utils_imported STATIC IMPORTED)
    set_property(TARGET utils_imported PROPERTY IMPORTED_LOCATION "${UTILS_LIB}")
    set(UTILS_TARGET utils_imported)
else()
    set(ENABLE_ICONV OFF CACHE BOOL "Libiconv is not needed.")
    add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../utils" "${CMAKE_BINARY_DIR}/utils")
    set(UTILS_TARGET utils)
endif()

include(CheckSymbolExists)
if (WIN32)
    check_symbol_exists(printf_s "stdio.h" HAVE_PRINTF_S)
    check_symbol_exists(sscanf_s "stdio.h" HAVE_SSCANF_S)
endif()

configure_file("${CMAKE_CURRENT_SOURCE_DIR}/ugoira_config.h.in" "${CMAKE_CURRENT_BINARY_DIR}/ugoira_config.h")

set(UGOIRA_FILES
ugoira.h
src/ugoira.c
"${CMAKE_CURRENT_BINARY_DIR}/ugoira_config.h"
)

include_directories("${CMAKE_CURRENT_SOURCE_DIR}/../utils")
include_directories("${CMAKE_CURRENT_BINARY_DIR}")
include_directories("${AVUTIL_INCLUDE_DIRS}")

add_library(ugoira STATIC "${UGOIRA_FILES}")
target_link_libraries(ugoira AVUTIL::AVUTIL AVFORMAT::AVFORMAT AVCODEC::AVCODEC SWSCALE::SWSCALE LIBZIP::LIBZIP ${UTILS_TARGET})
target_compile_definitions(ugoira PRIVATE BUILD_UGOIRA)

get_link_libraries(OUT ugoira)
file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/ugoira_dep.txt" "${OUT}")

add_executable(ugoira_cli src/main.cpp)
CHECK_INCLUDE_FILES(getopt.h HAVE_GETOPT_H)
if (NOT HAVE_GETOPT_H)
    add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../getopt" "${CMAKE_CURRENT_BINARY_DIR}/getopt")
    target_include_directories(ugoira_cli PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}/../getopt")
    target_link_libraries(ugoira_cli getopt)
endif()
target_link_libraries(ugoira_cli ugoira RapidJSON)
target_include_directories(ugoira_cli PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")
set_target_properties(ugoira_cli PROPERTIES OUTPUT_NAME "ugoira")
target_compile_features(ugoira_cli PRIVATE cxx_std_17)

install(TARGETS ugoira ugoira_cli)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/ugoira_dep.txt" DESTINATION ${CMAKE_INSTALL_PREFIX})
