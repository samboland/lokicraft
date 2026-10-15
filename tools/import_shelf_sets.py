"""Import shelf-set membership only from installed Houdini definitions."""
import argparse, hashlib, json
from pathlib import Path
import xml.etree.ElementTree as ET
parser = argparse.ArgumentParser()
parser.add_argument("houdini", type=Path)
args = parser.parse_args()
sets = []
sources = []
for filename, name in [("ShelfDefinitions.shelf", "shelf_set_1"), ("ShelfDefinitions.master_shelf", "shelf_set_2")]:
    source = args.houdini / "toolbar" / filename
    node = ET.parse(source).find(f"shelfSet[@name='{name}']")
    sets.append(dict(name=name, label=node.attrib["label"], shelves=[n.attrib["name"] for n in node.findall("memberToolshelf")]))
    sources.append(dict(file=filename, sha256=hashlib.sha256(source.read_bytes()).hexdigest()))
root = Path(__file__).resolve().parents[1]
(root / "assets/shelf-sets.json").write_text(json.dumps(sets, indent=2) + "\n")
(root / "assets/shelf-sets-provenance.json").write_text(json.dumps(dict(source="Houdini 22.0.466 Build desktop", split=[0.5, 0.5], definitions=sources, excluded=["scripts", "icons"]), indent=2) + "\n")
