# Improvements

Possible evolution paths for RETROVERSE, grouped by theme. Prioritize as needed.

## Performance and memory

1. **Streaming large ISOs** ✅ : streaming implémenté pour local, remote et archives (64 Ko chunks).
2. **Controlled concurrency**: ❌ queue (semaphore) non implémentée.
3. **Result caching** ✅ : skip si fichier inchangé (hashed_mtime/hashed_size).
4. **Virtual pagination** ✅ (virtualized list view).

## UX and interface

5. **Real hash cancellation** ✅ (`cancel_hash` dédié).
6. **Granular progress** ✅ (fichier actuel, taille, temps écoulé, historique).
7. **Advanced filters** ✅ (empreintes manquantes, sans métadonnées, récents, favoris, genre).
8. **Detail view tabs** ✅ ("Médias" ajouté).
9. **Dark/light mode** ✅ (déjà présent, sélecteur dans Paramètres).
10. **Fuzzy search** and autocomplete ❌.

## Metadata and referentials

11. **Manual field editing** ✅ (genre, dates, synopsis persistés).
12. **Multi-genre per game** ✅ (select multiple).
13. **Automatic referential sync** ✅ (déjà au démarrage).
14. **Metadata import/export** ✅ (export JSON, export DAT, preservation packs).
15. **Enrichment history** with revert capability ❌.

## Features

16. **Duplicate detection** ✅ (page Doublons).
17. **Collections/playlists** ✅ (page Collections, favoris, tags).
18. **Statistics** ✅ (graphiques par plateforme).
19. **Game launching** ✅ (bouton "Ouvrir le fichier").
20. **Incremental scan** ✅ (skip si size + mtime inchangés).
21. **Box art support** ✅ (covers dans les cartes et la fiche).
22. **Offline mode** ✅ (cache des covers via asset protocol).

## Quality and development

23. **Tests** 🔄 : tests unitaires Rust (identify : 11 tests), Playwright config + smoke tests.
24. **CI** 🔄 : eslint config ajouté, Playwright config (réseau requis).
25. **Logging** ✅ (tracing-appender + tracing-subscriber).
26. **Error handling** ✅ (messages explicites, validation des chemins).
27. **Security** ✅ (validation des chemins, path traversal).
28. **i18n** 🔄 (fr/en complets, autres langues fallback anglais).

## Architecture

29. **Refactoring** ✅ (hashing.rs, dat.rs).
30. **Database** ✅ (migrations v1-v27, auto-backup, list/restore).
31. **Configuration** ✅ (Settings page).

## Nouvelles pistes (2026-11)

### Performance
- **Index DB** : ajouter des index sur les clés étrangères (migration).
- **FTS5** : peupler la table `games_fts` pour la recherche plein texte.
- **Concurrence contrôlée** : semaphore pour limiter les calculs d'empreintes simultanés.
- **Mémoïsation** : `useMemo`/`useCallback` dans les composants lourds.
- **Buffers réutilisables** : réutiliser les buffers au lieu d'en allouer de nouveaux.

### Fonctionnalités
- **Recommandations** basées sur les genres/plateformes possédés.
- **Détection de jeux manquants** dans une série.
- **Tracking des prix** (marché rétro).
- **Alertes de nouveaux dumps** (No-Intro, Redump).
- **Import depuis d'autres gestionnaires** : LaunchBox, EmuDeck, ES-DE, Playnite.
- **Export vers formats standards** : DAT (No-Intro), XML (HyperSpin).
- **Mode présentation (kiosque)** pour afficher la collection sur TV.
- **Comparaison de versions** (régions, révisions) côte à côte.
- **Raccourcis clavier** pour navigation, recherche, actions.

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
- **RetroAchievements** pour les succès et le tracking.
