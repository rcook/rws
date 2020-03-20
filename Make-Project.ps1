[CmdletBinding()]
param()

Set-Location -Path $PSScriptRoot
Invoke-WebRequest -Uri https://gitlab.com/rcook/rbbt/-/raw/stable/rbbt.ps1 | Invoke-Expression
cargo build
