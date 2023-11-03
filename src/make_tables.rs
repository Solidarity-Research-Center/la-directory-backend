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

println!("Making pools");

let pool  = bb8::Pool::builder().build(manager).await.unwrap();

println!("making config client");

let configclient = pool.get().await.unwrap();

configclient.batch_execute("CREATE SCHEMA IF NOT EXISTS directory;").await.unwrap();

configclient.batch_execute("CREATE TABLE IF NOT EXISTS orgs (
    id text PRIMARY KEY,
    name text,
    description text,
    website text,
    phone text,
    banner_url text,
    profile_url text,
    university text,
    neighbourhood text,
    city text,
    state text,
    zip text,
    twitter text,
    instagram text,
    facebook text,
    auto_emails text[],
    categories text[]
)").await.unwrap();

println!("Creating base data");
}