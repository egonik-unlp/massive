use std::{collections::HashMap, fmt::Display};

use leptos::either::Either;
use leptos::logging::log;
use leptos::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageInference {
    pub ingredients: Vec<String>,
    pub inferred_category: Category,
    pub inferred_product_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Category {
    BPC,
    Foods,
    Home,
}
impl Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Category::BPC => "BPC",
            Category::Foods => "Foods",
            Category::Home => "Home",
        };
        write!(f, "{}", s)
    }
}

// In qdrant_client: type Payload = HashMap<String, serde_json::Value>;
pub type Payload = HashMap<String, Value>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ingredients {
    pub id: i32,
    pub actual_name: String,
    pub cita: String,
    pub info_para_reporte: String,

    // Accepts ["a","b"] or "a, b" or null/missing → []
    #[serde(default, deserialize_with = "de_vec_string")]
    pub sinonimos: Vec<String>,

    // Accepts 5.1 or "5.1"
    #[serde(deserialize_with = "de_f32_from_str_or_num")]
    pub total_risk: f32,
}

impl TryFrom<Payload> for Ingredients {
    type Error = anyhow::Error;

    fn try_from(payload: Payload) -> Result<Self, Self::Error> {
        // Convert the map into a Value, then into the struct
        let v = serde_json::to_value(payload)?;
        let mut ing: Ingredients = serde_json::from_value(v)?;

        // Optional: normalize, sort & dedup synonyms
        ing.sinonimos = ing
            .sinonimos
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        ing.sinonimos.sort_unstable();
        ing.sinonimos.dedup();

        Ok(ing)
    }
}

// --- helpers ---

fn de_vec_string<'de, D>(d: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(d)?;
    match v {
        Value::Null => Ok(vec![]),
        Value::String(s) => Ok(s
            .split(',')
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect()),
        Value::Array(arr) => Ok(arr
            .into_iter()
            .filter_map(|x| match x {
                Value::String(s) => {
                    let s = s.trim().to_string();
                    (!s.is_empty()).then_some(s)
                }
                _ => None,
            })
            .collect()),
        _ => Ok(vec![]), // or: Err(D::Error::custom("expected string/array/null"))
    }
}

fn de_f32_from_str_or_num<'de, D>(d: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let v = Value::deserialize(d)?;
    match v {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| serde::de::Error::custom("non-finite number"))
            .map(|f| f as f32),
        Value::String(s) => s.trim().parse::<f32>().map_err(serde::de::Error::custom),
        Value::Null => Ok(0.0), // or choose a default strategy
        _ => Err(serde::de::Error::custom("expected number or string")),
    }
}
// #[derive(Debug, Serialize, Clone, Deserialize, Default)]
// pub struct Ingredients {
//     pub id: i32,
//
//     pub score: f32,
//     pub actual_name: String,
//     pub info_para_reporte: String,
//     pub synonyms: Vec<String>,
//     pub cita: String,
// }
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
    ingredients: Vec<Ingredients>,
    url: String,
    score: Resource<Result<f32, ServerFnError>>,
) -> impl IntoView {
    //  TDOO: use read signal
    let (ing_value, _ing_set_value) = signal(ingredients);
    let has_field_populated = move || ing_value.get().len() != 0;
    view! {
        {move || {
            if has_field_populated() {
                Either::Left(view! {
                    <section class="mt-6 grid grid-cols-1 md:grid-cols-5 gap-6">
                        <div class="md:col-span-2 flex items-start justify-center">
                            <img class="w-full max-w-sm rounded-xl shadow-md ring-1 ring-black/10 object-contain" src=url.clone() alt="Uploaded image"/>
                        </div>
                        <div class="md:col-span-3 space-y-3">
                            <div class="flex items-center gap-3">
                                <span class="inline-flex items-center rounded-full bg-teal-100 text-teal-800 px-3 py-1 text-sm font-medium">
                                    Score: {match score.get() {
                                        Some(Ok(score)) => { log!("Score = {}", score); score }
                                        Some(Err(e)) => { log!("Error path {}", e); 0f32 }
                                        None => { log!("None path"); 0f32 }
                                    }}
                                </span>
                                <span class="text-sm text-slate-600">Detected {ing_value.get().len()} ingredients</span>
                            </div>
                            <Accordion response=ing_value.get() />
                        </div>
                    </section>
                })
            } else {
                Either::Right(view! { <p class="text-slate-500">Awaiting image...</p> })
            }
        }}
    }
}

#[component]
fn Accordion(response: Vec<Ingredients>) -> impl IntoView {
    view! {
        <div class="space-y-3">
            {response
                .into_iter()
                .enumerate()
                .map(|(n, ingredient)| {
                    let risk_color = match ingredient.total_risk as u32 {
                        0 => "bg-purple-100 text-purple-800",
                        1 => "bg-amber-100 text-amber-800",
                        3 => "bg-green-100 text-green-800",
                        _ => "bg-red-100 text-red-800",
                    };
                    view! {
                        <AccordionNode name_of_child=ingredient.clone().actual_name position=n>
                            <div class="space-y-2 text-sm leading-relaxed">
                                <p class="text-slate-700">{ingredient.info_para_reporte}</p>
                                <p class="inline-flex items-center px-2 py-1 rounded-full font-medium" >
                                    {format!("Risk score: {}", ingredient.total_risk)}
                                </p>
                            </div>
                        </AccordionNode>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
fn AccordionNode(
    children: ChildrenFragment,
    name_of_child: String,
    _position: usize,
) -> impl IntoView {
    let children = children().nodes.into_iter().map(|child| {
        view! {
            <details class="group rounded-xl border border-slate-200 bg-white/80 p-4 shadow-sm hover:shadow-md transition-shadow">
                <summary class="flex cursor-pointer list-none items-center justify-between gap-3">
                    <span class="font-medium text-slate-800">{name_of_child.clone()}</span>
                    <svg class="h-4 w-4 text-slate-500 transition-transform group-open:rotate-180" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                        <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 10.939l3.71-3.71a.75.75 0 111.06 1.061l-4.24 4.24a.75.75 0 01-1.06 0l-4.24-4.24a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
                    </svg>
                </summary>
                <div class="mt-3 border-t pt-3 text-slate-700">
                    {child}
                </div>
            </details>
        }
    }).collect_view();
    children
}

async fn mock_response() -> Result<Response, CloneableError> {
    let string = include_str!("../../sample_response.json");
    let response: Response =
        serde_json::from_str(string).map_err(|err| CloneableError(err.to_string()))?;
    return Ok(response);
}
