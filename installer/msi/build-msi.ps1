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
    # [string]$UpgradeCode
)

$ErrorActionPreference = 'Stop'

$ProjectName = "Atlas SQL ODBC"
$sourceDir = Get-Location
$resourceDir = Get-Location
$binDir = Get-Location
$objDIr = ".\objs\"
$WixPath = "C:\wixtools\bin\"
# for local building, most installations will be in the directory below
# $WixPath = "C:\Program Files (x86)\WiX Toolset v3.11\bin"
$wixUiExt = "$WixPath\WixUIExtension.dll"

# we currently only support x64, but we'll leave the 32 bit support here
# in case we eventually decide to provide a 32-bit driver
# upgradeCodes and productCodes are pre-generated for the next several releases 
if ($Arch -eq "x64") {
    switch ($VersionLabel) {
        "1.0" { 
            $upgradeCode = "a4b5342b-25fc-4978-8347-8686684ddba2" 
            $productCode = "72118595-650f-47a6-bd0f-8c21888bb115"
        }
        "1.1" { 
            $upgradeCode = "cf8912e0-e01f-4da7-ba06-5cc598c3c93e" 
            $productCode = "3e95f304-0ba8-413a-8c62-ed50b8e7f460"
        }
        "1.2" { 
            $upgradeCode = "5a54052b-5eae-42ea-a054-0c817d59225b" 
            $productCode = "d09237c7-806a-4ea4-ae01-acc131ffec24"
        }
        "1.3" { 
            $upgradeCode = "5dd94bac-17ba-4699-b215-579fc18cb4fd" 
            $productCode = "49c7e3fb-14c7-42bd-a27f-7727d3207118"
        }
        "1.4" { 
            $upgradeCode = "ff3db46e-d144-4f83-ac63-829a89097f95" 
            $productCode = "3e95f304-0ba8-413a-8c62-ed50b8e7f460"
        }
        "1.5" { 
            $upgradeCode = "0a03f1c2-f1aa-4ee9-9497-f5ebc1e61606" 
            $productCode = "a28f5345-090e-467b-ad01-805a583df560"
        }
        "1.6" { 
            $upgradeCode = "56f27a0d-96d4-419b-bd5d-90972016854d" 
            $productCode = "873c6fe3-30ff-4d4f-a513-8161b8c6c81e"
        }
        "1.7" { 
            $upgradeCode = "86b67d12-d709-4d88-8575-52d510ad11ac" 
            $productCode = "1da14301-8adf-4010-824a-ed0cac914aab"
        }
        "1.8" { 
            $upgradeCode = "25291a12-f569-41f3-afb7-806387ade571" 
            $productCode = "364eb145-3ff9-496c-92ab-7bb8f534c5ba"
        }
        "1.9" { 
            $upgradeCode = "a742e237-c2b3-4b25-a4ae-f6ff24f81ad6" 
            $productCode = "ca645af1-6665-4578-af95-fff16ab3e9eb"
        }
        "1.10" {
            $upgradeCode = "154e129a-a402-4540-aed3-3a8f3a3f3eec"
            $productCode = "3508aa68-c5f4-453d-9e9e-917c6974e67f"
        }
        "1.11" {
            $upgradeCode = "219726b7-f8ca-40c9-a33f-cf4c7a2fd54d"
            $productCode = "c4d183b6-ce72-4e94-8e3e-d3acdb3fcdde"
        }
        "1.12" {
            $upgradeCode = "2bc9c298-3c57-4d9b-93f2-e48515cd198c"
            $productCode = "84c73926-d7a2-4b25-92f5-9ab474e3d7ec"
        }
        "1.13" {
            $upgradeCode = "db3edfb4-0da0-44d2-a752-eb1c41e9c4a6"
            $productCode = "cfc74668-d90b-48f4-b3da-b989b82f946d"
        }
        "1.14" {
            $upgradeCode = "76d26c50-d376-4d37-b5b7-f06df6e57ce4"
            $productCode = "a0669045-8bb1-4b90-a793-9795d72641ba"
        }
        "1.15" {
            $upgradeCode = "e1ea15b4-a896-410d-9efb-bba0b8a15776"
            $productCode = "a0dc3052-f0ff-48fc-a3bd-e75a3727e85a"
        }
    }

}
else {
    $upgradeCode = "ade38aac-c8ca-11ed-afa1-0242ac120002"
    $productCode = "15e9a1ea-5c6e-4fe8-9f48-6dc23def5ec1"
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
