use chrono::Utc;
use sqlx::PgPool;

pub async fn upsert_user(
    pool: &PgPool,
    github_id: &str,
    email: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (github_id, last_login, auth_type, email)
        VALUES ($1, $2, 'github', $3)
        ON CONFLICT (github_id)
        DO UPDATE SET
            last_login = EXCLUDED.last_login,
            email = COALESCE(EXCLUDED.email, users.email)
        "#,
        github_id,
        Utc::now(),
        email
    )
    .execute(pool)
    .await?;

    Ok(())
}
