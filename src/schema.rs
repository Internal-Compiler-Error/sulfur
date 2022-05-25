table! {
    file_path (path) {
        path -> Text,
    }
}

table! {
    http_subdownload (id) {
        id -> Integer,
        url -> Text,
        offset -> Integer,
        total -> Integer,
        file_path -> Text,
    }
}

table! {
    url (full_text) {
        full_text -> Text,
    }
}

joinable!(http_subdownload -> file_path (file_path));
joinable!(http_subdownload -> url (url));

allow_tables_to_appear_in_same_query!(
    file_path,
    http_subdownload,
    url,
);
