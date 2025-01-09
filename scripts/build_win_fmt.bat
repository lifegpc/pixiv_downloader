@ECHO OFF
SETLOCAL
SET TOP=%CD%
SET PREFIX=%CD%\clib
IF NOT EXIST cbuild (
    MD cbuild || EXIT /B 1
)
CD cbuild || EXIT /B 1
git clone --depth 1 "https://github.com/fmtlib/fmt" || EXIT /B %ERRORLEVEL%
CD fmt || EXIT /B %ERRORLEVEL%
IF NOT EXIST build (
    MD build || EXIT /B 1
)
CD build || EXIT /B 1
cmake ^
    -G Ninja ^
    -DCMAKE_PREFIX_PATH=%PREFIX% ^
    -DCMAKE_BUILD_TYPE=Release ^
    -DCMAKE_INSTALL_PREFIX=%PREFIX% ^
    -DBUILD_SHARED_LIBS=ON ^
    -DFMT_DOC=OFF ^
    -DFMT_TEST=OFF ^
    ../ || EXIT /B %ERRORLEVEL%
ninja && ninja install || ninja && ninja install || EXIT /B %ERRORLEVEL%
ENDLOCAL