import os
from typing import Literal

import httpx
from mcp.server.fastmcp import FastMCP

mcp = FastMCP("web_search")

Freshness = Literal["noLimit", "oneDay", "oneWeek", "oneMonth", "oneYear"]


@mcp.tool()
async def web_search(query: str, freshness: Freshness = "noLimit") -> str:
    async with httpx.AsyncClient() as client:
        res = await client.post(
            "https://api.langsearch.com/v1/web-search",
            json={"query": query, "freshness": freshness},
            headers={"Authorization": f"Bearer {os.environ['LANGSEARCH_API_KEY']}"},
            timeout=30.0,
        )
        items = res.json()["data"]["webPages"]["value"]
        return "\n".join(
            f"{i['name']} {i['url'].removeprefix('https://').removeprefix('http://').rstrip('/')}"
            for i in items
        )


if __name__ == "__main__":
    mcp.run(transport="stdio")
