table! {
    one_time_video (id) {
        id -> Int4,
        video_id -> Int4,
        created_at -> Timestamp,
        one_time_pass -> Text,
    }
}

table! {
    user_permissions (id) {
        id -> Int4,
        permission -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        user_id -> Text,
        email -> Text,
        displayname -> Text,
        permissions -> Array<Int4>,
    }
}

table! {
    videos (id) {
        id -> Int4,
        video_id -> Text,
        video_path -> Text,
        video_name -> Text,
        video_length -> Int4,
        video_desc -> Text,
        owner_id -> Int4,
        thumbnail_path -> Nullable<Text>,
    }
}

joinable!(one_time_video -> videos (video_id));
joinable!(videos -> users (owner_id));

allow_tables_to_appear_in_same_query!(
    one_time_video,
    user_permissions,
    users,
    videos,
);
