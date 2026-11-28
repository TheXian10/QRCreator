; Instalador de Windows (Inno Setup 6).
; Se compila en GitHub Actions con:
;   ISCC.exe /DAppVersion=2.0.0 packaging\windows\installer.iss
; Las rutas son relativas a esta carpeta.

#ifndef AppVersion
  #define AppVersion "2.2.0"
#endif

[Setup]
AppId={{7E1F3C52-9B4A-4D8E-A6B1-2C5D9F0E8A31}
AppName=QR Creator
AppVersion={#AppVersion}
AppVerName=QR Creator {#AppVersion}
AppPublisher=TheXian10
AppPublisherURL=https://github.com/TheXian10/QRCreator
AppSupportURL=https://github.com/TheXian10/QRCreator/issues
DefaultDirName={autopf}\QR Creator
DefaultGroupName=QR Creator
DisableProgramGroupPage=yes
; Se instala sin pedir permisos de administrador (para el usuario actual);
; el instalador ofrece instalar para todos si se prefiere.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
OutputDir=..\..\dist
OutputBaseFilename=QRCreator-{#AppVersion}-Windows-Instalador
SetupIconFile=..\icons\icon.ico
UninstallDisplayIcon={app}\icon.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\release\creador-qr.exe"; DestDir: "{app}"; DestName: "QRCreator.exe"; Flags: ignoreversion
Source: "..\icons\icon.ico"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\QR Creator"; Filename: "{app}\QRCreator.exe"; IconFilename: "{app}\icon.ico"
Name: "{autodesktop}\QR Creator"; Filename: "{app}\QRCreator.exe"; IconFilename: "{app}\icon.ico"; Tasks: desktopicon

[Run]
Filename: "{app}\QRCreator.exe"; Description: "{cm:LaunchProgram,QR Creator}"; Flags: nowait postinstall skipifsilent
