use crate::components::upload_and_display::{ImageInference, IngredientsCard};
use crate::openai::server::server_api;
use crate::otro_upload;
use crate::otro_upload::{image_upload_to_server, upload_to_bucket};
use leptos::logging::log;
use leptos::{logging, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use reactive_stores::Store;
use server_fn::codec::MultipartData;
use thaw::{
    ConfigProvider, FileList, Toast, ToastBody, ToasterInjection, ToasterProvider, Upload,
    UploadDragger,
};
use web_sys::FormData;

#[derive(Clone, Debug, Default, Store)]
struct GlobalState {
    ingredientes: Option<Vec<i32>>,
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>

            </head>
            <body class="min-h-screen bg-gradient-to-br from-teal-50 to-slate-100 text-slate-800">
                <App/>
            </body>
        </html>
    }
}

#[server]
pub async fn cualquier_verga() -> Result<String, ServerFnError> {
    let st = include_str!("openai/server/text.txt");
    let ret: ImageInference =
        serde_json::from_str(st).map_err(|err| ServerFnError::new(err.to_string()))?;
    let rv = format!("{:?}", ret);
    Ok(rv)
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    provide_context(Store::new(GlobalState::default()));
    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/massive.css"/>
        // sets the document title
        <Title text="GVAPP"/>
        // content for this welcome page
        <Router>
            <main class="mx-auto max-w-6xl p-6 md:p-10">
                {move || view! {
                    <Routes fallback=move || "Page not found.">
                        <Route path=StaticSegment("") view=HomePage/>
                    </Routes>
                }}
            </main>
        </Router>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let (v, sv) = signal("".to_owned());
    view! {
        <header class="flex items-center justify-between rounded-2xl bg-white/70 shadow-sm ring-1 ring-black/5 p-4 md:p-6 mb-6">
            <div class="flex items-center gap-4">
                <img class="h-10 w-10 md:h-14 md:w-14 object-contain" src="/favicon.ico" alt="Good Vibes"/>
                <div>
                    <h1 class="text-xl md:text-2xl font-bold text-teal-800">Ingredient Recognition Prototype</h1>
                    <p class="text-sm text-slate-600">Upload a product image to extract ingredients</p>
                </div>
            </div>
        </header>

        <section class="grid grid-cols-1 gap-6">
            <div class="rounded-2xl bg-white shadow-md ring-1 ring-black/5 p-5 md:p-8">
                <ConfigProvider>
                    <ToasterProvider>
                        <UploadWidget />
                    </ToasterProvider>
                </ConfigProvider>
            </div>
        </section>
    }
}

#[component]
pub fn UploadWidget() -> impl IntoView {
    let (filename, filename_set) = signal("".to_owned());
    let toaster = ToasterInjection::expect_context();
    let custom_request = move |file_list: FileList| {
        let len = file_list.length();
        let mut filenames = vec![];
        let mut form_data = FormData::new().unwrap();
        for file_index in 0..len {
            if let Some(file) = file_list.get(file_index) {
                filenames.push(file.name());
                form_data
                    .append_with_blob_and_filename("image", &file, file.name().as_str())
                    .expect("Problemas con archiiis");
            }
        }
        logging::log!("{:?}", file_list);
        logging::log!("{:?}", form_data);
        toaster.dispatch_toast(
            move || {
                view! {
                    <Toast>
                        <ToastBody> { format!("Uploaded files {:?}", filenames) }</ToastBody>
                    </Toast>
                }
            },
            Default::default(),
        );
        let local_action = Action::new_local(|data: &FormData| {
            let data: MultipartData = data.to_owned().into();
            async move {
                let begin = web_time::Instant::now();
                let path_name = image_upload_to_server(data).await.unwrap();
                let timedelta = (begin - web_time::Instant::now()).as_millis();
                let remote_url = upload_to_bucket(path_name.clone())
                    .await
                    .expect("Error en fe uplading");
                println!("[PRINT] Successfully saved uploaded file as {}", remote_url);
                println!("[PRINT] time around upload = {} ", timedelta);
                logging::log!("[LOG] Successfully saved uploaded file as {}", path_name);
                logging::log!("[LOG] time around upload = {} ms", timedelta);
                remote_url
            }
        });
        let _re = local_action.dispatch_local(form_data);
        let path = local_action.value().get();
        Effect::new(move |_| {
            if let Some(value) = local_action.value().get() {
                log!("Valor de path = {}", value);
                filename_set.set(value);
            }
        });
    };
    view! {
        <div class="space-y-4">
            <div class="rounded-xl border-2 border-dashed border-teal-300/60 bg-teal-50/40 p-6 text-center hover:border-teal-400 transition-colors">
                <Upload custom_request>
                    <UploadDragger>
                        <span class="text-sm text-slate-600">Click or drag a file to upload</span>
                    </UploadDragger>
                </Upload>
            </div>

            <Show
                when=move || {filename.get() != ""}
                fallback= || view!{ <p class="text-slate-500">Waiting for an image...</p> }
            >
                <FetchHandler image_path=filename />
                <p class="text-xs text-slate-500">{filename}</p>
            </Show>
        </div>
    }
}

#[component]
fn FetchHandler(image_path: ReadSignal<String>) -> impl IntoView {
    let (score, set_score) = signal(0f32);
    let data = LocalResource::new(move || {
        let reactive_url = image_path.get();
        log!(
            "Entre al fetch handler. El valor de image_path es {}",
            reactive_url
        );
        async move {
            let inference = server_api::get_ingredients_from_image(reactive_url)
                .await
                .unwrap();
            (
                otro_upload::search_in_vector_database(inference.clone()).await,
                inference.inferred_category,
            )
        }
    });
    let ids_memo = Memo::new(move |_| {
        data.get().and_then(|res| {
            res.0
                .ok()
                .map(|value| value.iter().map(|ing| ing.id).collect::<Vec<i32>>())
        })
    });
    let score_res = Resource::new(
        move || ids_memo.get(),
        |maybe_ids| async move {
            match maybe_ids {
                Some(ids) if !ids.is_empty() => get_scores_for_ing(ids).await,
                _ => Ok(0.0),
            }
        },
    );

    view! {
        <Transition fallback=move || {
            view! {
                <div class="flex items-center justify-center p-6">
                    <svg class="h-6 w-6 animate-spin text-teal-600" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" fill="none"></circle>
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"></path>
                    </svg>
                </div>
            }
        }>
            
            {move || {
                data.get()
                    .map(|(inner, cat)| {
                        match inner {
                            Ok(value) => {
                        log!("Entered dibujo with value");
                                let ids: Vec<i32> = value.clone().into_iter().map(|ing|ing.id).collect();
                        log!("Pre spawn local");
                                view! {
                                    <>
                                        <div class="mb-3">
                                            <span class="inline-flex items-center rounded-full bg-slate-100 px-3 py-1 text-xs font-medium text-slate-700">
                                                {cat.to_string()}
                                            </span>
                                        </div>
                                        <IngredientsCard
                                            ingredients=value.to_owned()
                                            score=score_res
                                            url=image_path.get()
                                        />
                                    </>
                                }
                                    .into_any()
                            }
                            Err(_err) => {
                                log!("[ERROR] No se pudo responder {:?}", _err);
                                view! {
                                    <>
                                        <div class="error">
                                            <p>"Error haciendo solicitud "</p>
                                        </div>
                                    </>
                                }
                                    .into_any()
                            }
                        }
                    })
            }}
        </Transition>
    }
}

#[cfg(feature = "ssr")]
pub mod request_types {
    use serde::{Deserialize, Serialize};
    #[derive(Debug, Deserialize, Serialize)]
    pub struct ProductScore {
        pub score: f32,
    }
    #[derive(Serialize)]
    pub struct ScoreRequest {
        pub ingredients: Vec<i32>,
    }
}

#[server]
pub async fn get_scores_for_ing(ids: Vec<i32>) -> Result<f32, ServerFnError> {
    use request_types::{ProductScore, ScoreRequest};
    let client = Client::new();
    let score_req = ScoreRequest { ingredients: ids };
    use reqwest::Client;
    let score_string = client
        .post("https://api.goodvibes.work.gd/score_product")
        .json(&score_req)
        .send()
        .await?
        .text()
        .await?;
    let sco: ProductScore = serde_json::from_str(&score_string).unwrap();
    println!("[PRINT] Deserializado quedó así {:?}", sco);
    log!("[LOG] Deserializado quedó así {:?}", sco);
    println!(
        "[PRINT] Score calculado en la funcion de scores como string= {}",
        score_string
    );
    log!(
        "[LOG] Score calculado en la funcion de scores como string = {}",
        score_string
    );
    return Ok(sco.score);
}
