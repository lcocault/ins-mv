#!/usr/bin/env python3
"""Génère les catalogues de pouvoirs et de talents à partir de regles/ (lecture seule).

Usage : python3 -I moteur/outils/generer_catalogues.py
Écrit moteur/regles/donnees/{pouvoirs,talents}.json. À relancer si regles/ change.
"""
import json
import re
from pathlib import Path

RACINE = Path(__file__).resolve().parents[2]
SORTIE = RACINE / "moteur" / "regles" / "donnees"


def pouvoirs(livre: str, fichier: str):
    famille = categorie = ""
    resultat = []
    for ligne in (RACINE / "regles" / fichier).read_text(encoding="utf-8").splitlines():
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


FICHIERS_POUVOIRS = {
    "ange": ["ange-05-pouvoirs-offensifs", "ange-06-pouvoirs-defensifs", "commun-07-pouvoirs-communs",
             "ange-07-pouvoirs-utilitaires", "ange-08-pouvoirs-objets", "ange-09-pouvoirs-serviteurs",
             "ange-10-pouvoirs-limitations"],
    "demon": ["demon-05-pouvoirs-offensifs", "demon-06-pouvoirs-defensifs", "commun-07-pouvoirs-communs",
              "demon-07-pouvoirs-utilitaires", "demon-08-pouvoirs-objets", "demon-09-pouvoirs-serviteurs",
              "demon-10-pouvoirs-limitations"],
}
CARACS = {"Force": "FO", "Précision": "PR", "Volonté": "VO", "Perception": "PE", "Agilité": "AG", "Apparence": "AP"}


def variante(texte, livre):
    """Résout les variantes inline « {MV: a ; INS: b} » pour un livre."""
    etiquette = "MV" if livre == "ange" else "INS"

    def choisir(m):
        for morceau in m.group(1).split(" ; "):
            if morceau.strip().startswith(etiquette + ":"):
                return morceau.strip()[len(etiquette) + 1:].strip()
        return ""

    return re.sub(r"\{((?:MV|INS):[^{}]*)\}", choisir, texte)


def descriptions(livre):
    """Texte du livre pour chaque pouvoir : paragraphes propres, sinon introduction de la catégorie."""
    etiquette = "MV" if livre == "ange" else "INS"
    propres, intro_cat, caracs, cat_de = {}, {}, {}, {}
    categorie, courant = None, None
    for fichier in FICHIERS_POUVOIRS[livre]:
        actif, vu_contenu = "tous", False
        for ligne in (RACINE / "regles" / f"{fichier}.md").read_text(encoding="utf-8").splitlines():
            marque = re.fullmatch(r"\*\*\[(MV|INS)\]\*\*", ligne.strip())
            if marque:
                actif, vu_contenu = marque.group(1), False
                continue
            # Un bloc [MV]/[INS] couvre le paragraphe qui suit ; ensuite le texte est commun.
            if actif != "tous":
                if ligne.strip() and not ligne.startswith("<!--"):
                    vu_contenu = True
                elif not ligne.strip() and vu_contenu:
                    actif = "tous"
            if re.match(r"^#{1,6} ", ligne):
                actif = "tous"
                m_cat = re.match(r"^#### (\d{2})- ", ligne)
                m_pou = re.match(r"^##### (\d{3})- (.*?)(?: \((\w+)\))?\s*$", ligne)
                if m_cat:
                    categorie, courant = m_cat.group(1), None
                    intro_cat.setdefault(categorie, [])
                elif m_pou:
                    courant = int(m_pou.group(1))
                    propres[courant] = []
                    cat_de[courant] = m_pou.group(1)[:2]
                    if m_pou.group(3):
                        caracs[courant] = CARACS.get(m_pou.group(3))
                else:
                    courant = None
                continue
            if actif not in ("tous", etiquette):
                continue
            puce = re.match(r"^- \*\*(\d{3})- .*?\*\*(?: \((\w+)\))?", ligne.strip())
            if puce:
                code = int(puce.group(1))
                cat_de[code] = puce.group(1)[:2]
                if puce.group(2):
                    caracs[code] = CARACS.get(puce.group(2))
                continue
            if ligne.startswith("<!--") or ligne.startswith(">") or not ligne.strip():
                continue
            texte = variante(ligne.strip(), livre)
            if courant is not None:
                propres[courant].append(texte)
            elif categorie is not None:
                intro_cat[categorie].append(texte)
    resultat = {}
    for code, cat in cat_de.items():
        propre = " ".join(propres.get(code, [])).strip()
        generique = " ".join(intro_cat.get(cat, [])).strip()
        resultat[code] = {"description": propre or generique or None, "jet": caracs.get(code)}
    return resultat


def talents():
    texte = (RACINE / "regles" / "commun-04-talents.md").read_text(encoding="utf-8")
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


def entier(texte):
    """« -3 » → -3, « + 1 » → 1, « - » ou vide → 0."""
    t = texte.replace(" ", "")
    if t in ("", "-"):
        return 0
    return int(t)


def armes_et_protections():
    """Tables d'armes, de protections et notes de commun-09-confrontation-armes-combat.md."""
    texte = (RACINE / "regles" / "commun-09-confrontation-armes-combat.md").read_text(encoding="utf-8")
    armes, protections = [], []
    mode, notes = None, {}

    def clore():
        # Rattache à chaque entrée du tableau précédent les notes citées (a, b, c...).
        cibles = armes if mode in ("contact", "distance") else protections
        for entree in cibles:
            if entree.get("_lot") == mode and "notes" not in entree:
                entree["notes"] = [{"id": i, "texte": notes[i]} for i in entree.pop("_ids") if i in notes]
                entree.pop("_lot", None)

    for ligne in texte.splitlines():
        brut = ligne.strip()
        if brut.startswith("|"):
            cases = [c.strip() for c in brut.strip("|").split("|")]
            if cases[0] == "Arme de contact":
                clore(); mode, notes = "contact", {}
            elif cases[0] == "Arme à distance":
                clore(); mode, notes = "distance", {}
            elif cases[0] == "Nom":
                clore(); mode, notes = "protection", {}
            elif set(cases[0]) <= {"-"}:
                continue
            elif mode == "contact" and len(cases) == 5:
                nom, puiss, prec, talent, ids = cases
                armes.append({"nom": nom, "categorie": "contact", "puissance": None if puiss == "Spécial" else entier(puiss),
                              "puissance_speciale": puiss == "Spécial", "puissance_rafale": None,
                              "precision": entier(prec), "portee": None, "coups": None, "talent": talent,
                              "_ids": [i.strip() for i in ids.split(",") if i.strip()], "_lot": mode})
            elif mode == "distance" and len(cases) == 7:
                nom, puiss, prec, portee, coups, talent, ids = cases
                import re as _re
                m = _re.fullmatch(r"([+-]?\d+)?(?:\(([+-]?\d+)\))?", puiss.replace(" ", ""))
                base = int(m.group(1)) if m and m.group(1) else None
                rafale = int(m.group(2)) if m and m.group(2) else None
                armes.append({"nom": nom, "categorie": "distance", "puissance": base,
                              "puissance_speciale": puiss == "Spécial", "puissance_rafale": rafale,
                              "precision": entier(prec), "portee": None if portee in ("-", "") else int(portee),
                              "coups": None if coups == "-" else coups, "talent": talent,
                              "_ids": [i.strip() for i in ids.split(",") if i.strip()], "_lot": mode})
            elif mode == "protection" and len(cases) == 4:
                nom, prot, ajust, ids = cases
                protections.append({"nom": nom, "protection": int(prot), "ajustement": entier(ajust),
                                    "_ids": [i.strip() for i in ids.split(",") if i.strip()], "_lot": mode})
        else:
            m = __import__("re").match(r"- ([a-z]): (.*)", brut)
            if m and mode:
                notes[m.group(1)] = m.group(2)
            if brut.startswith("#### Les protections") or brut.startswith("#### Le combat"):
                clore(); mode = None if brut.startswith("#### Le combat") else mode
    clore()
    return armes, protections


def ecrire(nom, donnees):
    SORTIE.mkdir(parents=True, exist_ok=True)
    (SORTIE / nom).write_text(
        json.dumps(donnees, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
    print(f"{nom}: {len(donnees)} entrées")


if __name__ == "__main__":
    tous = pouvoirs("ange", "ange-17-liste-des-pouvoirs.md") + pouvoirs("demon", "demon-17-liste-des-pouvoirs.md")
    textes = {livre: descriptions(livre) for livre in ("ange", "demon")}
    for entree in tous:
        info = textes[entree["livre"]].get(entree["code"], {}) if entree["code"] else {}
        entree["description"] = info.get("description")
        entree["jet"] = info.get("jet")
    ecrire("pouvoirs.json", tous)
    ecrire("talents.json", talents())
    _armes, _protections = armes_et_protections()
    ecrire("armes.json", _armes)
    ecrire("protections.json", _protections)
