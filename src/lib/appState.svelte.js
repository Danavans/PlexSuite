import { invoke } from "@tauri-apps/api/core";

/**
 * @typedef {"info" | "success" | "error"} StatusType
 * @typedef {{ type: StatusType, message: string }} Status
 * @typedef {{ id: string, title: string }} PlexLibrary
 * @typedef {{ rating_key: string, title: string }} PlexShow
 * @typedef {{ rating_key: string, title: string, index?: number }} PlexSeason
 * @typedef {{ type: string, message: string, timestamp: string, tab: string }} LogEntry
 */

class AppState {
    // UI State
    activeTab = $state("trash");
    isBusy = $state(false);
    /** @type {Status} */
    status = $state({ type: "info", message: "" });
    /** @type {LogEntry[]} */
    appLogs = $state([]);
    
    // Connection State
    isConnected = $state(false);
    connectionMessage = $state("");
    
    // Credentials
    serverUrl = $state("");
    token = $state("");
    showToken = $state(false); // UI toggle
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
        this.status = { type, message };
        const timestamp = new Date().toLocaleTimeString("en-GB", { hour12: false });
        let tabLabel = "System";
        if (this.activeTab === "trash") tabLabel = "Trash";
        else if (this.activeTab === "subs") tabLabel = "Subtitles";
        else if (this.activeTab === "plexmatch") tabLabel = "PlexMatch";
        else if (this.activeTab === "settings") tabLabel = "Settings";

        this.appLogs = [
            { type, message, timestamp, tab: tabLabel },
            ...this.appLogs
        ].slice(0, 200);
    }

    async loadSettings() {
        try {
            const settings = await invoke("get_settings");
            this.serverUrl = settings.plex_url ?? "";
            this.token = settings.plex_token ?? "";
            this.tmdbKey = settings.tmdb_key ?? "";
            if (this.serverUrl && this.token) {
                await this.connectToPlex(true);
            } else {
                this.connectionMessage = "";
            }
        } catch (error) {
            this.setStatus("error", `Failed to load settings: ${error}`);
        }
    }

    async saveSettings() {
        if (!this.serverUrl.trim() || !this.token.trim()) {
            this.setStatus("error", "Plex URL and token are required.");
            return;
        }
        this.isBusy = true;
        this.setStatus("info", "Saving settings...");
        try {
            await invoke("save_settings", {
                plexUrl: this.serverUrl,
                plexToken: this.token,
                tmdbKey: this.tmdbKey
            });
            this.setStatus("success", "Settings saved.");
            await this.connectToPlex(false);
        } catch (error) {
            this.setStatus("error", `Save failed: ${error}`);
        } finally {
            this.isBusy = false;
        }
    }

    /**
     * @param {boolean} auto
     */
    async connectToPlex(auto) {
        if (!this.serverUrl.trim() || !this.token.trim()) {
            this.connectionMessage = "Enter Plex URL and token to connect.";
            this.isConnected = false;
            return;
        }
        this.isBusy = true;
        this.connectionMessage = auto ? "Connecting to Plex..." : "Connecting...";
        try {
            const data = await invoke("list_libraries", { serverUrl: this.serverUrl, token: this.token });
            this.libraries = data;
            this.selectedLibraryId = "";
            this.isConnected = true;
            this.connectionMessage = "Connected.";
            if (this.selectedLibraryId) {
                await this.loadShows();
            }
        } catch (error) {
            this.isConnected = false;
            this.connectionMessage = `Connection failed: ${error}`;
        } finally {
            this.isBusy = false;
        }
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
            const data = await invoke("list_shows", {
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
            const data = await invoke("list_seasons", {
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
