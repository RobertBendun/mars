#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut state = App {
        ratings: load_personal_ratings("ratings.csv")?,
        titles: load_titles("title.tsv")?,
        ratings_dist: [0.0; 10],
    };

    {
        for rating in &state.ratings {
            state.ratings_dist[rating.rating-1] += 1.0;
        }

        let sum: f32 = state.ratings_dist.iter().sum();
        for rating in &mut state.ratings_dist {
            *rating /= sum;
        }
    }

    let app = axum::Router::new().route("/", get(index)).with_state(Arc::new(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

struct App {
    ratings: Vec<Rating>,
    titles: HashMap<String, Vec<String>>,
    ratings_dist: [f32; 10],
}

#[derive(Deserialize)]
struct Filter {
    rating: Option<String>,
}

#[derive(Debug, Clone)]
struct Rating {
    id: String,
    rating: usize,
}

fn load_titles<P: AsRef<std::path::Path>>(path: P) -> Result<HashMap<String, Vec<String>>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b'\t')
        .quoting(false)
        .from_path(path)?;

    let mut map: HashMap<String, Vec<String>> = HashMap::new();

    for row in reader.records().flatten() {
        let id = &row[0];
        let title = row[1].to_string();
        if let Some(titles) = map.get_mut(id) {
            titles.push(title);
        } else {
            map.insert(id.to_string(), vec![title]);
        }
    }

    Ok(map)
}

fn load_personal_ratings<P: AsRef<std::path::Path>>(path: P) -> Result<Vec<Rating>> {
    let mut ratings: Vec<_> = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?
        .records()
        .flatten()
        .map(|record| Rating {
            id: record[0].to_string(),
            rating: record[1].parse::<usize>().unwrap(),
        })
        .collect();
    ratings.sort_by(|r1, r2| r1.rating.cmp(&r2.rating).reverse());
    Ok(ratings)
}

async fn index(Query(filter): Query<Filter>, State(app): State<Arc<App>>) -> Markup {
    let valid_ratings: Vec<usize> = if let Some(ratings) = filter.rating {
        ratings
            .split(",")
            .flat_map(|rating| rating.parse::<usize>())
            .filter(|rating| *rating >= 1 && *rating <= 10)
            .collect()
    } else {
        vec![]
    };

    let ratings = app
        .ratings
        .iter()
        .filter(|rating| valid_ratings.is_empty() || valid_ratings.contains(&rating.rating))
        .map(|rating| app.titles.get(&rating.id).map(|x| (&x[0], rating.rating)))
        .flatten();

    html! {
        (DOCTYPE)
        html {
            head {
                title { "MARS - rate your movies and shows" }
                style {
                    r#"
                        :root {
                            --text: #3d3d3d;
                            --background: #c56cf0;
                            --selection: #3ae374;
                            --link: #17c0eb;
                        }

                        a, a:visited, a:link {
                            color: var(--link);
                        }

                        body {
                            font-family: sans-serif;
                            color: var(--text);
                            background-color: var(--background);
                        }

                        ::selection {
                            background-color: var(--selection);
                        }

                        header {
                            text-align: center;
                            letter-spacing: 0.5em;
                            background-color: #67e6dc;
                            color: #c56cf0;
                            margin: 1em auto 1em auto;
                            width: 90vw;

                            display: grid;
                            grid-template-columns: 4em auto 4em;
                        }

                        article {
                            display: grid;
                            gap: 1em;
                            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
                        }

                        article section {
                            text-align: center;
                            background-color: #fff;
                            box-shadow: 0 4px 8px 0 rgba(0,0,0,0.2);
                            padding: 1em;
                            display: grid;
                            grid-template-columns: 1fr;
                            grid-template-rows: 4em 2em;
                            transition: box-shadow 300ms ease-in, background-color 300ms ease-in;
                            background-color: #e6e6e6;
                        }

                        article section:hover {
                            transition: box-shadow 300ms ease-out, background-color 300ms ease-out;
                            box-shadow: 0 8px 32px 0 rgba(0,0,0,0.4);
                            background-color: white;
                        }

                        #occurance {
                            display: grid;
                            grid-template-columns: repeat(2, 1fr);
                            padding: 0.5em;
                        }
                    "#
                }
            }
            body {
                header {
                    div #occurance {
                        @for d in app.ratings_dist {
                            div style=(format!("background-color: #c56cf0{alpha:x}", alpha = (d.sqrt() * 255.0).floor() as usize)) {
                            }
                        }
                    }
                    h1 {
                        "MARS"
                    }
                }
                article {
                    @for rating in ratings {
                        section {
                            h3 {
                                (rating.0)
                            }
                            p {
                                (rating.1)
                            }
                        }
                    }
                }
                script {
                    (PreEscaped(include_str!("index.js")))
                }
            }
        }
    }
}

use anyhow::Result;
use axum::extract::Query;
use axum::extract::State;
use axum::routing::get;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
