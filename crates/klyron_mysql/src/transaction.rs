use crate::error::Result;

pub struct Transaction<'a> {
    tx: sqlx::Transaction<'a, sqlx::MySql>,
}

impl<'a> Transaction<'a> {
    pub async fn begin(pool: &sqlx::mysql::MySqlPool) -> Result<Self> {
        let tx = pool.begin().await?;
        Ok(Self { tx })
    }

    pub async fn commit(self) -> Result<()> {
        self.tx.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> Result<()> {
        self.tx.rollback().await?;
        Ok(())
    }

    #[inline]
    pub async fn execute(&mut self, sql: &str) -> Result<u64> {
        Ok(sqlx::query(sql).execute(&mut *self.tx).await?.rows_affected())
    }

    #[inline]
    pub async fn query(&mut self, sql: &str) -> Result<Vec<crate::row::Row>> {
        let rows = sqlx::query(sql).fetch_all(&mut *self.tx).await?;
        Ok(rows.iter().map(crate::row::mysql_row_to_row).collect())
    }

    pub fn inner(&mut self) -> &mut sqlx::Transaction<'a, sqlx::MySql> {
        &mut self.tx
    }
}
