# INS/MV — Livres de règles (1re édition) : extraction Markdown

Extraction des deux livres de base du jeu de rôle *In Nomine Satanis / Magna Veritas* (1re édition) :

- **Livre des Anges** (*Magna Veritas*, « MV ») : `INS_MV_livrebase_1ed_ANGE.pdf`, 77 pages numérisées ;
- **Livre des Démons** (*In Nomine Satanis*, « INS ») : `INS_MV_livrebase_1ed_DEMON.pdf`, 78 pages numérisées.

## Organisation des fichiers

Les deux livres ont une structure parallèle. Les sections **identiques ou quasi identiques** sont mutualisées dans les fichiers `commun-*.md` ; le reste est dans les fichiers `ange-*.md` (livre des Anges) et `demon-*.md` (livre des Démons).

### Fichiers communs

- [commun-01-avertissements.md](commun-01-avertissements.md) — Avertissements
- [commun-02-des.md](commun-02-des.md) — Les dés
- [commun-03-caracteristiques.md](commun-03-caracteristiques.md) — Principes de jeu : les caractéristiques
- [commun-04-talents.md](commun-04-talents.md) — Les talents
- [commun-05-table-um.md](commun-05-table-um.md) — La table unique multiple (table UM)
- [commun-06-creation-personnages-humains.md](commun-06-creation-personnages-humains.md) — Création des personnages : généralités, humains, résumé
- [commun-07-pouvoirs-communs.md](commun-07-pouvoirs-communs.md) — Pouvoirs identiques (ou presque) dans les deux livres
- [commun-08-pouvoirs-annexes-biens.md](commun-08-pouvoirs-annexes-biens.md) — Annexes communes : aura / langage, biens matériels
- [commun-09-confrontation-armes-combat.md](commun-09-confrontation-armes-combat.md) — La confrontation : armes, protections, combat
- [commun-10-dommages-et-batiments.md](commun-10-dommages-et-batiments.md) — Visualisation des dommages, bâtiments et véhicules
- [commun-11-combats-de-masse.md](commun-11-combats-de-masse.md) — Les combats de masse
- [commun-12-hierarchie-rapports.md](commun-12-hierarchie-rapports.md) — Hiérarchie : rapports avec supérieurs et serviteurs
- [commun-13-experience.md](commun-13-experience.md) — Expérience

Dans les fichiers communs :

- le texte sans marque est **identique** dans les deux livres ;
- `{MV: … ; INS: …}` signale une **variante locale** (mot, chiffre, nom propre) selon le livre ;
- un bloc **[MV]** ou **[INS]** signale un passage **propre à un livre** ou trop différent pour être aligné (exemples de jeu, citations, tableaux spécifiques, etc.) ;
- un commentaire HTML `<!-- Sources : MV p.x ; INS p.y -->` donne les pages imprimées d'origine.

### Fichiers spécifiques

Chaque fichier `ange-*` / `demon-*` renvoie par une ligne « ↪ *Section commune* » vers le fichier commun à lire à cet endroit du livre.

## Ordre de lecture — livre des Anges (*Magna Veritas*)

Fichiers dans l'ordre où ils apparaissent dans le livre (les passages communs sont à lire au fil du texte, là où un fichier spécifique renvoie vers un fichier `commun-*`) :

1. [commun-01-avertissements.md](commun-01-avertissements.md) — Avertissements
2. [ange-01-introduction.md](ange-01-introduction.md) — Introduction
3. [commun-02-des.md](commun-02-des.md) — Les dés
4. [ange-02-regles-specifiques.md](ange-02-regles-specifiques.md) — Règles de base propres au livre des Anges
5. [commun-03-caracteristiques.md](commun-03-caracteristiques.md) — Principes de jeu : les caractéristiques
6. [commun-04-talents.md](commun-04-talents.md) — Les talents
7. [commun-05-table-um.md](commun-05-table-um.md) — La table unique multiple (table UM)
8. [commun-06-creation-personnages-humains.md](commun-06-creation-personnages-humains.md) — Création des personnages : généralités, humains, résumé
9. [ange-03-creation-personnages.md](ange-03-creation-personnages.md) — Création des personnages : Serviteurs, Soldats, Anges
10. [ange-04-pouvoirs-generalites.md](ange-04-pouvoirs-generalites.md) — Les pouvoirs des Anges : généralités
11. [ange-05-pouvoirs-offensifs.md](ange-05-pouvoirs-offensifs.md) — Pouvoirs offensifs
12. [ange-06-pouvoirs-defensifs.md](ange-06-pouvoirs-defensifs.md) — Pouvoirs défensifs
13. [commun-07-pouvoirs-communs.md](commun-07-pouvoirs-communs.md) — Pouvoirs identiques (ou presque) dans les deux livres
14. [ange-07-pouvoirs-utilitaires.md](ange-07-pouvoirs-utilitaires.md) — Pouvoirs utilitaires
15. [ange-08-pouvoirs-objets.md](ange-08-pouvoirs-objets.md) — Pouvoirs : objets, locaux, associations
16. [ange-09-pouvoirs-serviteurs.md](ange-09-pouvoirs-serviteurs.md) — Pouvoirs : serviteurs
17. [ange-10-pouvoirs-limitations.md](ange-10-pouvoirs-limitations.md) — Limitations
18. [ange-11-confrontation.md](ange-11-confrontation.md) — La confrontation
19. [commun-09-confrontation-armes-combat.md](commun-09-confrontation-armes-combat.md) — La confrontation : armes, protections, combat
20. [ange-12-blessures.md](ange-12-blessures.md) — Effet des blessures
21. [commun-10-dommages-et-batiments.md](commun-10-dommages-et-batiments.md) — Visualisation des dommages, bâtiments et véhicules
22. [commun-11-combats-de-masse.md](commun-11-combats-de-masse.md) — Les combats de masse
23. [ange-13-hierarchie.md](ange-13-hierarchie.md) — La hiérarchie
24. [commun-12-hierarchie-rapports.md](commun-12-hierarchie-rapports.md) — Hiérarchie : rapports avec supérieurs et serviteurs
25. [commun-13-experience.md](commun-13-experience.md) — Expérience
26. [ange-14-annexes.md](ange-14-annexes.md) — Annexes
27. [commun-08-pouvoirs-annexes-biens.md](commun-08-pouvoirs-annexes-biens.md) — Annexes communes : aura / langage, biens matériels
28. [ange-15-vie-des-anges-et-conseils.md](ange-15-vie-des-anges-et-conseils.md) — La vie privée des Anges, conseils de jeu
29. [ange-16-archanges.md](ange-16-archanges.md) — Les Archanges
30. [ange-17-liste-des-pouvoirs.md](ange-17-liste-des-pouvoirs.md) — Liste des pouvoirs, sources et crédits

## Ordre de lecture — livre des Démons (*In Nomine Satanis*)

1. [demon-01-introduction.md](demon-01-introduction.md) — Introduction
2. [commun-02-des.md](commun-02-des.md) — Les dés
3. [demon-02-regles-specifiques.md](demon-02-regles-specifiques.md) — Règles de base propres au livre des Démons
4. [commun-03-caracteristiques.md](commun-03-caracteristiques.md) — Principes de jeu : les caractéristiques
5. [commun-04-talents.md](commun-04-talents.md) — Les talents
6. [commun-05-table-um.md](commun-05-table-um.md) — La table unique multiple (table UM)
7. [commun-06-creation-personnages-humains.md](commun-06-creation-personnages-humains.md) — Création des personnages : généralités, humains, résumé
8. [demon-03-creation-personnages.md](demon-03-creation-personnages.md) — Création des personnages : mort-vivants, familiers, Démons
9. [demon-04-pouvoirs-generalites.md](demon-04-pouvoirs-generalites.md) — Les pouvoirs des Démons : généralités
10. [demon-05-pouvoirs-offensifs.md](demon-05-pouvoirs-offensifs.md) — Pouvoirs offensifs
11. [demon-06-pouvoirs-defensifs.md](demon-06-pouvoirs-defensifs.md) — Pouvoirs défensifs
12. [commun-07-pouvoirs-communs.md](commun-07-pouvoirs-communs.md) — Pouvoirs identiques (ou presque) dans les deux livres
13. [demon-07-pouvoirs-utilitaires.md](demon-07-pouvoirs-utilitaires.md) — Pouvoirs utilitaires
14. [demon-08-pouvoirs-objets.md](demon-08-pouvoirs-objets.md) — Pouvoirs : objets, locaux, planques
15. [demon-09-pouvoirs-serviteurs.md](demon-09-pouvoirs-serviteurs.md) — Pouvoirs : serviteurs
16. [demon-10-pouvoirs-limitations.md](demon-10-pouvoirs-limitations.md) — Limitations
17. [demon-11-confrontation.md](demon-11-confrontation.md) — La confrontation
18. [commun-09-confrontation-armes-combat.md](commun-09-confrontation-armes-combat.md) — La confrontation : armes, protections, combat
19. [demon-12-blessures.md](demon-12-blessures.md) — Effet des blessures
20. [commun-10-dommages-et-batiments.md](commun-10-dommages-et-batiments.md) — Visualisation des dommages, bâtiments et véhicules
21. [commun-11-combats-de-masse.md](commun-11-combats-de-masse.md) — Les combats de masse
22. [demon-13-hierarchie.md](demon-13-hierarchie.md) — La hiérarchie
23. [commun-12-hierarchie-rapports.md](commun-12-hierarchie-rapports.md) — Hiérarchie : rapports avec supérieurs et serviteurs
24. [commun-13-experience.md](commun-13-experience.md) — Expérience
25. [demon-14-annexes.md](demon-14-annexes.md) — Annexes
26. [commun-08-pouvoirs-annexes-biens.md](commun-08-pouvoirs-annexes-biens.md) — Annexes communes : aura / langage, biens matériels
27. [demon-15-vie-des-demons-et-conseils.md](demon-15-vie-des-demons-et-conseils.md) — La vie privée des Démons, conseils de jeu
28. [demon-16-princes-demons.md](demon-16-princes-demons.md) — Les Prince-Démons
29. [demon-17-liste-des-pouvoirs.md](demon-17-liste-des-pouvoirs.md) — Liste des pouvoirs, sources et crédits

## Notes sur la transcription

- Les PDF sont des **scans sans couche texte** : le contenu a été transcrit à partir de l'image de chaque page. La mise en page à deux colonnes a été aplatie en suivant l'ordre de lecture ; les coquilles et fautes d'orthographe de l'original ont été conservées.
- Les numéros de page `<!-- A p.N -->` / `<!-- D p.N -->` des fichiers spécifiques sont les numéros **imprimés** dans les livres (le numéro de page du PDF est donné entre parenthèses).
- Les **illustrations pleines pages** sont signalées par un commentaire HTML mais non reproduites. Les **sommaires** et la **fiche de personnage vierge** ne sont pas transcrits (les sommaires sont remplacés par la structure de fichiers ci-dessus ; la fiche est signalée en commentaire en fin de livre).
- Les tableaux « Liste des pouvoirs » de fin de livre (fichiers `*-17-*`) reprennent le récapitulatif imprimé, remis en forme en tableaux.
- La table UM est suivie d'une note de transcription (en italique) précisant la lecture des colonnes ; elle ne figure pas dans l'original.
- Dans le livre des Démons, le titre « La vie des privée des Démons » a été normalisé en « La vie privée des Démons » (titre du sommaire), et les sous-sections de cette partie ont été remontées d'un niveau de titre pour être cohérentes avec le livre des Anges.
