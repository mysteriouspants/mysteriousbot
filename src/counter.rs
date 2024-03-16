use r2d2::{Error as R2d2Error, Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Error as RusqliteError, OptionalExtension};
use serenity::model::id::UserId;
use snafu::{ResultExt, Snafu};
use std::{borrow::Cow, ops::Deref};

#[derive(Debug, Snafu)]
pub enum Error {
    Pool { source: R2d2Error },
    Db { source: RusqliteError },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct CounterFactory {
    pool: Pool<SqliteConnectionManager>,
}

impl CounterFactory {
    pub fn new(pool: Pool<SqliteConnectionManager>) -> Result<Self> {
        let connection = pool.get().context(PoolSnafu)?;

        // create the tracking table if it doesn't exist already;
        // figuring out how to version and migrate this is a problem
        // for future mysteriouspants. screw that guy.
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS \
                    counters ( \
                        id INTEGER PRIMARY KEY AUTOINCREMENT, \
                        counter TEXT NOT NULL, \
                        user_id INTEGER(64) NOT NULL, \
                        count INTEGER NOT NULL DEFAULT 0, \
                        CONSTRAINT \
                            one_count_per_user UNIQUE (counter, user_id) \
                                ON CONFLICT ROLLBACK);",
                [],
            )
            .context(DbSnafu)?;

        Ok(Self { pool })
    }

    pub fn make_counter(&self, counter_id: &str) -> Counter {
        Counter {
            pool: self.pool.clone(),
            counter_id: Cow::Owned(counter_id.to_owned()),
        }
    }
}

#[derive(Debug)]
pub struct Counter {
    pool: Pool<SqliteConnectionManager>,
    counter_id: Cow<'static, String>,
}

impl Counter {
    /// Gets the counter for a given subject.
    #[allow(dead_code)]
    pub fn get(&self, subject: UserId) -> Result<u64> {
        self.get_count(self.get_connection()?.deref(), subject)
    }

    /// Increments the counter for a given subject, returning the new
    /// value in the counter.
    pub fn increment(&self, subject: UserId) -> Result<u64> {
        let mut connection = self.get_connection()?;
        let tx = connection.transaction().context(DbSnafu)?;
        let mut count = self.get_count(&tx, subject)?;

        count += 1;
        self.set_count(&tx, subject, count)?;
        tx.commit().context(DbSnafu)?;

        Ok(count)
    }

    /// Decrements the counter for a given subject, returning the new
    /// value in the counter.
    #[allow(dead_code)]
    pub fn decrement(&self, subject: UserId) -> Result<u64> {
        let mut connection = self.get_connection()?;
        let tx = connection.transaction().context(DbSnafu)?;
        let mut count = self.get_count(&tx, subject)?;

        #[cfg(test)]
        assert_ne!(0, count);

        count -= 1;
        self.set_count(&tx, subject, count)?;
        tx.commit().context(DbSnafu)?;

        Ok(count)
    }

    /// Sets the counter for a given subject.
    #[allow(dead_code)]
    pub fn set(&self, subject: UserId, count: u64) -> Result<()> {
        let connection = self.get_connection()?;
        self.set_count(&connection, subject, count)?;
        Ok(())
    }

    /// The top counts for this counter.
    pub fn top_counts(&self, subject: UserId) -> Result<Vec<(UserId, u64)>> {
        let mut connection = self.get_connection()?;
        let tx = connection.transaction().context(DbSnafu)?;

        // get the top ten
        let mut top_counts = {
            let mut select = tx
                .prepare(
                    "SELECT user_id, count FROM counters \
                WHERE counter = ? \
                ORDER BY count DESC \
                LIMIT 10;",
                )
                .context(DbSnafu)?;
            let rows = select
                .query_map([&self.counter_id], |row| {
                    Ok((UserId::new(row.get(0)?), row.get::<_, u64>(1)?))
                })
                .context(DbSnafu)?;

            let mut top_counts = vec![];

            for row in rows {
                top_counts.push(row.context(DbSnafu)?);
            }

            top_counts
        };

        // if the subject isn't in the top ten, at least let them know
        // where they stand
        if !top_counts.iter().any(|tuple| tuple.0 == subject) {
            top_counts.pop();
            top_counts.push((subject, self.get_count(&tx, subject)?));
        }

        tx.commit().context(DbSnafu)?;

        Ok(top_counts)
    }

    /// Gets the count on a given connection for a subject.
    fn get_count(&self, connection: &Connection, subject: UserId) -> Result<u64> {
        Ok(
            match connection
                .query_row(
                    "SELECT count FROM counters \
                        WHERE counter = ? AND user_id = ? LIMIT 1;",
                    params![&self.counter_id, subject.get()],
                    |row| row.get(0),
                )
                .optional()
                .context(DbSnafu)?
            {
                Some(count) => count,
                None => 0,
            },
        )
    }

    /// Sets the count on a given connection for a subject.
    fn set_count(&self, connection: &Connection, subject: UserId, count: u64) -> Result<usize> {
        let rows_affected = connection
            .execute(
                "INSERT INTO counters (counter, user_id, count) \
                    VALUES(?, ?, ?) \
                    ON CONFLICT(counter, user_id) \
                    DO UPDATE SET count = excluded.count;",
                params![&self.counter_id, subject.get(), count],
            )
            .context(DbSnafu)?;

        #[cfg(test)]
        assert_eq!(1, rows_affected);

        Ok(rows_affected)
    }

    fn get_connection(&self) -> Result<PooledConnection<SqliteConnectionManager>> {
        Ok(self.pool.get().context(PoolSnafu)?)
    }
}

#[cfg(test)]
mod tests {
    use r2d2::Pool;
    use r2d2_sqlite::SqliteConnectionManager;
    use serenity::model::id::UserId;

    use super::CounterFactory;

    fn memory_pool() -> Pool<SqliteConnectionManager> {
        Pool::new(SqliteConnectionManager::memory()).unwrap()
    }

    #[test]
    fn counting() {
        let pool = memory_pool();
        let counter_factory = CounterFactory::new(pool).unwrap();
        let counter = counter_factory.make_counter("my_counter");
        let joe = UserId::new(1);

        assert!(matches!(counter.get(joe), Ok(0)));
        assert!(matches!(counter.increment(joe), Ok(1)));
        assert!(matches!(counter.decrement(joe), Ok(0)));
        assert!(matches!(counter.increment(joe), Ok(1)));

        let bob = UserId::new(2);

        assert!(matches!(counter.get(bob), Ok(0)));
        assert!(matches!(counter.increment(bob), Ok(1)));
        assert!(matches!(counter.decrement(bob), Ok(0)));

        assert!(matches!(counter.get(joe), Ok(1)));
    }
}
