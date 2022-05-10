cmake_minimum_required(VERSION 3.11)
find_package(PkgConfig)
if (PkgConfig_FOUND)
    pkg_check_modules(PC_AVCODEC QUIET IMPORTED_TARGET GLOBAL libavcodec)
endif()

if (PC_AVCODEC_FOUND)
    set(AVCODEC_FOUND TRUE)
    set(AVCODEC_VERSION ${PC_AVCODEC_VERSION})
    set(AVCODEC_VERSION_STRING ${PC_AVCODEC_STRING})
    set(AVCODEC_LIBRARYS ${PC_AVCODEC_LIBRARIES})
    if (USE_STATIC_LIBS)
        set(AVCODEC_INCLUDE_DIRS ${PC_AVCODEC_STATIC_INCLUDE_DIRS})
    else()
        set(AVCODEC_INCLUDE_DIRS ${PC_AVCODEC_INCLUDE_DIRS})
    endif()
    if (NOT AVCODEC_INCLUDE_DIRS)
        find_path(AVCODEC_INCLUDE_DIRS NAMES libavcodec/avcodec.h)
        if (AVCODEC_INCLUDE_DIRS)
            target_include_directories(PkgConfig::PC_AVCODEC INTERFACE ${AVCODEC_INCLUDE_DIRS})
        endif()
    endif()
    if (NOT TARGET AVCODEC::AVCODEC)
        add_library(AVCODEC::AVCODEC ALIAS PkgConfig::PC_AVCODEC)
    endif()
else()
    message(FATAL_ERROR "failed to find libavcodec.")
endif()

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(AVCODEC
    FOUND_VAR AVCODEC_FOUND
    REQUIRED_VARS
        AVCODEC_LIBRARYS
        AVCODEC_INCLUDE_DIRS
    VERSION_VAR AVCODEC_VERSION
)
