use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Postgres, QueryBuilder};
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

#[derive(sqlx::FromRow, Serialize)]
pub struct MAUGrowthAccountingResult {
    month: DateTime<Utc>,
    mau: i64,
    retained: i64,
    new: i64,
    resurrected: i64,
    churned: i64,
}

const BASE: &str = r#"
    wau AS (
        SELECT
            date_trunc('week',
                dt) AS week,
            user_id,
            sum(inc_amt) AS inc_amt
        FROM
            dau
        GROUP BY
            1,
            2
    ),
    mau AS (
        SELECT
            date_trunc('month',
                dt) AS month,
            user_id,
            sum(inc_amt) AS inc_amt
        FROM
            dau
        GROUP BY
            1,
            2
    ),
    -- This determines the cohort date of each user. In this case we are
    -- deriving it from DAU data but you can feel free to replace it with
    -- registration date if that's more appropriate.
    first_dt AS (
        SELECT
            user_id,
            min(dt) AS first_dt,
            date_trunc('week',
                min(dt)) AS first_week,
            date_trunc('month',
                min(dt)) AS first_month
        FROM
            dau
        GROUP BY
            1
    ),
    mau_decorated AS (
        SELECT
            d.month,
            d.user_id,
            d.inc_amt,
            f.first_month
        FROM
            mau d,
            first_dt f
        WHERE
            d.user_id = f.user_id
            AND inc_amt > 0
    ),
    "#;

// -- This is MAU growth accounting. Note that this does not require any
// -- information about inc_amt. As discussed in the articles, these
// -- quantities satisfy some identities:
// -- MAU(t) = retained(t) + new(t) + resurrected(t)
// -- MAU(t - 1 month) = retained(t) + churned(t)
const MAU_GROWTH_ACCOUNTING: &str = r#"
    mau_growth_accounting AS (
        SELECT
            coalesce(tm.month,
                lm.month + interval '1 month') AS month,
            count(DISTINCT tm.user_id) AS mau,
            count(DISTINCT CASE WHEN lm.user_id IS NOT NULL THEN
                    tm.user_id
                ELSE
                    NULL
                END) AS retained,
            count(DISTINCT CASE WHEN tm.first_month = tm.month THEN
                    tm.user_id
                ELSE
                    NULL
                END) AS new,
            count(DISTINCT CASE WHEN tm.first_month != tm.month
                    AND lm.user_id IS NULL THEN
                    tm.user_id
                ELSE
                    NULL
                END) AS resurrected,
            - 1 * count(DISTINCT CASE WHEN tm.user_id IS NULL THEN
                    lm.user_id
                ELSE
                    NULL
                END) AS churned
        FROM
            mau_decorated tm
        FULL OUTER JOIN mau_decorated lm ON (tm.user_id = lm.user_id
            AND tm.month = lm.month + interval '1 month')
    GROUP BY
        1
    ORDER BY
        1
    )
    SELECT
        month,
        mau,
        retained,
        new,
        resurrected,
        churned
    FROM
        mau_growth_accounting
    ORDER BY
        month ASC;
"#;

pub async fn mau_growth_accounting(
    pool: &PgPool,
    dau: String,
) -> Result<Vec<MAUGrowthAccountingResult>, sqlx::Error> {
    let q = format!(
        r#"
        WITH dau AS (
            {}
        ),
        {}
        {}
        "#,
        dau, BASE, MAU_GROWTH_ACCOUNTING
    );

    match sqlx::query_as::<_, MAUGrowthAccountingResult>(&q)
        .fetch_all(pool)
        .await
    {
        Ok(results) => Ok(results),
        Err(e) => {
            log::error!("Error fetching MAU growth accounting results: {}", e);
            Err(e)
        }
    }
}

const QUERY: &str = r#"
    WITH dau AS (
        SELECT
            author AS user_id,
            date_trunc('day',
                "date") AS dt,
            count(*) AS inc_amt
        FROM
            "commit"
        WHERE
            repository_id = $1
        GROUP BY
            1,
            2
    ),
    -- First, set up WAU and MAU tables for future use
    wau AS (
        SELECT
            date_trunc('week',
                dt) AS week,
            user_id,
            sum(inc_amt) AS inc_amt
        FROM
            dau
        GROUP BY
            1,
            2
    ),
    mau AS (
        SELECT
            date_trunc('month',
                dt) AS month,
            user_id,
            sum(inc_amt) AS inc_amt
        FROM
            dau
        GROUP BY
            1,
            2
    ),
    -- This determines the cohort date of each user. In this case we are
    -- deriving it from DAU data but you can feel free to replace it with
    -- registration date if that's more appropriate.
    first_dt AS (
        SELECT
            user_id,
            min(dt) AS first_dt,
            date_trunc('week',
                min(dt)) AS first_week,
            date_trunc('month',
                min(dt)) AS first_month
        FROM
            dau
        GROUP BY
            1
    ),
    mau_decorated AS (
        SELECT
            d.month,
            d.user_id,
            d.inc_amt,
            f.first_month
        FROM
            mau d,
            first_dt f
        WHERE
            d.user_id = f.user_id
            AND inc_amt > 0
    ),
    -- This is MAU growth accounting. Note that this does not require any
    -- information about inc_amt. As discussed in the articles, these
    -- quantities satisfy some identities:
    -- MAU(t) = retained(t) + new(t) + resurrected(t)
    -- MAU(t - 1 month) = retained(t) + churned(t)
    mau_growth_accounting AS (
        SELECT
            coalesce(tm.month,
                lm.month + interval '1 month') AS month,
            count(DISTINCT tm.user_id) AS mau,
            count(DISTINCT CASE WHEN lm.user_id IS NOT NULL THEN
                    tm.user_id
                ELSE
                    NULL
                END) AS retained,
            count(DISTINCT CASE WHEN tm.first_month = tm.month THEN
                    tm.user_id
                ELSE
                    NULL
                END) AS new,
            count(DISTINCT CASE WHEN tm.first_month != tm.month
                    AND lm.user_id IS NULL THEN
                    tm.user_id
                ELSE
                    NULL
                END) AS resurrected,
            - 1 * count(DISTINCT CASE WHEN tm.user_id IS NULL THEN
                    lm.user_id
                ELSE
                    NULL
                END) AS churned
        FROM
            mau_decorated tm
        FULL OUTER JOIN mau_decorated lm ON (tm.user_id = lm.user_id
            AND tm.month = lm.month + interval '1 month')
    GROUP BY
        1
    ORDER BY
        1
    ),
    -- This generates the familiar monthly cohort retention dataset.
    mau_retention_by_cohort AS (
        SELECT
            first_month,
            12 * extract(year FROM age(month,
                    first_month)) + extract(month FROM age(month,
                    first_month)) AS months_since_first,
            count(1) AS active_users,
            sum(inc_amt) AS inc_amt
        FROM
            mau_decorated
        GROUP BY
            1,
            2
        ORDER BY
            1,
            2
    ),
    -- This is the MRR growth accounting (or growth accounting of whatever
    -- value you put in inc_amt). These also satisfy some identities:
    -- MRR(t) = retained(t) + new(t) + resurrected(t) + expansion(t)
    -- MAU(t - 1 month) = retained(t) + churned(t) + contraction(t)
    mrr_growth_accounting AS (
        SELECT
            coalesce(tm.month,
                lm.month + interval '1 month') AS month,
            sum(tm.inc_amt) AS rev,
            sum(
                CASE WHEN tm.user_id IS NOT NULL
                    AND lm.user_id IS NOT NULL
                    AND tm.inc_amt >= lm.inc_amt THEN
                    lm.inc_amt
                WHEN tm.user_id IS NOT NULL
                    AND lm.user_id IS NOT NULL
                    AND tm.inc_amt < lm.inc_amt THEN
                    tm.inc_amt
                ELSE
                    0
                END) AS retained,
            sum(
                CASE WHEN tm.first_month = tm.month THEN
                    tm.inc_amt
                ELSE
                    0
                END) AS new,
            sum(
                CASE WHEN tm.month != tm.first_month
                    AND tm.user_id IS NOT NULL
                    AND lm.user_id IS NOT NULL
                    AND tm.inc_amt > lm.inc_amt
                    AND lm.inc_amt > 0 THEN
                    tm.inc_amt - lm.inc_amt
                ELSE
                    0
                END) AS expansion,
            sum(
                CASE WHEN tm.user_id IS NOT NULL
                    and(lm.user_id IS NULL
                        OR lm.inc_amt = 0)
                    AND tm.inc_amt > 0
                    AND tm.first_month != tm.month THEN
                    tm.inc_amt
                ELSE
                    0
                END) AS resurrected,
            - 1 * sum(
                CASE WHEN tm.month != tm.first_month
                    AND tm.user_id IS NOT NULL
                    AND lm.user_id IS NOT NULL
                    AND tm.inc_amt < lm.inc_amt
                    AND tm.inc_amt > 0 THEN
                    lm.inc_amt - tm.inc_amt
                ELSE
                    0
                END) AS contraction,
            - 1 * sum(
                CASE WHEN lm.inc_amt > 0
                    and(tm.user_id IS NULL
                        OR tm.inc_amt = 0) THEN
                    lm.inc_amt
                ELSE
                    0
                END) AS churned
        FROM
            mau_decorated tm
        FULL OUTER JOIN mau_decorated lm ON (tm.user_id = lm.user_id
            AND tm.month = lm.month + interval '1 month')
    GROUP BY
        1
    ORDER BY
        1
    ),
    -- These next tables are to compute LTV via the cohorts_cumulative table.
    -- The LTV here is being computed for weekly cohorts on weekly intervals.
    -- The queries can be modified to compute it for cohorts of any size
    -- on any time window frequency.
    wau_decorated AS (
        SELECT
            week,
            w.user_id,
            w.inc_amt,
            f.first_week
        FROM
            wau w,
            first_dt f
    WHERE
        w.user_id = f.user_id
    ),
    cohorts AS (
        SELECT
            first_week,
            week AS active_week,
            ceil(extract(DAYS FROM (week - first_week)) / 7.0) AS weeks_since_first,
            count(DISTINCT user_id) AS users,
            sum(inc_amt) AS inc_amt
        FROM
            wau_decorated
        GROUP BY
            1,
            2,
            3
        ORDER BY
            1,
            2
    ),
    cohort_sizes AS (
        SELECT
            first_week,
            users,
            inc_amt
        FROM
            cohorts
        WHERE
            weeks_since_first = 0
    ),
    cohorts_cumulative AS (
        -- A semi-cartesian join accomplishes the cumulative behavior.
        SELECT
            c1.first_week,
            c1.active_week,
            c1.weeks_since_first,
            c1.users,
            cs.users AS cohort_num_users,
            1.0 * c1.users / cs.users AS retained_pctg,
            c1.inc_amt,
            sum(c2.inc_amt) AS cum_amt,
            1.0 * sum(c2.inc_amt) / cs.users AS cum_amt_per_user
        FROM
            cohorts c1,
            cohorts c2,
            cohort_sizes cs
        WHERE
            c1.first_week = c2.first_week
            AND c2.weeks_since_first <= c1.weeks_since_first
            AND cs.first_week = c1.first_week
        GROUP BY
            1,
            2,
            3,
            4,
            5,
            6,
            7
        ORDER BY
            1,
            2
    ),
    -- monthly cumulative cohorts
    cohorts_m AS (
        SELECT
            first_month,
            month AS active_month,
            extract(month FROM month) - extract(month FROM first_month) + 12 * (extract(year FROM month) - extract(year FROM first_month)) AS months_since_first,
            count(DISTINCT user_id) AS users,
            sum(inc_amt) AS inc_amt
        FROM
            mau_decorated
        GROUP BY
            1,
            2,
            3
        ORDER BY
            1,
            2
    ),
    cohort_sizes_m AS (
        SELECT
            first_month,
            users,
            inc_amt
        FROM
            cohorts_m
        WHERE
            months_since_first = 0
    ),
    cohorts_cumulative_m AS (
        -- A semi-cartesian join accomplishes the cumulative behavior.
        SELECT
            c1.first_month,
            c1.active_month,
            c1.months_since_first,
            c1.users,
            cs.users AS cohort_num_users,
            1.0 * c1.users / cs.users AS retained_pctg,
            c1.inc_amt,
            sum(c2.inc_amt) AS cum_amt,
            1.0 * sum(c2.inc_amt) / cs.users AS cum_amt_per_user
        FROM
            cohorts_m c1,
            cohorts_m c2,
            cohort_sizes_m cs
        WHERE
            c1.first_month = c2.first_month
            AND c2.months_since_first <= c1.months_since_first
            AND cs.first_month = c1.first_month
        GROUP BY
            1,
            2,
            3,
            4,
            5,
            6,
            7
        ORDER BY
            1,
            2
    )
    SELECT
        to_char("month", 'MM-DD-YYYY') AS date,
        mau,
        retained,
        new,
        resurrected,
        churned
    FROM
        mau_growth_accounting
    ORDER BY
        month ASC;
"#;
