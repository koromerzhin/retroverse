# Plan d'amélioration — Performance

Objectif : améliorer la réactivité, la consommation mémoire et la scalabilité de RETROVERSE pour les grandes collections (plusieurs milliers de jeux).

## 1. Principes directeurs

- **Mesurer avant d'optimiser** : profiler avant de modifier.
- **Ne pas sacrifier la qualité** pour la performance.
- **Optimiser les chemins critiques** : scan, hash, recherche, affichage.
- **Éviter les allocations inutiles** et les copies mémoire.
- **Conserver les résultats** pour éviter les recalculs.

## 2. État actuel

Déjà implémentés :
- Streaming des fichiers lors du calcul des empreintes (64 Ko chunks).
- Cache des empreintes (skip si fichier inchangé via `hashed_mtime`/`hashed_size`).
- Pagination virtuelle (vue liste de la Bibliothèque).
- Scan incrémental (skip si size + mtime inchangés).
- Chargement différé des images (`loading="lazy"`).
- `color-scheme` et CSS variables.

Points faibles identifiés :
- `game_files` et `game_file_links` chargés en entier pour chaque jeu lors du scan.
- Requêtes SQL non indexées sur les clés étrangères.
- Pas de cache mémoire pour les plateformes et les métadonnées.
- Pas d'indexation FTS pour la recherche plein texte.
- Scan séquentiel (un dossier à la fois).

## 3. Axes d'amélioration

### 3.1 Base de données

- **Index** : ajouter des index sur `game_files.platform_id`, `game_file_links.game_id`, `game_file_links.file_id`, `metadata_records.game_id`, `game_preservation.game_id`.
- **VACUUM** : exécuter périodiquement pour compacter la base.
- **Batch writes** : utiliser des transactions par lot lors des scans (déjà partiellement fait).
- **Prepared statements** : réutiliser les statements compilés.
- **Connection pooling** : si nécessaire, utiliser un pool de connexions.

### 3.2 Calcul des empreintes

- **Streaming** ✅ (déjà fait pour local et remote).
- **Concurrence contrôlée** : limiter le nombre de calculs simultanés (semaphore) pour éviter la saturation CPU/RAM.
- **Cache** ✅ (skip si fichier inchangé).
- **Parallélisation** : traiter plusieurs fichiers en parallèle (avec limite).
- **Progression par bloc** : émettre des événements de progression pendant la lecture des gros fichiers.

### 3.3 Scan et identification

- **Scan incrémental** ✅ (skip si size + mtime inchangés).
- **Parallélisation du scan** : scanner plusieurs dossiers en parallèle.
- **Cache des plateformes** : charger une fois et réutiliser.
- **Batch insert** : insérer les fichiers par lot.

### 3.4 Affichage frontend

- **Pagination virtuelle** ✅ (vue liste).
- **Lazy loading des images** ✅ (covers).
- **Mémoïsation** : utiliser `useMemo` et `useCallback` pour éviter les recalculs inutiles.
- **Code splitting** : charger les pages à la demande (déjà fait avec `lazy`).
- **Debounce** : déjà implémenté pour la recherche (300 ms).

### 3.5 Recherche

- **FTS5** : utiliser la table `games_fts` pour la recherche plein texte.
- **Index** : ajouter des index sur les colonnes fréquemment recherchées.
- **Cache des résultats** : mémoriser les résultats de recherche récents.

### 3.6 Mémoire

- **Streaming** ✅ (pas de chargement complet des fichiers).
- **Libération des ressources** : fermer les fichiers et les connexions dès qu'ils ne sont plus nécessaires.
- **Buffers réutilisables** : réutiliser les buffers au lieu d'en allouer de nouveaux.
- **Weak references** : utiliser des références faibles pour les caches.

## 4. Plan d'implémentation

### Phase 1 — Base de données
1. Ajouter les index manquants (migration).
2. Implémenter le VACUUM périodique.
3. Optimiser les requêtes (prepared statements, batch writes).

### Phase 2 — Calcul des empreintes
4. Ajouter un semaphore pour limiter la concurrence.
5. Paralléliser le traitement des fichiers (avec limite).
6. Émettre des événements de progression par bloc.

### Phase 3 — Scan et identification
7. Paralléliser le scan des dossiers.
8. Mettre en cache les plateformes.
9. Batch insert des fichiers.

### Phase 4 — Affichage frontend
10. Mémoïser les composants coûteux (`useMemo`, `useCallback`).
11. Lazy loading des images (déjà fait).
12. Virtualisation de la vue cartes (si nécessaire).

### Phase 5 — Recherche
13. Implémenter la recherche FTS5.
14. Ajouter des index sur les colonnes recherchées.
15. Mettre en cache les résultats de recherche.

## 5. Schéma DB proposé (index)

```sql
CREATE INDEX IF NOT EXISTS idx_game_files_platform ON game_files(platform_id);
CREATE INDEX IF NOT EXISTS idx_game_files_hash ON game_files(hash);
CREATE INDEX IF NOT EXISTS idx_game_file_links_game ON game_file_links(game_id);
CREATE INDEX IF NOT EXISTS idx_game_file_links_file ON game_file_links(file_id);
CREATE INDEX IF NOT EXISTS idx_metadata_records_game ON metadata_records(game_id);
CREATE INDEX IF NOT EXISTS idx_game_preservation_status ON game_preservation(status);
```

## 6. Contraintes

- Ne pas modifier la logique métier.
- Ne pas augmenter la consommation mémoire.
- Respecter l'architecture existante.
- Mesurer avant et après chaque optimisation.

## 7. Résultat attendu

RETROVERSE doit pouvoir gérer des collections de plusieurs milliers de jeux sans ralentissement notable, avec une consommation mémoire maîtrisée, et une interface réactive même pendant les opérations de scan et de calcul d'empreintes.
EOF
echo done