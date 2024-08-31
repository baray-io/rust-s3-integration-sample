use std::io::Read;

use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject, Upload};

use uuid::Uuid;

use crate::utils::get_bucket;

pub type FilesSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

#[derive(Clone, SimpleObject)]
pub struct FileInfo {
    id: String,
    key: String,
    url: String,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn get_file_url(&self, _ctx: &Context<'_>, key: String) -> String {
        let bucket = get_bucket().await;
        let url = bucket.presign_get(&key, 3600, None).await.unwrap();
        url
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn single_upload(&self, ctx: &Context<'_>, file: Upload) -> FileInfo {
        let bucket = get_bucket().await;
        let id = Uuid::new_v4().to_string();
        let key = format!("/uploads/{}", &id);
        let mut upload = file.value(ctx).unwrap();
        let mut buffer = Vec::new();

        upload.content.read_to_end(&mut buffer).unwrap();
        bucket.put_object(&key, &buffer).await.unwrap();

        let url = bucket.presign_get(&key, 3600, None).await.unwrap();

        let info = FileInfo { id, key, url };
        info
    }
}
