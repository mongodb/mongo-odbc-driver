# Copyright (c) 2018-Present MongoDB Inc.
<#
.SYNOPSIS
    Builds an MSI for the MongoDB ODBC driver.
.DESCRIPTION
    .
.PARAMETER Arch
    The architecture, x86 or x64.
#>
Param(
  [string]$Arch,
  [string]$Version,
  [string]$VersionLabel,
  [string]$UpgradeCode
)

$ErrorActionPreference = 'Stop'

$ProjectName = "Atlas SQL ODBC"
$sourceDir = pwd
$resourceDir = pwd
$binDir = pwd
$objDIr = ".\objs\"
$WixPath = "C:\wixtools\bin\"
# for local building, most installations will be in the directory below
#$WixPath = "C:\Program Files (x86)\WiX Toolset v3.11\bin"
$wixUiExt = "$WixPath\WixUIExtension.dll"

# we currently only support x64, but we'll leave the 32 bit support here
# in case we eventually decide to provide a 32-bit driver
if ($Arch -eq "x64") {
    $upgradeCode = "a4303fe6-c8ca-11ed-afa1-0242ac120002"
} else {
    $upgradeCode = "ade38aac-c8ca-11ed-afa1-0242ac120002"
}


# compile wxs into .wixobjs
& $WixPath\candle.exe -wx `
    -dProductId="*" `
    -dPlatform="$Arch" `
    -dUpgradeCode="$UpgradeCode" `
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

if(-not $?) {
    exit 1
}

$artifactsDir = pwd

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
