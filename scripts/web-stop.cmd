@echo off
rem Lance "scripts/web.sh stop" dans WSL, depuis la racine du projet.
wsl.exe --cd "%~dp0.." bash scripts/web.sh stop
pause
