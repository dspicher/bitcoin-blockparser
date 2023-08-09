// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (height) {
        height -> Integer,
        version -> Integer,
        time -> Integer,
        encoded_target -> Integer,
        nonce -> BigInt,
        tx_count -> Integer,
        size -> Integer,
        weight -> BigInt,
    }
}
