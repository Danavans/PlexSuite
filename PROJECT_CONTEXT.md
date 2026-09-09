# PlexSuite - Project Context

## Resume rapide
- Projet dans `D:\Medias\Divers\Tools\PlexSuite\PlexSuite` (Tauri + Svelte).
- Objectif: app 4-en-1 pour Plex (Trash Selector, Sub Uploader, Sub Selector, Plexmatch Generator) avec UI en anglais.
- Theme: look Plex Trash Selector (fond sombre global, cards sombres, texte clair, accents orange). Aucun panel clair.

## Agent instructions
- See `AGENTS.md` for operational rules and editing guidance.
- If `AGENTS.md` conflicts with this document, follow `AGENTS.md`.

## Decisions UX principales
- Layout unique pour tous les onglets: contenu aligne en haut, meme marges, pas de centrage vertical.
- Background sombre applique a la racine (html/body/#app-root) pour couvrir 100% de la hauteur.
- Status global en bas (barre discrete) visible sur tous les onglets.
- Onglet Settings = credentials + logs (pas de popup obligatoire au demarrage).
- Header: badges "PlexSuite" a gauche et "Connected" a droite sur la meme ligne; onglets a gauche, description a droite.
- Tabs en style pilule (container + boutons actifs dynamiques), avec animations de chargement type "rise".
- Initialisation: Aucune library selectionnee par defaut au lancement ("Select a library"). Reset selection si on revient sur l'option par defaut.

## Onglets et layouts
- Trash Selector: layout 2 colonnes / 2 lignes. Gauche: Library Scope (haut) + Purge Controls (bas). Droite: Preview List (colonne pleine). Preview List scroll interne max-height 480px.
- Sub Uploader: Library & Target + Subtitle Source cote a cote (50/50). Preview en dessous pleine largeur. Choix dossier via bouton (pas de drag/drop).
- Sub Selector: deux colonnes responsives. A gauche, Library & Target avec le scan, puis le panneau compact Subtitle Cleanup. A droite, Available Subtitles avec les variantes exactes et Set as Default. Sur petit ecran, les colonnes s'empilent naturellement.
- Plexmatch Generator: TMDb lookup en haut, episodes + video files cote a cote, mapping dessous, preferences en bas, preview a droite.

## Sub Selector cleanup (September 2026)
- Scan and Set as Default use JSON and exact language/region/script, Forced and SDH variants. Set as Default changes only matching streams for the Plex user associated with the token; episodes without a match remain unchanged, and each media version/part is handled independently.
- Cleanup is compact and has unchecked-by-default Physical Sidecar, Plex Uploaded and Unknown External categories. Unknown External is hidden when its count is zero. Embedded has no selectable category and is always protected from deletion.
- Confirmation includes category-specific disk-deletion warnings. Removal uses Plex DELETE only and revalidates current scope, identity and selected category immediately before each stream deletion; changed, malformed, conflicting or unsafe streams are skipped.
- Root cause confirmed with a read-only JSON/XML comparison: Plex omits external stream index; python-plexapi defaults it to -1. PlexSuite now handles the absent attribute equivalently while retaining malformed values as uncertain.
- Development diagnostic: `scripts/diagnose-subtitle-metadata.ps1` (PowerShell 7, GET only, token-free classification fields).

## Sub Uploader (etat actuel)
- Choix dossier via bouton (pas de drag/drop).
- Champ Episode optionnel supprime.
- Preview affiche episode a gauche (format `S01E01 - Title`) et subtitle a droite, en bulles 50/50.
- Mapping Interactif: Drag & Drop supporté pour corriger manuellement l'association fichier <-> épisode.
- Interaction: Clic pour étendre le nom du fichier (multi-ligne), Drag pour déplacer.
- Statut affiche un rond vert (meme couleur que connected) si matched.
- Upload corrige: endpoint Plex utilise `/library/metadata/{ratingKey}/subtitles` avec body raw + params title/format.

## Plexmatch Generator (etat actuel)
- Layout stable avec mapping auto par ordre.
- Liste Fichiers: Reorder via drag-and-drop. Interaction unifiée "Clic pour étendre / Drag pour déplacer".
- Correctif CSS: Noms de fichiers longs tronqués proprement (pas de débordement sous la scrollbar).
- Root path depth: 0 = dossier parent, -1 = fichier seul, +n = remonte.
- Save .plexmatch via dialog uniquement.
- Confirmation de sauvegarde en modal theme, dialog affiche a chaque export.

## Trash Selector (etat actuel)
- Load shows auto sur selection library (pas de bouton).
- Target mode supprime.
- Preview list et actions se partagent la colonne gauche/droite.
- Confirmation purge + modal de fin de purge en theme app.

## Architecture (Refactoring Janvier 2026)
- Modularité: Application découpée en composants Svelte (`TrashTab`, `SubsTab`, `PlexmatchTab`, `SettingsTab`).
- État Centralisé: `appState.svelte.js` gère la logique métier et les données partagées via Svelte 5 Runes.
- Syntaxe: Migration complète vers Svelte 5 (`onclick`, `$state`).

## Plex HTTP transport (September 2026)
- Un unique `reqwest::Client` Plex, de durée de vie du processus, est partagé par les GET, DELETE, POST d'upload de sous-titres et PUT de sélection de sous-titres.
- Sa création ne génère aucun trafic en arrière-plan quand l'application est inactive. Les comportements de requête et timeouts existants restent inchangés.
- Toute future opération HTTP Plex doit réutiliser ce client partagé. TMDb conserve ses créations de client actuelles et n'a pas été modifié par ce refactoring.

## Logs
- Panneau Logs dans Settings, alimenté par `setStatus()`, avec timestamp + onglet. Les logs sont uniquement en session et ne sont pas persistés automatiquement.
- Debug mode est optionnel et désactivé par défaut. La rétention est de 200 logs normalement et de 10 000 quand Debug mode est actif.
- Les logs courants peuvent être exportés en texte brut. En Debug mode, Sub Selector et Trash Selector ajoutent des diagnostics détaillés de durée de scan.

## Icons
- Nouvelle icone generee (P stylise, fond sombre + accent orange) remplace toutes les PNG et icon.ico dans `src-tauri/icons`.
- `bundle.active` reste false pour build portable. `bundle.icon` reste defini.
- Tentative d'embed via tauri-winres retiree (conflit resource VERSION). En cas d'icone non mise a jour: vider cache Windows.

## Build / nettoyage
- Build portable Windows: `npm run tauri build` puis exe dans `src-tauri/target/release/plex-suite.exe`.
- Dossiers safe a supprimer: `src-tauri/target`, `node_modules`, `.svelte-kit`, `build`.

## Fichiers modifies principaux
- `src/lib/appState.svelte.js`: Store central.
- `src/lib/components/SubSelectorTab.svelte`: Scan, sélection de variantes exactes et cleanup sûr des sous-titres.
- `src/lib/components/SettingsTab.svelte`: Réglages, Debug mode et export des logs.
- `src/lib/components/TrashTab.svelte`: Preview Trash et diagnostics de scan.
- `src/lib/components/*.svelte`: Autres composants par onglet.
- `src/routes/+page.svelte`: Layout controller.
- `src/app.css`: theme global, layout grids, status dot, subs preview bubbles, etc.
- `src/app.html`: racine `#app-root` pour background global.
- `src-tauri/src/plex.rs`: Helpers Plex, client HTTP partagé, diagnostics et upload via metadata endpoint.
- `src-tauri/src/plex/subtitles.rs`: Parsing, classification et sécurité des sous-titres.
- `src-tauri/src/lib.rs`: Commandes Tauri, dont upload_subtitles avec episode_rating_key direct.
- `src-tauri/tauri.conf.json`: bundle inactive, bundle icon set, windows icon property removed.

## Notes
- UI en anglais partout.
- Pas de police blanche sur fond clair.
- Pas de card claire nulle part.
- Onglet par defaut: "trash" (activeTab initialise a "trash").
- Fenetre Tauri par defaut: 1200x815.

## Skill creee
- Skill Codex: `portable-desktop-app` dans `C:\Users\Danavans\.codex\skills\portable-desktop-app`.
- Declencheurs: "app portable", "portable desktop app", "Windows + Linux", "single portable executable".
- Stack par defaut: Tauri + Svelte + theme PlexSuite. Pas d'installers par defaut.

## Changelog
- 2026-01-13 05:30:00: Startup: Default library selection set to empty ("Select a library"). SubsTab resets selection when library cleared.
- 2026-01-12 22:50:00: Refactoring: Component split, Svelte 5 migration, Unified Drag & Drop + Click-to-expand UI for Subs and PlexMatch tabs.
- 2026-01-11 19:50:00: PlexMatch drag reorder from full file rows, remove reorder handle/arrows, auto-scroll while dragging, mapping drag for files and episodes with column-only feedback, wrap long file names without horizontal scroll, clamp path depth to max parent depth, replace mapping remove icon with blocked circle.
- 2026-01-11 19:50:00: PlexMatch save confirm modal + dialog always prompts, Trash purge completion modal, copy + spacing tweaks, PlexMatch spacing/preview width refinements.
- 2026-01-09 21:28:51: Update layouts (Trash/Subs), subtitle upload fix (metadata endpoint), logs panel added, tab names renamed, PlexMatch tweaks, dark theme refinements.
- 2026-01-09 21:45:12: Sub uploader preview formatting, status dot styling, logs + settings updates, icon generation, README updated, bundle config adjustments.
- 2026-01-10 00:00:00: Header rework (badges alignes, tabs a gauche, description a droite), tabs style/animations, panel lift animations, settings panel aligne a gauche, window height 815, badge size adjustments.
- 2026-09-09: Sub Selector ajoute: scan de variantes exactes, Set as Default par version/part et gestion des episodes sans correspondance.
- 2026-09-09: Subtitle Cleanup finalise: classification Physical Sidecar / Plex Uploaded / Unknown External, Embedded protege, avertissements et revalidation Plex DELETE avant suppression.
- 2026-09-09: UX Sub Selector stabilisee: layout desktop a deux colonnes responsive et panneau cleanup compact.
- 2026-09-09: Logs enrichis avec Debug mode optionnel, export texte brut et diagnostics de timing pour les scans Sub Selector et Trash Selector.
- 2026-09-09: Investigation de performance/transport: client reqwest Plex partage pour les operations GET, DELETE, POST et PUT, sans modifier les comportements ou timeouts existants.
