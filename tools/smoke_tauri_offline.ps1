param(
    [Parameter(Mandatory = $true)]
    [string] $Executable
)

$ErrorActionPreference = 'Stop'
$resolvedExecutable = (Resolve-Path -LiteralPath $Executable).Path
$runtimeRoot = Join-Path ${env:ProgramFiles(x86)} 'Microsoft\EdgeWebView\Application'
$webViewExecutable = Get-ChildItem -LiteralPath $runtimeRoot -Filter 'msedgewebview2.exe' -File -Recurse |
    Sort-Object FullName -Descending |
    Select-Object -First 1

if (-not $webViewExecutable) {
    throw "WebView2 runtime executable was not found under $runtimeRoot."
}

$ruleNames = @()
$process = $null
try {
    foreach ($program in @($resolvedExecutable, $webViewExecutable.FullName)) {
        $ruleName = 'MachineRoomOfflineSmoke-' + [guid]::NewGuid().ToString('N')
        New-NetFirewallRule -DisplayName $ruleName -Direction Outbound -Action Block -Program $program -Profile Any | Out-Null
        $ruleNames += $ruleName
    }

    $process = Start-Process -FilePath $resolvedExecutable -PassThru -WindowStyle Hidden
    $dataRoot = Join-Path $env:APPDATA 'com.machineroom.clab'
    $catalogPath = Join-Path $dataRoot 'catalog.sqlite'
    $learnerPath = Join-Path $dataRoot 'learner.sqlite'
    $deadline = (Get-Date).AddSeconds(45)
    while ((Get-Date) -lt $deadline) {
        if ($process.HasExited) {
            throw "Machine Room exited before catalog initialization (code $($process.ExitCode))."
        }
        if ((Test-Path -LiteralPath $catalogPath) -and (Test-Path -LiteralPath $learnerPath)) {
            break
        }
        Start-Sleep -Milliseconds 500
    }
    if (-not (Test-Path -LiteralPath $catalogPath) -or -not (Test-Path -LiteralPath $learnerPath)) {
        throw 'Machine Room did not initialize both local databases within 45 seconds.'
    }

    @'
import sqlite3, sys
catalog = sqlite3.connect(sys.argv[1])
learner = sqlite3.connect(sys.argv[2])
assert catalog.execute("select content_revision from catalog_meta").fetchone() == (1,)
assert catalog.execute("select count(*) from courses").fetchone() == (3,)
assert catalog.execute("select count(*) from chapters").fetchone() == (24,)
assert catalog.execute("select count(*) from catalog_items").fetchone() == (14,)
assert catalog.execute("select count(*) from lexicon_entries").fetchone() == (8,)
assert catalog.execute("select count(*) from search_documents where lower(search_text) like '%one-past%'").fetchone()[0] > 0
assert learner.execute("pragma user_version").fetchone() == (2,)
print("Offline launch, local catalog lookup, and learner migration passed.")
'@ | py -3 - $catalogPath $learnerPath
}
finally {
    if ($process -and -not $process.HasExited) {
        Stop-Process -Id $process.Id -Force -ErrorAction SilentlyContinue
    }
    foreach ($ruleName in $ruleNames) {
        Remove-NetFirewallRule -DisplayName $ruleName -ErrorAction SilentlyContinue
    }
}
