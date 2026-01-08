; =============================================================================
; IFA-LANG INSTALLER - Inno Setup Script
; The Yoruba Programming Language (Rust Edition)
; =============================================================================
; Packages the pre-built Rust binary - NO runtime dependencies!
; =============================================================================

#define MyAppName "Ifa-Lang"
#define MyAppVersion "1.1.0"
#define MyAppPublisher "Ayomide Alli"
#define MyAppURL "https://github.com/AAEO04/ifa-lang"
#define MyAppExeName "ifa.exe"

[Setup]
; App identification
AppId={{A1B2C3D4-E5F6-7890-ABCD-EF1234567890}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}/issues
AppUpdatesURL={#MyAppURL}/releases

; Installation settings
DefaultDirName={autopf}\ifa-lang
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=LICENSE
OutputDir=dist
OutputBaseFilename=ifa-lang-{#MyAppVersion}-windows-setup
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

; Privileges - allow non-admin install
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; Minimum Windows version
MinVersion=10.0

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "addtopath"; Description: "Add Ifa-Lang to PATH (required)"; GroupDescription: "Environment Configuration:"; Flags: checkedonce

[Files]
; === RUST BINARY - Single executable, no dependencies! ===
Source: "target\release\ifa.exe"; DestDir: "{app}\bin"; Flags: ignoreversion

; Standard library (Ifa files)
Source: "lib\std\*.ifa"; DestDir: "{app}\lib\std"; Flags: ignoreversion recursesubdirs; Check: DirExists(ExpandConstant('{src}\lib\std'))

; Examples
Source: "examples\*.ifa"; DestDir: "{app}\examples"; Flags: ignoreversion recursesubdirs skipifsourcedoesntexist

; Documentation
Source: "docs\index.html"; DestDir: "{app}\docs"; Flags: ignoreversion skipifsourcedoesntexist
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "TUTORIAL.md"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Ifa-Lang Command Line"; Filename: "{cmd}"; Parameters: "/k cd /d ""{app}"" && ifa --help"
Name: "{group}\Documentation"; Filename: "{app}\docs\index.html"
Name: "{group}\Tutorial"; Filename: "{app}\TUTORIAL.md"
Name: "{group}\Uninstall Ifa-Lang"; Filename: "{uninstallexe}"
Name: "{userdesktop}\Ifa-Lang"; Filename: "{cmd}"; Parameters: "/k cd /d ""{app}"" && ifa repl"; Comment: "Ifa-Lang REPL"

[Registry]
; Add to PATH for current user
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\bin"; Check: NeedsAddPath('{app}\bin'); Tasks: addtopath

[Run]
; Verify installation
Filename: "{app}\bin\ifa.exe"; Parameters: "--version"; Description: "Verify installation"; Flags: postinstall nowait skipifsilent runhidden

[UninstallDelete]
Type: filesandordirs; Name: "{app}\*.ifab"
Type: filesandordirs; Name: "{app}\target"

[Messages]
; =============================================================================
; SIMPLIFIED MESSAGES (No box art, Linus-approved)
; =============================================================================

WelcomeLabel1=Welcome to Ifa-Lang
WelcomeLabel2=You are about to install [name/ver] - The Yoruba Programming Language.%n%nThis is a native Rust binary - no runtime dependencies required!%n%nThe 16 Odu Domains provide:%n  - System operations (Ogbe)%n  - Math operations (Obara, Oturupon)%n  - String manipulation (Ika)%n  - File I/O (Odi)%n  - Networking (Otura)%n  - And more...

FinishedHeadingLabel=Installation Complete!
FinishedLabelNoIcons=[name] has been installed.%n%nOpen a new terminal and run:%n%n   ifa --help      Show all commands%n   ifa run app.ifa Run a program%n   ifa repl        Interactive REPL%n%nVS Code Extension: Search "Ifa-Lang" in Extensions Marketplace
FinishedLabel=[name] has been installed.%n%nOpen a new terminal and run:%n%n   ifa --help      Show all commands%n   ifa run app.ifa Run a program%n   ifa repl        Interactive REPL%n%nVS Code Extension: Search "Ifa-Lang" in Extensions Marketplace

SelectDirLabel3=Select installation folder:
SelectDirBrowseLabel=Install to:

ReadyLabel1=Ready to Install
ReadyLabel2a=Click Install to begin.
ReadyLabel2b=Review your settings, then click Install.

PreparingLabel=Preparing installation...
InstallingLabel=Installing files...
ExtractingLabel=Extracting...

StatusUninstalling=Removing Ifa-Lang...

ExitSetupTitle=Exit Installation
ExitSetupMessage=Installation is not complete. Exit anyway?

[Code]
// Check if path needs to be added
function NeedsAddPath(Param: string): boolean;
var
  OrigPath: string;
begin
  if not RegQueryStringValue(HKEY_CURRENT_USER, 'Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  Result := Pos(';' + Param + ';', ';' + OrigPath + ';') = 0;
end;

function DirExists(Path: string): boolean;
begin
  Result := DirExists(Path);
end;

function InitializeSetup(): Boolean;
begin
  Result := True;
end;

// Show post-install message
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    MsgBox('Installation Complete!' + #13#10 + #13#10 +
           'Ifa-Lang is now installed.' + #13#10 + #13#10 +
           'IMPORTANT: Open a NEW terminal window, then run:' + #13#10 +
           '   ifa --help' + #13#10 + #13#10 +
           'The PATH was updated but requires a new terminal session.',
           mbInformation, MB_OK);
  end;
end;
