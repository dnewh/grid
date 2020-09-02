// Copyright 2019 Cargill Incorporated
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

use crate::database::{
    helpers as db,
    models::{GridPropertyDefinition, GridSchema},
};
use crate::rest_api::{
    error::RestApiResponseError, routes::DbExecutor, AcceptServiceIdParam, AppState, QueryServiceId,
};

use actix::{Handler, Message, SyncContext};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaSlice {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub properties: Vec<GridPropertyDefinitionSlice>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl GridSchemaSlice {
    pub fn from_schema(schema: &GridSchema, properties: Vec<GridPropertyDefinition>) -> Self {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            owner: schema.owner.clone(),
            properties: properties
                .iter()
                .map(|prop| GridPropertyDefinitionSlice::from_definition(prop))
                .collect(),
            service_id: schema.service_id.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridPropertyDefinitionSlice {
    pub name: String,
    pub schema_name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
}

impl GridPropertyDefinitionSlice {
    pub fn from_definition(definition: &GridPropertyDefinition) -> Self {
        Self {
            name: definition.name.clone(),
            schema_name: definition.schema_name.clone(),
            data_type: definition.data_type.clone(),
            required: definition.required,
            description: definition.description.clone(),
            number_exponent: definition.number_exponent,
            enum_options: definition.enum_options.clone(),
            struct_properties: definition.struct_properties.clone(),
            service_id: definition.service_id.clone(),
        }
    }
}

struct ListGridSchemas {
    service_id: Option<String>,
}

impl Message for ListGridSchemas {
    type Result = Result<Vec<GridSchemaSlice>, RestApiResponseError>;
}

#[cfg(feature = "postgres")]
impl Handler<ListGridSchemas> for DbExecutor<diesel::pg::PgConnection> {
    type Result = Result<Vec<GridSchemaSlice>, RestApiResponseError>;

    fn handle(&mut self, msg: ListGridSchemas, _: &mut SyncContext<Self>) -> Self::Result {
        let mut properties = db::list_grid_property_definitions(
            &*self.connection_pool.get()?,
            msg.service_id.as_deref(),
        )?
        .into_iter()
        .fold(HashMap::new(), |mut acc, definition| {
            acc.entry(definition.schema_name.to_string())
                .or_insert_with(Vec::new)
                .push(definition);
            acc
        });

        let fetched_schemas =
            db::list_grid_schemas(&*self.connection_pool.get()?, msg.service_id.as_deref())?
                .iter()
                .map(|schema| {
                    GridSchemaSlice::from_schema(
                        schema,
                        properties.remove(&schema.name).unwrap_or_else(Vec::new),
                    )
                })
                .collect();
        Ok(fetched_schemas)
    }
}

#[cfg(feature = "postgres")]
pub async fn list_grid_schemas(
    state: web::Data<AppState<diesel::pg::PgConnection>>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(ListGridSchemas {
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|schemas| HttpResponse::Ok().json(schemas))
}

struct FetchGridSchema {
    name: String,
    service_id: Option<String>,
}

impl Message for FetchGridSchema {
    type Result = Result<GridSchemaSlice, RestApiResponseError>;
}

#[cfg(feature = "postgres")]
impl Handler<FetchGridSchema> for DbExecutor<diesel::pg::PgConnection> {
    type Result = Result<GridSchemaSlice, RestApiResponseError>;

    fn handle(&mut self, msg: FetchGridSchema, _: &mut SyncContext<Self>) -> Self::Result {
        let properties = db::list_grid_property_definitions_with_schema_name(
            &*self.connection_pool.get()?,
            &msg.name,
            msg.service_id.as_deref(),
        )?;
        let fetched_schema = match db::fetch_grid_schema(
            &*self.connection_pool.get()?,
            &msg.name,
            msg.service_id.as_deref(),
        )? {
            Some(schema) => GridSchemaSlice::from_schema(&schema, properties),
            None => {
                return Err(RestApiResponseError::NotFoundError(format!(
                    "Could not find schema with name: {}",
                    msg.name
                )));
            }
        };

        Ok(fetched_schema)
    }
}

#[cfg(feature = "postgres")]
pub async fn fetch_grid_schema(
    state: web::Data<AppState<diesel::pg::PgConnection>>,
    schema_name: web::Path<String>,
    query: web::Query<QueryServiceId>,
    _: AcceptServiceIdParam,
) -> Result<HttpResponse, RestApiResponseError> {
    state
        .database_connection
        .send(FetchGridSchema {
            name: schema_name.into_inner(),
            service_id: query.into_inner().service_id,
        })
        .await?
        .map(|schema| HttpResponse::Ok().json(schema))
}
