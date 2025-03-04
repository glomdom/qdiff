$size = 1KB
$path = "generated2.bin"
$rand = New-Object System.Security.Cryptography.RNGCryptoServiceProvider
$bytes = New-Object byte[] $size
$rand.GetBytes($bytes)

[System.IO.File]::WriteAllBytes($path, $bytes)
Write-Host "generated: $path ($size bytes)"
