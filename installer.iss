; ═══════════════════════════════════════════════════════════════════════════
; IFÁ-LANG INSTALLER - Inno Setup Script
; The Yoruba Programming Language
; ═══════════════════════════════════════════════════════════════════════════
; Theme: Cowrie Shells (Owó Eyo) & Opon Ifá (Divination Board)
; Includes bundled Python - NO external Python required!
; ═══════════════════════════════════════════════════════════════════════════

#define MyAppName "Ifá-Lang"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Ayomide Alli"
#define MyAppURL "https://github.com/AAEO04/ifa-lang"
#define MyAppExeName "ifa.bat"

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
DefaultDirName=C:\ifa-lang
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=LICENSE
OutputDir=dist
OutputBaseFilename=ifa-lang-{#MyAppVersion}-windows-setup
; SetupIconFile=vscode_extension\icon.png
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

; Privileges
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "addtopath"; Description: "Add Ifá-Lang to PATH (recommended)"; GroupDescription: "Environment Configuration:"; Flags: checkedonce

[Files]
; === BUNDLED PYTHON (No external Python required!) ===
Source: "python\*"; DestDir: "{app}\python"; Flags: ignoreversion recursesubdirs

; Core files
Source: "bin\*"; DestDir: "{app}\bin"; Flags: ignoreversion recursesubdirs
Source: "src\*.py"; DestDir: "{app}\src"; Flags: ignoreversion recursesubdirs
Source: "lib\std\*.py"; DestDir: "{app}\lib\std"; Flags: ignoreversion recursesubdirs
Source: "lib\ext\*.py"; DestDir: "{app}\lib\ext"; Flags: ignoreversion recursesubdirs
Source: "lib\Cargo.toml"; DestDir: "{app}\lib"; Flags: ignoreversion
Source: "lib\core.rs"; DestDir: "{app}\lib"; Flags: ignoreversion

; Examples
Source: "examples\*"; DestDir: "{app}\examples"; Flags: ignoreversion recursesubdirs

; Documentation
Source: "docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs
Source: "README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "DOCS.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "TUTORIAL.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\Ifá-Lang Documentation"; Filename: "{app}\docs\index.html"
Name: "{group}\Tutorial"; Filename: "{app}\TUTORIAL.md"
Name: "{group}\Command Prompt Here"; Filename: "{cmd}"; Parameters: "/k cd /d ""{app}"""
Name: "{group}\Uninstall Ifá-Lang"; Filename: "{uninstallexe}"

[Registry]
; Add to PATH for current user
Root: HKCU; Subkey: "Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\bin"; Tasks: addtopath; Check: NeedsAddPath('{app}\bin')

[Run]
; Install Python dependencies to bundled Python
Filename: "{app}\python\python.exe"; Parameters: "-m pip install --quiet --no-warn-script-location -r ""{app}\requirements.txt"""; Description: "Installing dependencies..."; StatusMsg: "The shells are aligning..."; Flags: runhidden; Check: FileExists(ExpandConstant('{app}\requirements.txt'))

[UninstallDelete]
Type: filesandordirs; Name: "{app}\__pycache__"
Type: filesandordirs; Name: "{app}\src\__pycache__"
Type: filesandordirs; Name: "{app}\lib\std\__pycache__"
Type: filesandordirs; Name: "{app}\python\Lib\site-packages"

[Messages]
; ═══════════════════════════════════════════════════════════════════════════
; COWRIE & OPON THEMED MESSAGES
; ═══════════════════════════════════════════════════════════════════════════

WelcomeLabel1=Welcome to the Opon Ifa
WelcomeLabel2=You are about to install [name/ver] - The Yoruba Programming Language.%n%nThis installer includes everything needed - no external Python required!%n%n%nThe 16 Odu Domains:%n%n   Ogbe (System)       Oyeku (Exit)%n   Iwori (Time)        Odi (Files)%n   Irosu (Output)      Owonrin (Random)%n   Obara (Math+)       Okanran (Errors)%n   Ogunda (Arrays)     Osa (Concurrency)%n   Ika (Strings)       Oturupon (Math-)%n   Otura (Network)     Irete (Crypto)%n   Ose (Graphics)      Ofun (Permissions)%n%nAse! (So be it!)

FinishedHeadingLabel=Ase! The Installation is Complete!
FinishedLabelNoIcons=[name] has been installed on your computer.%n%n%nQuick Start:%n%n   ifa --help          Show all commands%n   ifa run hello.ifa   Run a program%n   ifa repl            Interactive REPL%n   ifa build app.ifa   Compile to native%n%n%nVS Code Extension:%n   Search "Ifa-Lang" in Extensions%n%n%nRestart your terminal for PATH changes.%n%nMay the Odu guide your code!
FinishedLabel=[name] has been installed on your computer.%n%n%nQuick Start:%n%n   ifa --help          Show all commands%n   ifa run hello.ifa   Run a program%n   ifa repl            Interactive REPL%n   ifa build app.ifa   Compile to native%n%n%nVS Code Extension:%n   Search "Ifa-Lang" in Extensions%n%n%nRestart your terminal for PATH changes.%n%nMay the Odu guide your code!

SelectDirLabel3=The Opon will be placed in the following folder.
SelectDirBrowseLabel=Select where to install (location):

ReadyLabel1=The Cowries Are Ready to Cast
ReadyLabel2a=The Opon Ifa is prepared. Click Install to begin and install [name].
ReadyLabel2b=The Opon Ifa is prepared. Review your settings, then click Install to begin.

PreparingLabel=Preparing the Opon Ifa...
InstallingLabel=The shells are falling into place...
ExtractingLabel=Extracting the wisdom of the Odu...

StatusUninstalling=The shells return to their pouch...

ExitSetupTitle=Exit Installation
ExitSetupMessage=The installation is not complete. Are you sure you want to leave?

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

// No Python check needed - we bundle it!
function InitializeSetup(): Boolean;
begin
  Result := True;
end;

// Notify user to restart terminal
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    MsgBox('Ase! Installation Complete!' + #13#10 + #13#10 +
           'Ifa-Lang has been installed with its own Python runtime.' + #13#10 +
           'No external Python installation required!' + #13#10 + #13#10 +
           'IMPORTANT: Restart your terminal for PATH changes.' + #13#10 + #13#10 +
           'Then run:  ifa --help' + #13#10 + #13#10 +
           'May the 16 Odu guide your code!', mbInformation, MB_OK);
  end;
end;
