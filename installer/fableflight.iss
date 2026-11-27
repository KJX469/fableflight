; the windows installer: the release archive's files installed per user
; (no admin rights, so the launcher's updater can still replace them), a
; start menu entry for the launcher and an uninstaller. built by the release
; workflow: iscc /DAppVersion=1.2.0 /DSrcDir=<unpacked archive> /DOutName=<name> fableflight.iss

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef SrcDir
  #define SrcDir "..\target\release"
#endif
#ifndef OutName
  #define OutName "fableflight-setup"
#endif

[Setup]
AppId={{6F3B2A51-9C47-4C1E-8E2B-FAB1EF1A9001}
AppName=Fable Flight
AppVersion={#AppVersion}
AppPublisher=Fable Flight contributors
AppPublisherURL=https://github.com/Rolandjg/fableflight
DefaultDirName={localappdata}\Programs\Fable Flight
DefaultGroupName=Fable Flight
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputBaseFilename={#OutName}
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
LicenseFile={#SrcDir}\LICENSE
UninstallDisplayIcon={app}\fableflight-launcher.exe

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "{#SrcDir}\fableflight-launcher.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\fablegame.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\fableflight-mcp.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SrcDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Fable Flight"; Filename: "{app}\fableflight-launcher.exe"
Name: "{autodesktop}\Fable Flight"; Filename: "{app}\fableflight-launcher.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\fableflight-launcher.exe"; Description: "{cm:LaunchProgram,Fable Flight}"; Flags: nowait postinstall skipifsilent
