# Plan d'amélioration — Vue Bibliothèque

Objectif : transformer la vue Bibliothèque en outil de navigation, filtrage et gestion avancée de la collection.

## 1. Principes directeurs

- **Performance** : ne pas charger plus que nécessaire ; virtualiser l'affichage.
- **Réactivité** : filtres instantanés, recherche avec debounce.
- **Cohérence** : réutiliser les composants et styles existants.
- **Non-destructif** : aucune action irréversible sans confirmation.

## 2. État actuel

Déjà présents :
- Recherche textuelle (FTS via LIKE) avec debounce 300 ms.
- Filtres par plateforme, développeur, éditeur.
- Filtre "empreintes manquantes uniquement".
- Tri par titre, plateforme, développeur, éditeur, fichiers.
- Vues cartes / liste.
- Pagination (24 par page) + pagination virtuelle (vue liste).
- Export JSON.
- Badges de préservation.

Manquants :
- Recherche fuzzy / autocomplétion.
- Filtres combinés avancés (genre, année, statut de hash, métadonnées).
- Vues filtrées prédéfinies (sans métadonnées, sans empreintes, favoris).
- Sélection multiple + actions groupées.
- Groupement par plateforme/genre/année.
- Menu contextuel.
- Colonnes configurables.

## 3. Axes d'amélioration

### 3.1 Recherche et filtrage

- **Recherche fuzzy** : tolérance aux fautes, correspondance partielle, scoring.
- **Autocomplétion** : suggestions en temps réel basées sur les titres existants.
- **Filtres combinés** : panneau de filtres avec cases à cocher (plateforme, genre, développeur, éditeur, année, statut hash, présence métadonnées).
- **Filtres rapides** : boutons "Sans métadonnées", "Sans empreintes", "Favoris", "Récemment ajoutés".
- **Sauvegarde des filtres** : mémoriser dans l'URL (query params) et localStorage.
- **Recherche FTS5** : utiliser la table `games_fts` pour la recherche plein texte.

### 3.2 Affichage

- **Vue compacte** : tableau dense avec colonnes configurables (afficher/masquer).
- **Cartes enrichies** : cover, genre, année, badges de préservation.
- **Mode grille adaptative** : taille des cartes ajustable (slider).
- **Prévisualisation au survol** : agrandissement de la cover.
- **Groupement** : par plateforme, par genre, par développeur, par année, par collection.
- **Pagination virtuelle** ✅ (déjà fait pour la vue liste).

### 3.3 Organisation

- **Favoris / liste de souhaits** : icône étoile, filtre dédié, stockage dans une table `game_favorites`.
- **Tags personnalisés** : créer/assigner des tags, filtrer par tag.
- **Vue "Récemment ajoutés"** : tri par `created_at` desc, badge "Nouveau".
- **Vue "Sans métadonnées"** : jeux sans `metadata_json`.
- **Vue "Sans empreintes"** : jeux avec au moins un fichier sans hash.

### 3.4 Actions

- **Sélection multiple** : cases à cocher sur les cartes/lignes, sélection par plage (Shift), tout sélectionner.
- **Actions groupées** : hasher, enrichir, ajouter à une collection, supprimer.
- **Menu contextuel** : clic droit → ouvrir, hasher, enrichir, supprimer, ajouter à collection.
- **Drag & drop** : ajouter à une collection par glisser-déposer.
- **Actions rapides** : lancer le jeu, ouvrir le dossier.

### 3.5 Statistiques

- **Compteurs en temps réel** : nombre de jeux, plateformes, fichiers, hashes.
- **Graphiques** : répartition par plateforme, par genre, par décennie.
- **Progression de la collection** : % de jeux avec métadonnées, % avec hashes.

## 4. Plan d'implémentation

### Phase 1 — Filtrage avancé
1. Panneau de filtres combinés (plateforme, genre, développeur, éditeur, année).
2. Filtres rapides (sans métadonnées, sans empreintes, favoris, récemment ajoutés).
3. Sauvegarde des filtres dans l'URL.

### Phase 2 — Organisation
4. Table `game_favorites` + icône étoile.
5. Tags personnalisés + filtrage.
6. Vues filtrées prédéfinies.

### Phase 3 — Actions
7. Sélection multiple + actions groupées.
8. Menu contextuel.
9. Drag & drop vers collections.

### Phase 4 — Affichage
10. Vue compacte avec colonnes configurables.
11. Groupement par plateforme/genre/année.
12. Prévisualisation au survol.

### Phase 5 — Statistiques
13. Graphiques (réutiliser une lib légère ou SVG).
14. Progression de la collection.

## 5. Schéma DB proposé

```sql
CREATE TABLE game_favorites (
    game_id TEXT PRIMARY KEY REFERENCES games(id) ON DELETE CASCADE,
    created_at TEXT NOT NULL
);

CREATE TABLE game_tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE game_tag_links (
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES game_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, tag_id)
);
```

## 6. Contraintes

- Ne pas modifier la logique métier existante.
- Réutiliser les composants et styles existants.
- Préserver les fonctionnalités déjà opérationnelles.
- Respecter l'architecture Tauri/Rust.

## 7. Résultat attendu

La Bibliothèque devient un outil de gestion avancée : filtrage multi-critères, organisation par favoris/tags, actions groupées, et visualisation statistique de la collection.
EOF
echo done