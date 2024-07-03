@ECHO OFF
SETLOCAL
SET TOP=%CD%
SET SCRIPTS_DIR=%CD%\scripts
SET DOWNLOAD_RESOURCE=%SCRIPTS_DIR%\download_resource.bat
SET PREFIX=%CD%\clib
SET PKG_CONFIG_DIR=%PREFIX%\lib\pkgconfig
SET PATCH_DIR=%CD%\patch\zlib-msvc
IF NOT EXIST cbuild (
    MD cbuild || EXIT /B 1
)
CD cbuild || EXIT /B 1
git clone --depth 1 "https://github.com/lifegpc/zlib" || EXIT /B %ERRORLEVEL%
CD zlib || EXIT /B 1
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
    ../ || EXIT /B %ERRORLEVEL%
ninja && ninja install || ninja && ninja install || EXIT /B %ERRORLEVEL%
CD %PREFIX%\include || EXIT /B 1
patch -p1 zconf.h %PATCH_DIR%\zconf.h.patch || EXIT /B %ERRORLEVEL%
ENDLOCAL
