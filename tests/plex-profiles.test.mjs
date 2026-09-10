import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, unlinkSync } from 'node:fs';
import { compileModule } from 'svelte/compiler';

// Compile the real Svelte state module; only the Tauri boundary is mocked.
const source = readFileSync(new URL('../src/lib/appState.svelte.js', import.meta.url), 'utf8')
  .replace('import { invoke } from "@tauri-apps/api/core";', 'const invoke = (...args) => globalThis.__plexTestInvoke(...args);');
const generated = new URL('./.appState-test.mjs', import.meta.url);
writeFileSync(generated, compileModule(source, { filename: 'appState.svelte.js', generate: 'client' }).js.code);
let calls = [];
let stored;
let handler;
let iteration = 0;
const profile = (id, name = 'Same name') => ({ id, name, url: `http://${id}:32400`, token: `fixture-${id}` });
const tick = () => new Promise(resolve => setImmediate(resolve));
async function boot(servers = [profile('a'), profile('b')], active = servers[0]?.id ?? null) {
  calls = []; handler = null;
  stored = { plex: { servers, active_server_id: active }, tmdb_key: 'global-fixture' };
  globalThis.__plexTestInvoke = async (command, args) => {
    calls.push({ command, args });
    if (handler) { const outcome = handler(command, args); if (outcome !== undefined) return outcome; }
    if (command === 'get_settings') return structuredClone(stored);
    if (command === 'save_plex_config') { stored.plex = structuredClone(JSON.parse(JSON.stringify(args.plex))); return; }
    if (command === 'save_tmdb_key') { stored.tmdb_key = args.tmdbKey; return; }
    if (command === 'list_libraries') return [{ id: args.serverUrl, title: 'Library' }];
    if (command === 'list_shows') return [];
  };
  const { appState } = await import(`${generated.href}?case=${iteration++}`);
  while (appState.switching) await tick();
  return appState;
}

try {
  await test('Plex profiles and global context', async t => {
    await t.test('save offline, rename stable ID, duplicates, and global TMDb independence', async () => {
      const app = await boot([]);
      calls = [];
      await app.saveTmdbKey('new-global');
      await app.saveProfile(profile('a'));
      await app.saveProfile(profile('b'));
      await app.saveProfile({ ...profile('a'), name: 'Renamed' });
      assert.equal(app.activeServerId, 'a');
      assert.equal(app.servers[0].name, 'Renamed');
      assert.equal(app.servers.length, 2);
      assert.equal(stored.tmdb_key, 'new-global');
      assert.ok(calls.every(c => ['save_plex_config', 'save_tmdb_key'].includes(c.command)));
      assert.equal(app.connectionState, 'Unknown');
    });
    await t.test('switch invalidates shared data and component context; persists selection', async () => {
      const app = await boot();
      app.shows = [{ rating_key: 'old', title: 'Old' }];
      app.selectedLibraryId = 'old'; app.selectedLibraryTitle = 'Old';
      app.selectedShow = app.shows[0]; app.selectedShowKey = 'old';
      app.seasons = [{ rating_key: 'old', title: 'Old' }]; app.selectedSeasonKey = 'old';
      const version = app.contextVersion;
      await app.switchServer('b');
      assert.equal(app.activeServerId, 'b'); assert.equal(stored.plex.active_server_id, 'b');
      assert.equal(app.contextVersion, version + 1);
      assert.equal(app.selectedLibraryId, ''); assert.equal(app.selectedLibraryTitle, '');
      assert.equal(app.selectedShowKey, ''); assert.equal(app.selectedSeasonKey, '');
      assert.equal(app.shows.length, 0); assert.equal(app.seasons.length, 0);
      assert.equal(app.libraries[0].id, profile('b').url);
      const reloaded = await boot(stored.plex.servers, stored.plex.active_server_id);
      assert.equal(reloaded.activeServerId, 'b');
    });
    await t.test('failed connection keeps chosen server and clears previous libraries', async () => {
      const app = await boot();
      handler = command => command === 'list_libraries' ? Promise.reject(new Error('Offline')) : undefined;
      await app.switchServer('b');
      assert.equal(app.activeServerId, 'b'); assert.equal(app.connectionState, 'Unavailable');
      assert.equal(app.libraries.length, 0); assert.equal(app.isBusy, false);
    });
    await t.test('failed persistence leaves active server and data untouched', async () => {
      const app = await boot(); const version = app.contextVersion;
      handler = command => command === 'save_plex_config' ? Promise.reject(new Error('Read-only')) : undefined;
      await app.switchServer('b');
      assert.equal(app.activeServerId, 'a'); assert.equal(app.contextVersion, version);
      assert.equal(app.libraries[0].id, profile('a').url); assert.equal(app.isBusy, false);
    });
    await t.test('overlapping requests, logical operations, and mutations block switching', async () => {
      const app = await boot(); const pending = [];
      handler = command => command === 'list_shows' ? new Promise(resolve => pending.push(resolve)) : undefined;
      const first = app.plexInvoke('list_shows'); const second = app.plexInvoke('list_shows');
      app.isBusy = true; app.isBusy = true; app.isBusy = false;
      pending[0]([]); await first;
      await app.switchServer('b'); assert.equal(app.activeServerId, 'a');
      await assert.rejects(app.saveProfile({ ...profile('a'), token: 'changed' }));
      await assert.rejects(app.deleteProfile('a'));
      pending[1]([]); await second;
      await app.switchServer('b'); assert.equal(app.activeServerId, 'a');
      app.isBusy = false;
      await app.switchServer('b'); assert.equal(app.activeServerId, 'b');
    });
    await t.test('every Plex command blocks context changes until its request settles', async () => {
      const app = await boot();
      const commands = ['list_libraries', 'list_shows', 'list_seasons', 'list_episodes', 'preview_trash', 'purge_trash', 'preview_subtitles', 'upload_subtitles', 'scan_subtitle_streams', 'set_subtitle_variant', 'remove_selected_subtitles'];
      for (const command of commands) {
        let finish;
        handler = name => name === command ? new Promise(resolve => finish = resolve) : undefined;
        const operation = app.plexInvoke(command);
        assert.equal(app.switchBlocked, true, command);
        await app.switchServer('b'); assert.equal(app.activeServerId, 'a', command);
        finish([]); await operation;
        assert.equal(app.switchBlocked, false, command);
      }
      let finishTest;
      handler = name => name === 'test_plex_connection' ? new Promise(resolve => finishTest = resolve) : undefined;
      const testing = app.testProfile(profile('b'));
      await app.switchServer('b'); assert.equal(app.activeServerId, 'a');
      finishTest(); await testing; assert.equal(app.switchBlocked, false);
    });
    await t.test('transition blocks new requests and duplicate switches until commit and refresh finish', async () => {
      const app = await boot(); let finish;
      handler = command => command === 'save_plex_config' ? new Promise(resolve => finish = resolve) : undefined;
      const switching = app.switchServer('b');
      await assert.rejects(app.plexInvoke('list_shows'));
      await app.switchServer('a');
      assert.equal(app.activeServerId, 'a'); assert.equal(app.switchBlocked, true);
      finish(); await switching;
      assert.equal(app.activeServerId, 'b'); assert.equal(app.switchBlocked, false);
    });
    await t.test('inactive delete preserves context; active delete falls back; last delete clears', async () => {
      const app = await boot([profile('a'), profile('b'), profile('c')]);
      const version = app.contextVersion;
      await app.deleteProfile('c'); assert.equal(app.activeServerId, 'a'); assert.equal(app.contextVersion, version);
      await app.deleteProfile('a'); assert.equal(app.activeServerId, 'b');
      await app.deleteProfile('b');
      assert.equal(app.activeServerId, null); assert.equal(stored.plex.active_server_id, null);
      assert.equal(app.libraries.length, 0); assert.equal(app.isConnected, false);
      assert.equal(stored.tmdb_key, 'global-fixture');
    });
    await t.test('test unsaved values without saving or overwriting active status; failures redact secrets', async () => {
      const app = await boot(); calls = [];
      await app.testProfile({ ...profile('a'), url: 'http://draft:32400', token: 'draft-secret' });
      assert.equal(calls[0].args.serverUrl, 'http://draft:32400');
      assert.equal(calls[0].args.token, 'draft-secret');
      assert.equal(app.serverUrl, profile('a').url); assert.equal(calls.length, 1);
      handler = command => command === 'test_plex_connection' ? Promise.reject(new Error('failed draft-secret')) : undefined;
      await assert.rejects(app.testProfile({ ...profile('a'), token: 'draft-secret' }), error => !error.message.includes('draft-secret'));
      assert.equal(app.connectionState, 'Available');
      app.setStatus('error', `URL ?X-Plex-Token=${profile('b').token} and ${profile('a').token}`);
      assert.ok(!app.status.message.includes('fixture-')); assert.ok(!app.appLogs[0].message.includes('fixture-'));
    });
    await t.test('active test updates availability, lazy loads only Plex pages, request routing ignores old credentials', async () => {
      const app = await boot(); app.activeTab = 'plexmatch'; calls = [];
      await app.switchServer('b');
      assert.ok(!calls.some(c => c.command === 'list_libraries'));
      assert.equal(app.libraries.length, 0);
      await app.testProfile(profile('b')); assert.equal(app.connectionState, 'Available');
      app.activeTab = 'trash'; await app.ensureLibraries();
      assert.equal(app.libraries[0].id, profile('b').url);
      await app.plexInvoke('list_shows', { serverUrl: profile('a').url, token: profile('a').token });
      assert.equal(calls.at(-1).args.serverUrl, profile('b').url);
      handler = command => command === 'test_plex_connection' ? Promise.reject(new Error('401')) : undefined;
      await assert.rejects(app.testProfile(profile('b')));
      assert.equal(app.connectionState, 'Unavailable'); assert.equal(app.isConnected, false);
    });
  });
} finally { unlinkSync(generated); delete globalThis.__plexTestInvoke; }
