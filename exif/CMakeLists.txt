cmake_minimum_required(VERSION 3.9)

project(exif)

if (MSVC)
    add_compile_options(/utf-8)
    add_compile_options(/EHsc)
endif()

include(GNUInstallDirs)

option(UTILS_LIBRARY "The path of utils of library." "")
if (UTILS_LIBRARY)
    set(UTILS_TARGET "${UTILS_LIBRARY}")
else()
    set(ENABLE_ICONV OFF CACHE BOOL "Libiconv is not needed.")
    add_subdirectory("${CMAKE_CURRENT_SOURCE_DIR}/../utils" "${CMAKE_BINARY_DIR}/utils")
    set(UTILS_TARGET utils)
endif()

include_directories("${CMAKE_CURRENT_SOURCE_DIR}/../utils")

list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")
find_package(Exiv2)

include_directories(${Exiv2_INCLUDE_DIRS})
include_directories("${CMAKE_CURRENT_SOURCE_DIR}")
add_library(exif exif.h src/exif_priv.h src/exif.cpp)
target_compile_features(exif PRIVATE cxx_std_17)
target_link_libraries(exif ${Exiv2_LIBRARIES})
target_link_libraries(exif ${UTILS_TARGET})

if (BUILD_SHARED_LIBS AND WIN32)
    target_compile_definitions(exif PRIVATE -DBUILD_DLL -DWIN32_DLL)
endif()

get_target_property(OUT exif LINK_LIBRARIES)
file(WRITE "${CMAKE_CURRENT_BINARY_DIR}/exif_dep.txt" "${OUT}")

install(TARGETS exif)
install(FILES "${CMAKE_CURRENT_BINARY_DIR}/exif_dep.txt" DESTINATION ${CMAKE_INSTALL_PREFIX})
