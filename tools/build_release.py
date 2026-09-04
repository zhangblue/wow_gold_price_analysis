"""Build a self-contained release directory for the gold price service."""

from pathlib import Path
import shutil
import subprocess


ROOT = Path(__file__).resolve().parents[1]


def assemble_release(
    binary: Path, ui_dist: Path, config_template: Path, release_dir: Path
) -> None:
    """Copy build artifacts into a release directory without replacing local config."""
    release_dir.mkdir(parents=True, exist_ok=True)
    shutil.copy2(binary, release_dir / binary.name)
    release_dist = release_dir / "dist"
    if release_dist.exists():
        shutil.rmtree(release_dist)
    shutil.copytree(ui_dist, release_dist)

    config_dir = release_dir / "config"
    config_dir.mkdir(exist_ok=True)
    config_file = config_dir / ".env"
    if not config_file.exists():
        shutil.copy2(config_template, config_file)

    (release_dir / "logs").mkdir(exist_ok=True)


def main() -> None:
    subprocess.run(["npm", "run", "build"], cwd=ROOT / "frontend", check=True)
    subprocess.run(
        ["cargo", "build", "--release", "--manifest-path", "backend/Cargo.toml"],
        cwd=ROOT,
        check=True,
    )
    assemble_release(
        ROOT / "backend/target/release/gold-price-backend",
        ROOT / "frontend/dist",
        ROOT / "backend/config/.env.example",
        ROOT / "release",
    )


if __name__ == "__main__":
    main()
