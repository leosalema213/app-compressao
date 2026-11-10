param([Parameter(Mandatory)][string]$CurrentVersion)

$ErrorActionPreference = 'Stop'
$project = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$release = Join-Path $project 'release'

function Assert-OwnedDirectory($path) {
    $item = Get-Item -LiteralPath $path
    if ((Resolve-Path -LiteralPath $path).Path -ne $path -or
        $item.Attributes -band [IO.FileAttributes]::ReparsePoint) {
        throw "Pasta de release inesperada em $path."
    }
}

Assert-OwnedDirectory $release
$builds = @(Get-ChildItem -LiteralPath $release -Directory | Where-Object {
    $_.Name -match '^\d+\.\d+\.\d+(?:-preview\.[1-9]\d*)?$'
} | ForEach-Object {
    Assert-OwnedDirectory $_.FullName
    $directory = $_
    $names = @("Pacton_$($directory.Name)_x64-setup.exe", "LpsToolkit_$($directory.Name)_x64-setup.exe")
    $files = @('Pacton.exe', 'LpsToolkit.exe', 'latest.json') + @($names | ForEach-Object { $_; "$_.sig"; "$_.sha256" })
    $artifacts = $files |
        ForEach-Object { Join-Path $directory.FullName $_ } |
        Where-Object { Test-Path -LiteralPath $_ -PathType Leaf }
    if ($artifacts) {
        [pscustomobject]@{ Directory = $directory; Artifacts = @($artifacts) }
    }
})
$current = $builds | Where-Object { $_.Directory.Name -eq $CurrentVersion }
if (!$current) { throw 'Gere e verifique a compilação atual antes da limpeza.' }
$older = $builds | Where-Object { $_.Directory.Name -ne $CurrentVersion } |
    Sort-Object { $_.Directory.LastWriteTimeUtc } -Descending | Select-Object -Skip 1
foreach ($build in $older) {
    Assert-OwnedDirectory $release
    Assert-OwnedDirectory $build.Directory.FullName
    foreach ($artifact in $build.Artifacts) {
        $file = Get-Item -LiteralPath $artifact
        if ($file.Directory.Parent.FullName -ne $release -or
            $file.Attributes -band [IO.FileAttributes]::ReparsePoint) {
            throw 'Artefato fora da pasta de releases.'
        }
        Remove-Item -LiteralPath $file.FullName
    }
    if (!(Get-ChildItem -LiteralPath $build.Directory.FullName -Force)) {
        Remove-Item -LiteralPath $build.Directory.FullName
    }
}
