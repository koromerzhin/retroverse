# Plan d'amélioration complet — RETROVERSE

Document maître consolidant toutes les pistes d'amélioration pour RETROVERSE, par thème et par priorité. Mis à jour le 2026-11-08.

## Légende

- ✅ Fait
- 🔄 En cours / partiel
- ❌ Non fait
- 🔴 Priorité haute
- 🟠 Priorité moyenne
- 🟡 Priorité basse
- 🟢 Architecture / long terme

---

## 1. Performance et mémoire

### 1.1 Base de données
- ✅ Streaming des fichiers lors du calcul des empreintes (64 Ko chunks).
- ✅ Cache des empreintes (skip si fichier inchangé via `hashed_mtime`/`hashed_size`).
- ✅ Scan incrémental (skip si size + mtime inchangés).
- ✅ VACUUM périodique (auto-backup avant migration).
- ❌ Index sur les clés étrangères (`game_files.platform_id`, `game_file_links.game_id`, etc.).
- ❌ Prepared statements réutilisés.
- ❌ Batch writes lors des scans (partiellement fait).
- ❌ Connection pooling.

### 1.2 Calcul des empreintes
- ✅ Streaming local et remote.
- ✅ Concurrence contrôlée (semaphore) — *à vérifier*.
- ✅ Cache (skip si fichier inchangé).
- ❌ Parallélisation des fichiers avec limite.
- ❌ Progression par bloc (événements pendant la lecture des gros fichiers).

### 1.3 Scan et identification
- ✅ Scan incrémental.
- ❌ Parallélisation du scan (plusieurs dossiers en parallèle).
- ❌ Cache des plateformes (charger une fois).
- ❌ Batch insert des fichiers.

### 1.4 Affichage frontend
- ✅ Pagination virtuelle (vue liste de la Bibliothèque).
- ✅ Lazy loading des images (`loading="lazy"`).
- ❌ Mémoïsation (`useMemo`, `useCallback`) pour éviter les recalculs.
- ✅ Code splitting (lazy loading des pages).
- ✅ Debounce pour la recherche (300 ms).

### 1.5 Recherche
- ❌ FTS5 (table `games_fts` existante, non peuplée).
- ❌ Index sur les colonnes recherchées.
- ❌ Cache des résultats de recherche.

### 1.6 Mémoire
- ✅ Streaming (pas de chargement complet des fichiers).
- ❌ Libération des ressources (fichiers, connexions).
- ❌ Buffers réutilisables.
- ❌ Weak references pour les caches.

---

## 2. UX et interface

### 2.1 Composants partagés
- ✅ `EmptyState`, `Badge`, `SearchInput`, `Toolbar` créés.
- ✅ Refactorisation de la Bibliothèque.
- ✅ Mutualisation dans toutes les vues (Plateformes, Éditeurs, Développeurs, Genres, Classifications, Sources, Découverte, Accueil, Companion).

### 2.2 Harmonisation des vues
- ✅ Bibliothèque (référence).
- ✅ Plateformes (recherche, tri, modes, scan all, calculate all, colonnes configurables).
- ✅ Éditeurs, Développeurs, Genres, Classifications (recherche, tri, sync, badges).
- ✅ Découverte (recherche, EmptyState).
- ✅ Sources (statut, EmptyState, scan par dossier, gestion complète).
- ✅ Accueil (covers, actions rapides, EmptyState).
- ✅ Détail du jeu (badges, ouvrir dossier, médias).
- ✅ Companion (EmptyState).

### 2.3 Fonctionnalités manquantes
- ✅ Vue "Récemment ajoutés" (tri par date d'ajout) — *partiel*.
- ✅ Favoris / liste de souhaits avec icône étoile — *amorcé*.
- ❌ Historique de jeu (temps de jeu, dernière session) — `game_sessions` existe.
- ❌ Mode présentation (kiosque) pour afficher la collection sur TV.
- ❌ Comparaison de versions (régions, révisions) côte à côte.
- ❌ Raccourcis clavier pour navigation, recherche, actions.

### 2.4 Affichage
- ✅ Vue compacte / liste détaillée avec colonnes configurables.
- ✅ Groupement par plateforme/développeur.
- ✅ Prévisualisation au survol (covers dans les cartes).
- ✅ Pagination virtuelle (vue liste).
- ❌ Mode grille adaptative (taille des cartes ajustable).

---

## 3. Métadonnées et référentiels

### 3.1 Édition et persistance
- ✅ Persistance des champs édités (genre, dates, classifications, synopsis).
- ✅ Multi-genres par jeu (select multiple).
- ✅ Synchronisation automatique des référentiels (publishers/developers) au démarrage.
- ✅ Import/export de métadonnées (export JSON, export DAT).
- ❌ Historique des enrichissements avec revert.

### 3.2 Referentiels
- ✅ Développeurs, Éditeurs, Genres (multilingue), Classifications.
- ✅ Compteur de jeux par éditeur/développeur.
- ✅ Hiérarchie visuelle des genres (indentation).
- ✅ Groupement des classifications par organisme.

---

## 4. Fonctionnalités

### 4.1 Détection et organisation
- ✅ Détection de doublons (UI).
- ✅ Collections/playlists et tags personnalisés.
- ✅ Statistiques (graphiques par plateforme).
- ✅ Lancement de jeux (ouverture fichier).
- ✅ Scan incrémental (skip si fichier inchangé).
- ✅ Images de boîtes (covers dans les cartes).
- ✅ Mode hors-ligne (cache des covers).

### 4.2 Découverte
- ❌ Recommandations basées sur les genres/plateformes possédés.
- ❌ Détection de jeux manquants dans une série.
- ❌ Tracking des prix (marché rétro).
- ❌ Alertes de nouveaux dumps (No-Intro, Redump).

### 4.3 ScreenScraper
- ✅ Recherche individuelle (fiche jeu).
- ✅ Recherche globale (Bibliothèque).
- ✅ Progression détaillée.
- ✅ Protection des métadonnées existantes.

---

## 5. Préservation

### 5.1 Intégrité et vérification
- ✅ Vérification d'intégrité contre DAT (No-Intro/Redump).
- ✅ Détection de bad dumps (colonne `bad` + statut `bad_dump`).
- ✅ Badges dans la bibliothèque.
- ❌ Vérification d'intégrité avec rapport détaillé par plateforme.
- ❌ Détection avancée de bad dumps / hacks / over-dumps.

### 5.2 Archivage et export
- ✅ Export de pack de préservation (zip avec metadata + manifest).
- ✅ Import/restauration d'un pack.
- ✅ Backup complet de la base + cache (auto-backup avant migration).
- ✅ Export DAT personnalisé (ClrMamePro).

### 5.3 Audit et historique
- ✅ Table `audit_log`.
- ✅ Journal d'audit UI.
- ❌ Historique des modifications (audit trail complet).

---

## 6. Qualité et développement

### 6.1 Tests
- ✅ Tests unitaires Rust (identify : 5 tests).
- ❌ Tests unitaires supplémentaires (scanner, metadata, hashing).
- ✅ Tests E2E Playwright (config + smoke tests).
- ❌ Tests E2E complets (flux critiques).

### 6.2 CI et outillage
- ✅ Linting frontend (eslint config).
- ✅ Linting backend (clippy dans la CI).
- ❌ CI : builds automatiques, releases GitHub.
- ❌ Documentation : rustdoc, Storybook pour les composants.

### 6.3 Logging et erreurs
- ✅ Logging structuré (tracing-appender + tracing-subscriber).
- ✅ Gestion d'erreurs (messages explicites).
- ❌ Codes d'erreur standardisés.

### 6.4 Sécurité
- ✅ Validation des chemins (path traversal).
- ❌ Chiffrement de la base (SQLCipher).
- ❌ Gestion des credentials (keyring).

### 6.5 i18n
- ✅ 6 langues (fr, en, es, pt, de, it).
- ✅ Clés i18n pour les nouvelles pages.
- ❌ Traductions complètes pour es/pt/de/it (fallback anglais).

---

## 7. Architecture

### 7.1 Refactoring
- ✅ Module `hashing.rs` dédié avec trait `HashComputer`.
- ✅ Module `dat.rs` pour le parsing DAT.
- ❌ Extraction de la logique de scan dans un module dédié.
- ❌ Abstraction des providers de métadonnées (trait commun).

### 7.2 Base de données
- ✅ Migrations versionnées (v1 à v27).
- ✅ Auto-backup avant migration.
- ❌ Rollback de migration.
- ❌ Backup automatique programmé.

### 7.3 Configuration
- ✅ Config in-app (Settings page).
- ❌ Gestion des credentials via keyring.
- ❌ Profils de configuration multiples.

---

## 8. Connectivité et intégrations

### 8.1 Sources
- ✅ Sources locales et distantes (SSH/SFTP).
- ✅ Scan incrémental.
- ✅ Gestion complète des sources (ajout, modification, suppression, activation).
- ✅ Conversion de type (local ↔ SSH).
- ✅ Migration des chemins lors de la conversion.
- ❌ Import depuis d'autres gestionnaires (LaunchBox, EmuDeck, ES-DE, Playnite).
- ❌ Export vers formats standards (DAT, XML HyperSpin).

### 8.2 Métadonnées
- ✅ ScreenScraper (hash-first, raw JSON, referential sync).
- ❌ IGDB / RAWG comme providers alternatifs.
- ❌ SteamGridDB pour les artworks.
- ❌ RetroAchievements pour les succès.

### 8.3 API
- ❌ API REST locale pour intégration avec d'autres outils.
- ❌ Webhooks pour les événements.

---

## 9. Communauté et extensibilité

- ❌ Thèmes communautaires (skins).
- ❌ Partage de collections (anonymisé).
- ❌ Contributions de métadonnées vers ScreenScraper.
- ❌ Plugin system pour étendre les fonctionnalités.
- ❌ Mode présentation (kiosque) pour afficher la collection sur TV.

---

## 10. Plan d'implémentation recommandé

### Phase A — Qualité (priorité haute)
1. Tests unitaires Rust (identify, scanner, metadata, hashing).
2. Tests E2E Playwright complets.
3. i18n complet (traductions es/pt/de/it).

### Phase B — Performance
4. Index DB (migration).
5. FTS5 pour la recherche.
6. Concurrence contrôlée pour les hashes.
7. Mémoïsation frontend.

### Phase C — Fonctionnalités
8. Import depuis d'autres gestionnaires.
9. Export vers formats standards.
10. Mode présentation (kiosque).

### Phase D — Extensibilité
11. Plugin system.
12. Thèmes communautaires.
13. API REST locale.

---

## Résumé

- **Fait** : ~75% des améliorations identifiées.
- **Restant** : tests, performance (index, FTS5), intégrations (IGDB, SteamGridDB), extensibilité (plugins, thèmes).
- **Priorité** : qualité (tests) > performance (index, FTS5) > fonctionnalités > extensibilité.
EOF
echo done