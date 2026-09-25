param(
    [ValidateRange(1, 100000)]
    [int]$Frames = 660,
    [string]$Seed = "1337",
    [string]$Output = "aavajan.cast"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command asciinema -ErrorAction SilentlyContinue)) {
    throw "asciinema is required. Install it before running this script."
}

cargo build --release
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$command = "target\release\aavajan.exe --frames $Frames --seed $Seed"
asciinema rec --command $command $Output
exit $LASTEXITCODE
