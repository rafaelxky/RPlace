use std::sync::Arc;

use anyhow::Ok;
use anyhow::Result;
use axum::Router;

use crate::db::db_provider::Repo;
use crate::routes;
use crate::{
    models::app_state::{AppStateBuilder},
};



pub async fn app(repo: Arc<dyn Repo>) -> Result<Router>{
    let state = AppStateBuilder::new().db_provider(repo).build()?;
    let app = Router::new().merge(routes::router()).with_state(state);
    Ok(app)
}