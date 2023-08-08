from pathlib import Path
import requests
import plyvel

LEVEL_DB_PATH = "./signet/index"
MAX_BLOCK_HEIGHT = 2

def get_blockhash(height: int) -> str:
    return [
        "00000008819873e925422c1ff0f99f7cc9bbb232af63a077a480a3633bee1ef6",
        "00000086d6b2636cb2a392d45edc4ec544a10024d30141c9adf4bfd9de533b53",
        "00000032bb881de703dcc968e8258080c7ed4a2933e3a35888fa0b2f75f36029",
            ][height]

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
