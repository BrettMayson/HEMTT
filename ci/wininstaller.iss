#define target GetEnv('TARGET')

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{CD09919A-074B-41A5-910E-DE0730E3815A}
AppName=HEMTT
AppPublisher=SynixeBrett
AppVersion=stable
AppPublisherURL=https://github.com/SynixeBrett/HEMTT
AppSupportURL=https://github.com/SynixeBrett/HEMTT
AppUpdatesURL=https://github.com/SynixeBrett/HEMTT
DefaultDirName={pf}\HEMTT
DefaultGroupName=HEMTT
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
OutputBaseFilename=setup
Compression=lzma
SolidCompression=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "..\target\{#target}\release\hemtt.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{group}\HEMTT"; Filename: "{app}\bin\hemtt.exe"

[Registry]
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{pf}\HEMTT\bin"; Check: NeedsAddPath('{pf}\HEMTT\bin')

[Code]

function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_LOCAL_MACHINE,
    'SYSTEM\CurrentControlSet\Control\Session Manager\Environment',
    'Path', OrigPath)
  then begin
    Result := True;
    exit;
  end;
  { look for the path with leading and trailing semicolon }
  { Pos() returns 0 if not found }
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;
