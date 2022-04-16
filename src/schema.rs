table! {
    http_download (id) {
        id -> Integer,
        uri -> Text,
        progress -> Float,
        path -> Text,
    }
}

table! {
    sub_http_download (id) {
        id -> Integer,
        parent_id -> Integer,
        offset -> Integer,
        uri -> Text,
        progress -> Float,
    }
}

joinable!(sub_http_download -> http_download (parent_id));

allow_tables_to_appear_in_same_query!(
    http_download,
    sub_http_download,
);
