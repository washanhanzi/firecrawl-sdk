pub mod controller;

use axum::{Router, http::StatusCode, routing::post};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
