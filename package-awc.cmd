@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0package-awc.ps1" %*
exit /b %errorlevel%
