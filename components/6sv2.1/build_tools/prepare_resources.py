"""Export literal standard atmospheres and band labels from pinned source."""

import hashlib, json, re
from build import ROOT, SOURCE, verify_source

verify_source()
profiles = {}
for i, name in enumerate(["TROPIC", "MIDSUM", "MIDWIN", "SUBSUM", "SUBWIN", "US62"], 1):
    text = " ".join(
        line.expandtabs(8)[6:132]
        for line in (SOURCE / (name + ".f")).read_text().splitlines()
        if line and line[0] not in "cC*!"
    )
    arrays = {}
    for variable, body in re.findall(
        r"data\s*\(\s*(\w+)\(i\),i=1,\s*34\)\s*/(.*?)/", text, re.I | re.S
    ):
        arrays[variable.lower()] = [
            float(x.strip().replace("D", "e").replace("d", "e"))
            for x in body.split(",")
            if x.strip()
        ]
    columns = [arrays[prefix + str(i)] for prefix in ["z", "p", "t", "wh", "wo"]]
    assert all(len(x) == 34 for x in columns), (name, arrays.keys())
    profiles[str(i)] = {
        "name": name.lower(),
        "columns": [
            "altitude_km",
            "pressure_hpa",
            "temperature_k",
            "water_g_m3",
            "ozone_g_m3",
        ],
        "levels": [list(x) for x in zip(*columns)],
    }
text = " ".join(
    line.expandtabs(8)[6:132]
    for line in (SOURCE / "main.f").read_text().splitlines()
    if line and line[0] not in "cC*!"
)
body = re.search(r"data\s+nsat\s*/(.*?)/", text, re.I | re.S)[1]
names = re.findall(r"'([^']*)'", body)
assert len(names) == 200, len(names)
data = {
    "source_archive_sha256": "42422db29c095a49eaa98556b416405eb818be1ee30139d2a1913dbf3b0c7de1",
    "profiles": profiles,
    "bands": [
        {"id": i, "name": name.strip()} for i, name in enumerate(names) if i >= 2
    ],
}
p = ROOT / "crates/models/atmosphere/sixs/data/resources.json"
p.parent.mkdir(parents=True, exist_ok=True)
p.write_text(json.dumps(data, indent=2) + "\n", encoding="utf8")
print(
    "Exported 6 standard profiles and",
    len(data["bands"]),
    "original band labels; SHA256",
    hashlib.sha256(p.read_bytes()).hexdigest(),
)
