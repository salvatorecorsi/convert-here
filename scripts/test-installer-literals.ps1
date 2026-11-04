[CmdletBinding()]
param([string]$InstallerPath)
$ErrorActionPreference = 'Stop'
if (-not $InstallerPath) { $InstallerPath = Join-Path $PSScriptRoot '..\installer\TheConverter.iss' }

function Read-PascalLiteral([string]$Expression) {
    $value = [Text.StringBuilder]::new()
    $index = 0
    while ($index -lt $Expression.Length) {
        if ([char]::IsWhiteSpace($Expression[$index])) { $index++; continue }
        if ($Expression[$index] -eq [char]0x27) {
            $index++
            $closed = $false
            while ($index -lt $Expression.Length) {
                if ($Expression[$index] -eq [char]0x27) {
                    if ($index + 1 -lt $Expression.Length -and $Expression[$index + 1] -eq [char]0x27) {
                        [void]$value.Append([char]0x27)
                        $index += 2
                    } else { $index++; $closed = $true; break }
                } else { [void]$value.Append($Expression[$index]); $index++ }
            }
            if (-not $closed) { throw 'Pascal literal is not terminated.' }
            continue
        }
        $hex = [regex]::Match($Expression.Substring($index), '^#\$([0-9a-fA-F]{1,4})')
        if ($hex.Success) {
            [void]$value.Append([char][Convert]::ToInt32($hex.Groups[1].Value, 16))
            $index += $hex.Length
            continue
        }
        throw "Unsupported Pascal literal token at position $index."
    }
    return $value.ToString()
}

function Split-PascalArguments([string]$Arguments) {
    $result = [Collections.Generic.List[string]]::new()
    $current = [Text.StringBuilder]::new()
    $quoted = $false
    for ($index = 0; $index -lt $Arguments.Length; $index++) {
        $character = $Arguments[$index]
        if ($character -eq [char]0x27) {
            [void]$current.Append($character)
            if ($quoted -and $index + 1 -lt $Arguments.Length -and $Arguments[$index + 1] -eq [char]0x27) {
                [void]$current.Append($Arguments[++$index])
            } else { $quoted = -not $quoted }
        } elseif ($character -eq ',' -and -not $quoted) {
            $result.Add($current.ToString().Trim())
            [void]$current.Clear()
        } else { [void]$current.Append($character) }
    }
    if ($quoted) { throw 'Unterminated Pascal argument.' }
    $result.Add($current.ToString().Trim())
    return $result.ToArray()
}

$source = Get-Content -LiteralPath $InstallerPath -Raw -Encoding UTF8
$function = [regex]::Match($source, '(?ms)^function PowerShellLiteral\([^\r\n]+\r?\n.*?^end;')
if (-not $function.Success) { throw 'Production PowerShellLiteral function was not found.' }
if ($function.Value -notmatch "(?m)^\s*Result := '''' \+ Escaped \+ '''';\s*$") {
    throw 'PowerShellLiteral no longer uses the expected single-quoted wrapper.'
}
$rules = @(
    foreach ($call in [regex]::Matches($function.Value, '(?m)^\s*StringChangeEx\((.*)\);\s*$')) {
        $arguments = @(Split-PascalArguments $call.Groups[1].Value)
        if ($arguments.Count -ne 4 -or $arguments[0] -cne 'Escaped' -or $arguments[3] -cne 'True') {
            throw 'Unexpected StringChangeEx signature in production function.'
        }
        [pscustomobject]@{ From = (Read-PascalLiteral $arguments[1]); To = (Read-PascalLiteral $arguments[2]) }
    }
)
if (-not $rules.Count) { throw 'Production escaping rules were not found.' }

$fixtures = [Collections.Generic.List[object]]::new()
$fixtures.Add([pscustomobject]@{ Name = 'empty'; Text = '' })
foreach ($code in @(0x27, 0x2018, 0x2019, 0x201A, 0x201B)) {
    $fixtures.Add([pscustomobject]@{ Name = ('U+{0:X4}' -f $code); Text = ('left' + [char]$code + 'right') })
}
$allQuotes = -join (@(0x27, 0x2018, 0x2019, 0x201A, 0x201B) | ForEach-Object { [char]$_ })
$fixtures.Add([pscustomobject]@{
    Name = 'mixed path and inert command text'
    Text = ('C:\Convert Here\citt' + [char]0xE0 + ' ' + $allQuotes + ' `backtick $(Write-Output inatteso) $env:PATH "double quotes"' + [char]10 + 'second line')
})

$failures = @()
Write-Output ("PowerShell {0}: {1} production escaping rules" -f $PSVersionTable.PSVersion, $rules.Count)
foreach ($fixture in $fixtures) {
    $escaped = $fixture.Text
    foreach ($rule in $rules) { $escaped = $escaped.Replace($rule.From, $rule.To) }
    $literal = "'" + $escaped + "'"
    $tokens = $null
    $parseErrors = $null
    $ast = [Management.Automation.Language.Parser]::ParseInput($literal, [ref]$tokens, [ref]$parseErrors)
    $valid = $parseErrors.Count -eq 0 -and $ast.EndBlock.Statements.Count -eq 1
    if ($valid) {
        $statement = $ast.EndBlock.Statements[0]
        $valid = $statement -is [Management.Automation.Language.PipelineAst] -and $statement.PipelineElements.Count -eq 1
        if ($valid) {
            $element = $statement.PipelineElements[0]
            $valid = $element -is [Management.Automation.Language.CommandExpressionAst] -and
                $element.Expression -is [Management.Automation.Language.StringConstantExpressionAst] -and
                $element.Expression.Value -ceq $fixture.Text
        }
    }
    if ($valid) { Write-Output ("PASS " + $fixture.Name) }
    else {
        Write-Output ("FAIL " + $fixture.Name + ": parse errors=" + $parseErrors.Count)
        $failures += $fixture.Name
    }
}
if ($failures.Count) { throw ("Literal roundtrip failed: " + ($failures -join ', ')) }
Write-Output ("All {0} literal roundtrips passed without executing the generated text." -f $fixtures.Count)
