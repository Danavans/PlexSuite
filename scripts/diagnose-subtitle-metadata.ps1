param(
    [string]$SettingsPath = "$PSScriptRoot/../src-tauri/target/release/settings.json",
    [string]$Show = 'Barry',
    [int]$Season = 3
)
# Read-only comparison. Credentials stay in headers; output is an allowlist of
# classification fields, never response bodies, URLs, settings, or exceptions.
$ErrorActionPreference = 'Stop'
try {
    $connection = Get-Content -LiteralPath $SettingsPath -Raw | ConvertFrom-Json
    $baseUrl = $connection.plex_url.TrimEnd('/')
    $headers = @{ 'X-Plex-Token' = $connection.plex_token; Accept = 'application/json' }
    $sections = Invoke-RestMethod "$baseUrl/library/sections" -Headers $headers -TimeoutSec 20
    foreach ($library in @($sections.MediaContainer.Directory | Where-Object type -eq 'show')) {
        $shows = Invoke-RestMethod "$baseUrl/library/sections/$($library.key)/all?type=2" -Headers $headers -TimeoutSec 20
        $target = $shows.MediaContainer.Metadata | Where-Object title -eq $Show | Select-Object -First 1
        if (!$target) { continue }
        $episodes = Invoke-RestMethod "$baseUrl/library/metadata/$($target.ratingKey)/allLeaves" -Headers $headers -TimeoutSec 20
        $episode = $episodes.MediaContainer.Metadata | Where-Object parentIndex -eq $Season | Select-Object -First 1
        if (!$episode) { throw 'No episode in requested scope' }
        $endpoint = "$baseUrl/library/metadata/$($episode.ratingKey)?includeMedia=1&includeAllStreams=1"
        $jsonResponse = Invoke-WebRequest -UseBasicParsing $endpoint -Headers $headers -TimeoutSec 20
        Write-Output ('JSON request content type: ' + $jsonResponse.Headers['Content-Type'])
        $jsonText = if ($jsonResponse.Content -is [byte[]]) { [Text.Encoding]::UTF8.GetString($jsonResponse.Content) } else { $jsonResponse.Content }
        $json = $jsonText | ConvertFrom-Json -AsHashtable
        $headers.Accept = 'application/xml'
        [xml]$xml = (Invoke-WebRequest -UseBasicParsing $endpoint -Headers $headers -TimeoutSec 20).Content
        foreach ($stream in @($json.MediaContainer.Metadata.Media.Part.Stream | Where-Object { $_.streamType -eq 3 -and $_.key })) {
            [pscustomobject]@{ representation='JSON'; id=$stream.id; indexPresent=$stream.Contains('index'); index=$stream.index; transientPresent=$stream.Contains('transient'); transient=$stream.transient; keyMatches=($stream.key -eq "/library/streams/$($stream.id)") } | ConvertTo-Json -Compress
        }
        foreach ($stream in $xml.SelectNodes('//Stream[@streamType="3"][@key]')) {
            [pscustomobject]@{ representation='XML'; id=$stream.GetAttribute('id'); indexPresent=$stream.HasAttribute('index'); index=$stream.GetAttribute('index'); transientPresent=$stream.HasAttribute('transient'); transient=$stream.GetAttribute('transient'); keyMatches=($stream.GetAttribute('key') -eq "/library/streams/$($stream.GetAttribute('id'))") } | ConvertTo-Json -Compress
        }
        Write-Output 'Read-only JSON/XML comparison complete (first episode of requested season).'
        exit 0
    }
    throw 'Show not found'
} catch {
    Write-Output ('Read-only Plex diagnostic failed: ' + $_.Exception.GetType().Name + ' at line ' + $_.InvocationInfo.ScriptLineNumber)
    exit 1
}
