cmake_minimum_required(VERSION 3.11)
find_package(PkgConfig)
if (PkgConfig_FOUND)
    pkg_check_modules(PC_SWSCALE QUIET IMPORTED_TARGET GLOBAL libswscale)
endif()

if (PC_SWSCALE_FOUND)
    set(SWSCALE_FOUND TRUE)
    set(SWSCALE_VERSION ${PC_SWSCALE_VERSION})
    set(SWSCALE_VERSION_STRING ${PC_SWSCALE_STRING})
    set(SWSCALE_LIBRARYS ${PC_SWSCALE_LIBRARIES})
    if (USE_STATIC_LIBS)
        set(SWSCALE_INCLUDE_DIRS ${PC_SWSCALE_STATIC_INCLUDE_DIRS})
    else()
        set(SWSCALE_INCLUDE_DIRS ${PC_SWSCALE_INCLUDE_DIRS})
    endif()
    if (NOT SWSCALE_INCLUDE_DIRS)
        find_path(SWSCALE_INCLUDE_DIRS NAMES libswscale/swscale.h)
        if (SWSCALE_INCLUDE_DIRS)
            target_include_directories(PkgConfig::PC_SWSCALE INTERFACE ${SWSCALE_INCLUDE_DIRS})
        endif()
    endif()
    if (NOT TARGET SWSCALE::SWSCALE)
        add_library(SWSCALE::SWSCALE ALIAS PkgConfig::PC_SWSCALE)
    endif()
else()
    message(FATAL_ERROR "failed to find libswscale.")
endif()

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(SWSCALE
    FOUND_VAR SWSCALE_FOUND
    REQUIRED_VARS
        SWSCALE_LIBRARYS
        SWSCALE_INCLUDE_DIRS
    VERSION_VAR SWSCALE_VERSION
)
