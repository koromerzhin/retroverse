# Plan d'amélioration — Harmonisation des vues de RETROVERSE

Objectif : obtenir une expérience utilisateur cohérente, moderne et agréable dans toutes les vues de listing et de détail de RETROVERSE, en réutilisant les bonnes pratiques déjà mises en place dans la vue Bibliothèque.

## 1. Principes directeurs

- **Cohérence** : les éléments ayant la même fonction ont la même apparence et le même comportement.
- **Réutilisation** : mutualiser les composants et styles communs.
- **Simplicité** : éviter les abstractions inutiles.
- **Non-destructif** : ne pas supprimer de fonctionnalités existantes.
- **Performance** : lazy loading, virtualisation, chargement différé.

## 2. État actuel

Déjà harmonisés :
- Bibliothèque : recherche, filtres, tri, sélection multiple, actions groupées, badges, export.
- Plateformes : recherche, tri, modes cards/liste, scan all, calcul all, boutons harmonisés.
- Éditeurs, Développeurs, Genres, Classifications : recherche, tri, bouton sync, flexWrap.

Non harmonisés :
- Discovery : layout différent (sections), pas de recherche/filtres.
- Détails du scan : harmonisé avec hash (phase précédente).
- Détails du jeu : onglets (info, fichiers, empreintes, médias).

## 3. Axes d'amélioration

### 3.1 Composants partagés

Créer des composants réutilisables dans `src/components/` :

- `Toolbar` : barre d'actions standard (recherche, tri, filtres, boutons).
- `SearchInput` : champ de recherche avec debounce.
- `SortButton` : bouton de tri avec indicateur direction.
- `EmptyState` : message d'état vide avec icône.
- `LoadingSpinner` : spinner standard.
- `Badge` : badge avec variantes (success, warning, danger, muted).
- `Card` : conteneur standard avec titre, actions, contenu.
- `DataTable` : tableau avec en-tête sticky, colonnes configurables, virtualisation.

### 3.2 Harmonisation des vues de listing

Appliquer le pattern standard à toutes les vues :

```
┌─────────────────────────────────────────┐
│ Titre de la page                        │
│ Sous-titre                              │
├─────────────────────────────────────────┤
│ [Recherche] [Filtres] [Tri ▲▼] [Actions]│
├─────────────────────────────────────────┤
│ Contenu (cards / liste / tableau)       │
│ ...                                     │
├─────────────────────────────────────────┤
│ Compteur · Pagination                   │
└─────────────────────────────────────────┘
```

Chaque vue doit avoir :
- Titre + sous-titre.
- Barre de recherche (si applicable).
- Filtres et tri.
- Boutons d'action alignés.
- État de chargement (spinner).
- État vide (message + icône).
- Compteur de résultats.

### 3.3 Vues concernées

#### Plateformes
- ✅ Déjà harmonisé (recherche, tri, modes, scan all, calculate all).
- Amélioration : ajouter les covers de plateforme (si disponibles).

#### Discovery
- Reprendre le pattern standard avec recherche et filtres.
- Afficher les doublons et collections dans des cards harmonisées.
- Ajouter un filtre par statut (doublons, fichiers manquants).

#### Éditeurs / Développeurs
- ✅ Déjà harmonisé.
- Amélioration : ajouter le nombre de jeux par éditeur/développeur.

#### Genres
- ✅ Déjà harmonisé.
- Amélioration : afficher la hiérarchie (parent/enfant) de manière visuelle.

#### Classifications
- ✅ Déjà harmonisé.
- Amélioration : grouper par organisme.

### 3.4 Vue détaillée du jeu

Harmoniser avec les autres vues :
- Onglets : Informations, Fichiers, Empreintes, Médias.
- Actions : Sauvegarder, Rechercher ScreenScraper, Calculer empreintes, Ouvrir fichier.
- Badges de préservation.

### 3.5 États et feedback

- **Chargement** : spinner centré avec message.
- **Vide** : icône + message + action suggérée.
- **Erreur** : message d'erreur avec bouton "Réessayer".
- **Succès** : notification native (déjà en place).

### 3.6 Responsive

- Adapter les grilles à la taille de la fenêtre.
- Masquer les colonnes non essentielles sur petits écrans.
- Menu hamburger pour la sidebar sur petits écrans.

## 4. Plan d'implémentation

### Phase 1 — Composants partagés
1. Créer `src/components/Toolbar.tsx`, `SearchInput.tsx`, `EmptyState.tsx`, `Badge.tsx`.
2. Extraire les styles communs dans `App.css`.
3. Refactoriser la Bibliothèque pour utiliser les nouveaux composants.

### Phase 2 — Harmonisation des vues
4. Appliquer les composants partagés à Plateformes.
5. Appliquer à Discovery.
6. Appliquer à Éditeurs, Développeurs, Genres, Classifications.

### Phase 3 — Améliorations visuelles
7. Ajouter les covers de plateforme.
8. Afficher le nombre de jeux par éditeur/développeur.
9. Hiérarchie visuelle des genres.

### Phase 4 — Responsive
10. Adapter les grilles.
11. Menu hamburger pour la sidebar.

## 5. Contraintes

- Ne pas modifier la logique métier.
- Ne pas supprimer de fonctionnalités.
- Réutiliser les composants existants.
- Préserver les fonctionnalités spécifiques à chaque vue.

## 6. Résultat attendu

Une interface homogène, moderne et cohérente dans toutes les vues de RETROVERSE, avec des composants réutilisables et une expérience utilisateur uniforme.
EOF
echo done