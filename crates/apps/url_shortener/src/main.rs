
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},