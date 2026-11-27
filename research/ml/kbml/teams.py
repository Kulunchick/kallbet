"""`team_names::team_key` (sidecar/src/team_names.rs), ported line for line."""
NOISE = {"team", "esports", "esport", "gaming", "clan", "club", "gg"}

ALIASES = {
    "nip": "ninjasinpyjamas",
    "navi": "natusvincere",
    "tl": "liquid",
    "fq": "flyquest",
    "fly": "flyquest",
    "vit": "vitality",
    "imp": "imperial",
    "gl": "gamerlegion",
    "lvg": "lynnvision",
    "mongolz": "themongolz",
    "col": "complexity",
    "c9": "cloud9",
    "vp": "virtuspro",
    "pvision": "parivision",
    "pv": "parivision",
    "dendelecs": "dendele",
    "chinggis": "chinggiswarriors",
    "cw": "chinggiswarriors",
    "ef": "eternalfire",
    "100t": "100thieves",
    "fnc": "fnatic",
    "gaimin": "gaimingladiators",
    "gosh": "bcgame",
    "wc": "wildcard",
    "bb": "betboom",
    "bbteam": "betboom",
}

def team_key(name: str) -> str:
    lower = name.lower()
    words = []
    cur = []
    for ch in lower:
        if ch.isalnum():
            cur.append(ch)
        else:
            if cur:
                words.append("".join(cur))
            cur = []
    if cur:
        words.append("".join(cur))
    kept = "".join(w for w in words if w not in NOISE)
    key = kept if kept else "".join(words)
    return ALIASES.get(key, key)
