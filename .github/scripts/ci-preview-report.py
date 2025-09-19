#!/usr/bin/env python3
"""
GitHub Actions preview deployment report generator.
Reads needs JSON and generates a markdown table for PR comments.
"""

import json
import os
import sys


def get_status_emoji(result: str) -> str:
    """Map GitHub Actions job result to emoji."""
    emoji_map = {
        "success": "✅",
        "failure": "❌",
        "cancelled": "🚫",
        "skipped": "⏭️",
    }
    return emoji_map.get(result, "❓")


def format_url(url: str) -> str:
    """Format URL as markdown link or return dash if empty."""
    if url and url != "null":
        return f"[{url}]({url})"
    return "-"


def generate_report(needs_json: str) -> str:
    """
    Generate markdown report from needs JSON.

    Args:
        needs_json: JSON string containing GitHub Actions needs context
        pr_number: PR number for commenting

    Returns:
        Tuple of (pr_number, markdown_content)
    """
    try:
        needs = json.loads(needs_json)
    except json.JSONDecodeError as e:
        print(f"Error parsing needs JSON: {e}", file=sys.stderr)
        return ""

    # Build markdown content
    lines = [
        "## 🚀 Preview Deployments",
        "",
        "|Deployment|Status|Preview URL|",
        "|---|---|---|",
    ]

    # Process each job in needs
    for job_name in needs.keys():
        job_data = needs[job_name]

        result = job_data.get("result", "unknown")
        preview_url = job_data.get("outputs", {}).get("preview-url", "")

        deployment_name = job_name.replace("_", " ")
        status_emoji = get_status_emoji(result)
        url_formatted = format_url(preview_url)

        lines.append(f"| {deployment_name} | {status_emoji} | {url_formatted} |")

    return "\n".join(lines)


def main():
    """Main function to generate report from environment variables."""
    needs_json = os.getenv("NEEDS_JSON", "{}")

    content = generate_report(needs_json)

    # Write to body.md file
    with open("body.md", "w") as f:
        f.write(content)

    # Debug output
    print("Generated comment:")
    print(content)


if __name__ == "__main__":
    main()
