use crate::utils::get_bucket;
use actix_multipart::Multipart;
use actix_web::{
    get, post,
    web::{self, Buf as _, BytesMut},
    HttpResponse, Responder,
};
use futures_util::{StreamExt as _, TryStreamExt as _};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

const MAX_FILE_SIZE: usize = 250 * 1024 * 1024; // 250 MB

#[derive(Deserialize)]
struct Info {
    key: String,
}

// http://locahost:8080/files?key=/uploads/4bc274de-3fee-44f9-bed5-330432922ab2

#[get("/files")]
pub async fn get_file(query: web::Query<Info>) -> impl Responder {
    let bucket = get_bucket().await;

    let url = bucket
        .presign_get(query.key.to_string(), 3600, None)
        .await
        .unwrap();

    HttpResponse::Ok().json(json!({
      "key": query.key.to_owned(),
      "url": url
    }))
}

// http://locahost:8080/uploads

#[post("/uploads")]
pub async fn upload(mut payload: Multipart) -> impl Responder {
    // Set up S3 bucket
    let bucket = get_bucket().await;

    let mut url = String::new();
    let uuid = Uuid::new_v4().to_string();
    let s3_key = format!("/uploads/{}", uuid);
    // Iterate over multipart stream
    while let Ok(Some(mut field)) = payload.try_next().await {
        // Begin multipart upload
        let upload_id = bucket
            .initiate_multipart_upload(&s3_key, "application/octet-stream")
            .await
            .unwrap();

        let mut part_number = 1;
        let mut etags = Vec::new();
        let mut buffer = BytesMut::with_capacity(MAX_FILE_SIZE);

        // Read file chunks and upload to S3
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            buffer.extend_from_slice(&data);
            if buffer.len() >= MAX_FILE_SIZE {
                // Upload the accumulated buffer as a part
                let etag = bucket
                    .put_multipart_stream(
                        &mut data.reader(),
                        &s3_key,
                        part_number,
                        &upload_id.upload_id,
                        "application/octet-stream",
                    )
                    .await
                    .unwrap();
                etags.push(etag);
                buffer.clear();
                part_number += 1;
            }

            part_number += 1;
        }

        if !buffer.is_empty() {
            let etag = bucket
                .put_multipart_stream(
                    &mut buffer.reader(),
                    &s3_key,
                    part_number,
                    &upload_id.upload_id,
                    "application/octet-stream",
                )
                .await
                .unwrap();

            etags.push(etag);
        }

        // Complete multipart upload
        bucket
            .complete_multipart_upload(&s3_key, &upload_id.upload_id, etags)
            .await
            .unwrap();

        let expiration = Duration::from_secs(3600);

        let presigned_url = bucket
            .presign_get(&s3_key, expiration.as_secs() as u32, None)
            .await
            .unwrap();

        url = presigned_url;
    }

    HttpResponse::Ok().json(json!({
      "key": s3_key,
      "url": url
    }))
}
