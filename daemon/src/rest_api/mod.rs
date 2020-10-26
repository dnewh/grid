// Copyright 2019 Bitwise IO, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod error;
mod routes;

use std::sync::mpsc;
use std::thread;

use crate::config::Endpoint;
pub use crate::rest_api::error::RestApiServerError;
use crate::rest_api::routes::{
    BATCHES_MAX_VERSION, BATCHES_MIN_VERSION, fetch_agent, fetch_grid_schema, fetch_location, fetch_organization, fetch_product,
    fetch_record, fetch_record_property, get_batch_statuses, list_agents, list_grid_schemas,
    list_locations, list_organizations, list_products, list_records, submit_batches,
};

use crate::submitter::BatchSubmitter;
use actix::{Addr, SyncArbiter};
use actix_service::Service;
use actix_web::{
    dev,
    error::{Error as ActixError, ErrorBadRequest, ErrorInternalServerError}, dev::{ServiceResponse, ServiceRequest},
    web, App, FromRequest, HttpResponse, HttpRequest, HttpServer, Result,
};
use futures::executor::block_on;
use futures::future;
use futures::TryFutureExt;
use serde::{Deserialize, Serialize};

pub use self::routes::DbExecutor;

const SYNC_ARBITER_THREAD_COUNT: usize = 2;

pub struct AppState {
    batch_submitter: Box<dyn BatchSubmitter + 'static>,
    database_connection: Addr<DbExecutor>,
}

impl AppState {
    pub fn new(
        batch_submitter: Box<dyn BatchSubmitter + 'static>,
        db_executor: DbExecutor,
    ) -> Self {
        let database_connection =
            SyncArbiter::start(SYNC_ARBITER_THREAD_COUNT, move || db_executor.clone());

        AppState {
            batch_submitter,
            database_connection,
        }
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            batch_submitter: self.batch_submitter.clone(),
            database_connection: self.database_connection.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryServiceId {
    pub service_id: Option<String>,
}

pub struct AcceptServiceIdParam;

impl FromRequest for AcceptServiceIdParam {
    type Error = ActixError;
    type Future = future::Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let endpoint: Endpoint = if let Some(endpoint) = req.app_data::<Endpoint>() {
            endpoint.clone()
        } else {
            return future::err(ErrorInternalServerError("App state not found"));
        };

        let service_id =
            if let Ok(query) = web::Query::<QueryServiceId>::from_query(req.query_string()) {
                query.service_id.clone()
            } else {
                return future::err(ErrorBadRequest("Malformed query param"));
            };

        if service_id.is_some() && endpoint.is_sawtooth() {
            return future::err(ErrorBadRequest(
                "Circuit ID present, but grid is running in sawtooth mode",
            ));
        } else if service_id.is_none() && !endpoint.is_sawtooth() {
            return future::err(ErrorBadRequest(
                "Circuit ID is not present, but grid is running in splinter mode",
            ));
        }

        future::ok(AcceptServiceIdParam)
    }
}

pub struct RestApiShutdownHandle {
    server: dev::Server,
}

impl RestApiShutdownHandle {
    pub fn shutdown(&self) {
        block_on(self.server.stop(true));
    }
}

// pub struct Response(HttpResponse);

// impl From<HttpResponse> for Response {
//     fn from(res: HttpResponse) -> Self {
//         Self(res)
//     }
// }

// impl IntoFuture for Response {
//     type Item = HttpResponse;
//     type Error = ActixError;
//     type Future = FutureResult<HttpResponse, ActixError>;

//     fn into_future(self) -> Self::Future {
//         self.0.into_future()
//     }
// }

// impl std::fmt::Debug for Response {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f, "{:?}", self.0)
//     }
// }

/// A continuation indicates whether or not a guard should allow a given request to continue, or to
/// return a result.
// pub enum Continuation {
//     Continue,
//     Terminate(Box<dyn Future<Output = Result<ServiceResponse, ActixError>>>),
// }

// impl Continuation {
//     /// Wraps the given future in the Continuation::Terminate variant.
//     pub fn terminate<F>(fut: F) -> Continuation
//     where
//         F: Future<Output = Result<ServiceResponse, ActixError>> + 'static,
//     {
//         Continuation::Terminate(Box::new(fut))
//     }
// }

// /// A guard checks the request content in advance, and either continues the request, or
// /// returns a terminating result.
// pub trait RequestGuard: Send + Sync {
//     /// Evaluates the request and determines whether or not the request should be continued or
//     /// short-circuited with a terminating future.
//     fn evaluate(&self, req: &ServiceRequest, srv: &dyn Service) -> dyn Service;
// }

// // impl<F> RequestGuard for F
// // where
// //     F: Fn(&HttpRequest, &Service) -> Service + Sync + Send,
// // {
// //     fn evaluate(&self, req: &HttpRequest, srv: &Service) -> Service {
// //         (*self)(req, srv)
// //     }
// // }

// impl RequestGuard for Box<dyn RequestGuard> {
//     fn evaluate(&self, req: &ServiceRequest, srv: &dyn Service) -> dyn Service {
//         (**self).evaluate(req, srv)
//     }
// }

/// Guards requests based on a minimum protocol version.
///
/// A protocol version is specified via the HTTP header `"GridProtocolVersion"`.  This header
/// is a positive integer value.
#[derive(Clone)]
pub struct ProtocolVersionRangeGuard {
    min: u32,
    max: u32,
}

impl ProtocolVersionRangeGuard {
    /// Constructs a new protocol version guard with the given minimum.
    pub fn new(min: u32, max: u32) -> Self {
        Self { min, max }
    }
}

// impl RequestGuard for ProtocolVersionRangeGuard {
//     fn evaluate(&self, req: &ServiceRequest, srv: &dyn Service) -> dyn Service {
//         if let Some(header_value) = req.headers().get("GridProtocolVersion") {
//             let parsed_header = header_value
//                 .to_str()
//                 .map_err(|err| {
//                     format!(
//                         "Invalid characters in GridProtocolVersion header: {}",
//                         err
//                     )
//                 })
//                 .and_then(|val_str| {
//                     val_str.parse::<u32>().map_err(|_| {
//                         "GridProtocolVersion must be a valid positive integer".to_string()
//                     })
//                 });
//             match parsed_header {
//                 Err(msg) =>
//                     HttpResponse::BadRequest()
//                         .json(json!({
//                             "message": msg,
//                         }))
//                         .into_future(),
//                 Ok(version) if version < self.min =>
//                     HttpResponse::BadRequest()
//                         .json(json!({
//                             "message": format!(
//                                 "Client must support protocol version {} or greater.",
//                                 self.min,
//                             ),
//                             "requested_protocol": version,
//                             "grid_protocol": self.max,
//                             "gridd_version": format!(
//                                 "{}.{}.{}",
//                                 env!("CARGO_PKG_VERSION_MAJOR"),
//                                 env!("CARGO_PKG_VERSION_MINOR"),
//                                 env!("CARGO_PKG_VERSION_PATCH")
//                             )
//                         }))
//                         .into_future(),
//                 Ok(version) if version > self.max =>
//                     HttpResponse::BadRequest()
//                         .json(json!({
//                             "message": format!(
//                                 "Client requires a newer protocol than can be provided: {} > {}",
//                                 version,
//                                 self.max,
//                             ),
//                             "requested_protocol": version,
//                             "grid_protocol": self.max,
//                             "gridd_version": format!(
//                                 "{}.{}.{}",
//                                 env!("CARGO_PKG_VERSION_MAJOR"),
//                                 env!("CARGO_PKG_VERSION_MINOR"),
//                                 env!("CARGO_PKG_VERSION_PATCH")
//                             )
//                         }))
//                         .into_future(),
//                 Ok(_) => srv,
//             }
//         } else {
//             // Ignore the missing header, and assume the client will handle version mismatches by
//             // inspecting the output
//             srv
//         }
//     }
// }

// fn validate_protocol_version(
//     request: ServiceRequest,
//     service: dyn Service,
// ) -> dyn Service {
//     let accepted = match request.path() {
//         "/batches" => ProtocolVersionRangeGuard::new(
//             BATCHES_MIN_VERSION,
//             BATCHES_MAX_VERSION,
//         ),
//         _ => ProtocolVersionRangeGuard::new(
//             0,
//             u32::MAX
//         )
//     };

//     accepted.evaluate(&request, &service)
// }

pub fn run(
    bind_url: &str,
    db_executor: DbExecutor,
    batch_submitter: Box<dyn BatchSubmitter + 'static>,
    endpoint: Endpoint,
) -> Result<
    (
        RestApiShutdownHandle,
        thread::JoinHandle<Result<(), RestApiServerError>>,
    ),
    RestApiServerError,
> {
    let bind_url = bind_url.to_owned();
    let (tx, rx) = mpsc::channel();

    let join_handle = thread::Builder::new()
        .name("GridRestApi".into())
        .spawn(move || {
            let sys = actix::System::new("Grid-Rest-API");
            let state = AppState::new(batch_submitter, db_executor);

            let addr = HttpServer::new(move || {
                App::new()
                    .data(state.clone())
                    .app_data(endpoint.clone())
                    // .wrap_fn(|req, srv| { validate_protocol_version(req, srv) })
                    .wrap_fn(|req, srv| {
                        let accepted = match req.path() {
                            "/batches" => ProtocolVersionRangeGuard::new(
                                BATCHES_MIN_VERSION,
                                BATCHES_MAX_VERSION,
                            ),
                            _ => ProtocolVersionRangeGuard::new(
                                0,
                                u32::MAX
                            )
                        };

                        if let Some(header_value) = req.headers().get("GridProtocolVersion") {
                            let parsed_header = header_value
                                .to_str()
                                .map_err(|err| {
                                    format!(
                                        "Invalid characters in GridProtocolVersion header: {}",
                                        err
                                    )
                                })
                                .and_then(|val_str| {
                                    val_str.parse::<u32>().map_err(|_| {
                                        "GridProtocolVersion must be a valid positive integer".to_string()
                                    })
                                });
                            match parsed_header {
                                Err(msg) => {
                                    let fut = srv.call(req);
                                    async {
                                        let mut res = fut.await?;
                                        res = ServiceResponse::into_response(res, HttpResponse::BadRequest()
                                            .json(json!({
                                                "message": msg,
                                            }))
                                        );
                                        Ok(res)
                                    }
                                    // HttpResponse::BadRequest()
                                    //     .json(json!({
                                    //         "message": msg,
                                    //     })),
                                        // .into_future(),
                                }
                                Ok(version) if version < accepted.min => {
                                    let fut = srv.call(req);
                                    async {
                                        let mut res = fut.await?;
                                        res = ServiceResponse::into_response(res, HttpResponse::BadRequest()
                                            .json(json!({
                                                "message": format!(
                                                    "Client must support protocol version {} or greater.",
                                                    accepted.min,
                                                ),
                                                "requested_protocol": version,
                                                "grid_protocol": accepted.min,
                                                "gridd_version": format!(
                                                    "{}.{}.{}",
                                                    env!("CARGO_PKG_VERSION_MAJOR"),
                                                    env!("CARGO_PKG_VERSION_MINOR"),
                                                    env!("CARGO_PKG_VERSION_PATCH")
                                                )
                                            }))
                                        );
                                        Ok(res)
                                    }
                                }
                                    // HttpResponse::BadRequest()
                                    //     .json(json!({
                                    //         "message": format!(
                                    //             "Client must support protocol version {} or greater.",
                                    //             accepted.min,
                                    //         ),
                                    //         "requested_protocol": version,
                                    //         "grid_protocol": accepted.max,
                                    //         "gridd_version": format!(
                                    //             "{}.{}.{}",
                                    //             env!("CARGO_PKG_VERSION_MAJOR"),
                                    //             env!("CARGO_PKG_VERSION_MINOR"),
                                    //             env!("CARGO_PKG_VERSION_PATCH")
                                    //         )
                                    //     })),
                                        // .into_future(),
                                Ok(version) if version > accepted.max => {
                                    let fut = srv.call(req);
                                    async {
                                        let mut res = fut.await?;
                                        res = ServiceResponse::into_response(res, HttpResponse::BadRequest()
                                            .json(json!({
                                                "message": format!(
                                                    "Client must support protocol version {} or greater.",
                                                    accepted.max,
                                                ),
                                                "requested_protocol": version,
                                                "grid_protocol": accepted.max,
                                                "gridd_version": format!(
                                                    "{}.{}.{}",
                                                    env!("CARGO_PKG_VERSION_MAJOR"),
                                                    env!("CARGO_PKG_VERSION_MINOR"),
                                                    env!("CARGO_PKG_VERSION_PATCH")
                                                )
                                            }))
                                        );
                                        Ok(res)
                                    }
                                }
                                    // HttpResponse::BadRequest()
                                    //     .json(json!({
                                    //         "message": format!(
                                    //             "Client requires a newer protocol than can be provided: {} > {}",
                                    //             version,
                                    //             accepted.max,
                                    //         ),
                                    //         "requested_protocol": version,
                                    //         "grid_protocol": accepted.max,
                                    //         "gridd_version": format!(
                                    //             "{}.{}.{}",
                                    //             env!("CARGO_PKG_VERSION_MAJOR"),
                                    //             env!("CARGO_PKG_VERSION_MINOR"),
                                    //             env!("CARGO_PKG_VERSION_PATCH")
                                    //         )
                                    //     })),
                                        // .into_future(),
                                Ok(_) => srv.call(req),
                            }
                        } else {
                            // Ignore the missing header, and assume the client will handle version mismatches by
                            // inspecting the output
                            srv.call(req)
                        }
                    })
                    .service(web::resource("/batches")
                        .route(web::post()
                        .to(submit_batches)))
                    .service(
                        web::resource("/batch_statuses")
                            .name("batch_statuses")
                            .route(web::get().to(get_batch_statuses)),
                    )
                    .service(
                        web::scope("/agent")
                            .service(web::resource("").route(web::get().to(list_agents)))
                            .service(
                                web::resource("/{public_key}").route(web::get().to(fetch_agent)),
                            ),
                    )
                    .service(
                        web::scope("/location")
                            .service(web::resource("").route(web::get().to(list_locations)))
                            .service(web::resource("/{id}").route(web::get().to(fetch_location))),
                    )
                    .service(
                        web::scope("/organization")
                            .service(web::resource("").route(web::get().to(list_organizations)))
                            .service(
                                web::resource("/{id}").route(web::get().to(fetch_organization)),
                            ),
                    )
                    .service(
                        web::scope("/product")
                            .service(web::resource("").route(web::get().to(list_products)))
                            .service(web::resource("/{id}").route(web::get().to(fetch_product))),
                    )
                    .service(
                        web::scope("/schema")
                            .service(web::resource("").route(web::get().to(list_grid_schemas)))
                            .service(
                                web::resource("/{name}").route(web::get().to(fetch_grid_schema)),
                            ),
                    )
                    .service(
                        web::scope("/record")
                            .service(web::resource("").route(web::get().to(list_records)))
                            .service(
                                web::scope("/{record_id}")
                                    .service(web::resource("").route(web::get().to(fetch_record)))
                                    .service(
                                        web::resource("/property/{property_name}")
                                            .route(web::get().to(fetch_record_property)),
                                    ),
                            ),
                    )
            })
            .bind(bind_url)?
            .disable_signals()
            .system_exit()
            .run();

            tx.send(addr).map_err(|err| {
                RestApiServerError::StartUpError(format!("Unable to send Server Addr: {}", err))
            })?;
            sys.run()?;

            info!("Rest API terminating");

            Ok(())
        })?;

    let server = rx.recv().map_err(|err| {
        RestApiServerError::StartUpError(format!("Unable to receive Server Addr: {}", err))
    })?;

    Ok((RestApiShutdownHandle { server }, join_handle))
}
