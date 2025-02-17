use crate::*;

#[derive(Debug, thiserror::Error, ToSchema, Serialize)]
// XXX Fixme!
#[allow(dead_code)]
pub enum JokeBaseErr {
    #[error("joke already exists: {0}")]
    JokeExists(String),
    #[error("jokebase io failed: {0}")]
    JokeBaseIoError(String),
    #[error("no joke")]
    NoJoke,
    #[error("joke {0} doesn't exist")]
    JokeDoesNotExist(String),
    #[error("joke payload unprocessable: {0}")]
    JokeUnprocessable(String),
    #[error("database error: {0}")]
    DatabaseError(String),
}

impl From<std::io::Error> for JokeBaseErr {
    fn from(e: std::io::Error) -> Self {
        JokeBaseErr::JokeBaseIoError(e.to_string())
    }
}

impl From<sqlx::Error> for JokeBaseErr {
    fn from(e: sqlx::Error) -> Self {
        JokeBaseErr::DatabaseError(e.to_string())
    }
}

#[derive(Debug)]
pub struct JokeBaseError {
    pub status: StatusCode,
    pub error: JokeBaseErr,
}

impl<'s> ToSchema<'s> for JokeBaseError {
    fn schema() -> (&'s str, RefOr<Schema>) {
        let sch = ObjectBuilder::new()
            .property(
                "status",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .property(
                "error",
                ObjectBuilder::new().schema_type(SchemaType::String),
            )
            .example(Some(serde_json::json!({
                "status":"404","error":"no joke"
            })))
            .into();
        ("JokeBaseError", sch)
    }
}

impl Serialize for JokeBaseError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let status: String = self.status.to_string();
        let mut state = serializer.serialize_struct("JokeBaseError", 2)?;
        state.serialize_field("status", &status)?;
        state.serialize_field("error", &self.error)?;
        state.end()
    }
}

impl JokeBaseError {
    pub fn response(status: StatusCode, error: JokeBaseErr) -> Response {
        let error = JokeBaseError { status, error };
        (status, Json(error)).into_response()
    }
}

#[derive(Debug)]
pub struct JokeBase(pub Pool<Postgres>);

impl JokeBase {
    async fn to_joke(&self, row: &PgRow) -> Result<Joke, sqlx::Error> {
        let id = row.get("id");
        let tags = sqlx::query(r#"SELECT tag FROM tags WHERE id = $1"#)
            .bind(&id)
            .fetch_all(&self.0)
            .await?;
        let tags: HashSet<String> = tags
            .iter()
            .map(|row| row.get("tag"))
            .collect();
        let tags = if tags.is_empty() {
            None
        } else {
            Some(tags)
        };
        Ok(Joke {
            id,
            whos_there: row.get("whos_there"),
            answer_who: row.get("answer_who"),
            source: row.get("source"),
            tags,
        })
    }

    pub async fn new() -> Result<Self, Box<dyn Error>> {
        use std::env::var;
        
        let pwf = var("PG_PASSWORDFILE")?;
        let password = std::fs::read_to_string(pwf)?;
        let url = format!(
            "postgres://{}:{}@{}:5432/{}",
            var("PG_USER")?,
            password.trim(),
            var("PG_HOST")?,
            var("PG_DBNAME")?,
        );
        let pool = PgPool::connect(&url).await?;
        sqlx::migrate!()
            .run(&pool)
            .await?;
        Ok(JokeBase(pool))
    }

    pub async fn get_random(&self) -> Result<Joke, JokeBaseErr> {
        todo!()
    }

    pub async fn get<'a>(&self, _index: &str) -> Result<Joke, JokeBaseErr> {
        todo!()
    }

    pub async fn get_jokes<'a>(&self) -> Result<Vec<Joke>, JokeBaseErr> {
        let rows = sqlx::query("SELECT * FROM jokes;").fetch_all(&self.0).await?;
        let mut jokes: Vec<Joke> = Vec::with_capacity(rows.len());
        for j in rows.iter() {
            jokes.push(self.to_joke(j).await?);
        }
        Ok(jokes)
    }

    pub async fn add(&mut self, joke: Joke) -> Result<(), JokeBaseErr> {
        sqlx::query(
            r#"INSERT INTO jokes
            (id, whos_there, answer_who, source)
            VALUES ($1, $2, $3, $4);"#
        )
            .bind(&joke.id)
            .bind(&joke.whos_there)
            .bind(&joke.answer_who)
            .bind(&joke.source)
            .execute(&self.0)
            .await?;
        if let Some(tags) = &joke.tags {
            for tag in tags.iter() {
                sqlx::query(r#"INSERT INTO tags (id, tag) VALUES ($1, $2);"#)
                    .bind(&joke.id)
                    .bind(tag)
                    .execute(&self.0)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn delete(&mut self, _index: &str) -> Result<(), JokeBaseErr> {
        todo!()
    }

    pub async fn update(&mut self, _index: &str, _joke: Joke) -> Result<(), JokeBaseErr> {
        todo!()
    }
}
