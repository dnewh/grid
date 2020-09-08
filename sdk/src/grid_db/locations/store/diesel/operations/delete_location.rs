// Copyright 2018-2020 Cargill Incorporated
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

use super::LocationStoreOperations;
use crate::grid_db::locations::store::diesel::{
    schema::{location, location_attribute},
    LocationStoreError,
};

use crate::grid_db::locations::store::diesel::models::LocationModel;
use diesel::{dsl::delete, prelude::*, result::Error::NotFound};

pub(in crate::grid_db::locations) trait LocationStoreDeleteLocationOperation {
    fn delete_location(
        &self,
        location_id: &str,
        service_id: Option<String>,
    ) -> Result<(), LocationStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> LocationStoreDeleteLocationOperation
    for LocationStoreOperations<'a, diesel::pg::PgConnection>
{
    fn delete_location(
        &self,
        location_id: &str,
        service_id: Option<String>,
    ) -> Result<(), LocationStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, LocationStoreError, _>(|| {
                let loc = location::table
                    .filter(location::location_id.eq(&location_id))
                    .filter(location::service_id.eq(&service_id))
                    .first::<LocationModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| LocationStoreError::QueryError {
                        context: "Failed check for existing location".to_string(),
                        source: Box::new(err),
                    })?;

                if loc.is_none() {
                    return Err(LocationStoreError::NotFoundError(format!(
                        "Failed to find location: {}",
                        &location_id
                    )));
                }

                delete(
                    location::table
                        .filter(location::location_id.eq(&location_id))
                        .filter(location::service_id.eq(&service_id)),
                )
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| LocationStoreError::OperationError {
                    context: "Failed to delete location".to_string(),
                    source: Some(Box::new(err)),
                })?;

                delete(
                    location_attribute::table
                        .filter(location_attribute::location_id.eq(&location_id))
                        .filter(location_attribute::service_id.eq(&service_id)),
                )
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| LocationStoreError::OperationError {
                    context: "Failed to delete location attributes".to_string(),
                    source: Some(Box::new(err)),
                })?;

                Ok(())
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> LocationStoreDeleteLocationOperation
    for LocationStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn delete_location(
        &self,
        location_id: &str,
        service_id: Option<String>,
    ) -> Result<(), LocationStoreError> {
        self.conn
            .immediate_transaction::<_, LocationStoreError, _>(|| {
                let loc = location::table
                    .filter(location::location_id.eq(&location_id))
                    .filter(location::service_id.eq(&service_id))
                    .first::<LocationModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| LocationStoreError::QueryError {
                        context: "Failed check for existing location".to_string(),
                        source: Box::new(err),
                    })?;

                if loc.is_none() {
                    return Err(LocationStoreError::NotFoundError(format!(
                        "Failed to find location: {}",
                        &location_id
                    )));
                }

                delete(
                    location::table
                        .filter(location::location_id.eq(&location_id))
                        .filter(location::service_id.eq(&service_id)),
                )
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| LocationStoreError::OperationError {
                    context: "Failed to delete location".to_string(),
                    source: Some(Box::new(err)),
                })?;

                delete(
                    location_attribute::table
                        .filter(location_attribute::location_id.eq(&location_id))
                        .filter(location_attribute::service_id.eq(&service_id)),
                )
                .execute(self.conn)
                .map(|_| ())
                .map_err(|err| LocationStoreError::OperationError {
                    context: "Failed to delete location attributes".to_string(),
                    source: Some(Box::new(err)),
                })?;

                Ok(())
            })
    }
}
