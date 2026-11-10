$ErrorActionPreference = 'Stop'
$project = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$app = Join-Path $project 'app'
$version = (Get-Content -LiteralPath (Join-Path $app 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json).version
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw 'A publicação exige uma versão final.' }
$key = Join-Path $env:LOCALAPPDATA 'LpsToolkit\signing\updater.key'
if (!$env:TAURI_SIGNING_PRIVATE_KEY) {
    if (!(Test-Path -LiteralPath $key -PathType Leaf)) { throw 'A chave privada de atualização não foi encontrada.' }
    $env:TAURI_SIGNING_PRIVATE_KEY = $key
}
if (!(Test-Path Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD)) {
    $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ''
}
$release = Join-Path $project 'release'
if ((Test-Path -LiteralPath $release) -and
    (Get-Item -LiteralPath $release).Attributes -band [IO.FileAttributes]::ReparsePoint) {
    throw 'A pasta de releases não pode ser um link.'
}
$destination = Join-Path $project "release\$version"
if (Test-Path -LiteralPath $destination) { throw 'A pasta desta versão já existe. Confira a versão antes de compilar.' }
if (!$env:VSCMD_ARG_TGT_ARCH) {
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    $vs = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (!$vs) { throw 'Visual Studio Build Tools com C++ x64 não encontrado.' }
    Import-Module (Join-Path $vs 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll')
    Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64'
}
if ($env:VSCMD_ARG_TGT_ARCH -ne 'x64') { throw 'Use o ambiente de compilação x64.' }
$env:CARGO_TARGET_DIR = Join-Path $app 'src-tauri\target\validacao-local'
$env:CARGO_INCREMENTAL = '0'
Push-Location -LiteralPath $app
try {
    & npm.cmd run tauri -- build --ci --bundles nsis
    if ($LASTEXITCODE) { throw 'Falha ao compilar a release.' }
} finally { Pop-Location }
$name = "Pacton_${version}_x64-setup.exe"
$bundle = Join-Path $env:CARGO_TARGET_DIR 'release\bundle\nsis'
$source = Get-Item -LiteralPath (Join-Path $bundle $name)
$signature = (Get-Content -LiteralPath ($source.FullName + '.sig') -Raw).Trim()
if (!$signature) { throw 'O instalador não tem assinatura de atualização.' }
New-Item -ItemType Directory -Path $destination | Out-Null
Copy-Item -LiteralPath $source.FullName -Destination $destination
Copy-Item -LiteralPath ($source.FullName + '.sig') -Destination $destination
$installer = Join-Path $destination $name
$hash = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hash -ne (Get-FileHash -LiteralPath $source.FullName -Algorithm SHA256).Hash.ToLowerInvariant()) {
    throw 'Falha na verificação da cópia do instalador.'
}
[IO.File]::WriteAllText(($installer + '.sha256'), ($hash + '  ' + $name + [char]10), [Text.Encoding]::ASCII)
$legacyName = "LpsToolkit_${version}_x64-setup.exe"
Copy-Item -LiteralPath $installer -Destination (Join-Path $destination $legacyName)
Copy-Item -LiteralPath ($installer + '.sig') -Destination (Join-Path $destination ($legacyName + '.sig'))
[IO.File]::WriteAllText((Join-Path $destination ($legacyName + '.sha256')), ($hash + '  ' + $legacyName + [char]10), [Text.Encoding]::ASCII)
$manifest = @{
    version = $version
    notes = "Atualização Pacton $version"
    pub_date = [DateTime]::UtcNow.ToString('yyyy-MM-ddTHH:mm:ssZ')
    platforms = @{
        'windows-x86_64' = @{
            signature = $signature
            url = "https://github.com/leosalema213/app-compressao/releases/download/v$version/$legacyName"
        }
    }
}
[IO.File]::WriteAllText((Join-Path $destination 'latest.json'), ($manifest | ConvertTo-Json -Depth 5), [Text.UTF8Encoding]::new($false))
& (Join-Path $PSScriptRoot 'prune-releases.ps1') -CurrentVersion $version
Write-Output "Release preparada em $destination. Publique o instalador, .sig, .sha256 e latest.json juntos."
