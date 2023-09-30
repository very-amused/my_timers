!include "MUI.nsh"
!include "nsDialogs.nsh"
!include "LogicLib.nsh"

!define MUI_ABORTWARNING

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
Var StartMenuFolder
!insertmacro MUI_PAGE_STARTMENU Application $StartMenuFolder
Var PathCheckbox
Var AddToPath
Page custom pathPage pathPageLeave
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

Name "my_timers"
OutFile "my_timers-installer-x86_64.exe"
InstallDir "$PROGRAMFILES64\my_timers"
ShowInstDetails show

Function pathPage
	!insertmacro MUI_HEADER_TEXT "Add to PATH" "Choose options affecting your PATH."
	nsDialogs::Create 1018
	Pop $0

	${If} $0 == error
		Abort
	${EndIf}

	${NSD_CreateCheckBox} 0 0 100% 14u "Add my_timers to your PATH"
	Pop $PathCheckbox

	${NSD_SetState} $PathCheckbox $AddToPath

	nsDialogs::Show
FunctionEnd

Function pathPageLeave
	${NSD_GetState} $PathCheckbox $AddToPath
FunctionEnd

Function .onInit
	StrCpy $AddToPath ${BST_CHECKED}
FunctionEnd

!define ARP "Software\Microsoft\Windows\CurrentVersion\Uninstall\my_timers"
!define DISPLAY_VERSION "0.1.2"

Section "my_timers"
	SetOutPath $INSTDIR
	File README.md
	File LICENSE
	CreateDirectory $INSTDIR\bin
	File /oname=$INSTDIR\bin\my_timers.exe my_timers.exe
	WriteUninstaller $INSTDIR\uninstaller.exe

	# Add uninstall registry data
	WriteRegStr HKLM "${ARP}" \
			"DisplayName" "my_timers -- MariaDB/MySQL event runner"
	WriteRegStr HKLM "${ARP}" \
			"UninstallString" "$\"$INSTDIR\uninstaller.exe$\""
	WriteRegStr HKLM "${ARP}" \
			"QuietUninstallString" "$\"$INSTDIR\uninstaller.exe$\" /S"
	WriteRegStr HKLM "${ARP}" \
			"InstallLocation" "$\"$INSTDIR$\""
	WriteRegStr HKLM "${ARP}" \
			"Publisher" "Keith Scroggs <very-amused>"
	WriteRegStr HKLM "${ARP}" \
			"DisplayVersion" "${DISPLAY_VERSION}"

	# Create shortcuts
	!insertmacro MUI_STARTMENU_WRITE_BEGIN Application
		CreateDirectory $SMPROGRAMS\$StartMenuFolder
		CreateShortcut $SMPROGRAMS\$StartMenuFolder\my_timers.lnk \
				$INSTDIR\bin\my_timers.exe
		CreateShortcut $SMPROGRAMS\$StartMenuFolder\README.md.lnk \
				$INSTDIR\README.md
	!insertmacro MUI_STARTMENU_WRITE_END

	# Add to PATH
	${If} $AddToPath == ${BST_CHECKED}
		EnVar::AddValue "Path" "$INSTDIR\bin"
		Pop $0
	${EndIf}
SectionEnd

Section "uninstall"
	Delete $INSTDIR\bin\my_timers.exe
	RMDir $INSTDIR\bin
	Delete $INSTDIR\LICENSE
	Delete $INSTDIR\README.md
	Delete $INSTDIR\uninstaller.exe
	RMDir $INSTDIR

	# Remove uninstall registry data
	DeleteRegKey HKLM "${ARP}"

	# Remove any installed shortcuts
	!insertmacro MUI_STARTMENU_GETFOLDER Application $R0
	Delete $SMPROGRAMS\$R0\my_timers.lnk
	Delete $SMPROGRAMS\$R0\README.md.lnk
	RMDir $SMPROGRAMS\$R0

	# Remove from PATH
	EnVar::DeleteValue "Path" "$INSTDIR\bin"
SectionEnd
