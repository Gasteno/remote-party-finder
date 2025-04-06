use std::sync::Arc;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};
use crate::web::State;
use crate::ws::WsApiClient;

pub fn api(state: Arc<State>) -> BoxedFilter<(impl Reply, )> {
    warp::path("api")
        .and(
            ws(state.clone())
        )
        .boxed()
}

// fn listings(state: Arc<State>) -> BoxedFilter<(impl Reply, )> {
    // warp::get()
    //     .and(warp::path("listings"))
    //     .and_then(async || anyhow::Result::Ok("hi".into()))
    //     .boxed()
// }

fn ws(state: Arc<State>) -> BoxedFilter<(impl Reply,)> {
    let route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let state = Arc::clone(&state);
            ws.on_upgrade(move |websocket| {
                async move {
                    WsApiClient::run(state, websocket).await;
                }
            })
        });

    warp::get().and(route).boxed()
}
