By the end of this tutorial, you will understand how to create a simple yet complete full-stack application using the following technologies:

For the Frontend:
- Rust - Core Programming Language
- Web Assembly - For running Rust in the browser
- Yew - Rust Framework for building client web apps
- Trunk - For serving the frontend app
- Tailwind CSS - For styling the frontend

For the Backend:
- Rust - Core Programming Language
- Rocket - Rust Framework for building web servers

For the Database:
- Postgres - Relational Database
- Docker - Dockerfile and Docker Compose for running Postgres

Wow, so many technologies! But we'll keep the example as basic as possible to help you understand the core concepts. Let's get started! 

We will proceed with a bottom-up approach, starting with the database, then the backend, and finally the frontend.  

If you prefer a video tutorial, you can watch it here.

{% youtube D_76TcfzDTU %}

All the code is available on [GitHub](https://youtu.be/FYVbt6YFMsM) (link in video description)
## Architecture

Before we start, here is a simple architecture diagram of the application we are going to build:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/xr096aqa4ffgo2yyob1k.png)](https://youtu.be/FYVbt6YFMsM)

The front end will be built using Yew, a new Rust framework for building client web apps. Yew is inspired by Elm and React and is designed to be simple and easy to use. We will use Trunk to serve the frontend and Tailwind CSS for styling. All this will be compiled to Web Assembly and run in the browser.

The Backend will be built using Rocket, a web framework for Rust. Rocket is designed to maximize the Developer Experience. We will use Rocket to build a simple REST API that will interact with the database.

The Database will be Postgres, a relational database. We will use Docker to run Postgres in a container, and we will use no ORM to keep things simple. We will interact with the database using SQL queries written directly in the Rocket handlers.


## Prerequisites

Before we start, make sure you have the following installed on your machine:

- Rust
- Docker

That's it! If you have never used WASM or Trunk, no worries; I will show you the commands you need to run.

## Preparation.

We will have a folder that will contain the following subfolders:
- backend
- frontend

So, let's create a new folder, navigate it, and open it in any IDE you want. 

I will use Visual Studio Code.

```bash
mkdir rustfs
cd rustfs
code .
```

From the root folder, initialize a git repository.

```bash
git init
```

And create a `compose.yml` file (this will be used to run the Postgres database)

And you should have something like this:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/36o8tyr23xu225gn6k33.png)](https://youtu.be/FYVbt6YFMsM)

We are now ready to build our application. In the next section, we will set up the database.

## Setting up the Database

We will use Docker to run a Postgres database in a container. This will make it easy to run the database locally without installing Postgres on your machine.

Open the `compose.yml` file and add the following:

```yml
services:
  db:
    container_name: db
    image: postgres:12
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: postgres
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata: {}
```

- `db` is the name of the service
- `container_name` is the name of the container, we will use `db`
- `image` is the Postgres image (we will use Postgres 12)
- `ports` is the port mapping (5432:5432)
- `environment` is the environment variables for the Postgres instance
- `volumes` is the volume mapping for the Postgres data

We also define a volume `pgdata` that will be used to store the Postgres data.

Now, run the following command to start the Postgres database:

```bash
docker compose up
```

You should see the Postgres logs in the terminal. If you see `database system is ready to accept connections`, the database is probably running successfully.

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/ibt7wgojoxi4w74sf1cx.png)](https://youtu.be/FYVbt6YFMsM)

To make another test, you can go on the terminal and type:

```bash
docker ps -a
```

And you should see the database running:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/rzj8nrrtu5oirc76hzd7.png)](https://youtu.be/FYVbt6YFMsM)

You can also step into the database container by running:

```bash
docker exec -it db psql -U postgres
```

You can check the current databases by running:

```bash
\dt
```

And you should see the following output (Did not find any relations):

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/1asdk6dtkk68uv1kzty4.png)](https://youtu.be/FYVbt6YFMsM)

This is because we have not created any tables yet. We will do that in the next section.

## Setting up the Backend

We will use Rocket to build the backend. 

Rocket is a web framework for Rust that is designed to maximize the Developer Experience. We will use Rocket to build a simple REST API that will interact with the database.


Create a new Rust project called `backend`, without initializing a git repository:

```bash
cargo new backend --vcs none
```

Your project structure should look like this:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/m2cqsucz9g9efrks3vgh.png)](https://youtu.be/FYVbt6YFMsM)

Open the `Cargo.toml` file and add the following dependencies:

```toml
rocket = { version = "0.5", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tokio-postgres = "0.7.11"
rocket_cors = { version = "0.6.0", default-features = false }
```

- `rocket` is the Rocket web framework we will use to build the backend
- `serde` is a serialization/deserialization library
- `serde_json` is a JSON serialization/deserialization library
- `tokio` is an asynchronous runtime for Rust
- `tokio-postgres` is a Postgres client for Tokio
- `rocket_cors` is a CORS library for Rocket

Now, open the `/backend/main.rs` file and replace the contents with the following (explanation below):

```rust
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
```

In this part of the video, I explain the code above.

Explanation

- We make all the imports at the top of the file. We also definte a `macro_use` attribute to import the `rocket` macro.
- We define a `User` struct that will represent the user data. This struct will be serialized/deserialized to/from JSON (Note: The id is an `Option` because we don't want to provide an id when creating a new user, it will be assigned by the database).
- We define the `add_user` route that will insert a new user into the database. We use the `execute_query` function to execute the SQL query. We then call the `get_users` function to return all the users.
- We define the `get_users` route that will return all the users from the database.
- We define the `update_user` route that will update a user in the database. We use the `execute_query` function to execute the SQL query. We then call the `get_users` function to return all the users.
- We define the `delete_user` route that will delete a user from the database. We use the `execute_query` function to execute the SQL query.
- We define the `execute_query` function that will execute a SQL query on the database.
- We define the `rocket` function that will create the Rocket instance. We connect to the Postgres database and create the `users` table if it doesn't exist, using a SQL query. We then create the CORS options and attach them to the Rocket instance. Even if we are running the frontend and back end on the same machine, we need to enable CORS to allow the frontend to make requests to the backend.

We can now run the backend by running:

```bash
cargo run
```
And we should see the following output:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/8ef3f3w5yeigwsc3ctba.png)](https://youtu.be/FYVbt6YFMsM)

You can visit the following URL: `http://127.0.0.1:8000/api/users` and you should see an empty array `[]`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/lctpm4ic0ci0x1fdzteg.png)](https://youtu.be/FYVbt6YFMsM)

### Testing the APIs with Postman

You can test the APIs using Postman. 

You can get the list of the users by sending a `GET` request to `http://127.0.0.1:8000/api/users`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/r36hk2f89xgnta8zxykx.png)](https://youtu.be/FYVbt6YFMsM)

You can create a new user by sending a `POST` request to `http://127.0.0.1:8000/api/users` with the following JSON body:

```json
{
    "name": "AAA",
    "email": "aaa@mail.com"
}
```

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/er9ex1e6ih0r9rwh45qm.png)](https://youtu.be/FYVbt6YFMsM)


You can create 2 more users:

```json
{
    "name": "BBB",
    "email": "
}
```

```json
{
    "name": "CCC",
    "email": "
}
```

You should see the following output:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/bjecymzisay9noeh8nba.png)](https://youtu.be/FYVbt6YFMsM)


To Update a user, you can send a `PUT` request to `http://127.0.0.1:8000/api/users/2` with the following JSON body:

```json
{
    "name": "Francesco",
    "email": "francesco@mail"
}
```

And we should see the updated user:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/ybo563f6wszq7uj5w1sj.png)](https://youtu.be/FYVbt6YFMsM)

To delete a user, you can send a `DELETE` request to `http://127.0.0.1:8000/api/users/1`:

And we should get a 204 response (Resource was deleted):

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/yp3uf9afnj37gqal5mn1.png)](https://youtu.be/FYVbt6YFMsM) 

And if we try to get all the users, we should see the following output:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/mq8jkod0h9i4i0u1uj6r.png)](https://youtu.be/FYVbt6YFMsM)

We can see that this is consistent if we use the browser to check the users at the address `http://127.0.0.1:8000/api/users`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/7u7ynlld3vi9umw4mh28.png)](https://youtu.be/FYVbt6YFMsM)

We can also test it directly in the Postgres database by running:
(If you closed the terminal, you can step into the container by running `docker exec -it db psql -U postgres`)

```bash
\dt
select * from users;
```

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/tlf5dt0ik42dkf67mgdl.png)](https://youtu.be/FYVbt6YFMsM)

Congratulations! You have successfully set up the backend. In the next section, we will set up the frontend.

## Setting up the Frontend

Now, let's work on the front end. We will use Yew to build it. Yew is a Rust framework for building client web apps. We will use Trunk to build and bundle the front end and Tailwind CSS for styling. All this will be compiled to Web Assembly and run in the browser.

**IMPORTANT!** If you never used Wasm for Rust on your machine, you can install it by running:

```bash
rustup target add wasm32-unknown-unknown
```

**IMPORTANT!** You must also have `trunk` installed on your machine. You can install it by running:

```bash
cargo install trunk
```
You can verify that `trunk` is installed by running:

```bash
trunk --version
```

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/5ec96sei8ndvz5ivepsp.png)](https://youtu.be/FYVbt6YFMsM)

Now you can create a new Rust project called `frontend` (be sure to be in the `rustfs` folder):

```bash
cargo new frontend --vcs none
```

Now open the `frontend/Cargo.toml` file and add the following dependencies:

```toml
[package]
name = "frontend"
version = "0.1.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["console"] }
gloo = "0.6"
wasm-bindgen-futures = "0.4"  
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

- `yew` is the Yew framework (Rust framework for building client web apps)
- `wasm-bindgen` is a library that facilitates communication between WebAssembly and JavaScript
- `web-sys` is a library that provides bindings to Web APIs
- `gloo` is a library that provides utilities for WebAssembly
- `wasm-bindgen-futures` is a library that provides utilities for working with futures in WebAssembly
- `serde` is a serialization/deserialization library
- `serde_json` is a JSON serialization/deserialization library

Now create a new file called `index.html` in the `frontend` folder and add the following:

```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Yew + Tailwind</title>
    <script src="https://cdn.tailwindcss.com"></script>
  </head>
  <body>
    <div id="app"></div>
    <script type="module">
      import init from './pkg/frontend.js';
      init();
    </script>
  </body>
</html>
```

- We import the Tailwind CSS CDN in the head of the HTML file
- We create a div with the id `app` where the Yew app will be mounted
- We import the `frontend.js` file that will be generated by Trunk

Now open the `frontend/src/main.rs` file and replace the contents with the following:

```rust
use yew::prelude::*;
use serde::{ Deserialize, Serialize };
use gloo::net::http::Request;
use wasm_bindgen_futures::spawn_local;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```

- We import the necessary dependencies
- We define a `User` struct that will represent the user data
- We define the `main` function that will render the Yew app

But this is not enough. We need to add the `App` component. We can use an external file, but here for simplicity, we will add it directly in the `main.rs` file.

Below is the code you should add to the main.rs file.

This code defines a Yew function component named App that manages user data and interactions within a web application. The use_state hooks initialize states for managing user information `(user_state)`, messages `(message)`, and the list of users `(users)`. 

The component defines several callbacks for interacting with a backend API:

- `get_users:` Fetches the list of users from the backend API and updates the users state. If the request fails, it sets an error message.
- `create_user:` Sends a POST request to create a new user using the data from user_state. On success, it triggers the get_users callback to refresh the user list.
- `update_user:` Updates an existing user's information by sending a PUT request to the backend. If successful, it refreshes the user list and resets the user_state.
- `delete_user:` Deletes a user based on their ID by sending a DELETE request to the backend. On success, it refreshes the user list.
- `edit_user:` Prepares a user's information for editing by updating the user_state with the selected user's details.

These callbacks utilize asynchronous operations `(spawn_local)` to handle network requests without blocking the UI thread, ensuring a responsive user experience.

```rust
...
#[function_component(App)]
fn app() -> Html {
    let user_state = use_state(|| ("".to_string(), "".to_string(), None as Option<i32>));
    let message = use_state(|| "".to_string());
    let users = use_state(Vec::new);

    let get_users = {
        let users = users.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let users = users.clone();
            let message = message.clone();
            spawn_local(async move {
                match Request::get("http://127.0.0.1:8000/api/users").send().await {
                    Ok(resp) if resp.ok() => {
                        let fetched_users: Vec<User> = resp.json().await.unwrap_or_default();
                        users.set(fetched_users);
                    }

                    _ => message.set("Failed to fetch users".into()),
                }
            });
        })
    };

    let create_user = {
        let user_state = user_state.clone();
        let message = message.clone();
        let get_users = get_users.clone();
        Callback::from(move |_| {
            let (name, email, _) = (*user_state).clone();
            let user_state = user_state.clone();
            let message = message.clone();
            let get_users = get_users.clone();

            spawn_local(async move {
                let user_data = serde_json::json!({ "name": name, "email": email });

                let response = Request::post("http://127.0.0.1:8000/api/users")
                    .header("Content-Type", "application/json")
                    .body(user_data.to_string())
                    .send().await;

                match response {
                    Ok(resp) if resp.ok() => {
                        message.set("User created successfully".into());
                        get_users.emit(());
                    }

                    _ => message.set("Failed to create user".into()),
                }

                user_state.set(("".to_string(), "".to_string(), None));
            });
        })
    };

    let update_user = {
        let user_state = user_state.clone();
        let message = message.clone();
        let get_users = get_users.clone();

        Callback::from(move |_| {
            let (name, email, editing_user_id) = (*user_state).clone();
            let user_state = user_state.clone();
            let message = message.clone();
            let get_users = get_users.clone();

            if let Some(id) = editing_user_id {
                spawn_local(async move {
                    let response = Request::put(&format!("http://127.0.0.1:8000/api/users/{}", id))
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&(id, name.as_str(), email.as_str())).unwrap())
                        .send().await;

                    match response {
                        Ok(resp) if resp.ok() => {
                            message.set("User updated successfully".into());
                            get_users.emit(());
                        }

                        _ => message.set("Failed to update user".into()),
                    }

                    user_state.set(("".to_string(), "".to_string(), None));
                });
            }
        })
    };

    let delete_user = {
        let message = message.clone();
        let get_users = get_users.clone();

        Callback::from(move |id: i32| {
            let message = message.clone();
            let get_users = get_users.clone();

            spawn_local(async move {
                let response = Request::delete(
                    &format!("http://127.0.0.1:8000/api/users/{}", id)
                ).send().await;

                match response {
                    Ok(resp) if resp.ok() => {
                        message.set("User deleted successfully".into());
                        get_users.emit(());
                    }

                    _ => message.set("Failed to delete user".into()),
                }
            });
        })
    };

    let edit_user = {
        let user_state = user_state.clone();
        let users = users.clone();

        Callback::from(move |id: i32| {
            if let Some(user) = users.iter().find(|u| u.id == id) {
                user_state.set((user.name.clone(), user.email.clone(), Some(id)));
            }
        })
    };
...
```

You can check writing the code line by line in this part of the video

Now we need to add the HTML code rendered by the Yew component. Below is the code you should add in the main.rs file.

If you know React, this is similar to what happens in a JSX file.


This HTML part written using Yew's html! macro, defines the user interface of the Yew application. It consists of several key sections that provide functionalities for managing users.

- A main container with some padding and a nice layout using Tailwind CSS.
- A big title at the top says “User Management” to let users know what the app is about.
- Two input fields: one for the user's name and another for their email. As you type, it updates the state to track what’s entered.
- A button that changes its action and label depending on whether you're creating a new user or updating an existing one. It says `Create User` if you're adding a new one or `Update User` if you're editing.
- A space for messages to appear—like success or error messages—just below the input fields (the text color is always green; feel free to make it red in case of errors).
- A `Fetch User List` button that, when clicked, pulls the latest user data from the backend.
- A section that lists all the users fetched from the backend, showing their ID, name, and email.
- Each user in the list has a "Delete" button to remove them and an "Edit" button to load their details into the input fields for editing.

```rust
...
html! {
        <div class="container mx-auto p-4">
            <h1 class="text-4xl font-bold text-blue-500 mb-4">{ "User Management" }</h1>
                <div class="mb-4">
                    <input
                        placeholder="Name"
                        value={user_state.0.clone()}
                        oninput={Callback::from({
                            let user_state = user_state.clone();
                            move |e: InputEvent| {
                                let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                                user_state.set((input.value(), user_state.1.clone(), user_state.2));
                            }
                        })}
                        class="border rounded px-4 py-2 mr-2"
                    />
                    <input
                        placeholder="Email"
                        value={user_state.1.clone()}
                        oninput={Callback::from({
                            let user_state = user_state.clone();
                            move |e: InputEvent| {
                                let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                                user_state.set((user_state.0.clone(), input.value(), user_state.2));
                            }
                        })}
                        class="border rounded px-4 py-2 mr-2"
                    />

                    <button
                        onclick={if user_state.2.is_some() { update_user.clone() } else { create_user.clone() }}
                        class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                    >
                        { if user_state.2.is_some() { "Update User" } else { "Create User" } }
                        
                    </button>
                        if !message.is_empty() {
                        <p class="text-green-500 mt-2">{ &*message }</p>
                    }
                </div>

                <button
                    onclick={get_users.reform(|_| ())}  
                    class="bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded mb-4"
                >
                    { "Fetch User List" }
                </button>

                <h2 class="text-2xl font-bold text-gray-700 mb-2">{ "User List" }</h2>

                <ul class="list-disc pl-5">
                    { for (*users).iter().map(|user| {
                        let user_id = user.id;
                        html! {
                            <li class="mb-2">
                                <span class="font-semibold">{ format!("ID: {}, Name: {}, Email: {}", user.id, user.name, user.email) }</span>
                                <button
                                    onclick={delete_user.clone().reform(move |_| user_id)}
                                    class="ml-4 bg-red-500 hover:bg-red-700 text-white font-bold py-1 px-2 rounded"
                                >
                                    { "Delete" }
                                </button>
                                <button
                                    onclick={edit_user.clone().reform(move |_| user_id)}
                                    class="ml-4 bg-yellow-500 hover:bg-yellow-700 text-white font-bold py-1 px-2 rounded"
                                >
                                    { "Edit" }
                                </button>
                            </li>
                        }
                    })}

                </ul>
                    

        </div>
    }
...
```

You can check writing the code line by line in this part of the video

Here is the complete code for the `/frontend/src/main.rs` file:

```rust
use yew::prelude::*;
use serde::{ Deserialize, Serialize };
use gloo::net::http::Request;
use wasm_bindgen_futures::spawn_local;

#[function_component(App)]
fn app() -> Html {
    let user_state = use_state(|| ("".to_string(), "".to_string(), None as Option<i32>));
    let message = use_state(|| "".to_string());
    let users = use_state(Vec::new);

    let get_users = {
        let users = users.clone();
        let message = message.clone();
        Callback::from(move |_| {
            let users = users.clone();
            let message = message.clone();
            spawn_local(async move {
                match Request::get("http://127.0.0.1:8000/api/users").send().await {
                    Ok(resp) if resp.ok() => {
                        let fetched_users: Vec<User> = resp.json().await.unwrap_or_default();
                        users.set(fetched_users);
                    }

                    _ => message.set("Failed to fetch users".into()),
                }
            });
        })
    };

    let create_user = {
        let user_state = user_state.clone();
        let message = message.clone();
        let get_users = get_users.clone();
        Callback::from(move |_| {
            let (name, email, _) = (*user_state).clone();
            let user_state = user_state.clone();
            let message = message.clone();
            let get_users = get_users.clone();

            spawn_local(async move {
                let user_data = serde_json::json!({ "name": name, "email": email });

                let response = Request::post("http://127.0.0.1:8000/api/users")
                    .header("Content-Type", "application/json")
                    .body(user_data.to_string())
                    .send().await;

                match response {
                    Ok(resp) if resp.ok() => {
                        message.set("User created successfully".into());
                        get_users.emit(());
                    }

                    _ => message.set("Failed to create user".into()),
                }

                user_state.set(("".to_string(), "".to_string(), None));
            });
        })
    };

    let update_user = {
        let user_state = user_state.clone();
        let message = message.clone();
        let get_users = get_users.clone();

        Callback::from(move |_| {
            let (name, email, editing_user_id) = (*user_state).clone();
            let user_state = user_state.clone();
            let message = message.clone();
            let get_users = get_users.clone();

            if let Some(id) = editing_user_id {
                spawn_local(async move {
                    let response = Request::put(&format!("http://127.0.0.1:8000/api/users/{}", id))
                        .header("Content-Type", "application/json")
                        .body(serde_json::to_string(&(id, name.as_str(), email.as_str())).unwrap())
                        .send().await;

                    match response {
                        Ok(resp) if resp.ok() => {
                            message.set("User updated successfully".into());
                            get_users.emit(());
                        }

                        _ => message.set("Failed to update user".into()),
                    }

                    user_state.set(("".to_string(), "".to_string(), None));
                });
            }
        })
    };

    let delete_user = {
        let message = message.clone();
        let get_users = get_users.clone();

        Callback::from(move |id: i32| {
            let message = message.clone();
            let get_users = get_users.clone();

            spawn_local(async move {
                let response = Request::delete(
                    &format!("http://127.0.0.1:8000/api/users/{}", id)
                ).send().await;

                match response {
                    Ok(resp) if resp.ok() => {
                        message.set("User deleted successfully".into());
                        get_users.emit(());
                    }

                    _ => message.set("Failed to delete user".into()),
                }
            });
        })
    };

    let edit_user = {
        let user_state = user_state.clone();
        let users = users.clone();

        Callback::from(move |id: i32| {
            if let Some(user) = users.iter().find(|u| u.id == id) {
                user_state.set((user.name.clone(), user.email.clone(), Some(id)));
            }
        })
    };

    //html

    html! {
        <div class="container mx-auto p-4">
            <h1 class="text-4xl font-bold text-blue-500 mb-4">{ "User Management" }</h1>
                <div class="mb-4">
                    <input
                        placeholder="Name"
                        value={user_state.0.clone()}
                        oninput={Callback::from({
                            let user_state = user_state.clone();
                            move |e: InputEvent| {
                                let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                                user_state.set((input.value(), user_state.1.clone(), user_state.2));
                            }
                        })}
                        class="border rounded px-4 py-2 mr-2"
                    />
                    <input
                        placeholder="Email"
                        value={user_state.1.clone()}
                        oninput={Callback::from({
                            let user_state = user_state.clone();
                            move |e: InputEvent| {
                                let input = e.target_dyn_into::<web_sys::HtmlInputElement>().unwrap();
                                user_state.set((user_state.0.clone(), input.value(), user_state.2));
                            }
                        })}
                        class="border rounded px-4 py-2 mr-2"
                    />

                    <button
                        onclick={if user_state.2.is_some() { update_user.clone() } else { create_user.clone() }}
                        class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded"
                    >
                        { if user_state.2.is_some() { "Update User" } else { "Create User" } }
                        
                    </button>
                        if !message.is_empty() {
                        <p class="text-green-500 mt-2">{ &*message }</p>
                    }
                </div>

                <button
                    onclick={get_users.reform(|_| ())}  
                    class="bg-gray-500 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded mb-4"
                >
                    { "Fetch User List" }
                </button>

                <h2 class="text-2xl font-bold text-gray-700 mb-2">{ "User List" }</h2>

                <ul class="list-disc pl-5">
                    { for (*users).iter().map(|user| {
                        let user_id = user.id;
                        html! {
                            <li class="mb-2">
                                <span class="font-semibold">{ format!("ID: {}, Name: {}, Email: {}", user.id, user.name, user.email) }</span>
                                <button
                                    onclick={delete_user.clone().reform(move |_| user_id)}
                                    class="ml-4 bg-red-500 hover:bg-red-700 text-white font-bold py-1 px-2 rounded"
                                >
                                    { "Delete" }
                                </button>
                                <button
                                    onclick={edit_user.clone().reform(move |_| user_id)}
                                    class="ml-4 bg-yellow-500 hover:bg-yellow-700 text-white font-bold py-1 px-2 rounded"
                                >
                                    { "Edit" }
                                </button>
                            </li>
                        }
                    })}

                </ul>
                    

        </div>
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    id: i32,
    name: String,
    email: String,
}

fn main() {
    yew::Renderer::<App>::new().render();
}
```

## Building the Frontend

It's now time to run the frontend.

You can type:
```
cargo build --target wasm32-unknown-unknown
```

And then you can run the frontend by running:

```bash
trunk serve
```

You can now visit `http://127.0.0.1:8080` and click the `Fetch User List` button to fetch the users from the backend: 

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/b980sgazvkr9jhw1g89p.png)](https://youtu.be/FYVbt6YFMsM)

As you can see, we fetch users from the backend and display them in the front end. The front end also allows you to create, update, and delete users.

For example, we can create a user with name `yew` and email `yes@mail.com`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/yfl4w7mlgq1huu5xa0ii.png)](https://youtu.be/FYVbt6YFMsM)

The user should be displayed correctly on the frontend, with the message `User created successfully`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/jmuwyvtcu5onh3dyv40p.png)](https://youtu.be/FYVbt6YFMsM)

To check the consistency of data, we can use Postman, making a `GET` request to `http://127.0.0.1:8000/api/users`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/8jzza68jcbzm1kbsx3qb.png)](https://youtu.be/FYVbt6YFMsM)

We can also update a user, for example, the one with ID 3, by changing the name to `subscribe` and the email to `subscribe@mail.com`. Notice that when we hit the `Edit` button , the form is populated with the user data, and the button label is changed to `Update User`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/2cbg9apnoqdunas0av22.png)](https://youtu.be/FYVbt6YFMsM)

After hitting the `Update User` button, we should see the message `User updated successfully`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/9e6zg6th7a9kwd3348rt.png)](https://youtu.be/FYVbt6YFMsM)


The last test is to delete a user, for example, the one with ID 3. After hitting the `Delete` button, we should see the message `User deleted successfully`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/q3z2md59qmjnh88kpomu.png)](https://youtu.be/FYVbt6YFMsM)

After hitting the `Delete` button, we should see the message `User deleted successfully`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/fs48v7ulz8b3dqc8bcmd.png)](https://youtu.be/FYVbt6YFMsM)

**Note:** You should be able to see all the HTTP requests in the backend logs.

Let's create the last user, and let's call it `last` and let's use as email `last@mail.com`

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/9htsv8zhy9stpapax0tq.png)](https://youtu.be/FYVbt6YFMsM)

If we use Postman and make a `GET` request to `http://127.0.0.1:8000/api/users`, we should see the following output:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/gc1sjnta8487ju7yuzop.png)](https://youtu.be/FYVbt6YFMsM)

We can also see the data by opening a new tab and visiting `http://127.0.0.1:8000/api/users`:

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/ob1sb41al9njmqfiq1x1.png)](https://youtu.be/FYVbt6YFMsM)

The last test is to check it directly in the Postgres container. You can step into the container by running `docker exec -it db psql -U postgres` and then run:

```bash
\dt
select * from users;
```

[![Build a rust fullstack web application](https://dev-to-uploads.s3.amazonaws.com/uploads/articles/xq8aa3by5wdxabs1gypz.png)](https://youtu.be/FYVbt6YFMsM)

Well done!

## Conclusion

In this tutorial, we built a full-stack web application using Rust. We built a backend using Rocket and Postgres and a frontend using Yew, Tailwind CSS, and Trunk. We created, read, update, and deleted users from the database using the front end. We also tested the APIs using Postman and checked

If you prefer a video version:
{% youtube FYVbt6YFMsM %}

All the code is available on [GitHub](https://youtu.be/FYVbt6YFMsM) (link in video description)


