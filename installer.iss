[Setup]
AppName=Robocopy+
AppVersion=0.1.0-beta.1
AppPublisher=dcryptoniun
DefaultDirName={autopf}\Robocopy+
DefaultGroupName=Robocopy+
OutputDir=Output
OutputBaseFilename=RobocopyPlus_Setup
Compression=lzma
SolidCompression=yes
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
DisableProgramGroupPage=yes

[Files]
Source: "target\release\robocpyplusgui.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Robocopy+"; Filename: "{app}\robocpyplusgui.exe"
Name: "{autodesktop}\Robocopy+"; Filename: "{app}\robocpyplusgui.exe"; Tasks: desktopicon

[Tasks]
Name: "desktopicon"; Description: "Create a &desktop icon"; GroupDescription: "Additional icons:"
