mod api;
mod joke;
mod jokebase;
mod startup;
mod web;

use api::*;
use joke::*;
use jokebase::*;
use startup::*;
use web::*;

use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use clap::Parser;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};
extern crate serde_json;
use sqlx::{self, Pool, Row, postgres::{Postgres, PgPool, PgRow}};
extern crate thiserror;
use tokio::{self, sync::RwLock};
use tower_http::{services, trace};
extern crate tracing;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{
    openapi::schema::{ObjectBuilder, Schema, SchemaType},
    openapi::RefOr,
    OpenApi, ToSchema,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

const STYLESHEET: &str = "assets/static/knock-knock.css";

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Args {
    #[clap(short, long, default_value = "0.0.0.0:3000")]
    serve: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    startup(args.serve).await
}
