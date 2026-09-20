param(
    [switch]$Release,
    [switch]$Demo
)

$ErrorActionPreference = 'Stop'
$cargoExe = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
if (-not (Test-Path -LiteralPath $cargoExe)) {
    $cargoExe = (Get-Command cargo -ErrorAction SilentlyContinue).Source
}
if (-not $cargoExe) {
    throw 'Rust is required. Install it from https://rustup.rs first.'
}

$buildArguments = @('build', '--locked')
if ($Release) { $buildArguments += '--release' }
if ($Demo) { $buildArguments += @('--features', 'demo') }

Push-Location $PSScriptRoot
try {
    & $cargoExe @buildArguments
    if ($LASTEXITCODE -ne 0) { throw "Build failed with exit code $LASTEXITCODE" }
} finally {
    Pop-Location
}
