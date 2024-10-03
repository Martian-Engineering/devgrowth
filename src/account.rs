use sqlx::PgPool;

pub async fn upsert_account(
    pool: &PgPool,
    github_id: &str,
    email: Option<&str>,
) -> Result<i32, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO account (github_id, email, last_login)
        VALUES ($1, $2, NOW())
        ON CONFLICT (github_id)
        DO UPDATE SET
            last_login = NOW(),
            email = COALESCE(EXCLUDED.email, account.email)
        RETURNING account_id
        "#,
        github_id,
        email
    )
    .fetch_one(pool)
    .await?;

    Ok(result.account_id)
}
