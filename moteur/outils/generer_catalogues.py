#!/usr/bin/env python3
"""Génère les catalogues de pouvoirs et de talents à partir de livres/ (lecture seule).

Usage : python3 -I moteur/outils/generer_catalogues.py
Écrit moteur/regles/donnees/{pouvoirs,talents}.json. À relancer si livres/ change.
"""
import json
import re
from pathlib import Path

RACINE = Path(__file__).resolve().parents[2]
SORTIE = RACINE / "moteur" / "regles" / "donnees"


def pouvoirs(livre: str, fichier: str):
    famille = categorie = ""
    resultat = []
    for ligne in (RACINE / "livres" / fichier).read_text(encoding="utf-8").splitlines():
        if ligne.startswith("## "):
            famille = ligne[3:].strip()
        elif ligne.startswith("### "):
            categorie = ligne[4:].strip()
        elif ligne.startswith("|"):
            cases = [c.strip() for c in ligne.strip().strip("|").split("|")]
            if len(cases) != 5 or cases[0] in ("N°", "---"):
                continue
            code, nom, att, deff, cout = cases
            if not re.fullmatch(r"\d{3}|", code) or not nom:
                continue
            resultat.append({
                "livre": livre,
                "code": int(code) if code else None,
                "nom": nom,
                "famille": famille,
                "categorie": categorie,
                "attaque": att or None,
                "defense": deff or None,
                "cout": cout or None,
            })
    return resultat


def talents():
    texte = (RACINE / "livres" / "commun-04-talents.md").read_text(encoding="utf-8")
    categorie = ""
    vus, resultat = set(), []
    caracs = {"Force": "FO", "Précision": "PR", "Volonté": "VO",
              "Perception": "PE", "Agilité": "AG", "Apparence": "AP"}

    def ajouter(nom, malus, desc=""):
        if nom in vus:
            return
        vus.add(nom)
        trouvees = sorted({abbr for mot, abbr in caracs.items() if mot in desc})
        resultat.append({"nom": nom, "categorie": categorie, "malus": malus,
                         "caracteristiques": trouvees})

    for ligne in texte.splitlines():
        if ligne.startswith("###### "):
            categorie = ligne[7:].strip()
        m = re.match(r"- \*\*(.+?)\((-\d+|Variable)\):\*\*(.*)", ligne.strip())
        if m:
            nom, malus, desc = m.groups()
            ajouter(nom.strip(), None if malus == "Variable" else int(malus), desc)
        elif categorie == "Artistique" and "Danse(" in ligne:
            # Talents artistiques cités en prose : « Danse(-4) (Agilité), ... »
            for nom, malus, carac in re.findall(r"([A-Za-zéèêîô]+)\((-\d+)\) \((\w+)\)", ligne):
                ajouter(nom.capitalize(), int(malus), carac)
    # « comédie(-3) » dans le livre des Anges, « comédie(-4) » dans celui des Démons.
    resultat.append({"nom": "Comédie", "categorie": "Artistique", "malus": -3,
                     "malus_demon": -4, "caracteristiques": ["VO"]})
    return resultat


def ecrire(nom, donnees):
    SORTIE.mkdir(parents=True, exist_ok=True)
    (SORTIE / nom).write_text(
        json.dumps(donnees, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"{nom}: {len(donnees)} entrées")


if __name__ == "__main__":
    ecrire("pouvoirs.json",
           pouvoirs("ange", "ange-17-liste-des-pouvoirs.md")
           + pouvoirs("demon", "demon-17-liste-des-pouvoirs.md"))
    ecrire("talents.json", talents())
