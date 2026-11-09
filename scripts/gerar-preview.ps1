$ErrorActionPreference = 'Stop'
$project = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$app = Join-Path $project 'app'
$config = 'src-tauri/tauri.preview.conf.json'
$version = (Get-Content -LiteralPath (Join-Path $app $config) -Raw | ConvertFrom-Json).version
$previewPattern = '^(\d+\.\d+\.\d+)-preview\.([1-9]\d*)$'
if ($version -notmatch $previewPattern) {
    throw 'Informe uma versão MAJOR.MINOR.PATCH-preview.N na configuração de preview.'
}

function Assert-NoLink($item) {
    if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw "Link encontrado em $($item.FullName)."
    }
}

$release = Join-Path $project 'release'
if (Test-Path -LiteralPath $release) {
    Assert-NoLink (Get-Item -LiteralPath $release)
}
$destination = Join-Path $release $version
if (Test-Path -LiteralPath $destination) {
    throw "O preview $version já existe. Incremente o número para gerar outro."
}

if (!$env:VSCMD_ARG_TGT_ARCH) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    $vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$vs) { throw 'Visual Studio Build Tools com C++ x64 não encontrado.' }
    Import-Module (Join-Path $vs 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll')
    Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
if ($env:VSCMD_ARG_TGT_ARCH -ne 'x64') { throw 'Use o ambiente de compilação x64.' }

# Reutiliza o cache das verificações locais e gera somente o executável.
$env:CARGO_TARGET_DIR = Join-Path $app 'src-tauri\target\validacao-local'
$env:CARGO_INCREMENTAL = '0'
Push-Location -LiteralPath $app
try {
    & npm.cmd run tauri -- build --ci --no-bundle --config $config
    if ($LASTEXITCODE -ne 0) { throw 'Falha ao compilar o preview.' }
} finally {
    Pop-Location
}

$binary = Get-Item -LiteralPath (Join-Path $env:CARGO_TARGET_DIR 'release\lps-toolkit.exe')
if ($binary.Length -eq 0) { throw 'O executável compilado está vazio.' }
New-Item -ItemType Directory -Path $destination | Out-Null
$exe = Join-Path $destination 'LpsToolkit.exe'
Copy-Item -LiteralPath $binary.FullName -Destination $exe
if ((Get-FileHash -LiteralPath $exe).Hash -ne (Get-FileHash -LiteralPath $binary.FullName).Hash) {
    throw 'A cópia do executável não passou na verificação de integridade.'
}

$shortcutPath = Join-Path $project 'Testar LpsToolkit.lnk'
if (Test-Path -LiteralPath $shortcutPath) { Assert-NoLink (Get-Item -LiteralPath $shortcutPath) }
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $exe
$shortcut.WorkingDirectory = $destination
$shortcut.Description = "LpsToolkit $version"
$shortcut.Save()

& (Join-Path $PSScriptRoot 'prune-releases.ps1') -CurrentVersion $version

Write-Output "Preview: $exe"
Write-Output "Atalho: $shortcutPath"
