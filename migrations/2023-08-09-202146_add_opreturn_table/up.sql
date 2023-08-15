CREATE TABLE blocks (
    height INTEGER PRIMARY KEY NOT NULL,
    version INTEGER NOT NULL,
    time INTEGER NOT NULL,
    encoded_target INTEGER NOT NULL,
    nonce BIGINT NOT NULL,
    tx_count INTEGER NOT NULL,
    size INTEGER NOT NULL,
    weight BIGINT NOT NULL,
    turnover BIGINT NOT NULL,
    miner_reward BIGINT NOT NULL
);
