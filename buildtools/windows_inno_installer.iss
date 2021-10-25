; Script generated by the Inno Script Studio Wizard.
; SEE THE DOCUMENTATION FOR DETAILS ON CREATING INNO SETUP SCRIPT FILES!

; Notes:
;  (1) To run this script from the command line, prepare the following:
;      - Install Microsoft Windows SDK 10 (https://developer.microsoft.com/en-us/windows/downloads/windows-10-sdk/)
;      - Add the Microsoft Windows SDK 10 folder where "signtool.exe" is located to the path,
;        e.g. "C:\Program Files (x86)\Windows Kits\10\bin\10.0.18362.0\x64"
;      - Install Inno Setup (https://jrsoftware.org/isinfo.php)
;  (2) To run this script from the command line with Inno Setup console-mode compiler:
;      - "<path to console-mode compiler>\ISCC.exe" "/SSignTool=signtool sign
;         /tr http://timestamp.digicert.com /f "<path and filename of the certificate>"
;         /p <password used to create the certificate> $f"
;         "/DMyAppVersion=<base node version>" "windows_inno_installer.iss"
;  (3) To configure sign tools for Inno Script Studio:
;      - with 'Tools -> Configure Sign Tools...' (Example configuration), add
;        Name:
;          SignTool
;        Command:
;          signtool sign /tr http://timestamp.digicert.com
;          /f "<path and filename of the certificate>"
;          /p "<password used to create the certificate>" $f


#define MyOrgName "Tari"
;#define MyAppVersion "?.?.?-???????-release"
#define MyAppPublisher "The Tari Development Community"
#define MyAppURL "https://github.com/tari-project/tari"
#define MyAppSupp "Tari Website"
#define MyAppSuppURL "http://www.tari.com"
#define AllName "All"
#define AllExeName "start_all.bat"
#define BaseNodeName "Base Node"
#define BaseNodeExeName "start_tari_base_node.bat"
#define ConsoleWalletName "Console Wallet"
#define ConsoleWalletExeName "start_tari_console_wallet.bat"
#define MiningNodeName "Mining Node"
#define MiningNodeExeName "start_tari_mining_node.bat"
#define TorServicesName "Tor Services"
#define TorServicesExeName "start_tor.bat"
#define MergeMiningProxyName "Merge Mining Proxy"
#define MergeMiningProxyExeName "start_tari_merge_mining_proxy.bat"
#define MergeMiningName "XMRig"
#define MergeMiningExeName "start_xmrig.bat"
#define ReadmeName "README.txt"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{35C6E863-EDE5-4CBD-A824-E1418ECB1D1D}
AppName={#MyOrgName} {#BaseNodeName}
AppVersion={#MyAppVersion}
;AppVerName={#BaseNodeName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppSuppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={userdocs}\..\.tari-testnet
DefaultGroupName={#MyOrgName} - Testnet
AllowNoIcons=yes
LicenseFile=..\LICENSE
OutputBaseFilename=tari_base_node-Windows-{#MyAppVersion}
SetupIconFile=.\tari_logo_black.ico
Compression=lzma
SolidCompression=yes
MinVersion=0,6.1
VersionInfoCompany=The Tari Developer Community
VersionInfoProductName=tari_base_node
InfoAfterFile="..\applications\tari_base_node\windows\README.md"
;SignTool=SignTool

PrivilegesRequired=none

[CustomMessages]
TariGit=Tari on GitHub
TariWeb=Tari on the web

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}";
Name: "quicklaunchicon"; Description: "{cm:CreateQuickLaunchIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked; OnlyBelowVersion: 0,6.1

[PreCompile]
Name: ".\generate_config.bat"; Flags: abortonerror cmdprompt redirectoutput

[Files]
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.md"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\README.md"; DestDir: "{app}"; DestName: "README.txt"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\start_all.lnk"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\start_tari_base_node.lnk"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\applications\tari_console_wallet\windows\start_tari_console_wallet.lnk"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\applications\tari_mining_node\windows\start_tari_mining_node.lnk"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\applications\tari_merge_mining_proxy\windows\start_tari_merge_mining_proxy.lnk"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\applications\tari_merge_mining_proxy\windows\start_xmrig.lnk"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\start_tor.lnk"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\target\release\tari_base_node.exe"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\target\release\tari_console_wallet.exe"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\target\release\tari_mining_node.exe"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\target\release\tari_merge_mining_proxy.exe"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\runtime\start_all.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\runtime\start_tor.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\runtime\source_base_node_env.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_base_node\windows\runtime\start_tari_base_node.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_console_wallet\windows\runtime\source_console_wallet_env.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_console_wallet\windows\runtime\start_tari_console_wallet.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_mining_node\windows\runtime\source_mining_node_env.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_mining_node\windows\runtime\start_tari_mining_node.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_merge_mining_proxy\windows\runtime\source_merge_mining_proxy_env.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_merge_mining_proxy\windows\runtime\start_tari_merge_mining_proxy.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_merge_mining_proxy\windows\runtime\source_xmrig_env.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\applications\tari_merge_mining_proxy\windows\runtime\start_xmrig.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: ".\tari_config_example.toml"; DestDir: "{app}\config"; DestName: "config.toml"; Flags: ignoreversion
Source: "tari_logo_purple.ico"; DestDir: "{userdocs}\..\temp\tari_icons"; Flags: ignoreversion
Source: "tor.ico"; DestDir: "{userdocs}\..\temp\tari_icons"; Flags: ignoreversion
Source: "xmr_logo.ico"; DestDir: "{userdocs}\..\temp\tari_icons"; Flags: ignoreversion
Source: "install_tor_services.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "install_vs2019_redist.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "install_xmrig.bat"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "get_xmrig_win.ps1"; DestDir: "{app}\runtime"; Flags: ignoreversion
Source: "..\common\xmrig_config\config_example_stagenet.json"; DestDir: "{app}\config"; DestName: "xmrig_config_example_stagenet.json"; Flags: ignoreversion
Source: "..\common\xmrig_config\config_example_mainnet.json"; DestDir: "{app}\config"; DestName: "xmrig_config_example_mainnet.json"; Flags: ignoreversion
Source: "..\common\xmrig_config\config_example_mainnet_self_select.json"; DestDir: "{app}\config"; DestName: "xmrig_config_example_mainnet_self_select.json"; Flags: ignoreversion

[Icons]
Name: "{group}\Start {#AllName}"; Filename: "{app}\runtime\{#AllExeName}"; WorkingDir: "{app}"
Name: "{group}\Start {#BaseNodeName}"; Filename: "{app}\runtime\{#BaseNodeExeName}"; WorkingDir: "{app}"
Name: "{group}\Start {#ConsoleWalletName}"; Filename: "{app}\runtime\{#ConsoleWalletExeName}"; WorkingDir: "{app}"
Name: "{group}\Start {#MiningNodeName}"; Filename: "{app}\runtime\{#MiningNodeExeName}"; WorkingDir: "{app}"
Name: "{group}\Start {#MergeMiningProxyName}"; Filename: "{app}\runtime\{#MergeMiningProxyExeName}"; WorkingDir: "{app}"
Name: "{group}\Start {#MergeMiningName}"; Filename: "{app}\runtime\{#MergeMiningExeName}"; WorkingDir: "{app}"
Name: "{group}\Start {#TorServicesName}"; Filename: "{app}\runtime\{#TorServicesExeName}"; WorkingDir: "{app}"
Name: "{group}\{#ReadmeName}"; Filename: "{app}\{#ReadmeName}"; WorkingDir: "{app}"
Name: "{group}\{cm:ProgramOnTheWeb,{#BaseNodeName}}"; Filename: "{#MyAppURL}"
Name: "{group}\{cm:TariWeb,{#MyAppSupp}}"; Filename: "{#MyAppSuppURL}"
Name: "{group}\{cm:UninstallProgram,{#MyOrgName} - Testnet}"; Filename: "{uninstallexe}"
Name: "{userdesktop}\{#MyOrgName} {#AllName}"; Filename: "{app}\runtime\{#AllExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userdesktop}\{#MyOrgName} {#BaseNodeName}"; Filename: "{app}\runtime\{#BaseNodeExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userdesktop}\{#MyOrgName} {#ConsoleWalletName}"; Filename: "{app}\runtime\{#ConsoleWalletExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userdesktop}\{#MyOrgName} {#MiningNodeName}"; Filename: "{app}\runtime\{#MiningNodeExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userdesktop}\{#MyOrgName} {#MergeMiningProxyName}"; Filename: "{app}\runtime\{#MergeMiningProxyExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userdesktop}\{#MyOrgName} {#MergeMiningName}"; Filename: "{app}\runtime\{#MergeMiningExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userdesktop}\{#MyOrgName} - {#TorServicesName}"; Filename: "{app}\runtime\{#TorServicesExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
Name: "{userappdata}\Microsoft\Internet Explorer\Quick Launch\{#BaseNodeName}"; Filename: "{app}\{#BaseNodeExeName}"; Tasks: quicklaunchicon

;[Setup]
;PrivilegesRequired=admin

[Run]
Filename: "{app}\runtime\install_tor_services.bat"; Parameters: "NO_PAUSE"; Flags: runascurrentuser postinstall; Description: "Install Tor Services"
Filename: "{app}\runtime\install_xmrig.bat"; Parameters: "NO_PAUSE"; Flags: runascurrentuser postinstall; Description: "Install XMRig"
Filename: "{app}\runtime\install_vs2019_redist.bat"; Parameters: "NO_PAUSE"; Flags: runascurrentuser postinstall; Description: "Install Redistributable for Visual Studio 2019"

[InstallDelete]
Type: filesandordirs; Name: "{app}\config"
Type: filesandordirs; Name: "{app}\log"
Type: filesandordirs; Name: "{app}\runtime"
Type: files; Name: "{app}\LICENSE.md"
Type: files; Name: "{app}\LICENSE.txt"
Type: files; Name: "{app}\README.md"
Type: files; Name: "{app}\README.txt"
Type: files; Name: "{app}\start_all.lnk"
Type: files; Name: "{app}\start_tari_base_node.lnk"
Type: files; Name: "{app}\start_tari_console_wallet.lnk"
Type: files; Name: "{app}\start_tari_mining_node.lnk"
Type: files; Name: "{app}\start_tari_merge_mining_proxy.lnk"
Type: files; Name: "{app}\start_xmrig.lnk"
Type: files; Name: "{app}\start_tor.lnk"
Type: files; Name: "{userdesktop}\Tari All.lnk"
Type: files; Name: "{userdesktop}\Tari Base Node.lnk"
Type: files; Name: "{userdesktop}\Tari Console Wallet.lnk"
Type: files; Name: "{userdesktop}\Tari Mining Node.lnk"
Type: files; Name: "{userdesktop}\Tari Merge Mining Proxy.lnk"
Type: files; Name: "{userdesktop}\Tari XMRig.lnk"
Type: files; Name: "{userdesktop}\Tari - Tor Services.lnk"

[UninstallDelete]
Type: filesandordirs; Name: "{app}\config"
Type: filesandordirs; Name: "{app}\log"
Type: filesandordirs; Name: "{app}\runtime"
