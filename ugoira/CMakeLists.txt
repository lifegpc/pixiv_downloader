cmake_minimum_required(VERSION 3.9)

project(ugoira)

if (MSVC)
    add_compile_options(/utf-8)
endif()

include(GNUInstallDirs)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../cmake")
include(GetLinkLibraries)
find_package(AVUTIL REQUIRED)
find_package(AVFORMAT REQUIRED)
find_package(AVCODEC REQUIRED)
find_package(SWSCALE REQUIRED)
find_package(LIBZIP REQUIRED)

option(UTILS_LIBRARY "The path of utils of library." "")
if (UTILS_LIBRARY)
    set(UTILS_TARGET "${UTILS_LIBRARY}")
else()
    set(ENABLE_ICONV OFF CACHE BOOL "Libiconv is not needed.")
    add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../utils" "${CMAKE_BINARY_DIR}/utils")
    set(UTILS_TARGET utils)
endif()

include(CheckSymbolExists)
if (WIN32)
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

install(TARGETS ugoira)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/ugoira_dep.txt" DESTINATION ${CMAKE_INSTALL_PREFIX})
