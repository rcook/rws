$ErrorActionPreference = 'Stop'

Set-Location -Path $PSScriptRoot
& cargo watch
