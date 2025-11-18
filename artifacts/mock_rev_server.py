#!/usr/bin/env uv run
# /// script
# requires-python = ">=3.8"
# dependencies = [
#     "flask>=3.0.0",
#     "rich>=13.0.0",
# ]
# ///

"""
Mock Reverse Server - Flask echo server for testing webhook relays
Displays received requests with rich formatting

Single endpoint / that supports GET and POST methods.
"""

from flask import Flask, request, jsonify
from rich.console import Console
from rich.panel import Panel
from rich.table import Table
from rich.syntax import Syntax
from rich.json import JSON
from rich import print as rprint
from datetime import datetime
import json

app = Flask(__name__)
console = Console()


@app.route("/", methods=["GET", "POST"])
def echo():
    """Echo endpoint - displays received request data with rich formatting"""
    timestamp = datetime.now().isoformat()

    # Create a table for request info
    table = Table(title="📨 Incoming Request", show_header=True)
    table.add_column("Property", style="cyan", no_wrap=True)
    table.add_column("Value", style="magenta")

    # Request details
    table.add_row("Timestamp", timestamp)
    table.add_row("Method", request.method)
    table.add_row("URL", request.url)
    table.add_row("Path", request.path)
    table.add_row("Remote Address", request.remote_addr)
    table.add_row("Content Type", request.content_type or "N/A")
    table.add_row("Content Length", str(request.content_length or 0))

    # Headers table
    headers_table = Table(title="📋 Request Headers")
    headers_table.add_column("Header", style="green")
    headers_table.add_column("Value", style="yellow")

    for header, value in sorted(request.headers):
        headers_table.add_row(header, value)

    # Body/data
    body_content = ""
    body_data = None

    try:
        if request.method == "POST":
            if request.is_json:
                body_data = request.get_json()
                body_content = json.dumps(body_data, indent=2, ensure_ascii=False)
            else:
                body_data = request.get_data(as_text=True)
                body_content = body_data
    except Exception as e:
        body_content = str(e)

    # Print all sections
    console.print(table)
    console.print(headers_table)

    # Print body with syntax highlighting if JSON
    if body_data:
        if isinstance(body_data, dict):
            console.print(Panel("JSON Body", style="bold blue"))
            console.print(JSON.from_data(body_data))
        else:
            console.print(
                Panel(
                    body_content[:500] + ("..." if len(body_content) > 500 else ""),
                    style="red",
                )
            )
    elif request.method == "GET":
        console.print(Panel("GET request - no body", style="bold green"))

    # Separator
    console.print("\n" + "=" * 80 + "\n")

    # Response
    response_data = {
        "status": "success",
        "message": "Request received and echoed",
        "timestamp": timestamp,
        "received": {
            "method": request.method,
            "path": request.path,
            "content_type": request.content_type,
            "content_length": request.content_length,
        },
    }

    return jsonify(response_data), 200


if __name__ == "__main__":
    # Header
    console.print(
        Panel(
            "[bold green]🚀 Mock Reverse Server Starting[/]\n"
            "[dim]Flask echo server for testing webhook relays[/]",
            style="bold blue",
        )
    )

    console.print("\n[bold yellow]Available endpoints:[/]")
    console.print("  [cyan]GET  /[/cyan]  - Echo request data")
    console.print("  [cyan]POST /[/cyan]  - Echo webhook data\n")

    console.print("[bold yellow]Example usage:[/]")
    console.print("  [dim]curl -X POST http://localhost:5000/ \\[/dim]")
    console.print("[dim]    -H 'Content-Type: application/json' \\[/dim]")
    console.print('[dim]    -d \'{"test": "data"}\'[/dim]\n')

    console.print("[bold red]⚠️  Press Ctrl+C to stop[/]\n")
    console.print("=" * 80 + "\n")

    # Run server
    app.run(host="0.0.0.0", port=5000, debug=False)
