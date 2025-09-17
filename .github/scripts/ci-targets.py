#!/usr/bin/env python3

import sys


def read_file_lines(filepath):
    """Read a file and return a set of stripped lines."""
    with open(filepath, "r") as f:
        return set(line.strip() for line in f)


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} file1 file2")
        sys.exit(1)

    file1, file2 = sys.argv[1], sys.argv[2]

    set1 = read_file_lines(file1)
    set2 = read_file_lines(file2)

    intersection = set1 & set2  # set intersection

    for item in sorted(intersection):
        print(item)


if __name__ == "__main__":
    main()
