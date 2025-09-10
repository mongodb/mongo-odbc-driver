# Copyright (c) 2018-Present MongoDB Inc.
<#
.SYNOPSIS
    Builds an MSI for the MongoDB ODBC driver.
.DESCRIPTION
    .
.PARAMETER Arch
    The architecture, x86 or x64.
.PARAMETER Version
    The version of the driver.
.PARAMETER VersionLabel
    The version label of the driver.
#>
Param(
    [string]$Arch,
    [string]$Version,
    [string]$VersionLabel
)

$ErrorActionPreference = 'Stop'

$ProjectName = "Atlas SQL ODBC"
$sourceDir = Get-Location
$resourceDir = Get-Location
$binDir = Get-Location
$objDIr = ".\objs\"
$WixPath = "C:\wixtools\bin\"
if (-not (Test-Path -Path $WixPath)) {
    $WixPath = "C:\Program Files (x86)\WiX Toolset\bin"
}
if (-not (Test-Path -Path $WixPath)) {
    $WixPath = "C:\Program Files\WiX Toolset\bin"
}
if (-not (Test-Path -Path $WixPath)) {
    $WixPath = "C:\Program Files (x86)\WiX Toolset v3.11\bin"
}
if (-not (Test-Path -Path $WixPath)) {
    $WixPath = "C:\Program Files\WiX Toolset v3.11\bin"
}
if (-not (Test-Path -Path $WixPath)) {
    Write-Host "could not find Wix"
    exit 1
}
# for local building, most installations will be in the directory below
# $WixPath = "C:\Program Files (x86)\WiX Toolset v3.11\bin"
$wixUiExt = "$WixPath\WixUIExtension.dll"

# we currently only support x64, but we'll leave the 32 bit support here
# in case we eventually decide to provide a 32-bit driver
# we will need to add a product code should we ever want to have a v1.x and v2.x version side by side
if ($Arch -eq "x64") {
    $productCode = "72118595-650f-47a6-bd0f-8c21888bb116"
    $upgradeCode = "806440a5-1623-44b6-9d7c-8efbeaa8e316"
}
else {
    $productCode = "15e9a1ea-5c6e-4fe8-9f48-6dc23def5ec1"
    $upgradeCode = "8344fddd-a83e-4232-88f8-8b8bd387e0f8"
}


# compile wxs into .wixobjs
& $WixPath\candle.exe -wx `
    -dProductId="$productCode" `
    -dPlatform="$Arch" `
    -dUpgradeCode="$upgradeCode" `
    -dVersion="$Version" `
    -dVersionLabel="$VersionLabel" `
    -dProjectName="$ProjectName" `
    -dSourceDir="$sourceDir" `
    -dResourceDir="$resourceDir" `
    -dSslDir="$binDir" `
    -dBinaryDir="$binDir" `
    -dTargetDir="$objDir" `
    -dTargetExt=".msi" `
    -dTargetFileName="release" `
    -dOutDir="$objDir" `
    -dConfiguration="Release" `
    -arch "$Arch" `
    -out "$objDir" `
    -ext "$wixUiExt" `
    "$resourceDir\Product.wxs" `
    "$resourceDir\FeatureFragment.wxs" `
    "$resourceDir\BinaryFragment.wxs" `
    "$resourceDir\LicensingFragment.wxs" `
    "$resourceDir\UIFragment.wxs"

if (-not $?) {
    exit 1
}

$artifactsDir = Get-Location

# link wixobjs into an msi
& $WixPath\light.exe -wx `
    -cultures:en-us `
    -out "$artifactsDir\mongoodbc.msi" `
    -ext "$wixUiExt" `
    $objDir\Product.wixobj `
    $objDir\FeatureFragment.wixobj `
    $objDir\BinaryFragment.wixobj `
    $objDir\LicensingFragment.wixobj `
    $objDir\UIFragment.wixobj

trap {
    write-output $_
    exit 1
}
