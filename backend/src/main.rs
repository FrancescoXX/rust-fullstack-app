#[macro_use]
extern crate rocket;

use rocket::serde::{ Deserialize, Serialize, json::Json };
use rocket::{ State, response::status::Custom, http::Status };
use tokio_postgres::{ Client, NoTls };
use rocket_cors::{ CorsOptions, AllowedOrigins };

#[derive(Serialize, Deserialize, Clone)]
struct User {
    id: Option<i32>,
    name: String,
    email: String,
}

#[post("/api/users", data = "<user>")]
async fn add_user(
    conn: &State<Client>,
    user: Json<User>
) -> Result<Json<Vec<User>>, Custom<String>> {
    execute_query(
        conn,
        "INSERT INTO users (name, email) VALUES ($1, $2)",
        &[&user.name, &user.email]
    ).await?;
    get_users(conn).await
}

#[get("/api/users")]
async fn get_users(conn: &State<Client>) -> Result<Json<Vec<User>>, Custom<String>> {
    get_users_from_db(conn).await.map(Json)
}

async fn get_users_from_db(client: &Client) -> Result<Vec<User>, Custom<String>> {
    let users = client
        .query("SELECT id, name, email FROM users", &[]).await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))?
        .iter()
        .map(|row| User { id: Some(row.get(0)), name: row.get(1), email: row.get(2) })
        .collect::<Vec<User>>();

    Ok(users)
}

#[put("/api/users/<id>", data = "<user>")]
async fn update_user(
    conn: &State<Client>,
    id: i32,
    user: Json<User>
) -> Result<Json<Vec<User>>, Custom<String>> {
    execute_query(
        conn,
        "UPDATE users SET name = $1, email = $2 WHERE id = $3",
        &[&user.name, &user.email, &id]
    ).await?;
    get_users(conn).await
}

#[delete("/api/users/<id>")]
async fn delete_user(conn: &State<Client>, id: i32) -> Result<Status, Custom<String>> {
    execute_query(conn, "DELETE FROM users WHERE id = $1", &[&id]).await?;
    Ok(Status::NoContent)
}

async fn execute_query(
    client: &Client,
    query: &str,
    params: &[&(dyn tokio_postgres::types::ToSql + Sync)]
) -> Result<u64, Custom<String>> {
    client
        .execute(query, params).await
        .map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}

#[launch]
async fn rocket() -> _ {
    let (client, connection) = tokio_postgres
        ::connect("host=localhost user=postgres password=postgres dbname=postgres", NoTls).await
        .expect("Failed to connect to Postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Failed to connect to Postgres: {}", e);
        }
    });

    //Create the table if it doesn't exist
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL
            )",
            &[]
        ).await
        .expect("Failed to create table");

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .to_cors()
        .expect("Error while building CORS");

    rocket
        ::build()
        .manage(client)
        .mount("/", routes![add_user, get_users, update_user, delete_user])
        .attach(cors)
}
