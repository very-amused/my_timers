!include "MUI.nsh"

!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Name "my_timers"
OutFile "my_timers-installer-x86_64.exe"
InstallDir "$PROGRAMFILES64\my_timers"
ShowInstDetails show

Section "my_timers"
	SetOutPath $INSTDIR
	File README.md
	File LICENSE
	CreateDirectory $INSTDIR\bin
	File /oname=$INSTDIR\bin\my_timers.exe my_timers.exe
	WriteUninstaller $INSTDIR\uninstaller.exe
SectionEnd

Section "uninstall"
	Delete $INSTDIR\bin\my_timers.exe
	RMDir $INSTDIR\bin
	Delete $INSTDIR\LICENSE
	Delete $INSTDIR\README.md
	Delete $INSTDIR\uninstaller.exe
	RMDir $INSTDIR
SectionEnd
