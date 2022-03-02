find_path(Exiv2_INCLUDE_DIR
    NAMES exiv2/exiv2.hpp
)
find_library(Exiv2_LIBRARY
    NAMES exiv2
)
if (Exiv2_INCLUDE_DIR)
    if (EXISTS "${Exiv2_INCLUDE_DIR}/exiv2/exv_conf.h")
        file(STRINGS "${Exiv2_INCLUDE_DIR}/exiv2/exv_conf.h" EXV_CONF_H)
        foreach(LINE IN LISTS EXV_CONF_H)
            string(REGEX MATCH "^#define EXV_PACKAGE_VERSION \"([^\"]+)\"" OUTPUT "${LINE}")
            if (OUTPUT)
                set(EXIV2_VERSION "${CMAKE_MATCH_1}")
            endif()
        endforeach()
    endif()
endif()

if (Exiv2_INCLUDE_DIR AND Exiv2_LIBRARY)
    set(Exiv2_FOUND TRUE)
endif()

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(Exiv2
    FOUND_VAR Exiv2_FOUND
    REQUIRED_VARS
        Exiv2_LIBRARY
        Exiv2_INCLUDE_DIR
    VERSION_VAR
        EXIV2_VERSION
)

set(Exiv2_INCLUDE_DIRS "${Exiv2_INCLUDE_DIR}")
set(Exiv2_LIBRARIES "${Exiv2_LIBRARY}")
