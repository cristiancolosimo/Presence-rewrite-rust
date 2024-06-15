use sqlx::SqlitePool;

pub const INIT_SCRIPT: &str = include_str!("00_init.sql");

async fn connect_to_db() -> Result<SqlitePool,sqlx::Error >{
    SqlitePool::connect("sqlite:./database.db").await
}
fn create_db_file(db_file: &str){
    if !std::path::Path::new(db_file).exists() {
        println!("Database file not found, creating new database file");
        match std::fs::File::create(db_file) {
            Ok(_) => println!("Database file created"),
            Err(e) => panic!("Error creating database file: {}",e)
        }
    }
}

pub async fn get_database() -> SqlitePool  {
    println!("Initializing database");
    //check if file database.db exists
    create_db_file("./database.db");
    
    let db_pool = match connect_to_db().await {
        Ok(pool) => pool,
        Err(e) => panic!("Error connecting to database: {}",e)
    };
    
    match sqlx::query(INIT_SCRIPT).execute(&db_pool).await {
        Ok(_) => println!("Database initialized"),
        Err(e) => panic!("Error initializing database: {}",e)
    };

    db_pool
}


pub async fn get_session_database() -> tower_sessions_sqlx_store::SqliteStore {
    println!("Initializing session database");
    //check if file session.db exists
    create_db_file("./session.db");
    let session_pool = SqlitePool::connect("sqlite:./session.db").await.unwrap();
    let session_store = tower_sessions_sqlx_store::SqliteStore::new(session_pool);
    session_store.migrate().await.unwrap();
    
    
    session_store
}