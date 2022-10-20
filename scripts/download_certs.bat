@ECHO OFF
SETLOCAL
SET TOP=%CD%
SET SCRIPTS_DIR=%CD%\scripts
SET DOWNLOAD_RESOURCE=%SCRIPTS_DIR%\download_resource.bat
%DOWNLOAD_RESOURCE% -o "clib\ssl\cert.pem" "https://curl.se/ca/cacert.pem" || EXIT /B %ERRORLEVEL%
ENDLOCAL
