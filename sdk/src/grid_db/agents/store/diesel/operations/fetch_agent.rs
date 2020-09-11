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

use super::AgentStoreOperations;
use crate::grid_db::agents::store::diesel::{
    schema::{agent, role},
    Agent, AgentStoreError,
};

use crate::grid_db::agents::store::diesel::models::{AgentModel, RoleModel};
use crate::grid_db::commits::MAX_COMMIT_NUM;
use diesel::{prelude::*, result::Error::NotFound};

pub(in crate::grid_db::agents::store::diesel) trait AgentStoreFetchAgentOperation {
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<String>,
    ) -> Result<Option<Agent>, AgentStoreError>;
}

#[cfg(feature = "postgres")]
impl<'a> AgentStoreFetchAgentOperation for AgentStoreOperations<'a, diesel::pg::PgConnection> {
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<String>,
    ) -> Result<Option<Agent>, AgentStoreError> {
        self.conn
            .build_transaction()
            .read_write()
            .run::<_, AgentStoreError, _>(|| {
                let agent = agent::table
                    .filter(
                        agent::public_key
                            .eq(&pub_key)
                            .and(agent::service_id.eq(&service_id))
                            .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .first::<AgentModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| AgentStoreError::QueryError {
                        context: "Failed to fetch agent for pub_key".to_string(),
                        source: Box::new(err),
                    })?
                    .ok_or_else(|| {
                        AgentStoreError::NotFoundError(
                            format!("Failed to find agent: {}", pub_key,),
                        )
                    })?;

                let roles = role::table
                    .select(role::all_columns)
                    .filter(
                        role::public_key
                            .eq(&pub_key)
                            .and(role::service_id.eq(&service_id))
                            .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .load::<RoleModel>(self.conn)
                    .map(Some)
                    .map_err(|err| AgentStoreError::OperationError {
                        context: "Failed to fetch roles".to_string(),
                        source: Some(Box::new(err)),
                    })?
                    .ok_or_else(|| {
                        AgentStoreError::NotFoundError(
                            "Could not get all roles from storage".to_string(),
                        )
                    })?;

                Ok(Some(Agent::from_models(agent, roles)))
            })
    }
}

#[cfg(feature = "sqlite")]
impl<'a> AgentStoreFetchAgentOperation
    for AgentStoreOperations<'a, diesel::sqlite::SqliteConnection>
{
    fn fetch_agent(
        &self,
        pub_key: &str,
        service_id: Option<String>,
    ) -> Result<Option<Agent>, AgentStoreError> {
        self.conn
            .immediate_transaction::<_, AgentStoreError, _>(|| {
                let agent = agent::table
                    .filter(
                        agent::public_key
                            .eq(&pub_key)
                            .and(agent::service_id.eq(&service_id))
                            .and(agent::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .first::<AgentModel>(self.conn)
                    .map(Some)
                    .or_else(|err| if err == NotFound { Ok(None) } else { Err(err) })
                    .map_err(|err| AgentStoreError::QueryError {
                        context: "Failed to fetch agent for pub_key".to_string(),
                        source: Box::new(err),
                    })?
                    .ok_or_else(|| {
                        AgentStoreError::NotFoundError(
                            format!("Failed to find agent: {}", pub_key,),
                        )
                    })?;

                let roles = role::table
                    .select(role::all_columns)
                    .filter(
                        role::public_key
                            .eq(&pub_key)
                            .and(role::service_id.eq(&service_id))
                            .and(role::end_commit_num.eq(MAX_COMMIT_NUM)),
                    )
                    .load::<RoleModel>(self.conn)
                    .map(Some)
                    .map_err(|err| AgentStoreError::OperationError {
                        context: "Failed to fetch roles".to_string(),
                        source: Some(Box::new(err)),
                    })?
                    .ok_or_else(|| {
                        AgentStoreError::NotFoundError(
                            "Could not get all roles from storage".to_string(),
                        )
                    })?;

                Ok(Some(Agent::from_models(agent, roles)))
            })
    }
}
