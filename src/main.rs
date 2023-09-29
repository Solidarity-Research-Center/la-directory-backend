use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::middleware::DefaultHeaders;
use bb8::Pool;
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use firebase_auth::{FirebaseAuth, FirebaseUser};
use uuid::Uuid;

struct Org {
    id: String,
    name: String,
    description: String,
    website: String,
    phone: String,
    banner_url: String,
    profile_url: String,
    university: Option<String>,
    neighbourhood: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip: Option<String>,
    auth_emails: Vec<String>,
    categories: Vec<String>
}

#[actix_web::get("/")]
pub async fn index(pool: web::Data<bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>>, req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/plain"))
        .body("Hello world!")
}

#[actix_web::get("/gettime")]
pub async fn gettime(pool: web::Data<bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>>, req:HttpRequest) -> impl Responder {
    let conn = pool.get().await.unwrap();
    let rows = conn.query("SELECT NOW()", &[]).await.unwrap();
    let time: chrono::DateTime<chrono::Utc> = rows[0].get(0);
    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/plain"))
        .body(time.to_string())
}

#[derive(serde::Deserialize)]
struct MakeOrgForm {
    name: String,
    description: Option<String>,
    website: Option<String>,
    phone: Option<String>,
    banner_url: Option<String>,
    profile_url: Option<String>,
    university: Option<String>,
    neighbourhood: Option<String>,
    city: Option<String>,
    state: Option<String>,
    zip: Option<String>,
    auth_emails: Vec<String>,
    categories: Vec<String>
}

#[actix_web::post("/makeorg")]
pub async fn makeorg(pool: web::Data<bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>>, body: web::Json<MakeOrgForm>) ->  impl Responder {
    let conn = pool.get().await.unwrap();

    //get json body

    /*
        id text PRIMARY KEY,
    name text,
    description text,
    website string,
    phone string,
    banner_url string,
    profile_url string,
    university string,
    neighbourhood string,
    city string,
    state string,
    zip string,
    auto_emails string[],
    categories string[]
     */
    let insert = conn.query("INSERT INTO orgs (id, name, description, website, phone, banner_url, profile_url, university, neighbourhood, city, state, zip) ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    &[
        //id
        &Uuid::new_v4().to_string(),
        //name
        &body.name,
        &body.description,
        &body.website,
        &body.phone,
        &body.banner_url,
        &body.profile_url,
        &body.university,
        &body.neighbourhood,
        &body.city,
        &body.state,
        &body.zip,
        &body.auth_emails,
        &body.categories
    ]).await.unwrap();
    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/plain"))
        .body("Success")
}

#[actix_web::get("/testfirebase")]
async fn testfirebase(pool: web::Data<bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>>,firebase_auth: web::Data<FirebaseAuth>,req: HttpRequest) -> impl Responder {
    
        let token = (&req).headers().get("Authorization");

        let token = match token {
            Some(token) => token,
            None => return HttpResponse::Unauthorized().body("No token provided")
        };

    let user = firebase_auth.verify(token.to_str().unwrap());

    match user {
        Some(user) => {

            println!("user success");
            return HttpResponse::Ok().body(user.name.unwrap());
        },
        None => {
            return HttpResponse::Unauthorized().body("Invalid token");
        }
    }
    
   
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let postgresstring = arguments::parse(std::env::args())
    .unwrap()
    .get::<String>("postgres");

let postgresstring = postgresstring.unwrap();

   // Connect to the database.
   let manager: bb8_postgres::PostgresConnectionManager<NoTls> = bb8_postgres::PostgresConnectionManager::new(
    postgresstring.parse().unwrap(),
    NoTls,
);

let manager2: bb8_postgres::PostgresConnectionManager<NoTls> = bb8_postgres::PostgresConnectionManager::new(
    postgresstring.parse().unwrap(),
    NoTls,
);
let pool  = bb8::Pool::builder().build(manager).await.unwrap();

let pool2  = bb8::Pool::builder().build(manager2).await.unwrap();

let configclient = pool2.get().await.unwrap();

configclient.batch_execute("CREATE SCHEMA IF NOT EXISTS directory;").await.unwrap();

configclient.batch_execute("CREATE TABLE IF NOT EXISTS orgs (
    id text PRIMARY KEY,
    name text,
    description text,
    website string,
    phone string,
    banner_url string,
    profile_url string,
    university string,
    neighbourhood string,
    city string,
    state string,
    zip string,
    auto_emails string[],
    categories string[]
)").await.unwrap();

println!("Creating base data");

let firebase_auth = tokio::task::spawn_blocking(|| FirebaseAuth::new("la-movement-directory"))
.await
.expect("panic init FirebaseAuth");

    // Create a new HTTP server.
    let builder = HttpServer::new(move || {
        App::new()
            .wrap(
                DefaultHeaders::new()
                    .add(("Server", "LADirectory"))
                    .add(("Access-Control-Allow-Origin", "http://localhost:5173"))
                    .add(("Access-Control-Allow-Origin","https://directory.laforall.org"))
                    .add(("Access-Control-Allow-Credentials","true"))
                    .add(("Access-Control-Expose-Headers", "Server, hash, server, Hash"))
            )
        .app_data(actix_web::web::Data::new(pool.clone()))
        .app_data(actix_web::web::Data::new(firebase_auth.clone()))
        .service(index)
        .service(gettime)
        .service(testfirebase)
    })
    .workers(4);

    

    println!("everything set up");

    // Bind the server to port 38472
    let _ = builder.bind("127.0.0.1:38472").unwrap().run().await;

    Ok(())
}
