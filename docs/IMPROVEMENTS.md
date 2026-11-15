# Improvements

Possible evolution paths for RETROVERSE, grouped by theme. Prioritize as needed.

## Performance and memory

1. **Streaming large ISOs** ✅ : streaming implémenté pour local, remote et archives (64 Ko chunks).
2. **Controlled concurrency** ✅ : semaphore (2 permits) dans AppState.
3. **Result caching** ✅ : skip si fichier inchangé (hashed_mtime/hashed_size).
4. **Virtual pagination** ✅ (virtualized list view).

## UX and interface

5. **Real hash cancellation** ✅ (`cancel_hash` dédié).
6. **Granular progress** ✅ (fichier actuel, taille, temps écoulé, historique).
7. **Advanced filters** ✅ (empreintes manquantes, sans métadonnées, récents, favoris, genre).
8. **Detail view tabs** ✅ ("Médias" ajouté).
9. **Dark/light mode** ✅ (déjà présent, sélecteur dans Paramètres).
10. **Fuzzy search** and autocomplete ✅ (backend strsim + dropdown autocomplete).

## Metadata and referentials

11. **Manual field editing** ✅ (genre, dates, synopsis persistés).
12. **Multi-genre per game** ✅ (select multiple).
13. **Automatic referential sync** ✅ (déjà au démarrage).
14. **Metadata import/export** ✅ (export JSON, export DAT, preservation packs).
15. **Enrichment history** with revert capability ✅ (table enrichment_history + revert).

## Features

16. **Duplicate detection** ✅ (page Doublons).
17. **Collections/playlists** ✅ (page Collections, favoris, tags).
18. **Statistics** ✅ (graphiques par plateforme).
19. **Game launching** ✅ (bouton "Ouvrir le fichier").
20. **Incremental scan** ✅ (skip si size + mtime inchangés).
21. **Box art support** ✅ (covers dans les cartes et la fiche).
22. **Offline mode** ✅ (cache des covers via asset protocol).

## Quality and development

23. **Tests** ✅ : 11 tests Rust (identify) + Playwright E2E (navigation, pages, raccourcis).
24. **CI** 🔄 : eslint config, Playwright config (réseau requis).
25. **Logging** ✅ (tracing-appender + tracing-subscriber).
26. **Error handling** ✅ (messages explicites, validation des chemins).
27. **Security** ✅ (validation des chemins, path traversal).
28. **i18n** ✅ (fr/en/es/pt/de/it complets : 179 clés chacun).

## Architecture

29. **Refactoring** ✅ (hashing.rs, dat.rs).
30. **Database** ✅ (migrations v1-v29, auto-backup, list/restore).
31. **Configuration** ✅ (Settings page).

## Nouvelles pistes (2026-11)

### Performance
- **Index DB** ✅ : 8 index ajoutés (migration v28).
- **FTS5** ✅ : table `games_fts` rebuildée (migration v29), recherche robuste avec fallback LIKE.
- **Concurrence contrôlée** ✅ : semaphore (2 permits) dans AppState.
- **Mémoïsation** ✅ : `sortedGames` dans Library.tsx.
- **Parallélisation du scan** ✅ : dossiers top-level traités en parallèle (std::thread::scope).
- **Buffers réutilisables** ✅ : téléchargements distants streamés sur disque avec buffer 64 Ko réutilisable.

### Fonctionnalités
- **Recommandations** basées sur les genres/plateformes possédés.
- **Détection de jeux manquants** dans une série.
- **Tracking des prix** (marché rétro).
- **Alertes de nouveaux dumps** (No-Intro, Redump).
- **Import depuis d'autres gestionnaires** : LaunchBox, EmuDeck, ES-DE, Playnite.
- **Export vers formats standards** ✅ : DAT (clrmamepro) + XML (HyperSpin).
- **Mode présentation (kiosque)** ✅ : page /kiosk plein écran avec diaporama auto.
- **Comparaison de versions** ✅ : onglet "Versions" dans la fiche jeu (fichiers + jeux liés).
- **Raccourcis clavier** ✅ : Ctrl/⌘+K recherche, Ctrl/⌘+1..9 navigation, ? aide, Échap.

### Qualité
- **Tests unitaires** supplémentaires (scanner, metadata, hashing).
- **Tests E2E** Playwright complets.
- **i18n complet** (traductions es/pt/de/it).
- **Documentation** : rustdoc, Storybook pour les composants.

### Extensibilité
- **Thèmes communautaires** (skins).
- **Partage de collections** (anonymisé).
- **Contributions de métadonnées** vers ScreenScraper.
- **Plugin system** pour étendre les fonctionnalités.
- **API REST locale** pour intégration avec d'autres outils.
- **SteamGridDB** pour les artworks supplémentaires.
- **RetroAchievements** ✅ : recherche, récupération et affichage des succès (onglet "Succès").
