use actix_web::{middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web::middleware::DefaultHeaders;
use bb8::Pool;
use r2d2_postgres::{postgres::NoTls, PostgresConnectionManager};
use firebase_auth::{FirebaseAuth, FirebaseUser};

#[actix_web::get("/")]
pub async fn index(pool: web::Data<bb8::Pool<bb8_postgres::PostgresConnectionManager<NoTls>>>, req: HttpRequest) -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/plain"))
        .body("Hello world!")
}

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
let pool  = bb8::Pool::builder().build(manager).await.unwrap();

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
    })
    .workers(4);

    

    println!("everything set up");

    // Bind the server to port 38472
    let _ = builder.bind("127.0.0.1:38472").unwrap().run().await;

    Ok(())
}
