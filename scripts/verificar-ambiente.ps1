<##
.SYNOPSIS
Exibe um inventário somente leitura das ferramentas locais necessárias ao MediaToolkit.

.DESCRIPTION
O script consulta versões e disponibilidade; não instala, altera ou envia dados.
##>

Set-StrictMode -Version Latest

function Get-ToolStatus {
    param(
        [Parameter(Mandatory)]
        [string]$Name,

        [string[]]$VersionArguments = @('--version')
    )

    $commandInfo = Get-Command $Name -ErrorAction SilentlyContinue | Select-Object -First 1
    $executable = if ($null -ne $commandInfo) { $commandInfo.Source } else { $null }
    $foundOutsidePath = $false

    if ($null -eq $executable -and $Name -in @('rustup', 'rustc', 'cargo')) {
        $cargoExecutable = Join-Path (Join-Path $env:USERPROFILE '.cargo\bin') "$Name.exe"
        if (Test-Path -LiteralPath $cargoExecutable) {
            $executable = $cargoExecutable
            $foundOutsidePath = $true
        }
    }

    if ($null -eq $executable) {
        return [PSCustomObject]@{
            Tool    = $Name
            Status  = 'ausente do PATH'
            Version = ''
        }
    }

    if ($Name -eq 'npm') {
        $npmCmd = Get-Command npm.cmd -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($null -ne $npmCmd) {
            $executable = $npmCmd.Source
        }
    }

    $versionLines = & $executable @VersionArguments 2>&1
    $exitCode = $LASTEXITCODE
    $versionOutput = ($versionLines | Select-Object -First 1) -join ''
    $status = if ($exitCode -eq 0) {
        if ($foundOutsidePath) { 'ok (fora do PATH)' } else { 'ok' }
    } else {
        "exit $exitCode"
    }

    return [PSCustomObject]@{
        Tool    = $Name
        Status  = $status
        Version = $versionOutput
    }
}

$windows = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
Write-Output "Windows: $($windows.ProductName) $($windows.DisplayVersion) (build $($windows.CurrentBuild).$($windows.UBR))"
Write-Output "Arquitetura: $([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture)"
Write-Output ''

@(
    Get-ToolStatus -Name 'git'
    Get-ToolStatus -Name 'node'
    Get-ToolStatus -Name 'npm'
    Get-ToolStatus -Name 'rustup' -VersionArguments @('-V')
    Get-ToolStatus -Name 'rustc' -VersionArguments @('-V')
    Get-ToolStatus -Name 'cargo' -VersionArguments @('-V')
    Get-ToolStatus -Name 'ffmpeg'
    Get-ToolStatus -Name 'ffprobe'
    Get-ToolStatus -Name 'yt-dlp'
) | Format-Table -AutoSize

$rustupCommand = Get-Command rustup -ErrorAction SilentlyContinue | Select-Object -First 1
$rustupExecutable = if ($null -ne $rustupCommand) { $rustupCommand.Source } else { $null }
if ($null -eq $rustupExecutable) {
    $rustupFallback = Join-Path (Join-Path $env:USERPROFILE '.cargo\bin') 'rustup.exe'
    if (Test-Path -LiteralPath $rustupFallback) {
        $rustupExecutable = $rustupFallback
    }
}

if ($null -ne $rustupExecutable) {
    Write-Output ''
    Write-Output "Toolchain ativa: $((& $rustupExecutable show active-toolchain 2>&1 | Select-Object -First 1))"
}

$webViewPaths = @(
    'HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients',
    'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients'
)
$webView = foreach ($registryPath in $webViewPaths) {
    if (Test-Path $registryPath) {
        Get-ChildItem $registryPath |
            ForEach-Object { Get-ItemProperty $_.PSPath } |
            Where-Object { $_.name -match 'WebView2' } |
            Select-Object -First 1
    }
}

Write-Output ''
if ($webView) {
    Write-Output "WebView2: $($webView.name) $($webView.pv)"
} else {
    Write-Output 'WebView2: não encontrado nas chaves consultadas'
}

$vsWhere = 'C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe'
if (Test-Path -LiteralPath $vsWhere) {
    $installationPath = (& $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>&1 | Select-Object -First 1)
    if ($installationPath) {
        Write-Output "C++ Build Tools: MSVC encontrado em $installationPath"
    } else {
        Write-Output 'C++ Build Tools: componente MSVC não confirmado'
    }
} else {
    Write-Output 'C++ Build Tools: vswhere não encontrado'
}
