<#
.SYNOPSIS
    Downloads and configures portable Windows binaries for BAMP (Nginx, Apache, PHP 8.3, MariaDB, Mailpit).

.DESCRIPTION
    This script downloads official portable zip releases from upstream sources, extracts them into
    the `bin/` directory, and sets up minimal runtime configurations so that BAMP can run all services
    on Windows without requiring system-wide installations.

.PARAMETER Services
    Comma-separated list of services to download. Default is "all" (nginx, apache, php, mysql, mailpit).

.PARAMETER Force
    Overwrites existing binaries even if already present.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/install-binaries-win.ps1
    powershell -ExecutionPolicy Bypass -File scripts/install-binaries-win.ps1 -Services nginx,php
#>

[CmdletBinding()]
param(
    [string]$Services = "all",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RootDir = Split-Path -Parent $ScriptDir
$BinDir = Join-Path $RootDir "bin"
$TempDir = Join-Path $RootDir ".temp_binaries"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  BAMP: Portable Windows Binaries Downloader & Setup     " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "Target Root : $RootDir"
Write-Host "Target Bin  : $BinDir"

# Ensure directories exist
if (-not (Test-Path $BinDir)) { New-Item -ItemType Directory -Path $BinDir -Force | Out-Null }
if (-not (Test-Path $TempDir)) { New-Item -ItemType Directory -Path $TempDir -Force | Out-Null }

$TargetList = if ($Services -eq "all") {
    @("nginx", "apache", "php", "mysql", "mailpit")
} else {
    $Services.Split(",").Trim().ToLower()
}

function Download-And-Extract {
    param(
        [string]$Name,
        [string]$Url,
        [string]$ZipName,
        [string]$DestinationDir,
        [string]$Referer = "",
        [scriptblock]$PostExtract
    )

    Write-Host "`n>>> Checking $Name..." -ForegroundColor Yellow

    if (-not (Test-Path $DestinationDir)) {
        New-Item -ItemType Directory -Path $DestinationDir -Force | Out-Null
    }

    $ZipPath = Join-Path $TempDir $ZipName
    $ExtractTemp = Join-Path $TempDir "extracted_$Name"

    Write-Host "Downloading $Name from $Url ..." -ForegroundColor Cyan
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13
    
    # Use WebClient with standard browser User-Agent and optional Referer
    $webClient = New-Object System.Net.WebClient
    $webClient.Headers.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
    $webClient.Headers.Add("Accept", "*/*")
    if ($Referer -ne "") {
        $webClient.Headers.Add("Referer", $Referer)
    }
    $webClient.DownloadFile($Url, $ZipPath)

    # Validate ZIP integrity (Magic Bytes PK\x03\x04 and minimum size)
    $fileInfo = Get-Item $ZipPath
    if ($fileInfo.Length -lt 100000) {
        $sample = [System.IO.File]::ReadAllText($ZipPath)
        if ($sample -like "*<html*") {
            throw "Downloaded $Name ($ZipName) is an HTML error page instead of a ZIP archive. Response: $sample"
        }
        throw "Downloaded $Name ($ZipName) is too small ($($fileInfo.Length) bytes) to be a valid ZIP archive."
    }

    $stream = [System.IO.File]::OpenRead($ZipPath)
    $header = New-Object byte[] 4
    $bytesRead = $stream.Read($header, 0, 4)
    $stream.Close()
    if ($bytesRead -lt 4 -or $header[0] -ne 0x50 -or $header[1] -ne 0x4B) {
        throw "Downloaded file $ZipName is corrupted (magic bytes PK not found)."
    }

    Write-Host "Extracting $ZipName..." -ForegroundColor Cyan
    if (Test-Path $ExtractTemp) { Remove-Item -Recurse -Force $ExtractTemp }
    Expand-Archive -Path $ZipPath -DestinationPath $ExtractTemp -Force

    if ($PostExtract) {
        & $PostExtract $ExtractTemp $DestinationDir
    }

    # Cleanup temp extraction
    if (Test-Path $ExtractTemp) { Remove-Item -Recurse -Force $ExtractTemp }
    if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }

    Write-Host "[OK] $Name successfully installed into $DestinationDir" -ForegroundColor Green
}

# 1. NGINX
if ($TargetList -contains "nginx") {
    $NginxDir = Join-Path $BinDir "nginx"
    $NginxExe = Join-Path $NginxDir "nginx.exe"

    if ((Test-Path $NginxExe) -and (-not $Force)) {
        Write-Host "`n[SKIP] Nginx already present at $NginxExe" -ForegroundColor DarkGray
    } else {
        Download-And-Extract `
            -Name "Nginx" `
            -Url "https://nginx.org/download/nginx-1.26.2.zip" `
            -ZipName "nginx.zip" `
            -DestinationDir $NginxDir `
            -PostExtract {
                param($src, $dst)
                # Nginx zip contains a root folder like nginx-1.26.2/
                $subFolder = Get-ChildItem -Path $src -Directory | Select-Object -First 1
                if ($subFolder) {
                    Copy-Item -Path "$($subFolder.FullName)\*" -Destination $dst -Recurse -Force
                } else {
                    Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force
                }
            }
    }
}

# 2. APACHE HTTPD
if ($TargetList -contains "apache") {
    $ApacheDir = Join-Path $BinDir "apache"
    $ApacheExe = Join-Path $ApacheDir "bin\httpd.exe"

    if ((Test-Path $ApacheExe) -and (-not $Force)) {
        Write-Host "`n[SKIP] Apache HTTPD already present at $ApacheExe" -ForegroundColor DarkGray
    } else {
        # Dynamically resolve latest Apache Lounge zip with reliable fallback
        $apacheUrl = "https://www.apachelounge.com/download/VS17/binaries/httpd-2.4.66-251206-Win64-VS17.zip"
        try {
            $wc = New-Object System.Net.WebClient
            $wc.Headers.Add("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            $html = $wc.DownloadString("https://www.apachelounge.com/download/VS17/")
            if ($html -match 'href="(/download/VS17/binaries/httpd-2\.4\.[^"]+-Win64-VS17\.zip)"') {
                $apacheUrl = "https://www.apachelounge.com" + $matches[1]
            }
        } catch {
            Write-Host "Notice: Using fallback Apache Lounge URL: $apacheUrl" -ForegroundColor DarkGray
        }

        Download-And-Extract `
            -Name "Apache HTTPD" `
            -Url $apacheUrl `
            -ZipName "httpd.zip" `
            -Referer "https://www.apachelounge.com/download/VS17/" `
            -DestinationDir $ApacheDir `
            -PostExtract {
                param($src, $dst)
                $apacheFolder = Join-Path $src "Apache24"
                if (Test-Path $apacheFolder) {
                    Copy-Item -Path "$apacheFolder\*" -Destination $dst -Recurse -Force
                } else {
                    Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force
                }
            }
    }
}

# 3. PHP 8.3 (NTS x64)
if ($TargetList -contains "php") {
    $PhpDir = Join-Path $BinDir "php\php-8.3"
    $PhpCgiExe = Join-Path $PhpDir "php-cgi.exe"

    if ((Test-Path $PhpCgiExe) -and (-not $Force)) {
        Write-Host "`n[SKIP] PHP 8.3 already present at $PhpCgiExe" -ForegroundColor DarkGray
    } else {
        Download-And-Extract `
            -Name "PHP 8.3 NTS" `
            -Url "https://windows.php.net/downloads/releases/archives/php-8.3.16-nts-Win32-vs16-x64.zip" `
            -ZipName "php83.zip" `
            -DestinationDir $PhpDir `
            -PostExtract {
                param($src, $dst)
                Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force

                # Setup default php.ini if missing
                $iniDev = Join-Path $dst "php.ini-development"
                $iniFile = Join-Path $dst "php.ini"
                if ((Test-Path $iniDev) -and (-not (Test-Path $iniFile))) {
                    Copy-Item $iniDev $iniFile
                    # Enable standard extensions
                    (Get-Content $iniFile) |
                        ForEach-Object { $_ -replace '^;extension=curl', 'extension=curl' } |
                        ForEach-Object { $_ -replace '^;extension=mbstring', 'extension=mbstring' } |
                        ForEach-Object { $_ -replace '^;extension=openssl', 'extension=openssl' } |
                        ForEach-Object { $_ -replace '^;extension=pdo_mysql', 'extension=pdo_mysql' } |
                        ForEach-Object { $_ -replace '^;extension=mysqli', 'extension=mysqli' } |
                        ForEach-Object { $_ -replace '^;extension_dir = "ext"', 'extension_dir = "ext"' } |
                        Set-Content $iniFile
                }
            }
    }
}

# 4. MARIADB
if ($TargetList -contains "mysql") {
    $MySqlDir = Join-Path $BinDir "mysql"
    $MariaDbExe = Join-Path $MySqlDir "bin\mariadbd.exe"

    if ((Test-Path $MariaDbExe) -and (-not $Force)) {
        Write-Host "`n[SKIP] MariaDB already present at $MariaDbExe" -ForegroundColor DarkGray
    } else {
        Download-And-Extract `
            -Name "MariaDB 10.11" `
            -Url "https://archive.mariadb.org/mariadb-10.11.11/winx64-packages/mariadb-10.11.11-winx64.zip" `
            -ZipName "mariadb.zip" `
            -DestinationDir $MySqlDir `
            -PostExtract {
                param($src, $dst)
                $subFolder = Get-ChildItem -Path $src -Directory | Select-Object -First 1
                if ($subFolder) {
                    Copy-Item -Path "$($subFolder.FullName)\*" -Destination $dst -Recurse -Force
                } else {
                    Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force
                }

                # Ensure data/mysql/data is initialized if mariadb provides default data
                $embeddedData = Join-Path $dst "data"
                $runtimeData = Join-Path $RootDir "data\mysql\data"
                if ((Test-Path $embeddedData) -and (-not (Test-Path $runtimeData))) {
                    New-Item -ItemType Directory -Path (Join-Path $RootDir "data\mysql") -Force | Out-Null
                    Copy-Item -Path "$embeddedData\*" -Destination $runtimeData -Recurse -Force
                }
            }
    }
}

# 5. MAILPIT
if ($TargetList -contains "mailpit") {
    $MailpitDir = Join-Path $BinDir "mailpit"
    $MailpitExe = Join-Path $MailpitDir "mailpit.exe"

    if ((Test-Path $MailpitExe) -and (-not $Force)) {
        Write-Host "`n[SKIP] Mailpit already present at $MailpitExe" -ForegroundColor DarkGray
    } else {
        Download-And-Extract `
            -Name "Mailpit" `
            -Url "https://github.com/axllent/mailpit/releases/download/v1.22.3/mailpit-windows-amd64.zip" `
            -ZipName "mailpit.zip" `
            -DestinationDir $MailpitDir `
            -PostExtract {
                param($src, $dst)
                Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force
            }
    }
}

# Cleanup temp folder
if (Test-Path $TempDir) {
    Remove-Item -Recurse -Force $TempDir
}

Write-Host "`n==========================================================" -ForegroundColor Green
Write-Host "  All requested service binaries are installed in bin/!   " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green
