# PlexTools - Project Context

## Resume rapide
- Projet dans `D:\Medias\Divers\Tools\PlexTools\PlexTools` (Tauri + Svelte).
- Objectif: app 3-en-1 pour Plex (Trash Selector, Sub Uploader, Plexmatch Generator) avec UI en anglais.
- Theme: look Plex Trash Selector (fond sombre global, cards sombres, texte clair, accents orange). Aucun panel clair.

## Agent instructions
- See `AGENTS.md` for operational rules and editing guidance.
- If `AGENTS.md` conflicts with this document, follow `AGENTS.md`.

## Decisions UX principales
- Layout unique pour tous les onglets: contenu aligne en haut, meme marges, pas de centrage vertical.
- Background sombre applique a la racine (html/body/#app-root) pour couvrir 100% de la hauteur.
- Status global en bas (barre discrete) visible sur tous les onglets.
- Onglet Settings = credentials + logs (pas de popup obligatoire au demarrage).
- Header: badges "PlexTools" a gauche et "Connected" a droite sur la meme ligne; onglets a gauche, description a droite.
- Tabs en style pilule (container + boutons actifs dynamiques), avec animations de chargement type "rise".

## Onglets et layouts
- Trash Selector: layout 2 colonnes / 2 lignes. Gauche: Library Scope (haut) + Purge Controls (bas). Droite: Preview List (colonne pleine). Preview List scroll interne max-height 480px.
- Sub Uploader: Library & Target + Subtitle Source cote a cote (50/50). Preview en dessous pleine largeur. Choix dossier via bouton (pas de drag/drop).
- Plexmatch Generator: TMDb lookup en haut, episodes + video files cote a cote, mapping dessous, preferences en bas, preview a droite.

## Sub Uploader (etat actuel)
- Choix dossier via bouton (pas de drag/drop).
- Champ Episode optionnel supprime.
- Preview affiche episode a gauche (format `S01E01 - Title`) et subtitle a droite, en bulles 50/50.
- Statut affiche un rond vert (meme couleur que connected) si matched.
- Upload corrige: endpoint Plex utilise `/library/metadata/{ratingKey}/subtitles` avec body raw + params title/format.

## Plexmatch Generator (etat actuel)
- Layout stable avec mapping auto par ordre.
- Reorder fichiers via drag-handle + boutons up/down.
- Root path depth: 0 = dossier parent, -1 = fichier seul, +n = remonte.
- Save .plexmatch via dialog uniquement.
- Confirmation de sauvegarde en modal theme, dialog affiche a chaque export.

## Trash Selector (etat actuel)
- Load shows auto sur selection library (pas de bouton).
- Target mode supprime.
- Preview list et actions se partagent la colonne gauche/droite.
- Confirmation purge + modal de fin de purge en theme app.

## Logs
- Nouveau panneau Logs dans Settings, alimente par setStatus().
- Logs avec timestamp + onglet.

## Icons
- Nouvelle icone generee (P stylise, fond sombre + accent orange) remplace toutes les PNG et icon.ico dans `src-tauri/icons`.
- `bundle.active` reste false pour build portable. `bundle.icon` reste defini.
- Tentative d'embed via tauri-winres retiree (conflit resource VERSION). En cas d'icone non mise a jour: vider cache Windows.

## Build / nettoyage
- Build portable Windows: `npm run tauri build` puis exe dans `src-tauri/target/release/plex-tools.exe`.
- Dossiers safe a supprimer: `src-tauri/target`, `node_modules`, `.svelte-kit`, `build`.

## Fichiers modifies principaux
- `src/routes/+page.svelte`: layout onglets, logique Trash/Subs/Plexmatch, logs, UI changes.
- `src/app.css`: theme global, layout grids, status dot, subs preview bubbles, etc.
- `src/app.html`: racine `#app-root` pour background global.
- `src-tauri/src/plex.rs`: upload subtitles via metadata endpoint.
- `src-tauri/src/lib.rs`: upload_subtitles utilise episode_rating_key direct.
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
- Stack par defaut: Tauri + Svelte + theme PlexTools. Pas d'installers par defaut.

## Changelog
- 2026-01-11 00:00:00: PlexMatch save confirm modal + dialog always prompts, Trash purge completion modal, copy + spacing tweaks, PlexMatch spacing/preview width refinements.
- 2026-01-09 21:28:51: Update layouts (Trash/Subs), subtitle upload fix (metadata endpoint), logs panel added, tab names renamed, PlexMatch tweaks, dark theme refinements.
- 2026-01-09 21:45:12: Sub uploader preview formatting, status dot styling, logs + settings updates, icon generation, README updated, bundle config adjustments.
- 2026-01-10 00:00:00: Header rework (badges alignes, tabs a gauche, description a droite), tabs style/animations, panel lift animations, settings panel aligne a gauche, window height 815, badge size adjustments.

## Derniere mise a jour
- 2026-01-11 00:00:00
