# Plan d'amélioration — Préservation

Objectif : faire de RETROVERSE un outil de préservation fiable, non destructif et vérifiable, pour les collections de jeux vidéo.

## 1. Principes directeurs

- **Non-destructif par défaut** : aucune modification des fichiers sources sans confirmation explicite.
- **Traçabilité** : chaque action (scan, hash, enrichissement, correction) est journalisée avec provenance.
- **Vérifiabilité** : les empreintes et métadonnées permettent de prouver l'intégrité et l'authenticité.
- **Reproductibilité** : un export de préservation doit être restaurable à l'identique.
- **Compatibilité** : respect des formats standards (No-Intro, Redump, DAT, XML HyperSpin).

## 2. État actuel

Déjà présents :
- Calcul d'empreintes (SHA-256, CRC32, MD5, SHA-1, SHA-512) pour fichiers et archives.
- Stockage des hashes dans `game_files` + empreintes d'archive.
- Migrations SQLite avec auto-backup.
- Page "Préservation" (placeholder).
- Détection de doublons (par titre + plateforme).
- Collections et métadonnées (ScreenScraper).

Manquants :
- Vérification d'intégrité contre des DAT (No-Intro/Redump).
- Détection de bad dumps / hacks / over-dumps.
- Export de packs de préservation.
- Archivage des métadonnées avec les ROMs.
- Historique et audit trail.
- Restauration depuis un pack.

## 3. Axes d'amélioration

### 3.1 Intégrité et vérification

- **Import de DAT** : parser les fichiers DAT (No-Intro, Redump, TOSEC) et stocker les empreintes de référence.
- **Vérification d'un jeu** : comparer les empreintes calculées avec la référence DAT ; statut : OK, bad dump, missing, unknown.
- **Vérification globale** : lancer sur toute la bibliothèque, afficher un rapport (OK / problèmes / inconnus).
- **Détection de bad dumps** : flags (overdump, underdump, bad, verified) selon la référence.
- **Détection de hacks/traductions** : comparaison avec les DAT "hacks" et identification des variantes.
- **Alerte de corruption** : si un fichier change (mtime/hash) sans action utilisateur, signaler.

### 3.2 Archivage et export

- **Pack de préservation** : archive (zip/7z) contenant pour chaque jeu :
  - Les ROMs (ou références aux fichiers sources),
  - Le fichier de métadonnées (JSON) avec titre, plateforme, hashes, métadonnées ScreenScraper,
  - Un manifest (liste des fichiers + hashes du pack).
- **Export DAT personnalisé** : générer un DAT à partir de la collection locale.
- **Export CSV/JSON** : déjà amorcé (export JSON).
- **Backup complet** : base SQLite + cache d'images + config, dans une archive versionnée.
- **Restauration** : importer un pack et reconstruire la base.

### 3.3 Audit et historique

- **Table `audit_log`** : enregistrer chaque action (scan, hash, enrich, correction, suppression) avec timestamp, cible, auteur, détails.
- **Vue "Historique"** : afficher les actions récentes, filtrables par type/date.
- **Provenance des métadonnées** : déjà présent (`provenance` dans `metadata_records`) ; étendre à tous les champs.
- **Diff de métadonnées** : comparer deux versions d'une fiche.

### 3.4 Organisation et classification

- **Statut de préservation** par jeu : `verified`, `unverified`, `bad_dump`, `missing`, `unknown`.
- **Badges visuels** dans la bibliothèque et la fiche jeu.
- **Filtres** : par statut de préservation, par présence de hash, par DAT.
- **Tableau de bord** : couverture (% de jeux vérifiés par plateforme).

### 3.5 Compatibilité et standards

- **Parser DAT** (No-Intro, Redump, TOSEC) : formats XML et ClrMamePro.
- **Export DAT** au format ClrMamePro.
- **Support des CHD** (MAME) : calcul d'empreintes sur les CHD.
- **Support des archives** : déjà amorcé (zip/7z) ; ajouter 7z solid, rar.
- **Hashes supplémentaires** : SHA-256 (déjà), ajouter les hashes attendus par les DAT.

## 4. Plan d'implémentation

### Phase 1 — Fondations (v1.x)
1. Créer la table `audit_log` (migration).
2. Ajouter le champ `preservation_status` à `games` (ou table dédiée).
3. Parser DAT No-Intro/Redump (module `dat.rs`).
4. Stocker les DAT importés (`dat_files`, `dat_entries`).
5. Vérification d'un jeu contre un DAT → statut.

### Phase 2 — Vérification globale
6. Commande `verify_library` → rapport par plateforme.
7. Page "Préservation" avec tableau de bord et liste des problèmes.
8. Badges dans la bibliothèque.

### Phase 3 — Archivage
9. Export de pack de préservation (zip avec ROMs + metadata + manifest).
10. Import/restauration d'un pack.
11. Backup complet de la base + cache.

### Phase 4 — Avancé
12. Détection de bad dumps / hacks.
13. Export DAT personnalisé.
14. Historique et audit trail UI.

## 5. Schéma DB proposé

```sql
CREATE TABLE audit_log (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    action TEXT NOT NULL,
    target_type TEXT,
    target_id TEXT,
    details TEXT,
    provenance TEXT
);

CREATE TABLE dat_files (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    system TEXT,
    version TEXT,
    imported_at TEXT NOT NULL
);

CREATE TABLE dat_entries (
    id TEXT PRIMARY KEY,
    dat_id TEXT NOT NULL REFERENCES dat_files(id),
    name TEXT NOT NULL,
    size INTEGER,
    crc32 TEXT,
    md5 TEXT,
    sha1 TEXT,
    sha256 TEXT
);

CREATE TABLE game_preservation (
    game_id TEXT PRIMARY KEY REFERENCES games(id),
    status TEXT NOT NULL DEFAULT 'unknown',
    dat_entry_id TEXT REFERENCES dat_entries(id),
    verified_at TEXT,
    notes TEXT
);
```

## 6. Contraintes

- Ne pas modifier les fichiers sources.
- Ne pas augmenter la consommation mémoire (streaming pour les DAT volumineux).
- Respecter l'architecture existante (commandes Tauri, migrations versionnées).
- Compatibilité avec les bases existantes (migrations additives).

## 7. Résultat attendu

RETROVERSE devient un outil de préservation fiable : vérification d'intégrité, badges de statut, export de packs, audit trail, et compatibilité avec les standards de la communauté (No-Intro, Redump, TOSEC).
EOF
echo done