@echo off 
set version=__VERSION__
set version_label=__VERSION_LABEL__
set mongodb="HKEY_LOCAL_MACHINE\SOFTWARE\MongoDB\Atlas SQL ODBC 1.7"
set odbcinst="HKEY_LOCAL_MACHINE\SOFTWARE\ODBC\ODBCINST.INI"
set driver_key="%odbcinst%\MongoDB Atlas SQL ODBC Driver"
set odbc_drivers="%odbcinst%\ODBC Drivers"


reg add %mongodb% /v "Version" /d %version% /f

reg add %odbc_drivers% /v "MongoDB Atlas SQL ODBC Driver" /d "Installed" /f
reg add %driver_key% /v Driver /d "%~dp0atsql.dll" /f
reg add %driver_key% /v Setup /d "%~dp0atsqls.dll" /f

pause