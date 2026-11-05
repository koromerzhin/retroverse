# Améliorations possibles pour RETROVERSE

Liste de pistes d'évolution pour l'application, classées par thème. À prioriser selon les besoins.

## Performance et mémoire

1. **Streaming des ISO volumineux** (amorcé) : étendre le streaming à tous les chemins (remote, archives) et ajouter un buffer configurable.
2. **Concurrence contrôlée** : file d'attente (semaphore) pour limiter le nombre de calculs d'empreintes simultanés sur les gros fichiers.
3. **Cache des résultats** : ne pas recalculer les empreintes si le fichier n'a pas changé (mtime/size).
4. **Pagination virtuelle** dans la Bibliothèque pour très grandes collections.

## UX et interface

5. **Annulation réelle du calcul d'empreintes** : implémenter un `cancel_hash` (le bouton « Arrêter » appelle actuellement `cancelScan`).
6. **Progression granulaire** avec pourcentage par fichier et temps restant estimé.
7. **Filtres avancés** : par hash manquant, par taille, par date d'ajout, par présence de métadonnées.
8. **Vue détaillée avec onglets** (amorcée) : ajouter onglet "Médias" (covers/screenshots) et "Fichiers liés".
9. **Mode sombre/clair** avec thèmes personnalisables.
10. **Recherche fuzzy** et autocomplétion.

## Métadonnées et référentiels

11. **Édition manuelle des champs** : sauvegarder genre/dates/classifications/synopsis édités (actuellement seuls publisher/developer sont persistés).
12. **Multi-genres par jeu** : le champ Genre est un select unique ; passer à un multi-select.
13. **Synchronisation automatique** des référentiels (publishers/developers) au démarrage, comme les genres.
14. **Import/export de métadonnées** (JSON/CSV) pour sauvegarde/migration.
15. **Historique des enrichissements** avec possibilité de revert.

## Fonctionnalités

16. **Détection de doublons** (amorcée) : ajouter fusion manuelle et résolution assistée.
17. **Collections/playlists** et tags personnalisés.
18. **Statistiques** : graphiques par plateforme, par décennie, par genre.
19. **Lancement de jeux** : intégration avec émulateurs (détection déjà présente) + bouton "Jouer".
20. **Scan incrémental** : ne scanner que les nouveaux/modifiés fichiers (mtime).
21. **Support des images de boîtes** via ScreenScraper (medias déjà dans metadata_json).
22. **Mode hors-ligne** : cache des métadonnées et des images.

## Qualité et développement

23. **Tests** : ajouter des tests unitaires Rust (identify, scanner) et des tests frontend (React Testing Library).
24. **CI** : ajouter lint frontend (eslint) et tests E2E (Playwright).
25. **Logging** : tracing structuré avec niveaux configurables.
26. **Gestion d'erreurs** : messages utilisateur plus précis, codes d'erreur.
27. **Sécurité** : validation des chemins (path traversal), sandboxing.
28. **i18n** : compléter les traductions (actuellement partiel).

## Architecture

29. **Refactoring** : extraire la logique de hash dans un module dédié avec traits.
30. **Base de données** : migrations versionnées avec rollback, backup automatique.
31. **Configuration** : interface de gestion des providers (ScreenScraper credentials) dans l'app.
EOF
echo done