[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$assetDirectory = Join-Path (Split-Path -Parent $PSScriptRoot) 'assets'
$source = Join-Path $assetDirectory 'app.svg'
$svg = [System.Xml.XmlDocument]::new()
$svg.XmlResolver = $null
$svg.LoadXml([IO.File]::ReadAllText($source))
$group = $svg.GetElementsByTagName('g')[0]
$strokeColor = [Drawing.ColorTranslator]::FromHtml($group.GetAttribute('stroke'))
$strokeWidth = [single]::Parse($group.GetAttribute('stroke-width'), [Globalization.CultureInfo]::InvariantCulture)
$viewBox = $svg.DocumentElement.GetAttribute('viewBox').Split(' ', [StringSplitOptions]::RemoveEmptyEntries)
if (($viewBox -join ' ') -ne '0 0 256 256') { throw 'The icon renderer expects a 256 × 256 SVG viewBox.' }

# Keep app.svg as the editable source. The small path reader handles only the
# move, line and cubic commands used by this mark; unsupported edits fail clearly.
$paths = [Collections.Generic.List[Drawing.Drawing2D.GraphicsPath]]::new()
foreach ($element in $svg.GetElementsByTagName('path')) {
    $tokens = [regex]::Matches($element.GetAttribute('d'), '[A-Za-z]|-?\d+(?:\.\d+)?')
    $path = [Drawing.Drawing2D.GraphicsPath]::new()
    $index = 0
    $current = [Drawing.PointF]::new(0, 0)
    while ($index -lt $tokens.Count) {
        $command = $tokens[$index++].Value
        switch ($command) {
            'M' {
                $x = [single]::Parse($tokens[$index++].Value, [Globalization.CultureInfo]::InvariantCulture)
                $y = [single]::Parse($tokens[$index++].Value, [Globalization.CultureInfo]::InvariantCulture)
                $current = [Drawing.PointF]::new($x, $y)
                $path.StartFigure()
            }
            'L' {
                $x = [single]::Parse($tokens[$index++].Value, [Globalization.CultureInfo]::InvariantCulture)
                $y = [single]::Parse($tokens[$index++].Value, [Globalization.CultureInfo]::InvariantCulture)
                $next = [Drawing.PointF]::new($x, $y)
                $path.AddLine($current, $next)
                $current = $next
            }
            'C' {
                $coordinates = [single[]]::new(6)
                for ($coordinate = 0; $coordinate -lt 6; $coordinate++) {
                    $coordinates[$coordinate] = [single]::Parse($tokens[$index++].Value, [Globalization.CultureInfo]::InvariantCulture)
                }
                $first = [Drawing.PointF]::new($coordinates[0], $coordinates[1])
                $second = [Drawing.PointF]::new($coordinates[2], $coordinates[3])
                $next = [Drawing.PointF]::new($coordinates[4], $coordinates[5])
                $path.AddBezier($current, $first, $second, $next)
                $current = $next
            }
            default { $path.Dispose(); throw "Unsupported SVG command: $command" }
        }
    }
    $paths.Add($path)
}

function New-IconBitmap([int]$Size) {
    $supersampling = 4
    $large = [Drawing.Bitmap]::new($Size * $supersampling, $Size * $supersampling, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [Drawing.Graphics]::FromImage($large)
    $pen = [Drawing.Pen]::new($strokeColor, $strokeWidth)
    try {
        $graphics.Clear([Drawing.Color]::Transparent)
        $graphics.SmoothingMode = [Drawing.Drawing2D.SmoothingMode]::AntiAlias
        $graphics.PixelOffsetMode = [Drawing.Drawing2D.PixelOffsetMode]::HighQuality
        $graphics.ScaleTransform(($Size * $supersampling / 256.0), ($Size * $supersampling / 256.0))
        $pen.StartCap = [Drawing.Drawing2D.LineCap]::Round
        $pen.EndCap = [Drawing.Drawing2D.LineCap]::Round
        $pen.LineJoin = [Drawing.Drawing2D.LineJoin]::Round
        foreach ($path in $paths) { $graphics.DrawPath($pen, $path) }
        $bitmap = [Drawing.Bitmap]::new($Size, $Size, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
        $resample = [Drawing.Graphics]::FromImage($bitmap)
        try {
            $resample.CompositingMode = [Drawing.Drawing2D.CompositingMode]::SourceCopy
            $resample.InterpolationMode = [Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
            $resample.PixelOffsetMode = [Drawing.Drawing2D.PixelOffsetMode]::HighQuality
            $resample.DrawImage($large, [Drawing.Rectangle]::new(0, 0, $Size, $Size))
        } finally { $resample.Dispose() }
        return $bitmap
    } finally { $pen.Dispose(); $graphics.Dispose(); $large.Dispose() }
}

try {
    $png = New-IconBitmap 256
    try { $png.Save((Join-Path $assetDirectory 'app.png'), [Drawing.Imaging.ImageFormat]::Png) }
    finally { $png.Dispose() }

    $storeIcon = New-IconBitmap 300
    try {
        $storeIconPath = Join-Path (Split-Path -Parent $PSScriptRoot) 'docs/store-assets/store-icon-300.png'
        $storeIcon.Save($storeIconPath, [Drawing.Imaging.ImageFormat]::Png)
    } finally { $storeIcon.Dispose() }

    $sizes = @(16, 24, 32, 48, 64, 128, 256)
    $images = [Collections.Generic.List[byte[]]]::new()
    foreach ($size in $sizes) {
        $bitmap = New-IconBitmap $size
        $memory = [IO.MemoryStream]::new()
        try { $bitmap.Save($memory, [Drawing.Imaging.ImageFormat]::Png); $images.Add($memory.ToArray()) }
        finally { $memory.Dispose(); $bitmap.Dispose() }
    }
    $output = [IO.File]::Create((Join-Path $assetDirectory 'app.ico'))
    $writer = [IO.BinaryWriter]::new($output)
    try {
        $writer.Write([uint16]0)
        $writer.Write([uint16]1)
        $writer.Write([uint16]$sizes.Count)
        $offset = [uint32](6 + 16 * $sizes.Count)
        for ($index = 0; $index -lt $sizes.Count; $index++) {
            $size = $sizes[$index]
            $dimension = if ($size -eq 256) { 0 } else { $size }
            $writer.Write([byte]$dimension)
            $writer.Write([byte]$dimension)
            $writer.Write([byte]0)
            $writer.Write([byte]0)
            $writer.Write([uint16]1)
            $writer.Write([uint16]32)
            $writer.Write([uint32]$images[$index].Length)
            $writer.Write($offset)
            $offset += [uint32]$images[$index].Length
        }
        foreach ($bytes in $images) { $writer.Write($bytes) }
    } finally { $writer.Dispose(); $output.Dispose() }
} finally { foreach ($path in $paths) { $path.Dispose() } }

Write-Output 'Updated transparent app.png, app.ico and the 300px Store icon from app.svg.'
