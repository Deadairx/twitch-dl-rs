#!/usr/bin/env python3

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


TWITCH_GQL_URL = "https://gql.twitch.tv/gql"
TWITCH_CLIENT_ID = "kimne78kx3ncx6brgo4mv6wki5h1ko"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Backfill missing title fields in VOD artifact metadata.json files."
    )
    parser.add_argument(
        "output_root",
        nargs="?",
        help="Artifact root directory. Defaults to VOD_PIPELINE_OUTPUT_ROOT.",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite existing non-empty title fields.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print planned changes without writing files.",
    )
    parser.add_argument(
        "--delay-seconds",
        type=float,
        default=0.0,
        help="Optional delay between Twitch requests.",
    )
    return parser.parse_args()


def resolve_output_root(cli_value: str | None) -> Path:
    root = cli_value or os.environ.get("VOD_PIPELINE_OUTPUT_ROOT")
    if not root:
        raise SystemExit(
            "error: provide <output_root> or set VOD_PIPELINE_OUTPUT_ROOT"
        )

    path = Path(root).expanduser().resolve()
    if not path.exists():
        raise SystemExit(f"error: output root does not exist: {path}")
    if not path.is_dir():
        raise SystemExit(f"error: output root is not a directory: {path}")
    return path


def artifact_dirs(output_root: Path) -> list[Path]:
    return sorted(
        entry
        for entry in output_root.iterdir()
        if entry.is_dir() and entry.name.isascii() and entry.name.isdigit()
    )


def fetch_vod_title(video_id: str) -> str:
    payload = json.dumps(
        {
            "query": "query($id: ID!) { video(id: $id) { id title } }",
            "variables": {"id": video_id},
        }
    ).encode("utf-8")
    request = urllib.request.Request(
        TWITCH_GQL_URL,
        data=payload,
        headers={
            "Client-ID": TWITCH_CLIENT_ID,
            "Content-Type": "application/json",
            "User-Agent": "vod-pipeline-title-backfill/1.0",
        },
        method="POST",
    )

    try:
        with urllib.request.urlopen(request, timeout=20) as response:
            body = response.read().decode("utf-8")
    except urllib.error.HTTPError as error:
        detail = error.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"HTTP {error.code}: {detail}") from error
    except urllib.error.URLError as error:
        raise RuntimeError(f"network error: {error.reason}") from error

    try:
        parsed = json.loads(body)
    except json.JSONDecodeError as error:
        raise RuntimeError(f"invalid Twitch response: {error}") from error

    errors = parsed.get("errors")
    if errors:
        raise RuntimeError(f"Twitch API error: {errors}")

    video = parsed.get("data", {}).get("video")
    if not video:
        raise RuntimeError("video not found")

    title = video.get("title")
    if not isinstance(title, str) or not title.strip():
        raise RuntimeError("video title missing from response")

    return title


def process_artifact(
    artifact_dir: Path,
    overwrite: bool,
    dry_run: bool,
) -> tuple[str, str]:
    metadata_path = artifact_dir / "metadata.json"
    if not metadata_path.exists():
        return ("skipped", f"{artifact_dir.name}: missing metadata.json")

    try:
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        return ("failed", f"{artifact_dir.name}: invalid JSON: {error}")
    except OSError as error:
        return ("failed", f"{artifact_dir.name}: read failed: {error}")

    if not isinstance(metadata, dict):
        return ("failed", f"{artifact_dir.name}: metadata root is not an object")

    existing_title = metadata.get("title")
    if isinstance(existing_title, str) and existing_title.strip() and not overwrite:
        return ("skipped", f"{artifact_dir.name}: title already present")

    video_id = metadata.get("video_id")
    if not isinstance(video_id, str) or not video_id.strip():
        video_id = artifact_dir.name

    if not video_id.isdigit():
        return ("failed", f"{artifact_dir.name}: invalid video_id {video_id!r}")

    try:
        title = fetch_vod_title(video_id)
    except RuntimeError as error:
        return ("failed", f"{artifact_dir.name}: {error}")

    metadata["title"] = title

    if dry_run:
        return ("updated", f"{artifact_dir.name}: would set title to {title!r}")

    try:
        metadata_path.write_text(
            json.dumps(metadata, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
    except OSError as error:
        return ("failed", f"{artifact_dir.name}: write failed: {error}")

    return ("updated", f"{artifact_dir.name}: set title to {title!r}")


def main() -> int:
    args = parse_args()
    output_root = resolve_output_root(args.output_root)
    scanned = 0
    updated = 0
    skipped = 0
    failed = 0

    for artifact_dir in artifact_dirs(output_root):
        scanned += 1
        status, message = process_artifact(
            artifact_dir,
            overwrite=args.overwrite,
            dry_run=args.dry_run,
        )
        print(message)

        if status == "updated":
            updated += 1
        elif status == "skipped":
            skipped += 1
        else:
            failed += 1

        if args.delay_seconds > 0:
            time.sleep(args.delay_seconds)

    mode = "dry-run" if args.dry_run else "write"
    print(
        f"summary ({mode}): scanned={scanned} updated={updated} skipped={skipped} failed={failed}"
    )
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
