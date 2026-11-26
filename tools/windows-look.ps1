# Starts mortise-desktop on an app, waits for its window, adds the app with Ctrl Enter
# and writes a shot of the desktop before and after.
# The job "Windows looked at" of ci.yml runs it. It needs a session with a desktop.
#
#   tools/windows-look.ps1 -Exe target\debug\mortise-desktop.exe -App fixtures\shop-api -Out look
#
# It writes add-app.png, modules.png, facts.txt, stdout.txt and stderr.txt into -Out.
# It fails when no window shows within 90 seconds, or when Mortise ends by itself.
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [Parameter(Mandatory = $true)][string]$App,
    [Parameter(Mandatory = $true)][string]$Out
)

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

New-Item -ItemType Directory -Force -Path $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$facts = Join-Path $Out 'facts.txt'

function Note([string]$line) {
    Write-Host $line
    Add-Content -Path $facts -Value $line
}

function Shot([string]$name) {
    $bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
    $bitmap = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
    $bitmap.Save((Join-Path $Out $name), [System.Drawing.Imaging.ImageFormat]::Png)
    $graphics.Dispose()
    $bitmap.Dispose()
}

# A hosted runner starts at 1024 by 768, which is smaller than the window of Mortise.
# The cmdlet exists on Windows Server only, so a failure here is not a failure of the look.
try {
    Set-DisplayResolution -Width 1920 -Height 1080 -Force | Out-Null
} catch {
    Note "Set-DisplayResolution failed: $_"
}

Note "Windows: $([System.Environment]::OSVersion.VersionString)"
Note "Session: $env:SESSIONNAME, interactive: $([System.Environment]::UserInteractive)"
Note "Screen: $([System.Windows.Forms.SystemInformation]::VirtualScreen)"

$process = Start-Process -FilePath $Exe -ArgumentList "`"$App`"" -PassThru `
    -RedirectStandardOutput (Join-Path $Out 'stdout.txt') `
    -RedirectStandardError (Join-Path $Out 'stderr.txt')

try {
    $deadline = (Get-Date).AddSeconds(90)
    while ($true) {
        $process.Refresh()
        if ($process.HasExited) {
            Note "Mortise ended by itself with exit code $($process.ExitCode)."
            Shot 'no-window.png'
            exit 1
        }
        if ($process.MainWindowHandle -ne [IntPtr]::Zero) { break }
        if ((Get-Date) -gt $deadline) {
            Note 'No window after 90 seconds.'
            Shot 'no-window.png'
            exit 1
        }
        Start-Sleep -Milliseconds 500
    }
    # Bring the window to the front, then give it time to read the folder and draw.
    $shell = New-Object -ComObject WScript.Shell
    Note "Brought to the front: $($shell.AppActivate($process.Id))"
    Start-Sleep -Seconds 8
    Shot 'add-app.png'

    # Mortise opens on the Add app panel with the folder found. Ctrl Enter adds the app,
    # which writes settings.json in MORTISE_CONFIG_DIR and nothing in the app.
    Note "Brought to the front again: $($shell.AppActivate($process.Id))"
    [System.Windows.Forms.SendKeys]::SendWait('^{ENTER}')
    Start-Sleep -Seconds 8
    Shot 'modules.png'

    $process.Refresh()
    Note "Window title: $($process.MainWindowTitle)"
    $process.Refresh()
    if ($process.HasExited) {
        Note "Mortise ended by itself with exit code $($process.ExitCode)."
        exit 1
    }
    Note 'Mortise was still running after the shot.'
} finally {
    if (-not $process.HasExited) { Stop-Process -Id $process.Id -Force }
}
