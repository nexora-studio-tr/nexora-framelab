$ErrorActionPreference='Stop'
$rootPath=(Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$outPath=Join-Path $rootPath 'artifacts\windows'
$version=(Get-Content -Raw -LiteralPath (Join-Path $rootPath 'package.json') | ConvertFrom-Json).version
if ($version -notmatch '^\d+\.\d+\.\d+-alpha\.\d+$') {throw 'Unexpected prerelease version'}
$portablePath=Join-Path $outPath "Nexora-FrameLab-$version"
New-Item -ItemType Directory -Force -Path (Join-Path $portablePath 'presentmon') | Out-Null
Copy-Item -LiteralPath (Join-Path $rootPath 'target\release\nexora-framelab.exe') -Destination $portablePath
Copy-Item -LiteralPath (Join-Path $rootPath 'target\release\framelab-cli.exe') -Destination $portablePath
Copy-Item -LiteralPath (Join-Path $rootPath 'tools\presentmon\PresentMon.exe') -Destination (Join-Path $portablePath 'presentmon')
Copy-Item -LiteralPath (Join-Path $rootPath 'tools\presentmon\LICENSE.txt') -Destination (Join-Path $portablePath 'presentmon')
Copy-Item -LiteralPath (Join-Path $rootPath 'tools\notices\THIRD-PARTY-LICENSES.txt') -Destination $portablePath
foreach ($name in @('LICENSE','THIRD_PARTY_NOTICES.md','README.md','README.tr.md')) {Copy-Item -LiteralPath (Join-Path $rootPath $name) -Destination $portablePath}
Compress-Archive -LiteralPath $portablePath -DestinationPath (Join-Path $outPath "nexora-framelab-$version-windows-x64-portable.zip") -Force
$installer=Get-ChildItem -LiteralPath (Join-Path $rootPath 'target\release\bundle\nsis') -Filter "*_${version}_x64-setup.exe" | Select-Object -First 1
if (-not $installer) {throw 'NSIS installer not found'}
Copy-Item -LiteralPath $installer.FullName -Destination (Join-Path $outPath "nexora-framelab-$version-windows-x64-setup.exe")
$lines=@()
foreach ($name in @("nexora-framelab-$version-windows-x64-portable.zip","nexora-framelab-$version-windows-x64-setup.exe")) {$hash=(Get-FileHash -LiteralPath (Join-Path $outPath $name) -Algorithm SHA256).Hash.ToLowerInvariant();$lines+="$hash  $name"}
$lines | Set-Content -LiteralPath (Join-Path $outPath 'SHA256SUMS.txt') -Encoding ascii
Write-Output 'Windows installer and portable package prepared with SHA-256 checksums.'
