"""Reproduce the non-theme icon files from pinned upstream artwork."""

import argparse
from pathlib import Path
import subprocess
from urllib.request import urlopen
import xml.etree.ElementTree as ET


DEST = (
    Path(__file__).resolve().parents[1] / "crates/nine_dev_site_lib/assets/images/icons"
)
RAW = "https://raw.githubusercontent.com/"
BOOTSTRAP = "twbs/icons/ce0e49dd063243118a115f17ad1fe1fe7576d552/icons/"
MONO = "mono-company/mono-icons/43fac75e3d5ff38a55588e8c94a02ed96003a6ac/svg/"
ANILIST = "AniList/submission-manual/c30e98afdb152869756458e94a3054041d107305/docs/.vuepress/public/logo.png"
NS = "{http://www.w3.org/2000/svg}"


def fetch(path):
    with urlopen(RAW + path, timeout=30) as response:
        return response.read()


def symbol(view_box, paths, transform=None):
    root = ET.Element("svg", xmlns=NS[1:-1])
    parent = ET.SubElement(
        root, "symbol", id="icon", viewBox=view_box, fill="currentColor"
    )
    if transform:
        parent = ET.SubElement(parent, "g", transform=transform)
    for path in paths:
        if path.tag != NS + "path" or not path.get("d"):
            raise ValueError("Expected a path")
        attrs = dict(path.attrib)
        attrs.pop("fill", None)  # The original solid paint now inherits currentColor.
        if attrs.keys() - {"d", "fill-rule", "clip-rule"}:
            raise ValueError(f"Unexpected path attributes: {attrs.keys()}")
        ET.SubElement(parent, "path", attrs)
    ET.indent(root)
    return ET.tostring(root, encoding="utf-8") + b"\n"


def anilist():
    # Isolate both white and blue letters from the dark background, keeping a
    # square canvas. Potrace emits a Y-axis transform that must stay on the paths.
    bitmap = subprocess.run(
        [
            "magick",
            "png:-",
            "-crop",
            "204x204+26+26",
            "+repage",
            "-channel",
            "B",
            "-separate",
            "+channel",
            "-threshold",
            "50%",
            "-negate",
            "-compress",
            "none",
            "pbm:-",
        ],
        input=fetch(ANILIST),
        stdout=subprocess.PIPE,
        check=True,
    ).stdout
    traced = subprocess.run(
        ["potrace", "--svg", "--flat", "-o", "-"],
        input=bitmap,
        stdout=subprocess.PIPE,
        check=True,
    ).stdout
    svg = ET.fromstring(traced)
    group = svg.find(NS + "g")
    return symbol(svg.attrib["viewBox"], group, group.attrib["transform"])


def icons():
    for name in (
        "angle-down",
        "bluesky",
        "discord",
        "facebook",
        "github",
        "gitlab",
        "instagram",
        "linkedin",
        "mastodon",
        "signal",
        "spotify",
        "threads",
        "twitch",
        "website",
        "x",
        "youtube",
        "download",
        "email",
    ):
        stem = {"angle-down": "chevron-down", "website": "globe", "x": "twitter-x"}.get(
            name, name
        )
        prefix = MONO if name in {"download", "email"} else BOOTSTRAP
        svg = ET.fromstring(fetch(prefix + stem + ".svg"))
        yield name, symbol(svg.attrib["viewBox"], svg)
    yield "anilist", anilist()
    branch = ET.fromstring(
        fetch(
            "primer/octicons/c65a99647735412c633053fed85b1c1f5eee3099/icons/git-branch-24.svg"
        )
    )
    yield "git-branch", symbol(branch.attrib["viewBox"], branch)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="compare without writing")
    check = parser.parse_args().check
    for name, data in icons():
        path = DEST / (name + ".svg")
        if check:
            if path.read_bytes() != data:
                raise SystemExit(f"Upstream reproduction differs: {path}")
        else:
            path.write_bytes(data)
