import radix_engine_toolkit as ret

def encode_prices(prices) -> bytes:
    elements = [f'{{"kind":"Tuple","fields":[' +
                    f'{{"kind":"String","value":"{price["pair"]}"}},' +
                    f'{{"kind":"Decimal","value":"{price["quote"]}"}},' +
                    f'{{"kind":"I64","value":"{price["timestamp"]}"}}' +
                f']}}' for price in prices]
    
    sbor_string = f'{{"kind":"Array","element_kind":"Tuple","elements":[{','.join(elements)}]}}'
    sbor_programmatic_json = ret.ScryptoSborString.PROGRAMMATIC_JSON(sbor_string)
    sbor_bytes = ret.scrypto_sbor_encode_string_representation(sbor_programmatic_json)

    return sbor_bytes


# EXAMPLE
prices= [
   {
     "pair": "BTC/USD",
     "quote": 63457.174703230005,
     "timestamp": "1713207309"
   },
    {
      "pair": "ETH/USD",
      "quote": 4295.0,
      "timestamp": "1713207309"
    },
    {
      "pair": "SOL/USD",
      "quote": 200.0,
      "timestamp": "1713207309"
    }
 ]
sbor_bytes = encode_prices(prices)

# --- hash & sign bytes ---

# output bytes as hex
print(sbor_bytes.hex())