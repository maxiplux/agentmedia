# Install the latest stable public release, or select -Version. No administrator access.
param([string]$Version = '', [string]$BinDir = '', [switch]$NoModifyPath, [switch]$Force)
$ErrorActionPreference = 'Stop'
# Preserve modern system TLS defaults; enable TLS 1.2 for older PowerShell defaults.
if ([int][Net.ServicePointManager]::SecurityProtocol -ne 0) {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
}
$architecture = [Environment]::GetEnvironmentVariable('PROCESSOR_ARCHITEW6432')
if (-not $architecture) { $architecture = [Environment]::GetEnvironmentVariable('PROCESSOR_ARCHITECTURE') }
if ($architecture -ne 'AMD64') { throw 'This release supports Windows x86-64. Build from source for another architecture.' }
$temporary = Join-Path ([IO.Path]::GetTempPath()) ('mediaagent-install-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $temporary | Out-Null
try {
    $base = 'https://github.com/maxiplux/agentmedia/releases'
    if (-not $Version) {
        Invoke-WebRequest -UseBasicParsing -Uri "$base/latest/download/version.txt" -OutFile (Join-Path $temporary 'version.txt')
        $Version = (Get-Content -LiteralPath (Join-Path $temporary 'version.txt') -Raw).Trim()
    }
    $Version = $Version -replace '^v', ''
    if ($Version -notmatch '\A[0-9]+\.[0-9]+\.[0-9]+\z') { throw 'Invalid stable release version' }
    $asset = 'mediaagent-x86_64-pc-windows-msvc.exe'
    $binary = Join-Path $temporary 'mediaagent.exe'
    $checksum = Join-Path $temporary 'checksum'
    Invoke-WebRequest -UseBasicParsing -Uri "$base/download/v$Version/$asset" -OutFile $binary
    Invoke-WebRequest -UseBasicParsing -Uri "$base/download/v$Version/$asset.sha256" -OutFile $checksum
    $expected = ((Get-Content -LiteralPath $checksum -Raw).Trim() -split '\s+')[0]
    if ($expected -notmatch '^[a-fA-F0-9]{64}$' -or (Get-FileHash -LiteralPath $binary -Algorithm SHA256).Hash -ine $expected) { throw 'Checksum mismatch; nothing was installed' }
    if ((& $binary --version) -ne "mediaagent $Version" -or $LASTEXITCODE -ne 0) { throw 'Executable version check failed' }
    $arguments = @('install')
    if ($BinDir) { $arguments += @('--bin-dir', $BinDir) }
    if ($NoModifyPath) { $arguments += '--no-modify-path' }
    if ($Force) { $arguments += '--force' }
    & $binary @arguments
    if ($LASTEXITCODE -ne 0) { throw 'mediaagent installation failed' }
    if (-not $NoModifyPath) {
        $installedBin = if ($BinDir) { [IO.Path]::GetFullPath($BinDir) } else { Join-Path $env:LOCALAPPDATA 'mediaagent\bin' }
        $env:Path = $installedBin + ';' + $env:Path
    }
} finally { Remove-Item -LiteralPath $temporary -Recurse -Force }
