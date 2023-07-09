// @generated automatically by Diesel CLI.

diesel::table! {
    link_history (id) {
        id -> Bigint,
        #[max_length = 4000]
        origin_url -> Varchar,
        link_type -> Nullable<Integer>,
        expire_date -> Nullable<Datetime>,
        active -> Bool,
        #[max_length = 48]
        link_hash -> Varchar,
        create_date -> Nullable<Datetime>,
        update_date -> Nullable<Datetime>,
    }
}
