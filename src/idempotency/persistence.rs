use super::IdempotencyKey;
use sqlx::postgres::PgHasArrayType;
use sqlx::PgPool;
use tide::{Response, StatusCode};
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}
impl PgHasArrayType for HeaderPairRecord {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_header_pair")
    }
}

pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
) -> Result<Option<Response>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
        SELECT
            response_status_code,
            response_headers as "response_headers: Vec<HeaderPairRecord>",
            response_body
        FROM idempotency
        WHERE
            user_id = $1 AND
            idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = saved_response {
        let status_code = StatusCode::try_from(r.response_status_code as u16)
            .map_err(|_| anyhow::anyhow!("invalid status code saved in database."))?;
        let mut response = Response::new(status_code);
        for HeaderPairRecord { name, value } in r.response_headers {
            response.append_header(name.as_str(), String::from_utf8_lossy(&value));
        }
        response.set_body(r.response_body);
        Ok(Some(response))
    } else {
        Ok(None)
    }
}

pub async fn save_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    mut http_response: Response,
) -> Result<Response, anyhow::Error> {
    let status_code = http_response.status() as i16;
    let headers = {
        let mut h = Vec::new();
        for name in http_response.header_names() {
            let val = http_response
                .header(name)
                .expect("the relative value must exists.");
            let name = name.as_str().to_owned();
            let value = val.as_str().as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value })
        }
        h
    };
    // note from zero to production in rust, why we need `http_response` ownership.
    // Pulling a chunk of data from the payload stream requires a mutable reference to the stream itself - once the chunk has been read, there is no way to “replay” the stream and read it again.
    // There is a common pattern to work around this:
    // • Getownershipofthebodyvia.into_parts();
    // • Bufferthewholebodyinmemoryviato_bytes;
    // • Dowhateveryouhavetodowiththebody;
    // • Re-assembletheresponseusing.set_body()ontherequesthead.
    let body = http_response
        .take_body()
        .into_bytes()
        .await
        .expect("the given response body should always be able to convert to bytes.");

    sqlx::query_unchecked!(
        r#"
            INSERT INTO idempotency (
                user_id,
                idempotency_key,
                response_status_code,
                response_headers,
                response_body,
                created_at
    )
    VALUES ($1, $2, $3, $4, $5, now()) "#,
        user_id,
        idempotency_key.as_ref(),
        status_code,
        headers,
        body
    )
    .execute(pool)
    .await?;

    http_response.set_body(body);
    Ok(http_response)
}
