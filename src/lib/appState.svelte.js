import { invoke } from "@tauri-apps/api/core";

/**
 * @typedef {"info" | "success" | "error" | "debug"} StatusType
 * @typedef {{ type: StatusType, message: string }} Status
 * @typedef {{ id: string, title: string }} PlexLibrary
 * @typedef {{ rating_key: string, title: string }} PlexShow
 * @typedef {{ rating_key: string, title: string, index?: number }} PlexSeason
 * @typedef {{ type: string, message: string, timestamp: string, tab: string }} LogEntry
 */

class AppState {
    // UI State
    activeTab = $state("trash");
    activityCount = $state(0);
    plexRequests = $state(0);
    switching = $state(true);
    settingsWriting = $state(false);
    contextVersion = $state(0);
    librariesLoaded = $state(false);
    /** @type {{id: string, name: string, url: string, token: string}[]} */
    servers = $state([]);
    /** @type {string | null} */
    activeServerId = $state(null);
    /** @type {Record<string, string>} */
    serverStates = $state({});
    get activeServer() { return this.servers.find(s => s.id === this.activeServerId) ?? null; }
    get serverUrl() { return this.activeServer?.url ?? ""; }
    get token() { return this.activeServer?.token ?? ""; }
    get isBusy() { return this.activityCount > 0 || this.plexRequests > 0 || this.switching; }
    set isBusy(value) { this.activityCount = Math.max(0, this.activityCount + (value ? 1 : -1)); }
    get switchBlocked() { return this.isBusy || this.settingsWriting; }
    get connectionState() { return this.activeServerId ? this.serverStates[this.activeServerId] ?? "Unknown" : "No server configured"; }
    /** @type {Status} */
    status = $state({ type: "info", message: "" });
    /** @type {LogEntry[]} */
    appLogs = $state([]);
    debugMode = $state(false);
    
    // Connection State
    isConnected = $state(false);
    connectionMessage = $state("");
    
    // Credentials
    tmdbKey = $state("");

    // Plex Data
    /** @type {PlexLibrary[]} */
    libraries = $state([]);
    /** @type {PlexShow[]} */
    shows = $state([]);
    /** @type {PlexSeason[]} */
    seasons = $state([]);

    // Selection State
    selectedLibraryId = $state("");
    selectedLibraryTitle = $state("");
    /** @type {PlexShow | null} */
    selectedShow = $state(null);
    selectedShowKey = $state("");
    /** @type {PlexSeason | null} */
    selectedSeason = $state(null);
    selectedSeasonKey = $state("");

    // Helpers
    showsRequestId = 0;

    constructor() {
        // Auto-load settings on init
        this.loadSettings();
    }

    /**
     * @param {StatusType} type
     * @param {string} message
     */
    setStatus(type, message) {
        message = this.sanitize(message);
        this.status = { type, message };
        const timestamp = new Date().toLocaleTimeString("en-GB", { hour12: false });
        let tabLabel = "System";
        if (this.activeTab === "trash") tabLabel = "Trash";
        else if (this.activeTab === "subselector") tabLabel = "Sub Selector";
        else if (this.activeTab === "subs") tabLabel = "Subtitles";
        else if (this.activeTab === "plexmatch") tabLabel = "PlexMatch";
        else if (this.activeTab === "settings") tabLabel = "Settings";

        this.addLog(type, message, tabLabel);
    }

    /** @param {StatusType} type @param {string} message @param {string} tab */
    addLog(type, message, tab) {
        message = this.sanitize(message);
        this.appLogs = [{ type, message, timestamp: new Date().toLocaleTimeString("en-GB", { hour12: false }), tab }, ...this.appLogs]
            .slice(0, this.debugMode ? 10000 : 200);
    }

    /** @param {string} message @param {string} tab */
    addDebugLog(message, tab) {
        if (this.debugMode) this.addLog("debug", message, tab);
    }

    /** @param {boolean} enabled */
    setDebugMode(enabled) {
        this.debugMode = enabled;
        if (!enabled) this.appLogs = this.appLogs.slice(0, 200);
    }

    /** @param {string} message */
    sanitize(message) {
        let safe = String(message).replace(/([?&](?:X-Plex-Token|token|api_key)=)[^&\s]+/gi, "$1[REDACTED]")
            .replace(/(X-Plex-Token[=:]\s*)[^\s,]+/gi, "$1[REDACTED]");
        for (const secret of [...this.servers.map(s => s.token), this.tmdbKey].filter(Boolean)) {
            safe = safe.split(secret).join("[REDACTED]").split(encodeURIComponent(secret)).join("[REDACTED]");
        }
        return safe;
    }

    clearPlexContext() {
        this.contextVersion++;
        this.showsRequestId++;
        this.libraries = []; this.shows = []; this.seasons = [];
        this.selectedLibraryId = ""; this.selectedLibraryTitle = "";
        this.selectedShow = null; this.selectedShowKey = "";
        this.selectedSeason = null; this.selectedSeasonKey = "";
        this.isConnected = false; this.librariesLoaded = false;
        this.connectionMessage = "";
        this.setStatus("info", this.activeServerId ? "Plex context changed. Test or select this server to connect." : "No Plex server configured.");
        if (this.activeServerId) this.serverStates[this.activeServerId] = "Unknown";
    }

    /** @template T @param {string} command @param {Record<string, any>} [args] @param {boolean} [transition] @returns {Promise<T>} */
    async plexInvoke(command, args = {}, transition = false) {
        if (this.switching && !transition) throw new Error("Plex server is switching. Please retry.");
        const profile = this.activeServer;
        if (!profile) throw new Error("No Plex server configured.");
        const version = this.contextVersion;
        this.plexRequests++;
        try {
            const result = await invoke(command, { ...args, serverUrl: profile.url, token: profile.token });
            if (version !== this.contextVersion) throw new Error("Plex context changed; result discarded.");
            this.serverStates[profile.id] = "Available"; this.isConnected = true;
            return result;
        } catch (error) {
            this.serverStates[profile.id] = "Unavailable"; this.isConnected = false;
            throw new Error(this.sanitize(String(error)));
        } finally { this.plexRequests--; }
    }

    async loadSettings() {
        try {
            const settings = await invoke("get_settings");
            this.servers = settings.plex.servers;
            this.activeServerId = settings.plex.active_server_id;
            this.tmdbKey = settings.tmdb_key ?? "";
            this.clearPlexContext();
            await this.connectToPlex(true);
        } catch (error) { this.setStatus("error", `Failed to load settings: ${error}`); }
        finally { this.switching = false; }
    }

    /** @param {{id: string, name: string, url: string, token: string}[]} servers @param {string | null} activeId @param {boolean} [refresh] */
    async persistProfiles(servers, activeId, refresh = false) {
        if (this.switchBlocked) throw new Error("Wait for the current operation before changing Plex servers.");
        this.switching = true;
        const old = this.activeServer;
        const next = servers.find(s => s.id === activeId);
        const changed = old?.id !== next?.id || old?.url !== next?.url || old?.token !== next?.token;
        try {
            await invoke("save_plex_config", { plex: { servers, active_server_id: activeId } });
            this.servers = servers; this.activeServerId = activeId;
            if (changed || refresh) this.clearPlexContext();
            if (refresh) await this.connectToPlex(true);
        } finally { this.switching = false; }
    }

    /** @param {string} id */
    async switchServer(id) {
        if (this.switchBlocked || !this.servers.some(s => s.id === id)) return;
        try { await this.persistProfiles(this.servers, id, true); }
        catch (error) { this.setStatus("error", `Server selection failed: ${error}`); }
    }

    /** @param {{id: string, name: string, url: string, token: string}} profile */
    async saveProfile(profile) {
        const servers = this.servers.some(s => s.id === profile.id)
            ? this.servers.map(s => s.id === profile.id ? profile : s) : [...this.servers, profile];
        const previous = this.servers.find(s => s.id === profile.id);
        await this.persistProfiles(servers, this.activeServerId ?? profile.id);
        if (!previous || previous.url !== profile.url || previous.token !== profile.token) this.serverStates[profile.id] = "Unknown";
    }

    /** @param {string} id */
    async deleteProfile(id) {
        const servers = this.servers.filter(s => s.id !== id);
        const activeId = this.activeServerId === id ? servers[0]?.id ?? null : this.activeServerId;
        await this.persistProfiles(servers, activeId, this.activeServerId === id);
        delete this.serverStates[id];
    }

    /** @param {string} key */
    async saveTmdbKey(key) {
        if (this.settingsWriting || this.switching) throw new Error("Settings are being saved. Please retry.");
        this.settingsWriting = true;
        try { await invoke("save_tmdb_key", { tmdbKey: key }); this.tmdbKey = key.trim(); }
        finally { this.settingsWriting = false; }
    }

    /** @param {{id: string, url: string, token: string}} profile */
    async testProfile(profile) {
        if (this.switchBlocked) throw new Error("Wait for the current operation before testing Plex.");
        this.plexRequests++;
        const saved = this.servers.find(s => s.id === profile.id && s.url === profile.url && s.token === profile.token);
        try {
            await invoke("test_plex_connection", { serverUrl: profile.url, token: profile.token });
            if (saved) { this.serverStates[saved.id] = "Available"; if (saved.id === this.activeServerId) this.isConnected = true; }
        } catch (error) {
            if (saved) { this.serverStates[saved.id] = "Unavailable"; if (saved.id === this.activeServerId) this.isConnected = false; }
            throw new Error(String(error).split(profile.token || "\0").join("[REDACTED]").split(encodeURIComponent(profile.token) || "\0").join("[REDACTED]"));
        } finally { this.plexRequests--; }
    }

    /** @param {boolean} [transition] */
    async connectToPlex(transition = false) {
        if (!this.activeServer) return;
        this.connectionMessage = "Connecting...";
        this.setStatus("info", "Checking Plex connection...");
        this.serverStates[this.activeServer.id] = "Checking";
        try {
            if (["trash", "subs", "subselector"].includes(this.activeTab)) {
                this.libraries = await this.plexInvoke("list_libraries", {}, transition);
                this.librariesLoaded = true;
            } else { await this.plexInvoke("test_plex_connection", {}, transition); }
            this.connectionMessage = "Connected.";
            this.setStatus("success", `Connected to ${this.activeServer?.name ?? "Plex"}.`);
        } catch (error) {
            this.connectionMessage = `Connection failed: ${error}`;
            this.setStatus("error", this.connectionMessage);
        }
    }

    async ensureLibraries() {
        if (this.isBusy || this.librariesLoaded || !this.activeServer || !this.isConnected) return;
        await this.connectToPlex();
    }

    /**
     * @param {string | null} [query]
     * @param {string | null} [libraryIdOverride]
     */
    async loadShows(query = null, libraryIdOverride = null) {
        const libraryId = libraryIdOverride ?? this.selectedLibraryId;
        if (!libraryId) {
            // Only warn if explicitly requested, otherwise silent fail on init
            if (!query && !libraryIdOverride && this.isConnected) {
                 // this.setStatus("error", "Select a library first.");
            }
            return;
        }
        const requestId = ++this.showsRequestId;
        const currentLibrary = this.libraries.find((item) => item.id === libraryId);
        this.selectedLibraryTitle = currentLibrary?.title ?? "";
        
        this.isBusy = true;
        this.setStatus("info", "Loading shows...");
        try {
            const data = await this.plexInvoke("list_shows", {
                serverUrl: this.serverUrl,
                token: this.token,
                libraryId,
                query
            });
            if (requestId !== this.showsRequestId) {
                return;
            }
            this.shows = data;
            
            // Reset selection
            this.selectedShow = null;
            this.selectedShowKey = "";
            this.seasons = [];
            this.selectedSeason = null;
            this.selectedSeasonKey = "";
            
            this.setStatus("success", `Loaded ${this.shows.length} shows.`);
        } catch (error) {
            this.setStatus("error", `Show load failed: ${error}`);
        } finally {
            this.isBusy = false;
        }
    }

    /**
     * @param {PlexShow | null} show
     */
    async loadSeasons(show) {
        if (!show) return;
        this.isBusy = true;
        this.setStatus("info", `Loading seasons for ${show.title}...`);
        try {
            const data = await this.plexInvoke("list_seasons", {
                serverUrl: this.serverUrl,
                token: this.token,
                showRatingKey: show.rating_key
            });
            this.seasons = data;
            this.selectedSeason = null;
            this.selectedSeasonKey = "";
            this.setStatus("success", `Loaded ${this.seasons.length} seasons.`);
        } catch (error) {
            this.setStatus("error", `Season load failed: ${error}`);
        } finally {
            this.isBusy = false;
        }
    }

    /**
     * @param {PlexLibrary} library
     */
    selectLibrary(library) {
        this.selectedLibraryId = library.id;
        this.selectedLibraryTitle = library.title;
        // Reset lower levels
        this.shows = [];
        this.seasons = [];
        this.selectedShow = null;
        this.selectedShowKey = "";
        this.selectedSeason = null;
        this.selectedSeasonKey = "";
        this.loadShows(null, library.id);
    }

    /**
     * @param {PlexShow} show
     */
    selectShow(show) {
        this.selectedShow = show;
        this.selectedShowKey = show?.rating_key ?? "";
        this.selectedSeason = null;
        this.selectedSeasonKey = "";
        this.loadSeasons(show);
    }

    /**
     * @param {PlexSeason | null} season
     */
    selectSeason(season) {
        this.selectedSeason = season;
        this.selectedSeasonKey = season?.rating_key ?? "";
    }
}

export const appState = new AppState();

/** @template T @param {string} command @param {Record<string, any>} [args] @returns {Promise<T>} */
export function invokePlex(command, args) { return appState.plexInvoke(command, args); }
