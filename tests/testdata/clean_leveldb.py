from pathlib import Path
import requests
import plyvel

LEVEL_DB_PATH = "./testnet3/index"
MAX_BLOCK_HEIGHT = 120
API_ENDPOINT = "https://blockstream.info/testnet/api/"

def get_blockhash(height: int) -> str:
    return requests.get(f"{API_ENDPOINT}block-height/{height}").text.strip()

def make_prefix_copy(db: plyvel.DB) -> None:
    path = Path("./pruned-db")
    path.mkdir()
    db2 = plyvel.DB(str(path), create_if_missing=True, error_if_exists=True)
    for height in range(MAX_BLOCK_HEIGHT + 1):
        block_hash = get_blockhash(height)
        key = b'b' + bytes.fromhex(block_hash)[::-1]
        db2.put(key, db.get(key))
        if height % 10 == 0:
            print(f"processed height {height} of {MAX_BLOCK_HEIGHT}")
    db2.close()

if __name__ == "__main__":
    db = plyvel.DB(LEVEL_DB_PATH)
    make_prefix_copy(db)
    db.close()
