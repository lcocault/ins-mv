@echo off
rem Lance "scripts/web.sh start" dans WSL, depuis la racine du projet.
wsl.exe --cd "%~dp0.." bash scripts/web.sh start
pause
