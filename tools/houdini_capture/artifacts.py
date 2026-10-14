"""Output containment and collision rules shared by capture and tests."""
from pathlib import Path


def new_output(root, name):
    root = Path(root).resolve()
    path = (root / name).resolve()
    if not path.is_relative_to(root) or path == root:
        raise ValueError('Output must stay inside the run directory')
    if path.exists():
        raise FileExistsError(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    return path


def capture_outputs(root, name):
    # Keep foo.json parameter exports separate from foo.png.json capture metadata.
    image = new_output(root, name)
    metadata = new_output(root, name + '.json')
    return image, metadata
