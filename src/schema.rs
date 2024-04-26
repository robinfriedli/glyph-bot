// @generated automatically by Diesel CLI.

diesel::table! {
    aiode_supporter (user_id) {
        user_id -> Numeric,
        creation_timestamp -> Timestamptz,
    }
}
