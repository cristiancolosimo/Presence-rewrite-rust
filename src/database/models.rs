use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use sqlx::SqlitePool;
use serde::{Serialize, Deserialize};

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct User {
    id: i32,
    username: String,
    password: String,
    email: String,
    #[sqlx(rename = "createdAt")]
    created_at: String,
    #[sqlx(rename = "updatedAt")]
    updated_at: String,
    enabled: bool
}
pub struct InsertUser {
    username: String,
    password: String,
    email: String,
    created_at: String,
    updated_at: String,
    enabled: bool
}

impl InsertUser {
    pub fn new(username: &str, password: &str, email: &str) -> Self {
        let now = chrono::Utc::now();
        InsertUser {
            username: username.to_string(),
            password: password.to_string(),
            email: email.to_string(),
            created_at: now.to_string(),
            updated_at: now.to_string(),
            enabled: true
        }
    }

    pub async fn save(&self, pool: &SqlitePool) -> Result<(),Box<dyn std::error::Error >> {
        let salt = SaltString::generate(&mut OsRng);
        
        let password = match Argon2::default().hash_password(self.password.as_bytes(), &salt) {
            Ok(p) => p.to_string(),
            Err(e) => return Err("Error hashing password".into())   
        };
        sqlx::query("INSERT INTO users (username,password,email,created_at,updated_at,enabled) VALUES (?,?,?,?,?,?)")
        .bind(&self.username)
        .bind(&password)
        .bind(&self.email)
        .bind(&self.created_at)
        .bind(&self.updated_at)
        .bind(&self.enabled)
        .execute(pool).await?;
        Ok(())
    }

}


impl User {
    pub fn check_password(&self, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(&self.password).unwrap();
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()

    }
    pub async fn login(pool: SqlitePool, username: &str, password: &str) -> Option<User> {
        let user = sqlx::query_as::<_,User>("SELECT * FROM User WHERE username = ? LIMIT 1")
        .bind(username)
        .fetch_one(&pool).await;
        if let Err(_) = user {
            return None;
        }
        let user = user.unwrap();
        if user.check_password(password) {
            return Some(user);
        }
        return None;
    }
}