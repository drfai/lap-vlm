param([Parameter(Mandatory = $true)][string]$MsiPath)
$ErrorActionPreference = 'Stop'
$Installer = New-Object -ComObject WindowsInstaller.Installer
$Database = $Installer.OpenDatabase((Get-Item $MsiPath).FullName, 0)

function Query-Row([string]$Sql) {
    $View = $Database.OpenView($Sql)
    # COM calls can emit null values; keep only the fetched record in the output.
    $null = $View.Execute()
    $Record = $View.Fetch()
    $null = $View.Close()
    return $Record
}

# Verify the fragment survives linking into the MSI, rather than just compiling.
$Condition = Query-Row 'SELECT `Condition`, `Description` FROM `LaunchCondition` WHERE `Condition` = ''REMOVE~="ALL" OR LAP_VC_RUNTIME_AVAILABLE'''
if (-not $Condition -or $Condition.StringData(2) -notlike 'Lap requires Microsoft Visual C++ Runtime*') {
    throw 'MSI is missing the VC++ runtime launch condition.'
}
$Search = Query-Row 'SELECT `Signature_` FROM `AppSearch` WHERE `Property` = ''LAP_VC_RUNTIME_AVAILABLE'''
if (-not $Search) { throw 'MSI is missing the VC++ runtime file search.' }
$Signature = Query-Row 'SELECT `FileName`, `MinVersion` FROM `Signature` WHERE `Signature` = ''LapVcRuntimeDll'''
if (-not $Signature -or $Signature.StringData(1) -ne 'msvcp140.dll') {
    throw 'MSI is missing the versioned VC++ DLL signature.'
}
$Directory = Query-Row 'SELECT `Path` FROM `DrLocator` WHERE `Signature_` = ''LapVcRuntimeDirectory'''
if (-not $Directory -or $Directory.StringData(1) -ne '[System64Folder]') {
    throw 'MSI runtime search does not use the 64-bit system directory.'
}
$Expected = if ($env:VCToolsVersion) { [version]$env:VCToolsVersion.Trim() } else { [version]'14.40.33810.0' }
$Actual = [version]$Signature.StringData(2)
if ($Actual -lt $Expected) {
    throw "MSI requires an older runtime than its toolset: MSI minimum=$Actual, toolset=$Expected, exported minimum=$env:TAURI_LAP_VC_RUNTIME_MIN_VERSION."
}

# Evaluate the actual compiled condition with Windows Installer, without installing.
$Session = $Installer.OpenPackage((Get-Item $MsiPath).FullName, 0)
foreach ($Case in @(
    @{ Available = ''; Remove = ''; Installed = ''; Expected = 0 },
    @{ Available = ''; Remove = ''; Installed = '1'; Expected = 0 },
    @{ Available = 'C:\Windows\System32\msvcp140.dll'; Remove = ''; Installed = ''; Expected = 1 },
    @{ Available = ''; Remove = 'ALL'; Installed = '1'; Expected = 1 }
)) {
    $Session.Property('LAP_VC_RUNTIME_AVAILABLE') = $Case.Available
    $Session.Property('REMOVE') = $Case.Remove
    $Session.Property('Installed') = $Case.Installed
    if ($Session.EvaluateCondition($Condition.StringData(1)) -ne $Case.Expected) {
        throw 'MSI condition incorrectly handles installation or uninstall.'
    }
}
Write-Host "MSI VC++ runtime check verified: $($Signature.StringData(2))"
