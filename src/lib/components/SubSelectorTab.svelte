<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { appState } from "../appState.svelte.js";
  type Key = { languageTag: string; forced: boolean; hearingImpaired: boolean };
  type Variant = { key: Key; language: string; languageCode: string | null; episodeCount: number; totalEpisodes: number; selectedCount: number; missing: string[] };
  type Scan = { variants: Variant[]; counts: Record<string, number>; tracks: number; scanned: number; errors: string[]; episodes: { parts: { streams: { key: string | null; source: string }[] }[] }[] };
  type Result = { applied: number; removed: number; missing: string[]; errors: string[]; skippedUnsafe: number };
  let scan = $state<Scan | null>(null);
  let selected = $state("");
  let confirmOpen = $state(false);
  let summary = $state("");
  let details = $state<string[]>([]);
  let scannedScope = $state("");
  const scope = $derived(JSON.stringify([appState.serverUrl, appState.token, appState.selectedLibraryId, appState.selectedShowKey, appState.selectedSeasonKey]));
  const valid = $derived(scan !== null && scannedScope === scope);
  const uploadedKeys = $derived(valid ? [...new Set(scan!.episodes.flatMap(e => e.parts.flatMap(p => p.streams.filter(s => s.source === "Plex Uploaded" && s.key).map(s => s.key!))))] : []);
  const names = new Intl.DisplayNames(["en"], { type: "language" });
  const regions = new Intl.DisplayNames(["en"], { type: "region" });
  const scripts = new Intl.DisplayNames(["en"], { type: "script" });
  function language(v: Variant) {
    try { return names.of(new Intl.Locale(v.key.languageTag).language) || v.language; } catch { return v.language; }
  }
  function label(v: Variant) {
    let place = "Unspecified variant";
    try { const locale = new Intl.Locale(v.key.languageTag); place = [locale.region ? regions.of(locale.region) : "", locale.script ? scripts.of(locale.script) : ""].filter(Boolean).join(" · ") || "Unspecified variant"; } catch { /* metadata fallback */ }
    return `${place} · ${[v.key.forced ? "Forced" : "", v.key.hearingImpaired ? "SDH" : ""].filter(Boolean).join(" · ") || "Normal"}`;
  }
  const groups = $derived(valid ? [...new Set(scan!.variants.map(language))].sort().map(name => ({ name, variants: scan!.variants.filter(v => language(v) === name) })) : []);
  function args() { return { serverUrl: appState.serverUrl, token: appState.token, libraryId: appState.selectedLibraryId, showRatingKey: appState.selectedShowKey, seasonRatingKey: appState.selectedSeasonKey || null }; }
  function reset() { scan = null; selected = ""; confirmOpen = false; summary = ""; details = []; }
  function changeLibrary() {
    reset();
    const library = appState.libraries.find(l => l.id === appState.selectedLibraryId);
    if (library) appState.selectLibrary(library);
    else { appState.shows = []; appState.selectedLibraryTitle = ""; clearShow(); }
  }
  function clearShow() { appState.selectedShow = null; appState.selectedShowKey = ""; appState.seasons = []; appState.selectSeason(null); }
  function changeShow() { reset(); const show = appState.shows.find(s => s.rating_key === appState.selectedShowKey); if (show) appState.selectShow(show); else clearShow(); }
  function showConfirmation(node: HTMLDialogElement) {
    const previous = document.activeElement;
    node.showModal();
    return { destroy() { node.close(); if (previous instanceof HTMLElement) previous.focus(); } };
  }
  async function run(action: "scan" | "apply" | "remove") {
    if (appState.isBusy || !appState.isConnected || !appState.selectedShowKey || !appState.selectedLibraryId) return;
    if (action !== "scan" && !valid) return;
    const variant = scan?.variants.find(v => JSON.stringify(v.key) === selected);
    if (action === "apply" && !variant) return;
    const request = args(); const requestScope = scope;
    const reviewedKeys = [...uploadedKeys];
    confirmOpen = false; appState.isBusy = true;
    summary = action === "scan" ? "Scanning subtitles..." : action === "apply" ? "Selecting subtitle streams..." : "Removing uploaded subtitles...";
    appState.setStatus("info", summary); details = [];
    try {
      if (action !== "scan") {
        const result = await invoke<Result>(action === "apply" ? "set_subtitle_variant" : "remove_uploaded_subtitles", { ...request, variant: variant?.key, reviewedKeys });
        summary = action === "apply" ? `Applied: ${result.applied} · Missing: ${result.missing.length} · Errors: ${result.errors.length}` : `Removed: ${result.removed} · Errors: ${result.errors.length} · Skipped unsafe: ${result.skippedUnsafe}`;
        details = [...result.missing.map(code => `Missing: ${code}`), ...result.errors];
        appState.setStatus(result.errors.length ? "error" : "success", summary);
      }
      scan = null;
      const fresh = await invoke<Scan>("scan_subtitle_streams", request);
      if (scope === requestScope) { scan = fresh; scannedScope = requestScope; }
      if (action === "scan") { summary = `${fresh.scanned} episodes scanned · ${fresh.tracks} subtitle tracks found · Errors: ${fresh.errors.length}`; appState.setStatus(fresh.errors.length ? "error" : "success", summary); }
      else if (fresh.errors.length) appState.setStatus("error", `${summary}. Refresh has ${fresh.errors.length} error(s).`);
    } catch (error) { scan = null; summary = `${action === "scan" ? "Scan" : "Action or refresh"} failed: ${error}`; appState.setStatus("error", summary); }
    finally { appState.isBusy = false; }
  }
</script>

<section class="grid sub-selector-grid">
  <div class="panel lift-1">
    <h2>Library &amp; Target</h2>
    <div class="field"><label for="selector-library">TV Library</label><select id="selector-library" bind:value={appState.selectedLibraryId} onchange={changeLibrary} disabled={!appState.isConnected}>
      <option value="">Select a library</option>{#each appState.libraries as library}<option value={library.id}>{library.title}</option>{/each}
    </select></div>
    <div class="field"><label for="selector-show">Show</label><select id="selector-show" bind:value={appState.selectedShowKey} onchange={changeShow} disabled={!appState.selectedLibraryId}>
      <option value="">Select a show</option>{#each appState.shows as show}<option value={show.rating_key}>{show.title}</option>{/each}
    </select></div>
    <div class="field"><label for="selector-season">Season</label><select id="selector-season" bind:value={appState.selectedSeasonKey} onchange={() => { reset(); appState.selectSeason(appState.seasons.find(s => s.rating_key === appState.selectedSeasonKey) ?? null); }} disabled={!appState.selectedShow}>
      <option value="">All Seasons</option>{#each appState.seasons as season}<option value={season.rating_key}>{season.title}</option>{/each}
    </select></div>
  </div>
  <div class="panel lift-2">
    <h2>Subtitle Scan</h2>
    <p class="kicker">Scan the selected show or season to compare subtitle variants and review uploaded subtitles.</p>
    <div class="actions"><button data-variant="primary" onclick={() => run("scan")} disabled={appState.isBusy || !appState.isConnected || !appState.selectedShowKey}>Scan Subtitles</button></div>
    {#if summary}<p class="status info" role="status">{summary}</p>{/if}
    {#if details.length}<details><summary>Action details ({details.length})</summary><div class="debug">{#each details as line}<p>{line}</p>{/each}</div></details>{/if}
    {#if valid && scan!.errors.length}<details><summary>Scan errors ({scan!.errors.length})</summary><div class="debug">{#each scan!.errors as error}<p>{error}</p>{/each}</div></details>{/if}
  </div>
  <div class="panel sub-selector-wide lift-3">
    <h2>Available Subtitles</h2>
    {#if !valid}<p class="kicker">Scan subtitles to see available variants.</p>
    {:else if !groups.length}<p class="kicker">No subtitle tracks found in the scanned episodes.</p>
    {:else}<div class="list sub-selector-list">{#each groups as group}<details class="sub-selector-language"><summary>{group.name}</summary>
      {#each group.variants as variant}<div class="sub-selector-variant">
        <label class="list-item"><input type="radio" name="subtitle-variant" bind:group={selected} value={JSON.stringify(variant.key)} disabled={appState.isBusy} /><span>{label(variant)}<br /><small class="kicker">{variant.episodeCount} / {variant.totalEpisodes} episodes · Currently selected: {variant.selectedCount}<br />{variant.key.languageTag}</small></span></label>
        {#if variant.missing.length}<details class="kicker"><summary>Missing ({variant.missing.length})</summary><p>{variant.missing.join(", ")}</p></details>{/if}
      </div>{/each}
    </details>{/each}</div>{/if}
    <p class="kicker">Selects matching tracks for the current Plex user. Episodes without this variant remain unchanged. Each media version and part is matched independently.</p>
    <div class="actions"><button data-variant="primary" onclick={() => run("apply")} disabled={!valid || !selected || appState.isBusy}>Set as Default</button></div>
  </div>
  <div class="panel sub-selector-wide lift-3">
    <h2>Subtitle Cleanup</h2>
    {#if valid}<div class="chip-group">{#each Object.entries(scan!.counts) as [name, count]}<span class="status info">{name}: {count}</span>{/each}</div>{/if}
    <p class="kicker">Only subtitles identified as Plex uploads are eligible. Embedded, physical sidecar and unknown external subtitles are preserved.</p>
    <div class="actions"><button data-variant="primary" onclick={() => confirmOpen = true} disabled={!valid || !uploadedKeys.length || appState.isBusy}>Remove All Uploaded Subtitles</button></div>
  </div>
</section>
{#if confirmOpen && valid}
  <dialog class="modal sub-selector-dialog" use:showConfirmation oncancel={() => confirmOpen = false} aria-labelledby="subtitle-remove-title">
    <h3 id="subtitle-remove-title">Remove {uploadedKeys.length} uploaded subtitles?</h3>
    <p>This will remove subtitles uploaded into Plex from <strong>{appState.selectedShow?.title} — {appState.selectedSeason?.title || "All Seasons"}</strong>.</p>
    <p>Embedded subtitles and physical sidecar subtitles will not be touched. Uncertain streams will be skipped.</p>
    <div class="actions"><button data-variant="ghost" onclick={() => confirmOpen = false}>Cancel</button><button data-variant="primary" onclick={() => run("remove")}>Remove {uploadedKeys.length}</button></div>
  </dialog>
{/if}
