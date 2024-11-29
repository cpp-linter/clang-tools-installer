from pathlib import Path
from typing import Dict, Any
import yaml
import mkdocs_gen_files
from cli_gen import generate_cli_doc

FILENAME = "cli.md"

with mkdocs_gen_files.open(FILENAME, "w") as io_doc:
    options_versions = Path(__file__).parent / "cli.yml"
    versions: Dict[str, Any] = yaml.safe_load(options_versions.read_bytes())

    print("# Command Line Interface\n", file=io_doc)
    doc = generate_cli_doc(versions["commands"])
    # print(doc)
    print(doc, file=io_doc)

mkdocs_gen_files.set_edit_path(FILENAME, "gen_cli_doc.py")
