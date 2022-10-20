@ECHO OFF
SETLOCAL
SET PREFIX=%CD%\clib
SET PKG_CONFIG_DIR=%PREFIX%\lib\pkgconfig
IF NOT EXIST cbuild (
    MD cbuild || EXIT /B 1
)
CD cbuild || EXIT /B 1
git clone --depth 1 "https://github.com/nih-at/libzip" || EXIT /B %ERRORLEVEL%
CD libzip || EXIT /B 1
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
    -DENABLE_COMMONCRYPTO=OFF ^
    -DENABLE_GNUTLS=OFF ^
    -DENABLE_MBEDTLS=OFF ^
    -DENABLE_OPENSSL=ON ^
    -DENABLE_WINDOWS_CRYPTO=ON ^
    -DENABLE_BZIP2=OFF ^
    -DENABLE_LZMA=OFF ^
    -DENABLE_ZSTD=OFF ^
    -DBUILD_REGRESS=OFF ^
    -DBUILD_EXAMPLES=OFF ^
    -DBUILD_DOC=OFF ^
    -DBUILD_TOOLS=OFF ^
    ../ || EXIT /B %ERRORLEVEL%
ninja && ninja install || ninja && ninja install || EXIT /B %ERRORLEVEL%
ENDLOCAL
