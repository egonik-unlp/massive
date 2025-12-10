use anyhow::Context;
use dotenvy;
use leptos::logging;
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use std::env::VarError;
use thiserror::Error;

pub use crate::components::upload_and_display::{ImageInference, Ingredients};

#[derive(Error, Debug)]
pub enum InferenceError {
    #[error("Cannot retrieve openai API KEY")]
    ApiRetrieval(#[from] VarError),
    #[error(transparent)]
    Serialization(#[from] anyhow::Error),
}
pub type InferenceResult<T> = Result<T, InferenceError>;
fn fetch_type_description() -> &'static str {
    include_str!("../../components/upload_and_display.rs")
}
pub struct Model {
    pub(crate) inference_provider: Box<dyn llm::LLMProvider>,
}
impl Model {
    pub fn new() -> InferenceResult<Self> {
        dotenvy::dotenv().expect("No esta el .env");
        let api_key = std::env::var("OPENAI_API_KEY")?;
        let llm = LLMBuilder::new()
            .backend(LLMBackend::OpenAI)
            .api_key(api_key)
            .model("gpt-4.1-nano")
            .max_tokens(512)
            .temperature(0.7)
            .build()
            .expect("Model Building error");
        Ok(Model {
            inference_provider: llm,
        })
    }
    pub async fn infer_from_image(
        &self,
        image_url: String,
    ) -> InferenceResult<Option<ImageInference>> {
        let structure = "{ingredients : [\"palmitate\", \"niacin\"], inferred_category: Foods, inferred_product_name: Chocolinas}";
        let url = image_url.clone();
        println!("Entrando a openai, me aparece = {}", url);
        let messages = vec![
            ChatMessage::user().content("You are a system used to recognize ingredients from a product label. You have three tasks").build(),
            ChatMessage::user().content("First task: Recognize all ingredients in the label").build(),
            ChatMessage::user().content("Second task: Infer the category of the product").build(),
            ChatMessage::user().content("Third task: Infer the brand name of the product and the type of product it is").build(),
            ChatMessage::assistant().content("What choices do I have to decide the category?").build(),
            ChatMessage::user().content("It can be: BPC -> Beauty and personal cate, Foods: food products, Home: Home products, like cleaning for example.The naming is strict. The only allowed categories are Foods, BPC, Beauty. if you write something else it will cause an error").build(),
            ChatMessage::assistant().content("What is the inferred_product_name?").build(),
            ChatMessage::user().content("It is the product name and inferred type you think corresponds to the ingredient label you are seeing").build(),
            ChatMessage::assistant().content("What is the structure the data should have?").build(),
            ChatMessage::user().content("you should extract the parameters as : ingredients is an  array of strings, and inferred_category is a PascalCase string of the categories previously mentioned. inferred_product_name is a string. I will provide the json structure in the next message.").build(),
            ChatMessage::user().content(structure).build(),
            ChatMessage::assistant().content("Can you provide this information in a way that is easier for me to understand?").build(),
            ChatMessage::user().content(format!("You should follow the specification for InferenceResult as defined here {}",fetch_type_description() )).build(),
            ChatMessage::user().content("you should not include linebreaks. Also do not escape quoates. It should be valid JSON").build(),
            ChatMessage::user().content("Use this image!").image_url(url).build(),
        ];
        let inference = match self.inference_provider.chat(&messages).await {
            Ok(response) => {
                if let Some(text) = response.text() {
                    println!("text = {}", text);
                    let result: ImageInference = serde_json::from_str(&text)
                        .with_context(|| format!("Error deserailizando.\nPayload:\n{}", text))?;
                    Some(result)
                } else {
                    None
                }
            }
            Err(err) => {
                eprintln!("Error {}", err);
                None
            }
        };
        Ok(inference)
    }

    pub fn new_embedding(model: &str) -> anyhow::Result<Self> {
        dotenvy::dotenv().expect("No esta el .env");
        let api_key = std::env::var("OPENAI_API_KEY")?;
        let llm = LLMBuilder::new()
            .backend(LLMBackend::OpenAI)
            .api_key(api_key)
            .model(model)
            .max_tokens(512)
            .temperature(0.7)
            .build()
            .expect("Model Building error");
        Ok(Model {
            inference_provider: llm,
        })
    }
    pub async fn embed_vector(self, queries: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
        println!("Got here");
        logging::log!("Got here");
        println!("With input = {:?}", queries);
        let embeddings = self
            .inference_provider
            .embed(queries)
            .await
            .context("could not embed queries")?;

        Ok(embeddings)
    }
}
