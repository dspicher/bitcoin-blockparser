// @generated automatically by Diesel CLI.

diesel::table! {
    opreturns (id) {
        id -> Integer,
        height -> Integer,
        txid -> Text,
        vout -> Integer,
        message -> Text,
    }
}
