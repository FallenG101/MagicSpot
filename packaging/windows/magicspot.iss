; The Windows installer, built with Inno Setup 6.3 or later from a release
; binary (the release workflow does this on every tag):
;
;   iscc /DVersion=0.1.4 /DArch=x86_64 /DBinary=...\magicspot.exe ^
;        /DOutputDir=dist packaging\windows\magicspot.iss
;
; Arch is x86_64 or aarch64, as in the Rust target triple, so the installer
; is named like the zip next to it. It needs no administrator rights: the
; program goes to the user's own Programs folder with a Start menu entry,
; and a running copy is closed before an update replaces it.

#ifndef Version
  #error Version must be defined on the ISCC command line
#endif
#ifndef Arch
  #error Arch must be defined on the ISCC command line (x86_64 or aarch64)
#endif
#ifndef Binary
  #error Binary must be defined on the ISCC command line
#endif
#ifndef OutputDir
  #error OutputDir must be defined on the ISCC command line
#endif
#if Arch == "aarch64"
  #define InnoArch "arm64"
#else
  #define InnoArch "x64compatible"
#endif

#define AppName "MagicSpot"
#define AppExeName "magicspot.exe"

[Setup]
; Never change: this is how Windows tells an update from a new program.
AppId={{F7FE6CFD-2D98-48B8-A040-C9976412D8B6}
AppName={#AppName}
AppVersion={#Version}
AppVerName={#AppName} {#Version}
AppPublisher=MagicSpot
DefaultDirName={localappdata}\Programs\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed={#InnoArch}
ArchitecturesInstallIn64BitMode={#InnoArch}
MinVersion=10.0
LicenseFile=..\..\LICENSE
OutputDir={#OutputDir}
OutputBaseFilename=magicspot-v{#Version}-{#Arch}-pc-windows-msvc-setup
SetupIconFile=magicspot.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no
UninstallDisplayIcon={app}\{#AppExeName}
; The file version has to be numbers: a release candidate's -rc1 comes off.
#define Dash Pos("-", Version)
#if Dash > 0
  #define NumericVersion Copy(Version, 1, Dash - 1)
#else
  #define NumericVersion Version
#endif
VersionInfoVersion={#NumericVersion}.0

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
Source: "{#Binary}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExeName}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExeName}"; Tasks: desktopicon

[Registry]
; Spotify links (spotify:track:…) open in MagicSpot. Registered for this
; user only, like the program itself. The official client registers the same
; scheme when it is installed; whichever was set up last has the links, and
; Settings > Apps > Default apps can hand them to the other, where MagicSpot
; is listed through the capabilities below.
Root: HKCU; Subkey: "Software\Classes\spotify"; ValueType: string; ValueName: ""; ValueData: "URL:Spotify link"
Root: HKCU; Subkey: "Software\Classes\spotify"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\spotify\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeName}"",0"
Root: HKCU; Subkey: "Software\Classes\spotify\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeName}"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\MagicSpot.spotify"; ValueType: string; ValueName: ""; ValueData: "URL:Spotify link"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\MagicSpot.spotify"; ValueType: string; ValueName: "URL Protocol"; ValueData: ""
Root: HKCU; Subkey: "Software\Classes\MagicSpot.spotify\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeName}"",0"
Root: HKCU; Subkey: "Software\Classes\MagicSpot.spotify\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExeName}"" ""%1"""
Root: HKCU; Subkey: "Software\{#AppName}\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#AppName}"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\{#AppName}\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "A native Spotify client"
Root: HKCU; Subkey: "Software\{#AppName}\Capabilities\URLAssociations"; ValueType: string; ValueName: "spotify"; ValueData: "MagicSpot.spotify"
Root: HKCU; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "{#AppName}"; ValueData: "Software\{#AppName}\Capabilities"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#AppExeName}"; Description: "Launch {#AppName}"; Flags: nowait postinstall skipifsilent

[Code]
// The spotify: scheme key is shared with whatever else opens the links, so
// uninstalling takes it away only while it still names this program.
procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
var
  Command: String;
  Exe: String;
begin
  if CurUninstallStep <> usUninstall then
    Exit;
  if not RegQueryStringValue(HKCU, 'Software\Classes\spotify\shell\open\command', '', Command) then
    Exit;
  Exe := Lowercase(ExpandConstant('{app}\{#AppExeName}'));
  if Pos(Exe, Lowercase(Command)) > 0 then
    RegDeleteKeyIncludingSubkeys(HKCU, 'Software\Classes\spotify');
end;
