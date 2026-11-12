#define MyAppName "TODDY PC CONTROL"
#define MyAppVersion "1.0.0"
#define MyAppExeName "pc-remote-control.exe"

[Setup]
AppId={{A7D31E82-5B49-4F0C-9A61-2D8E7C31B405}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=TODDY PC CONTROL
DefaultDirName={localappdata}\Programs\TODDY PC CONTROL
DefaultGroupName=TODDY PC CONTROL
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
OutputDir=dist-installer
OutputBaseFilename=TODDY-PC-CONTROL-Setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
Uninstallable=yes

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"

[Files]
Source: "target\release\pc-remote-control.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\TODDY PC CONTROL"; Filename: "{app}\pc-remote-control.exe"
Name: "{autodesktop}\TODDY PC CONTROL"; Filename: "{app}\pc-remote-control.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\pc-remote-control.exe"; Description: "Launch TODDY PC CONTROL"; Flags: nowait postinstall skipifsilent
