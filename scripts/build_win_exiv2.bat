@ECHO OFF
SETLOCAL
SET TOP=%CD%
SET PREFIX=%CD%\clib
SET PKG_CONFIG_DIR=%PREFIX%\lib\pkgconfig
SET PATCH_DIR=%CD%\exif\patchs
IF NOT EXIST cbuild (
    MD cbuild || EXIT /B 1
)
CD cbuild || EXIT /B 1
git clone --depth 1 "https://github.com/Exiv2/exiv2" || EXIT /B %ERRORLEVEL%
CD exiv2 || EXIT /B %ERRORLEVEL%
git apply %PATCH_DIR%\basicio.cpp.patch || EXIT /B %ERRORLEVEL%
IF NOT EXIST build (
    MD build || EXIT /B 1
)
CD build || EXIT /B 1
cmake ^
    -G Ninja ^
    -DCMAKE_PREFIX_PATH=%PREFIX% ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_INSTALL_PREFIX=%PREFIX% ^
    -DINSTALL_PKGCONFIG_DIR=%PKG_CONFIG_DIR% ^
    -DEXIV2_ENABLE_BROTLI=OFF ^
    -DEXIV2_ENABLE_INIH=OFF ^
    ../ || EXIT /B %ERRORLEVEL%
ninja && ninja install || ninja && ninja install || EXIT /B %ERRORLEVEL%
ENDLOCAL
