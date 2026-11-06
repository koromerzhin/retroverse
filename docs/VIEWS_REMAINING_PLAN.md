# Plan d'amélioration — Vues restantes de RETROVERSE

Objectif : améliorer les vues qui n'ont pas encore bénéficié de l'harmonisation et des améliorations apportées à la Bibliothèque.

## 1. État actuel

Vues déjà harmonisées :
- Bibliothèque ✅ (recherche, filtres, tri, sélection multiple, actions groupées, badges, export, pagination virtuelle)
- Plateformes ✅ (recherche, tri, modes cards/liste, scan all, calculate all)
- Éditeurs ✅ (recherche, tri, compteur de jeux, sync)
- Développeurs ✅ (recherche, tri, compteur de jeux, sync)
- Genres ✅ (recherche, tri, sync, hiérarchie visuelle)
- Classifications ✅ (recherche, tri, sync, groupement par organisme)
- Collections ✅ (création, sélection, affichage)
- Statistiques ✅ (totaux, graphiques par plateforme)
- Doublons ✅ (groupes de doublons)
- Préservation ✅ (intégrité, sauvegarde/restauration, DAT, audit, badges)
- Paramètres ✅ (thème, langue, maintenance, providers)

Vues restantes à améliorer :
- **Accueil** (Home)
- **Sources**
- **Découverte** (Discovery)
- **Companion**
- **Détail du jeu** (GameDetail) — partiellement harmonisé

## 2. Axes d'amélioration

### 2.1 Accueil (Home)

État actuel : dashboard avec statistiques basiques.

Améliorations :
- **Widgets configurables** : l'utilisateur choisit les widgets affichés.
- **Activité récente** : derniers scans, enrichissements, calculs d'empreintes.
- **Raccourcis rapides** : boutons vers les actions fréquentes.
- **Graphiques** : mini-graphiques (jeux par plateforme, progression de la collection).
- **Notifications** : affichage des alertes (fichiers manquants, doublons détectés).

### 2.2 Sources

État actuel : gestion des sources locales/distants.

Améliorations :
- **Statut de disponibilité** : indicateur vert/rouge pour chaque source.
- **Dernier scan** : date et résultat du dernier scan.
- **Actions rapides** : scanner, tester la connexion, ouvrir le dossier.
- **Statistiques par source** : nombre de fichiers, de jeux découverts.
- **Drag & drop** pour ajouter des dossiers.

### 2.3 Découverte (Discovery)

État actuel : doublons, collections, jeux à redécouvrir.

Améliorations :
- **Recherche et filtres** : ajouter la recherche textuelle et les filtres.
- **Affichage harmonisé** : utiliser les composants partagés (SearchInput, EmptyState, Badge).
- **Filtre par statut** : doublons, fichiers manquants, sans métadonnées.
- **Actions rapides** : enrichir, hasher, ajouter à une collection.
- **Suggestions** : jeux similaires, jeux de la même série.

### 2.4 Companion

État actuel : companion app.

Améliorations :
- **Harmoniser le style** avec les autres vues.
- **Afficher les informations** de manière cohérente.
- **Actions** : lancer, ouvrir, etc.

### 2.5 Détail du jeu (GameDetail)

État actuel : onglets (info, fichiers, empreintes, médias).

Améliorations :
- **Badge de préservation** : afficher le statut dans la fiche.
- **Onglet "Fichiers liés"** : afficher les fichiers liés avec leurs empreintes.
- **Actions rapides** : lancer le jeu, ouvrir le dossier, copier le chemin.
- **Historique** : afficher les modifications récentes.
- **Médias** : afficher toutes les images (covers, screenshots, etc.).

## 3. Plan d'implémentation

### Phase 1 — Découverte
1. Ajouter la recherche et les filtres.
2. Harmoniser l'affichage avec les composants partagés.
3. Ajouter les actions rapides.

### Phase 2 — Sources
4. Ajouter le statut de disponibilité.
5. Afficher le dernier scan et les statistiques.
6. Ajouter le drag & drop.

### Phase 3 — Accueil
7. Ajouter les widgets configurables.
8. Afficher l'activité récente.
9. Ajouter les raccourcis rapides.

### Phase 4 — Détail du jeu
10. Ajouter le badge de préservation.
11. Ajouter l'onglet "Fichiers liés".
12. Ajouter les actions rapides.

### Phase 5 — Companion
13. Harmoniser le style.
14. Ajouter les actions.

## 4. Contraintes

- Ne pas modifier la logique métier.
- Ne pas supprimer de fonctionnalités.
- Réutiliser les composants existants.
- Préserver les fonctionnalités spécifiques à chaque vue.

## 5. Résultat attendu

Toutes les vues de RETROVERSE bénéficient d'une interface homogène, moderne et cohérente, avec des composants réutilisables et une expérience utilisateur uniforme.
EOF
echo done