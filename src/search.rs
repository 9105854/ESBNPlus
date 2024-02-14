use rocket_dyn_templates::{context, Template};

#[get("/search?<q>")]
pub async fn simple_search(q: &str) -> Template {
    // TODO: get data from IGDB Api
    Template::render("index", context![])
}
