mod api;
mod schema;
pub mod utils;
use actix_web::{
    guard,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use api::{get_file, upload};
use async_graphql::{
    http::{GraphiQLSource, MultipartOptions},
    EmptySubscription, Schema,
};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use schema::{FilesSchema, MutationRoot, QueryRoot};

async fn index(schema: web::Data<FilesSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn gql_playgound() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint("/").finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    let schema = Schema::build(QueryRoot, MutationRoot, EmptySubscription).finish();

    println!("GraphiQL IDE: http://localhost:8000");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(
                web::resource("/graphql")
                    .guard(guard::Post())
                    .to(index)
                    .app_data(MultipartOptions::default().max_num_files(3)),
            )
            .service(
                web::resource("/graphql")
                    .guard(guard::Get())
                    .to(gql_playgound),
            )
            .service(get_file)
            .service(upload)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
