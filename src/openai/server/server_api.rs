use crate::components::upload_and_display::{ImageInference, Ingredients};
// use crate::openai::server::model_server::{ImageInference, Ingredients};
use leptos::{leptos_dom::logging, prelude::*};
#[server]
pub async fn get_ingredients_from_image(
    image_url: String,
) -> Result<ImageInference, ServerFnError> {
    use crate::openai::server::model_server::Model;
    // let _url = format!("http://proto-api.work.gd/{}", image_url);
    // let url =
    //     "https://images.openfoodfacts.org/images/products/762/221/064/6712/ingredients_es.21.400.jpg"
    // .to_string();
    println!("[PRINT] url para request = {}", image_url);
    let model = Model::new().map_err(|err| ServerFnError::new(err.to_string()))?;
    let ings = model
        .infer_from_image(image_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
        .ok_or(ServerFnError::new("No se devolvio nada de openai"))?;
    Ok(ings)
}
