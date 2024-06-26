import qrcode
import asyncio
import json
from aiohttp import ClientSession

async def get_feeds(session, pair_ids) -> dict:
    feeds = {}
    headers = {
        'Content-Type': 'application/json',
    }
    async with session.get(
        f'https://hermes.pyth.network/v2/price_feeds',
        headers=headers) as response:
    
        data = await response.json()
        for pair_id in pair_ids:
            for feed in data:
                if pair_id == feed['attributes']['symbol'][7:]:
                    feeds[pair_id] = feed
                    break

    return feeds

async def get_prices(session, pair_ids) -> dict:
    feeds = await get_feeds(session, pair_ids)
    feed_ids = {feed['id']:pair_id for pair_id, feed in feeds.items()}

    prices = {}
    headers = {
        'Content-Type': 'application/json',
    }
    query = '?' + '&'.join([f'ids[]={feed_id}' for feed_id in feed_ids.keys()])
    async with session.get(
        f'https://hermes.pyth.network/v2/updates/price/latest{query}',
        headers=headers) as response:
    
        data = await response.json()
        for update in data['parsed']:
            pair_id = feed_ids[update['id']]
            sig = int(update['price']['price'])
            exp = int(update['price']['expo'])
            price = sig * 10 ** exp
            prices[pair_id] = price

    return prices