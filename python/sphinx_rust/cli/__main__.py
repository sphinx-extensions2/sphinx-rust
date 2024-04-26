import argparse
from pathlib import Path
import sys

from sphinx_rust.sphinx_rust import analyze_crate


def main() -> None:
    parser = argparse.ArgumentParser(description="Simple CLI to analyze a Rust crate")
    parser.add_argument(
        "crate", metavar="PATH", help="Path to the crate directory to analyze"
    )
    parser.add_argument(
        "--output",
        metavar="PATH",
        help="Path to the output directory",
        default="_analysis",
    )
    parser.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite the output directory if it exists",
    )

    args = parser.parse_args()
    output = Path(args.output).resolve()
    if output.exists() and not args.overwrite:
        print(  # noqa: T201
            f"Output directory {output} already exists. Use --overwrite to overwrite it."
        )
        sys.exit(1)
    output.mkdir(parents=True, exist_ok=True)
    result = analyze_crate(args.crate, str(output))
    print("Written analysis to", output)  # noqa: T201
    print(result)  # noqa: T201


if __name__ == "__main__":
    main()
