use crate::mongo::get_current_listings;
use crate::web::State;
use crate::ws::WsApiClient;
use std::convert::Infallible;
use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::http::StatusCode;
use warp::{Filter, Reply};

pub fn api(state: Arc<State>) -> BoxedFilter<(impl Reply,)> {
    warp::path("api")
        .and(ws(state.clone()).or(listings(state.clone())))
        .boxed()
}

fn listings(state: Arc<State>) -> BoxedFilter<(impl Reply,)> {
    async fn logic(state: Arc<State>) -> Result<warp::reply::Response, Infallible> {
        let listings = get_current_listings(state.collection()).await;

        match listings {
            Ok(listings) => Ok(warp::reply::json(&listings).into_response()),
            Err(_) => Ok(warp::reply::with_status(warp::reply(), StatusCode::INTERNAL_SERVER_ERROR).into_response())
        }
    }

    warp::get()
        .and(warp::path("listings"))
        .and(warp::path::end())
        .and_then(move || logic(state.clone()))
        .boxed()
}

fn ws(state: Arc<State>) -> BoxedFilter<(impl Reply,)> {
    let route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::end())
        .map(move |ws: warp::ws::Ws| {
            let state = Arc::clone(&state);
            ws.on_upgrade(move |websocket| async move {
                WsApiClient::run(state, websocket).await;
            })
        });

    warp::get().and(route).boxed()
}
