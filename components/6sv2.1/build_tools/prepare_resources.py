"""Export literal standard atmospheres and band metadata from pinned source."""

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
statements = " ".join(
    f"{line[:5]} {line[6:132]}" if len(line) > 5 else line
    for line in (SOURCE / "main.f").read_text().splitlines()
    if line and line[0] not in "cC*!"
)
dispatch = re.search(r"18\s+goto\s*\((.*?)\),iwave", statements, re.I)[1]
labels = [int(label) for label in dispatch.split(",")]
assert len(labels) == 199
calls = {
    int(label): (sensor.lower(), int(offset or 0))
    for label, sensor, offset in re.findall(
        r"(\d+)\s+call (\w+)(?:\(iwave-(\d+)\))?\s+go\s*to 19", statements, re.I
    )
}
band_sources = {p.stem.lower(): p for p in SOURCE.glob("*.f")}
band_metadata = []
for band_id in range(2, 200):
    sensor, offset = calls[labels[band_id - 1]]
    source = band_sources[sensor].read_text()
    index = band_id - offset
    bound_names = (
        ("wlinf", "wlsup") if sensor == "meteo" else (f"wli({index})", f"wls({index})")
    )
    bounds = [
        float(
            re.search(rf"^\s*{re.escape(name)}\s*=\s*([.\d]+)", source, re.I | re.M)[1]
        )
        for name in bound_names
    ]
    assert 0.25 <= bounds[0] <= bounds[1] <= 4.0, (band_id, bounds)
    band_metadata.append(
        dict(
            id=band_id,
            name=names[band_id].strip(),
            sensor=sensor,
            lower_um=bounds[0],
            upper_um=bounds[1],
        )
    )
data = {
    "source_archive_sha256": "42422db29c095a49eaa98556b416405eb818be1ee30139d2a1913dbf3b0c7de1",
    "profiles": profiles,
    "bands": band_metadata,
}
p = ROOT / "crates/models/atmosphere/sixs/data/resources.json"
p.parent.mkdir(parents=True, exist_ok=True)
p.write_text(json.dumps(data, indent=2) + "\n", encoding="utf8", newline="\r\n")
print(
    "Exported 6 standard profiles and",
    len(data["bands"]),
    "original bands; SHA256",
    hashlib.sha256(p.read_bytes()).hexdigest(),
)
