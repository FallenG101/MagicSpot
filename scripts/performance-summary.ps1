param(
    [Parameter(Mandatory = $true)]
    [string[]] $Path,

    # Label playback runs according to how they were prepared. Page runs use
    # the app's warm-header and warm-row flags instead.
    [string] $PlaybackScenario = 'unlabeled'
)

Set-StrictMode -Version Latest
$samples = [System.Collections.Generic.List[object]]::new()

foreach ($file in (Resolve-Path -Path $Path)) {
    $pages = @{}
    foreach ($line in [System.IO.File]::ReadLines($file.ProviderPath)) {
        if ($line -match 'page timing #(?<id>\d+) start kind=(?<kind>album|playlist) source=(?<source>\w+) warm_header=(?<header>true|false) warm_rows=(?<rows>true|false)') {
            $warm = if ($Matches.header -eq 'true' -and $Matches.rows -eq 'true') {
                'warm'
            } elseif ($Matches.header -eq 'false' -and $Matches.rows -eq 'false') {
                'cold'
            } else {
                'partial'
            }
            $pages[$Matches.id] = "$($Matches.kind)/$warm/$($Matches.source)"
            continue
        }
        if ($line -match 'page timing #(?<id>\d+) (?<metric>metadata_response|first_page_response|header_frame|rows_frame)_ms=(?<ms>\d+)') {
            $id = $Matches.id
            if ($pages.ContainsKey($id)) {
                $samples.Add([pscustomobject]@{
                    Scenario = $pages[$id]
                    Metric = $Matches.metric
                    Ms = [int] $Matches.ms
                })
            }
            continue
        }
        if ($line -match 'playback timing #\d+ summary: kind=(?<kind>\S+).*first_audio=(?<ms>\d+|n/a) ms') {
            if ($Matches.ms -ne 'n/a') {
                $samples.Add([pscustomobject]@{
                    Scenario = "playback/$PlaybackScenario/$($Matches.kind)"
                    Metric = 'first_audio_queued'
                    Ms = [int] $Matches.ms
                })
            }
            continue
        }
        if ($line -match 'startup first_ui_ms=(?<ms>\d+)') {
            $samples.Add([pscustomobject]@{
                Scenario = 'startup'
                Metric = 'first_ui_frame'
                Ms = [int] $Matches.ms
            })
        }
    }
}

$summary = foreach ($group in ($samples | Group-Object -Property Scenario, Metric)) {
    $ordered = @($group.Group | ForEach-Object { $_.Ms } | Sort-Object)
    $count = $ordered.Count
    $middle = [int] [Math]::Floor($count / 2)
    $median = if ($count % 2 -eq 0) {
        ($ordered[$middle - 1] + $ordered[$middle]) / 2
    } else {
        $ordered[$middle]
    }
    $p95Index = [Math]::Max(0, [int] [Math]::Ceiling($count * 0.95) - 1)
    [pscustomobject]@{
        Scenario = $group.Group[0].Scenario
        Metric = $group.Group[0].Metric
        Samples = $count
        MedianMs = $median
        P95Ms = $ordered[$p95Index]
    }
}

$summary | Sort-Object Scenario, Metric
