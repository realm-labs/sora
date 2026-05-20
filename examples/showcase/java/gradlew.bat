@echo off
setlocal
set PROJECT_DIR=%~dp0.
call "%PROJECT_DIR%\..\kotlin\gradlew.bat" -p "%PROJECT_DIR%" %*
exit /b %ERRORLEVEL%
