<#
.SYNOPSIS
    Installs specific service versions for BAMP on Windows (e.g. PHP 8.4, 8.2, 8.1, 7.4).

.DESCRIPTION
    This script downloads and configures portable Windows packages for specific service versions.
    It can be run independently or invoked automatically by BAMP's Package Hub GUI.

.PARAMETER Service
    Target service name: "php", "apache", "nginx", "mysql", "mailpit". Default is "php".

.PARAMETER Version
    Specific version string (e.g. "8.4", "8.2", "8.1", "7.4").

.PARAMETER Php
    Convenience shortcut to specify a PHP version directly (e.g. -Php 8.4).

.PARAMETER Force
    Re-download and overwrite even if already present.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts/install-service-version.ps1 -Php 8.4
    powershell -ExecutionPolicy Bypass -File scripts/install-service-version.ps1 -Php 7.4
    powershell -ExecutionPolicy Bypass -File scripts/install-service-version.ps1 -Service php -Version 8.2
#>

[CmdletBinding()]
param(
    [string]$Service = "php",
    [string]$Version = "",
    [string]$Php = "",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# Handle -Php shortcut
if ($Php -ne "") {
    $Service = "php"
    $Version = $Php
}

# Clean version format (e.g., "php-8.4" -> "8.4")
if ($Version.StartsWith("php-")) {
    $Version = $Version.Substring(4)
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RootDir = Split-Path -Parent $ScriptDir
$BinDir = Join-Path $RootDir "bin"
$TempDir = Join-Path $RootDir ".temp_binaries"

if (-not (Test-Path $BinDir)) { New-Item -ItemType Directory -Path $BinDir -Force | Out-Null }
if (-not (Test-Path $TempDir)) { New-Item -ItemType Directory -Path $TempDir -Force | Out-Null }

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12 -bor [Net.SecurityProtocolType]::Tls13

function Download-And-Extract-Zip {
    param(
        [string]$Name,
        [string]$Url,
        [string]$DestinationDir,
        [scriptblock]$PostExtract
    )

    Write-Host "`n>>> Processing $Name from $Url" -ForegroundColor Cyan

    if (-not (Test-Path $DestinationDir)) {
        New-Item -ItemType Directory -Path $DestinationDir -Force | Out-Null
    }

    $ZipName = "temp_pkg_" + [guid]::NewGuid().ToString("N").Substring(0, 8) + ".zip"
    $ZipPath = Join-Path $TempDir $ZipName
    $ExtractTemp = Join-Path $TempDir ("extract_" + [guid]::NewGuid().ToString("N").Substring(0, 8))

    Write-Host "Downloading $Name..." -ForegroundColor Yellow
    $webClient = New-Object System.Net.WebClient
    $webClient.Headers.Add("User-Agent", "BAMP-Installer/1.0 (Windows NT 10.0; Win64; x64)")
    $webClient.DownloadFile($Url, $ZipPath)

    Write-Host "Extracting package..." -ForegroundColor Yellow
    Expand-Archive -Path $ZipPath -DestinationPath $ExtractTemp -Force

    if ($PostExtract) {
        & $PostExtract $ExtractTemp $DestinationDir
    } else {
        Copy-Item -Path "$ExtractTemp\*" -Destination $DestinationDir -Recurse -Force
    }

    # Cleanup temp
    if (Test-Path $ExtractTemp) { Remove-Item -Recurse -Force $ExtractTemp }
    if (Test-Path $ZipPath) { Remove-Item -Force $ZipPath }

    Write-Host "[OK] $Name successfully installed into $DestinationDir" -ForegroundColor Green
}

# -------------------------------------------------------------
# PHP Version Management
# -------------------------------------------------------------
if ($Service.ToLower() -eq "php") {
    $phpUrlMap = @{
        "8.4" = "https://windows.php.net/downloads/releases/php-8.4.6-nts-Win32-vs17-x64.zip"
        "8.3" = "https://windows.php.net/downloads/releases/php-8.3.17-nts-Win32-vs16-x64.zip"
        "8.2" = "https://windows.php.net/downloads/releases/php-8.2.27-nts-Win32-vs16-x64.zip"
        "8.1" = "https://windows.php.net/downloads/releases/php-8.1.31-nts-Win32-vs16-x64.zip"
        "7.4" = "https://windows.php.net/downloads/releases/archives/php-7.4.33-nts-Win32-vc15-x64.zip"
    }

    if ($Version -eq "") {
        $Version = "8.3"
    }

    if (-not $phpUrlMap.ContainsKey($Version)) {
        Write-Error "Unsupported PHP version '$Version'. Supported versions: $($phpUrlMap.Keys -join ', ')"
        exit 1
    }

    $TargetDir = Join-Path $BinDir "php\php-$Version"
    $PhpCgi = Join-Path $TargetDir "php-cgi.exe"

    if ((Test-Path $PhpCgi) -and (-not $Force)) {
        Write-Host "[SKIP] PHP $Version is already installed at $TargetDir" -ForegroundColor DarkGray
        exit 0
    }

    $Url = $phpUrlMap[$Version]
    Download-And-Extract-Zip -Name "PHP $Version NTS" -Url $Url -DestinationDir $TargetDir -PostExtract {
        param($src, $dst)
        Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force

        # Configure default php.ini
        $iniDev = Join-Path $dst "php.ini-development"
        $iniProd = Join-Path $dst "php.ini-production"
        $iniTarget = Join-Path $dst "php.ini"

        $iniSource = if (Test-Path $iniDev) { $iniDev } elseif (Test-Path $iniProd) { $iniProd } else { $null }

        if (($iniSource) -and (-not (Test-Path $iniTarget))) {
            Copy-Item $iniSource $iniTarget
            (Get-Content $iniTarget) |
                ForEach-Object { $_ -replace '^;extension=curl', 'extension=curl' } |
                ForEach-Object { $_ -replace '^;extension=mbstring', 'extension=mbstring' } |
                ForEach-Object { $_ -replace '^;extension=openssl', 'extension=openssl' } |
                ForEach-Object { $_ -replace '^;extension=pdo_mysql', 'extension=pdo_mysql' } |
                ForEach-Object { $_ -replace '^;extension=mysqli', 'extension=mysqli' } |
                ForEach-Object { $_ -replace '^;extension_dir = "ext"', 'extension_dir = "ext"' } |
                Set-Content $iniTarget
        }
    }
}
elseif ($Service.ToLower() -eq "apache") {
    $ApacheDir = Join-Path $BinDir "apache"
    $ApacheUrl = "https://www.apachelounge.com/download/VS17/binaries/httpd-2.4.63-250207-win64-VS17.zip"
    Download-And-Extract-Zip -Name "Apache HTTPD" -Url $ApacheUrl -DestinationDir $ApacheDir -PostExtract {
        param($src, $dst)
        $sub = Join-Path $src "Apache24"
        if (Test-Path $sub) {
            Copy-Item -Path "$sub\*" -Destination $dst -Recurse -Force
        } else {
            Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force
        }
    }
}
elseif ($Service.ToLower() -eq "nginx") {
    $NginxDir = Join-Path $BinDir "nginx"
    $NginxUrl = "https://nginx.org/download/nginx-1.26.2.zip"
    Download-And-Extract-Zip -Name "Nginx" -Url $NginxUrl -DestinationDir $NginxDir -PostExtract {
        param($src, $dst)
        $sub = Get-ChildItem -Path $src -Directory | Select-Object -First 1
        if ($sub) {
            Copy-Item -Path "$($sub.FullName)\*" -Destination $dst -Recurse -Force
        } else {
            Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force
        }
    }
}
elseif ($Service.ToLower() -eq "mysql" -or $Service.ToLower() -eq "mariadb") {
    $MySqlDir = Join-Path $BinDir "mysql"
    $MariaDbUrl = "https://archive.mariadb.org/mariadb-10.11.11/winx64-packages/mariadb-10.11.11-winx64.zip"
    Download-And-Extract-Zip -Name "MariaDB 10.11 LTS" -Url $MariaDbUrl -DestinationDir $MySqlDir -PostExtract {
        param($src, $dst)
        $sub = Get-ChildItem -Path $src -Directory | Select-Object -First 1
        if ($sub) {
            Copy-Item -Path "$($sub.FullName)\*" -Destination $dst -Recurse -Force
        } else {
            Copy-Item -Path "$src\*" -Destination $dst -Recurse -Force
        }
    }
}
elseif ($Service.ToLower() -eq "mailpit") {
    $MailpitDir = Join-Path $BinDir "mailpit"
    $MailpitUrl = "https://github.com/axllent/mailpit/releases/download/v1.22.3/mailpit-windows-amd64.zip"
    Download-And-Extract-Zip -Name "Mailpit" -Url $MailpitUrl -DestinationDir $MailpitDir
}
else {
    Write-Error "Unknown service '$Service'. Supported: php, apache, nginx, mysql, mailpit"
    exit 1
}

# Cleanup temp folder
if (Test-Path $TempDir) {
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}

Write-Host "`n>>> Done! $Service version setup completed." -ForegroundColor Green
