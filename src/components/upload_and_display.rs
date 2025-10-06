use leptos::ev::{DragEvent, SubmitEvent};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::{either::Either, ev::Event};
use leptos::{html, logging};
use leptos_router::components::{Route, Router, Routes, A};
use serde::{Deserialize, Serialize};
use thaw::{
    ConfigProvider, FileList, Toast, ToastBody, ToasterInjection, ToasterProvider, Upload,
    UploadDragger,
};

#[derive(Debug, Serialize, Clone, Deserialize, Default)]
pub struct Ingredients {
    pub id: i32,
    pub doc: String,
    pub score_db: f32,
    pub info: String,
}
#[derive(Debug, Serialize, Clone, Deserialize, Default)]
pub struct Response {
    pub duration_seconds: f32,
    pub ingredientes: Vec<Ingredients>,
    pub len: u8,
    pub len90: u8,
}
impl Response {
    fn get_ingredient_ids(self) -> Vec<i32> {
        self.ingredientes.into_iter().map(|ing| ing.id).collect()
    }
}
#[derive(Debug, Clone)]
struct CloneableError(String);

impl std::fmt::Display for CloneableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[component]
pub fn IngredientsCard(
    ingredients: Response,
    url: String,
    score: Resource<Result<f32, ServerFnError>>,
) -> impl IntoView {
    //  TDOO: use read signal
    let (ing_value, _ing_set_value) = signal(ingredients);
    let has_field_populated = move || ing_value.get().ingredientes.len() != 0;
    view! {
        <h3>"Este otro va aca"</h3>
        <p class="w-6xl">"Ingredientes encontrados:  "</p>
        {move || {
            if has_field_populated() {
                Either::Left(
                    view! {
                        <h3>"Score "{
                            match score.get() {
                                Some(Ok(score)) => {
                                    log!("Score = {}", score);
                                    score
                                },
                                Some(Err(e)) => {
                                    log!("Error path {}", e);
                                    0f32
                                },
                                None => {
                                    log!("None path");
                                    0f32
                                }

                            }
                        }</h3>
                        <h4>
                            "Se detectaron  "{ing_value.get().ingredientes.len()}" ingredientes"
                        </h4>
                        <img class= "imaginate" src=url.clone() />
                        <Accordion response=ing_value.get() />
                    },
                )
            } else {
                Either::Right(view! { <p>Si no me das nada no te doy nada</p> })
            }
        }}
    }
}

#[component]
fn Accordion(response: Response) -> impl IntoView {
    view! {
        <h3>"Este otro va aca"</h3>
        <div class="container-sm">
            <div class="accordion accordion-flush" id="accordionFlushExample">
                {response
                    .ingredientes
                    .into_iter()
                    .enumerate()
                    .map(|(n, ingredient)| {
                        view! {
                            <AccordionNode name_of_child=ingredient.doc position=n>
                                <div>
                                    <p>"Informacion sobre el ingrediente :  "{ingredient.info}</p>
                                    <p style=match ingredient.score_db as u32 {
                                        0 => "color: purple;",
                                        1 => "color: orange;",
                                        3 => "color: green;",
                                        _ => "color: red;",
                                    }>"Puntaje en nuestra db:  "{ingredient.score_db}</p>
                                </div>
                            </AccordionNode>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
fn AccordionNode(
    children: ChildrenFragment,
    name_of_child: String,
    position: usize,
) -> impl IntoView {
    let children = children()
        .nodes
        .into_iter()
        .map(|child| {
            let unique_id = format!("flush-collapseOne-{}", position);
            log!("unique_id = {}", unique_id);
            log!("Rendering accordion node with ID = {}", unique_id);
            view! {
                <div class="accordion-item bg-[#84a4b4] w-6xl">
                    <h2 class="accordion-header">
                        <button
                            class="accordion-button collapsed w-2xl"
                            type="button"
                            data-bs-toggle="collapse"
                            data-bs-target=format!("#{}", unique_id)

                            aria-expanded="false"
                            aria-controls=unique_id.clone()
                        >
                            <p class="w-2xl">{name_of_child.clone()}</p>
                        </button>
                    </h2>
                    <div
                        id=unique_id.clone()
                        class="accordion-collapse collapse w-2xl"
                        data-bs-parent="#accordionFlushExample"
                    >
                        <div class="accordion-body text-black w-2xl">
                            {child} "  \ncontenido permanente"
                        </div>
                    </div>
                </div>
            }
        })
        .collect_view();
    return children;
}

async fn mock_response() -> Result<Response, CloneableError> {
    let string = include_str!("../../sample_response.json");
    let response: Response =
        serde_json::from_str(string).map_err(|err| CloneableError(err.to_string()))?;
    return Ok(response);
}
