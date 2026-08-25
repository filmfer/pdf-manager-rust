Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing

$type = @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class W {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT r);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr hWnd, StringBuilder s, int n);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint pid);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc cb, IntPtr lParam);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L; public int T; public int R; public int B; }
    public static IntPtr Find(uint pid, string titleMatch) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((h, l) => {
            if (!IsWindowVisible(h)) return true;
            uint p; GetWindowThreadProcessId(h, out p);
            if (p != pid) return true;
            int len = 1024; var sb = new StringBuilder(len);
            GetWindowTextW(h, sb, len);
            if (sb.ToString().IndexOf(titleMatch, StringComparison.OrdinalIgnoreCase) >= 0) {
                found = h; return false;
            }
            return true;
        }, IntPtr.Zero);
        return found;
    }
}
'@
Add-Type -TypeDefinition $type

Get-Process -Name 'pdf-manager-rust' -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep -Milliseconds 300
$proc = Start-Process -FilePath 'D:\scripts\pdf-manager-rust\target\release\pdf-manager-rust.exe' -PassThru
Start-Sleep -Seconds 3

$hwnd = [W]::Find([uint32]$proc.Id, 'simple')
Write-Host "Found HWND: $hwnd"
if ($hwnd -eq [IntPtr]::Zero) {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    exit 1
}
[W]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 500
$rect = New-Object W+RECT
[W]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
$w = $rect.R - $rect.L
$h = $rect.B - $rect.T
Write-Host "Window rect: $($rect.L),$($rect.T) - $w x $h"
$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($rect.L, $rect.T, 0, 0, $bmp.Size)
$bmp.Save('D:\scripts\pdf-manager-rust\screenshot.png', [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
Write-Host "Screenshot saved."
