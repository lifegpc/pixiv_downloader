@ECHO OFF
SETLOCAL
SET TOP=%CD%
SET SCRIPTS_DIR=%CD%\scripts
SET DOWNLOAD_RESOURCE=%SCRIPTS_DIR%\download_resource.bat
SET PREFIX=%CD%\clib
SET PKG_CONFIG_DIR=%PREFIX%\lib\pkgconfig
SET OPENSSLDIR=%PREFIX%\ssl
IF NOT EXIST cbuild (
    MD cbuild || EXIT /B 1
)
CD cbuild || EXIT /B 1
CALL %DOWNLOAD_RESOURCE% -o "openssl-3.0.5.tar.gz" "https://www.openssl.org/source/openssl-3.0.5.tar.gz" || EXIT /B %ERRORLEVEL%
tar -xzvf "openssl-3.0.5.tar.gz" || EXIT /B %ERRORLEVEL%
CD openssl-3.0.5 || EXIT /B 1
perl Configure shared zlib-dynamic --prefix=%PREFIX% --openssldir=%OPENSSLDIR% || EXIT /B %ERRORLEVEL%
nmake || EXIT /B %ERRORLEVEL%
nmake install || EXIT /B %ERRORLEVEL%
ENDLOCAL
