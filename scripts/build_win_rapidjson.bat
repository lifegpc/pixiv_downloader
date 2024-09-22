@ECHO OFF
SETLOCAL
SET PREFIX=%CD%\clib
SET PKG_CONFIG_DIR=%PREFIX%\lib\pkgconfig
IF NOT EXIST cbuild (
    MD cbuild || EXIT /B 1
)
CD cbuild || EXIT /B 1
git clone --depth 1 "https://github.com/Tencent/rapidjson" || EXIT /B %ERRORLEVEL%
CD rapidjson || EXIT /B 1
IF NOT EXIST build (
    MD build || EXIT /B 1
)
CD build || EXIT /B 1
cmake ^
    -G Ninja ^
    -DCMAKE_PREFIX_PATH=%PREFIX% ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_INSTALL_PREFIX=%PREFIX% ^
    -DRAPIDJSON_BUILD_DOC=OFF ^
    -DRAPIDJSON_BUILD_EXAMPLES=OFF ^
    -DRAPIDJSON_BUILD_TESTS=OFF ^
    ../ || EXIT /B %ERRORLEVEL%
ninja && ninja install || ninja && ninja install || EXIT /B %ERRORLEVEL%
ENDLOCAL
