CREATE TABLE opreturns (
    id INTEGER PRIMARY KEY NOT NULL,
    height INTEGER NOT NULL,
    txid TEXT NOT NULL,
    vout INTEGER NOT NULL,
    message TEXT NOT NULL
);