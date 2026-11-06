# Plan d'amélioration — Vues de RETROVERSE

Objectif : améliorer l'expérience utilisateur, la cohérence visuelle et les fonctionnalités des vues de RETROVERSE.

## 1. État actuel

Vues existantes :
- **Accueil** (Home) — dashboard avec statistiques.
- **Bibliothèque** — recherche, filtres, tri, sélection multiple, actions groupées, badges, export.
- **Plateformes** — cards/liste, scan all, calculate all, hash par plateforme.
- **Sources** — gestion des sources locales/distants.
- **Companion** — companion app.
- **Découverte** — doublons, collections, jeux à redécouvrir.
- **Développeurs** — liste avec recherche/tri, compteur de jeux.
- **Éditeurs** — liste avec recherche/tri, compteur de jeux.
- **Genres** — liste multilingue avec recherche/tri/sync.
- **Classifications** — liste avec recherche/tri/sync.
- **Collections** — création, sélection, affichage des jeux.
- **Statistiques** — totaux, graphiques par plateforme.
- **Doublons** — groupes de doublons.
- **Préservation** — intégrité, sauvegarde/restauration, DAT, audit.
- **Paramètres** — thème, langue, maintenance, providers.

Composants partagés : `EmptyState`, `Badge`, `SearchInput`, `Toolbar`.

## 2. Axes d'amélioration

### 2.1 Cohérence visuelle

- **Unifier les barres d'actions** : toutes les vues doivent utiliser le même pattern (recherche, filtres, tri, actions).
- **Composant `Toolbar`** : réutiliser dans toutes les vues (actuellement inline).
- **États vides** : utiliser `EmptyState` partout (actuellement inline dans certaines vues).
- **Badges** : utiliser `Badge` pour les statuts (préservation, hash, etc.).
- **Espacements** : standardiser les marges/paddings (actuellement variables).

### 2.2 Fonctionnalités manquantes

#### Découverte
- Ajouter la recherche et les filtres (actuellement pas de recherche).
- Ajouter un filtre par statut (doublons, fichiers manquants).
- Afficher les jeux découverts dans un tableau harmonisé.

#### Statistiques
- Ajouter des graphiques (camembert par genre, courbes par année).
- Ajouter des métriques (temps de jeu, derniers ajouts).
- Exporter les statistiques (PNG/CSV).

#### Préservation
- Ajouter les badges de préservation dans la Bibliothèque ✅ (fait).
- Afficher le statut de vérification par jeu dans la fiche jeu.
- Ajouter un tableau de bord global avec progression.

#### Paramètres
- Ajouter une gestion des thèmes personnalisés.
- Ajouter l'import/export de la configuration.
- Ajouter la gestion des providers (déjà présent pour ScreenScraper).

### 2.3 Navigation et ergonomie

- **Menu contextuel** : clic droit sur un jeu → actions rapides.
- **Raccourcis clavier** : navigation, recherche, actions.
- **Favoris** : icône étoile dans les cartes (déjà fait).
- **Vue compacte** : tableau dense avec colonnes configurables (déjà fait).
- **Groupement** : par plateforme, genre, année (déjà fait pour plateforme/développeur).

### 2.4 Performance

- **Lazy loading des images** : déjà fait pour les covers.
- **Virtualisation** : déjà fait pour la vue liste.
- **Cache des données** : utiliser React Query avec staleTime approprié.

### 2.5 Accessibilité

- **Navigation au clavier** : focus visible, tab order.
- **ARIA labels** : pour les boutons et les contrôles.
- **Contraste** : vérifier les ratios de contraste (WCAG AA).
- **Screen reader** : annonces pour les changements d'état.

## 3. Plan d'implémentation

### Phase 1 — Composants partagés (amorcé)
1. Créer `Toolbar`, `SearchInput`, `EmptyState`, `Badge` ✅.
2. Extraire les styles communs dans `App.css`.
3. Refactoriser la Bibliothèque ✅ (partiel).

### Phase 2 — Harmonisation des vues (amorcé)
4. Appliquer les composants partagés à Plateformes, Discovery, Éditeurs, Développeurs, Genres, Classifications ✅ (partiel).

### Phase 3 — Améliorations visuelles
5. Ajouter les covers de plateforme (si disponibles).
6. Afficher le nombre de jeux par éditeur/développeur ✅ (fait).
7. Hiérarchie visuelle des genres.
8. Grouper les classifications par organisme.

### Phase 4 — Responsive
9. Adapter les grilles ✅ (fait).
10. Menu hamburger pour la sidebar ✅ (fait).

### Phase 5 — Accessibilité
11. Navigation au clavier.
12. ARIA labels.
13. Vérification du contraste.

## 4. Contraintes

- Ne pas modifier la logique métier.
- Ne pas supprimer de fonctionnalités.
- Réutiliser les composants existants.
- Préserver les fonctionnalités spécifiques à chaque vue.

## 5. Résultat attendu

Une interface homogène, moderne, accessible et cohérente dans toutes les vues de RETROVERSE, avec des composants réutilisables et une expérience utilisateur uniforme.
EOF
echo done