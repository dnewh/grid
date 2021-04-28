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

use super::ProductStoreOperations;

use crate::products::{
    store::{
        diesel::{
            models::{Product as ModelProduct, ProductPropertyValue},
            schema::{product, product_property_value},
        },
        error::ProductStoreError,
        Product, PropertyValue,
    },
    MAX_COMMIT_NUM,
};
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::products) trait GetProductOperation {
    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> GetProductOperation for ProductStoreOperations<'a, diesel::pg::PgConnection> {
    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = product::table
                .into_boxed()
                .select(product::all_columns)
                .filter(
                    product::product_id
                        .eq(product_id)
                        .and(product::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(product::service_id.eq(service_id));
            } else {
                query = query.filter(product::service_id.is_null());
            }

            let product = query
                .first(conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    ProductStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            match product {
                Some(product) => {
                    let root_values = product_property_value::table
                        .select(product_property_value::all_columns)
                        .filter(
                            product_property_value::product_id
                                .eq(product_id)
                                .and(product_property_value::parent_property.is_null())
                                .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .load::<ProductPropertyValue>(conn)?;

                    let mut values = Vec::new();

                    for root_value in root_values {
                        let children = product_property_value::table
                            .select(product_property_value::all_columns)
                            .filter(product_property_value::parent_property.eq(&root_value.parent_property))
                            .load(conn)?;

                        if children.is_empty() {
                            values.push(PropertyValue::from(root_value));
                        } else {
                            values.push(PropertyValue::from((
                                root_value,
                                get_property_values(conn, children)?,
                            )));
                        }
                    }

                    Ok(Some(Product::from((product, values))))
                }
                None => Ok(None),
            }
        })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> GetProductOperation for ProductStoreOperations<'a, diesel::sqlite::SqliteConnection> {
    fn get_product(
        &self,
        product_id: &str,
        service_id: Option<&str>,
    ) -> Result<Option<Product>, ProductStoreError> {
        self.conn.transaction::<_, PikeStoreError, _>(|| {
            let mut query = product::table
                .into_boxed()
                .select(product::all_columns)
                .filter(
                    product::product_id
                        .eq(product_id)
                        .and(product::end_commit_num.eq(MAX_COMMIT_NUM)),
                );

            if let Some(service_id) = service_id {
                query = query.filter(product::service_id.eq(service_id));
            } else {
                query = query.filter(product::service_id.is_null());
            }

            let product = query
                .first(conn)
                .map(Some)
                .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                .map_err(|err| {
                    ProductStoreError::InternalError(InternalError::from_source(Box::new(err)))
                })?;

            match product {
                Some(product) => {
                    let root_values = product_property_value::table
                        .select(product_property_value::all_columns)
                        .filter(
                            product_property_value::product_id
                                .eq(product_id)
                                .and(product_property_value::parent_property.is_null())
                                .and(product_property_value::end_commit_num.eq(MAX_COMMIT_NUM)),
                        )
                        .load::<ProductPropertyValue>(conn)?;

                    let mut values = Vec::new();

                    for root_value in root_values {
                        let children = product_property_value::table
                            .select(product_property_value::all_columns)
                            .filter(product_property_value::parent_property.eq(&root_value.parent_property))
                            .load(conn)?;

                        if children.is_empty() {
                            values.push(PropertyValue::from(root_value));
                        } else {
                            values.push(PropertyValue::from((
                                root_value,
                                get_property_values(conn, children)?,
                            )));
                        }
                    }

                    Ok(Some(Product::from((product, values))))
                }
                None => Ok(None),
            }
        })
    }
}
