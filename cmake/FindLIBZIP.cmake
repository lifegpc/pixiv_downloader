find_package(PkgConfig)
if (PkgConfig_FOUND)
    pkg_check_modules(PC_LIBZIP QUIET IMPORTED_TARGET GLOBAL libzip)
endif()

if (PC_LIBZIP_FOUND)
    set(LIBZIP_FOUND TRUE)
    set(LIBZIP_VERSION ${PC_LIBZIP_VERSION})
    set(LIBZIP_VERSION_STRING ${PC_LIBZIP_STRING})
    set(LIBZIP_LIBRARYS ${PC_LIBZIP_LIBRARIES})
    if (USE_STATIC_LIBS)
        set(LIBZIP_INCLUDE_DIRS ${PC_LIBZIP_STATIC_INCLUDE_DIRS})
    else()
        set(LIBZIP_INCLUDE_DIRS ${PC_LIBZIP_INCLUDE_DIRS})
    endif()
    if (NOT TARGET LIBZIP::LIBZIP)
        add_library(LIBZIP::LIBZIP ALIAS PkgConfig::PC_LIBZIP)
    endif()
else()
    message(FATAL_ERROR "failed.")
endif()

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(LIBZIP
    FOUND_VAR LIBZIP_FOUND
    REQUIRED_VARS
        LIBZIP_LIBRARYS
    VERSION_VAR LIBZIP_VERSION
)
