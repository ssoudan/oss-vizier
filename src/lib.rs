// Copyright 2022 Sebastien Soudan.
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

//! Unofficial OSS Vizier Client API.
//!
//! See https://github.com/google/vizier for OSS Vizier backend.
//!
//! ```no_run
//! let endpoint = std::env::var("ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_string());
//!
//! let service = VizierServiceClient::connect(endpoint).await.unwrap();
//!
//! let owner = "owner".to_string();
//!
//! let mut client = VizierClient::new_with_service(owner, service)
//!
//! let request = client
//!     .mk_list_studies_request_builder()
//!     .with_page_size(2)
//!     .build();
//!
//! let studies = client.service.list_studies(request).await.unwrap();
//! let study_list = &studies.get_ref().studies;
//! for t in study_list {
//!     println!("- {}", &t.display_name);
//! }
//! ```

use std::time::Duration;

use prost::bytes::Bytes;
pub use prost_types;
use tokio::time::sleep;
use tonic::codegen::http::uri::InvalidUri;
use tonic::codegen::{Body, StdError};
use tonic::transport::Channel;

use crate::google::longrunning::{operation, GetOperationRequest, Operation};
use crate::model::{study, trial};
use crate::study::StudyName;
use crate::trial::complete::FinalMeasurementOrReason;
use crate::trial::{early_stopping, optimal, stop, TrialName};
use crate::vizier::vizier_service_client::VizierServiceClient;
use crate::vizier::{
    AddTrialMeasurementRequest, CheckTrialEarlyStoppingStateRequest, CompleteTrialRequest,
    CreateTrialRequest, DeleteStudyRequest, DeleteTrialRequest, GetStudyRequest, GetTrialRequest,
    ListOptimalTrialsRequest, Measurement, StopTrialRequest, SuggestTrialsRequest,
    SuggestTrialsResponse, Trial,
};

pub mod model;
pub mod util;

/// google protos.
#[allow(missing_docs)]
pub mod google {
    /// google.apis protos.
    pub mod api {
        #![allow(clippy::derive_partial_eq_without_eq)]
        tonic::include_proto!("google.api");
    }

    /// google.rpc protos.
    pub mod rpc {
        #![allow(clippy::derive_partial_eq_without_eq)]
        tonic::include_proto!("google.rpc");
    }

    /// google.longrunning protos.
    pub mod longrunning {
        #![allow(clippy::derive_partial_eq_without_eq)]
        tonic::include_proto!("google.longrunning");
    }
}

/// vizier oss proto
#[allow(missing_docs)]
pub mod vizier {
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("vizier");
}

/// Vizier client.
#[derive(Clone)]
pub struct VizierClient<T> {
    owner: String,
    /// The Vizier service client.
    pub service: VizierServiceClient<T>,
}

/// Errors that can occur when using [VizierClient].
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Transport error
    #[error("tonic transport error - {0}")]
    Tonic(#[from] tonic::transport::Error),
    /// Invalid URI.
    #[error("{0}")]
    InvalidUri(#[from] InvalidUri),
    /// Decoding error.
    #[error("{0}")]
    DecodingError(#[from] util::Error),
    /// Vizier service error.
    #[error("Status: {}", .0.message())]
    Status(#[from] tonic::Status),
}

impl VizierClient<Channel> {
    /// Creates a new [VizierClient].
    pub fn new_with_service(owner: String, service: VizierServiceClient<Channel>) -> Self {
        Self { owner, service }
    }
}

impl<T> VizierClient<T>
where
    T: tonic::client::GrpcService<tonic::body::BoxBody>,
    T::Error: Into<StdError>,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    /// Creates a new [crate::vizier::CreateStudyRequest] builder.
    pub fn mk_study_request_builder(&self) -> study::create::RequestBuilder {
        study::create::RequestBuilder::new(self.owner.clone())
    }

    /// Creates a new [GetStudyRequest].
    pub fn mk_get_study_request(&self, study_name: StudyName) -> GetStudyRequest {
        study::get::RequestBuilder::new(study_name).build()
    }

    /// Creates a new [DeleteStudyRequest].
    pub fn mk_delete_study_request(&self, study_name: StudyName) -> DeleteStudyRequest {
        study::delete::RequestBuilder::new(study_name).build()
    }

    /// Creates a new [crate::vizier::ListStudiesRequest] builder.
    pub fn mk_list_studies_request_builder(&self) -> study::list::RequestBuilder {
        study::list::RequestBuilder::new(self.owner.clone())
    }

    /// Creates a new [GetTrialRequest].
    pub fn mk_get_trial_request(&self, trial_name: TrialName) -> GetTrialRequest {
        trial::get::RequestBuilder::new(trial_name).build()
    }

    /// Creates a new [SuggestTrialsRequest].
    pub fn mk_suggest_trials_request(
        &self,
        study_name: StudyName,
        suggestion_count: i32,
        client_id: String,
    ) -> SuggestTrialsRequest {
        trial::suggest::RequestBuilder::new(study_name, suggestion_count, client_id).build()
    }

    /// Creates a new [CreateTrialRequest].
    pub fn mk_create_trial_request(
        &self,
        study_name: StudyName,
        trial: Trial,
    ) -> CreateTrialRequest {
        trial::create::RequestBuilder::new(study_name, trial).build()
    }

    /// Creates a new [DeleteTrialRequest].
    pub fn mk_delete_trial_request(&self, trial_name: TrialName) -> DeleteTrialRequest {
        trial::delete::RequestBuilder::new(trial_name).build()
    }

    /// Creates a new [crate::vizier::ListTrialsRequest] builder.
    pub fn mk_list_trials_request_builder(
        &self,
        study_name: StudyName,
    ) -> trial::list::RequestBuilder {
        trial::list::RequestBuilder::new(study_name)
    }

    /// Creates a new [AddTrialMeasurementRequest].
    pub fn mk_add_trial_measurement_request(
        &self,
        trial_name: TrialName,
        measurement: Measurement,
    ) -> AddTrialMeasurementRequest {
        trial::add_measurement::RequestBuilder::new(trial_name, measurement).build()
    }

    /// Creates a new [CompleteTrialRequest].
    pub fn mk_complete_trial_request(
        &self,
        trial_name: TrialName,
        final_measurement: FinalMeasurementOrReason,
    ) -> CompleteTrialRequest {
        trial::complete::RequestBuilder::new(trial_name, final_measurement).build()
    }

    /// Creates a new [CheckTrialEarlyStoppingStateRequest].
    pub fn mk_check_trial_early_stopping_state_request(
        &self,
        trial_name: TrialName,
    ) -> CheckTrialEarlyStoppingStateRequest {
        early_stopping::RequestBuilder::new(trial_name).build()
    }

    /// Creates a new [StopTrialRequest].
    pub fn mk_stop_trial_request(&self, trial_name: TrialName) -> StopTrialRequest {
        stop::RequestBuilder::new(trial_name).build()
    }

    /// Creates a new [ListOptimalTrialsRequest].
    pub fn mk_list_optimal_trials_request(
        &self,
        study_name: StudyName,
    ) -> ListOptimalTrialsRequest {
        optimal::RequestBuilder::new(study_name).build()
    }

    /// Creates a [TrialName] (of the form
    /// "owners/{owner}/studies/{study}/trials/{trial}"). #
    /// Arguments
    /// * `study` - The study number - {study} in the pattern.
    /// * `trial` - The trial number - {trial} in the pattern.
    pub fn trial_name(&self, study: String, trial: String) -> TrialName {
        TrialName::new(self.owner.clone(), study, trial)
    }

    /// Creates a [TrialName] from a [StudyName] and trial number.
    /// # Arguments
    /// * `study_name` - The [StudyName].
    /// * `trial` - The trial number.
    pub fn trial_name_from_study(
        &self,
        study_name: &StudyName,
        trial: impl Into<String>,
    ) -> TrialName {
        TrialName::from_study(study_name, trial.into())
    }

    /// Creates a [StudyName] (of the form
    /// "owners/{owner}/studies/{study}").  
    /// # Arguments
    ///  * `study` - The study number - {study} in the pattern.
    pub fn study_name(&self, study: impl Into<String>) -> StudyName {
        StudyName::new(self.owner.clone(), study.into())
    }

    /// Waits for an operation to be completed.
    /// Makes `retries` attempts and return the error if it still fails.
    /// # Arguments
    /// * `retries` - The number of retries.
    /// * `operation` - The operation to wait for.
    pub async fn wait_for_operation(
        &mut self,
        mut retries: usize,
        mut operation: Operation,
    ) -> Result<Option<operation::Result>, Error> {
        while !operation.done {
            let mut wait_ms = 500;
            let resp = loop {
                match self
                    .service
                    .get_operation(GetOperationRequest {
                        name: operation.name.clone(),
                    })
                    .await
                {
                    Err(_) if retries > 0 => {
                        retries -= 1;
                        sleep(Duration::from_millis(wait_ms)).await;
                        wait_ms *= 2;
                    }
                    res => break res,
                }
            }?;

            operation = resp.into_inner();
        }

        Ok(operation.result)
    }

    /// Gets the [operation::Result] of an [Operation] specified by its name.
    pub async fn get_operation(
        &mut self,
        operation_name: String,
    ) -> Result<Option<operation::Result>, Error> {
        let resp = self
            .service
            .get_operation(GetOperationRequest {
                name: operation_name,
            })
            .await?;

        let operation = resp.into_inner();

        if operation.done {
            Ok(operation.result)
        } else {
            Ok(None)
        }
    }

    /// Suggests trials to a study.
    pub async fn suggest_trials(
        &mut self,
        request: SuggestTrialsRequest,
    ) -> Result<SuggestTrialsResponse, Error> {
        let trials = self.service.suggest_trials(request).await?;
        let operation = trials.into_inner();

        let result = loop {
            if let Some(result) = self.get_operation(operation.name.clone()).await? {
                break result;
            }
            sleep(Duration::from_millis(100)).await;
        };

        // parse the result into trials
        let resp: SuggestTrialsResponse =
            util::decode_operation_result_as(result, "SuggestTrialsResponse")?;

        Ok(resp)
    }
}

#[cfg(test)]
mod trials {
    use std::time::Duration;

    use tonic::Code;

    use super::common::{create_dummy_study, test_client};
    use crate::trial::complete::FinalMeasurementOrReason;
    use crate::util::decode_operation_result_as;
    use crate::vizier::{measurement, Measurement};
    use crate::SuggestTrialsResponse;

    #[tokio::test]
    async fn it_can_get_a_trial() {
        let mut client = test_client().await;

        let study_name = "it_can_get_a_trial".to_string();

        // create a study
        create_dummy_study(&mut client, study_name.clone()).await;

        let study_name = client.study_name(study_name);

        // suggest a trial
        let _resp = client
            .suggest_trials(client.mk_suggest_trials_request(
                study_name.clone(),
                1,
                "it_can_get_a_trial".to_string(),
            ))
            .await
            .unwrap();

        // get the trial
        let trial = "1".to_string();

        dbg!(&study_name);
        let trial_name = client.trial_name_from_study(&study_name, trial);
        dbg!(&trial_name);
        let request = client.mk_get_trial_request(trial_name);

        let trial = client.service.get_trial(request).await.unwrap();
        let trial = trial.get_ref();
        dbg!(trial);
    }

    #[tokio::test]
    async fn it_deletes_a_trial() {
        let mut client = test_client().await;

        let study = "53316451264".to_string();
        let trial = "2".to_string();

        let study_name = client.study_name(study);
        let trial_name = client.trial_name_from_study(&study_name, trial);

        let request = client.mk_delete_trial_request(trial_name);

        match client.service.delete_trial(request).await {
            Ok(study) => {
                let study = study.get_ref();
                dbg!(study);
            }
            Err(err) => {
                // dbg!(&err);
                assert_eq!(err.code(), Code::Unknown);
            }
        }
    }

    #[tokio::test]
    async fn it_suggests_trials_raw() {
        let mut client = test_client().await;

        let study_name = "it_suggests_trials_raw".to_string();

        // create a study
        create_dummy_study(&mut client, study_name.clone()).await;

        let study_name = client.study_name(study_name);

        let client_id = "it_can_suggest_trials".to_string();

        let request = client.mk_suggest_trials_request(study_name, 1, client_id);

        let resp = client.service.suggest_trials(request).await.unwrap();
        let operation = resp.into_inner();

        if let Some(result) = client.wait_for_operation(3, operation).await.unwrap() {
            // parse the result into trials
            let resp: SuggestTrialsResponse =
                decode_operation_result_as(result, "SuggestTrialsResponse").unwrap();

            dbg!(&resp);

            assert_eq!(resp.trials.len(), 1);
        } else {
            panic!("no result");
        }
    }

    #[tokio::test]
    async fn it_suggests_trials() {
        let mut client = test_client().await;

        let study_name = "it_suggests_trials".to_string();

        // create a study
        create_dummy_study(&mut client, study_name.clone()).await;

        let study_name = client.study_name(study_name);

        let client_id = "it_can_suggest_trials".to_string();

        let request = client.mk_suggest_trials_request(study_name, 1, client_id);

        let resp = client.suggest_trials(request).await.unwrap();

        dbg!(resp);
    }

    #[tokio::test]
    async fn it_lists_trials() {
        let mut client = test_client().await;

        let study_name = "it_lists_trials".to_string();

        // create a study
        create_dummy_study(&mut client, study_name.clone()).await;

        let study_name = client.study_name(study_name);

        // suggest 3 trials
        let client_id = "it_can_list_trials".to_string();
        let request = client.mk_suggest_trials_request(study_name.clone(), 3, client_id);

        let _resp = client.suggest_trials(request).await.unwrap();

        // list the trials
        let request = client
            .mk_list_trials_request_builder(study_name.clone())
            .with_page_size(2)
            .build();

        let trials = client.service.list_trials(request).await.unwrap();
        let trial_list = &trials.get_ref().trials;
        for t in trial_list {
            dbg!(&t);
        }

        if !trials.get_ref().next_page_token.is_empty() {
            let mut page_token = trials.get_ref().next_page_token.clone();

            while !page_token.is_empty() {
                println!("There is more! - {:?}", &page_token);

                let request = client
                    .mk_list_trials_request_builder(study_name.clone())
                    .with_page_token(page_token)
                    .with_page_size(2)
                    .build();

                let trials = client.service.list_trials(request).await.unwrap();
                let trial_list = &trials.get_ref().trials;
                for t in trial_list {
                    dbg!(&t);
                }

                page_token = trials.get_ref().next_page_token.clone();
            }
        }
    }

    #[tokio::test]
    async fn it_can_add_trial_measurement() {
        let mut client = test_client().await;

        let study_name = "it_can_add_trial_measurement".to_string();

        // create a study
        create_dummy_study(&mut client, study_name.clone()).await;

        let study_name = client.study_name(study_name);

        // create trials
        let client_id = "it_can_add_trial_measurement".to_string();

        let request = client.mk_suggest_trials_request(study_name.clone(), 1, client_id);

        let resp = client.service.suggest_trials(request).await.unwrap();
        let operation = resp.into_inner();

        if let Some(result) = client.wait_for_operation(3, operation).await.unwrap() {
            // parse the result into trials
            let resp: SuggestTrialsResponse =
                decode_operation_result_as(result, "SuggestTrialsResponse").unwrap();

            dbg!(&resp);

            assert_eq!(resp.trials.len(), 1);
        } else {
            panic!("no result");
        }

        // do something with the trials

        let trial = "1".to_string();

        let trial_name = client.trial_name_from_study(&study_name, trial);

        let measurement = Measurement {
            elapsed_duration: Some(Duration::from_secs(10).try_into().unwrap()),
            step_count: 13,
            metrics: vec![measurement::Metric {
                metric_id: "m1".to_string(),
                value: 2.1,
            }],
        };

        let request = client.mk_add_trial_measurement_request(trial_name, measurement);

        let trial = client.service.add_trial_measurement(request).await.unwrap();
        let trial = trial.get_ref();
        dbg!(trial);
    }

    #[tokio::test]
    async fn it_can_complete_a_trial() {
        let mut client = test_client().await;

        let study = "blah2".to_string();
        let trial = "3".to_string();

        let study_name = client.study_name(study);
        let trial_name = client.trial_name_from_study(&study_name, trial);

        let final_measurement_or_reason = FinalMeasurementOrReason::FinalMeasurement(Measurement {
            elapsed_duration: Some(Duration::from_secs(100).try_into().unwrap()),
            step_count: 14,
            metrics: vec![measurement::Metric {
                metric_id: "m1".to_string(),
                value: 3.1,
            }],
        });

        let request = client.mk_complete_trial_request(trial_name, final_measurement_or_reason);

        match client.service.complete_trial(request).await {
            Ok(trial) => {
                let trial = trial.get_ref();
                dbg!(trial);
            }
            Err(e) => {
                dbg!(e);
            }
        };
    }

    #[tokio::test]
    async fn it_can_check_trial_early_stopping_state() {
        let mut client = test_client().await;

        let study_name = "it_can_check_trial_early_stopping_state".to_string();

        // create a study
        create_dummy_study(&mut client, study_name.clone()).await;

        let study_name = client.study_name(study_name);

        // create trials
        let client_id = "it_can_check_trial_early_stopping_state".to_string();

        let request = client.mk_suggest_trials_request(study_name.clone(), 1, client_id);

        let resp = client.service.suggest_trials(request).await.unwrap();
        let operation = resp.into_inner();

        if let Some(result) = client.wait_for_operation(3, operation).await.unwrap() {
            // parse the result into trials
            let resp: SuggestTrialsResponse =
                decode_operation_result_as(result, "SuggestTrialsResponse").unwrap();

            dbg!(&resp);

            assert_eq!(resp.trials.len(), 1);
        } else {
            panic!("no result");
        }

        // do something with the trials

        let trial = "1".to_string();

        let trial_name = client.trial_name_from_study(&study_name, trial);

        let request = client.mk_check_trial_early_stopping_state_request(trial_name);

        let resp = client
            .service
            .check_trial_early_stopping_state(request)
            .await
            .unwrap();

        dbg!(&resp);

        let result = resp.into_inner();

        dbg!(result);
    }

    #[tokio::test]
    async fn it_can_stop_a_trial() {
        let mut client = test_client().await;

        let study = "blah2".to_string();
        let trial = "3".to_string();

        let study_name = client.study_name(study);
        let trial_name = client.trial_name_from_study(&study_name, trial);

        let request = client.mk_stop_trial_request(trial_name);

        match client.service.stop_trial(request).await {
            Ok(trial) => {
                let trial = trial.get_ref();
                dbg!(trial);
            }
            Err(err) => {
                dbg!(err);
            }
        };
    }

    #[tokio::test]
    async fn it_lists_optimal_trials() {
        let mut client = test_client().await;

        let study_name = "it_lists_optimal_trials".to_string();

        // create a study
        create_dummy_study(&mut client, study_name.clone()).await;

        let study_name = client.study_name(study_name);

        let request = client.mk_list_optimal_trials_request(study_name);

        let trials = client.service.list_optimal_trials(request).await.unwrap();
        let trial_list = &trials.get_ref().optimal_trials;
        for t in trial_list {
            dbg!(&t.name);
        }
    }
}

#[cfg(test)]
mod studies {
    use tonic::Code;

    use super::common::{create_dummy_study, test_client};
    use crate::study::spec::StudySpecBuilder;
    use crate::vizier::study_spec::metric_spec::GoalType;
    use crate::vizier::study_spec::parameter_spec::{
        DoubleValueSpec, IntegerValueSpec, ParameterValueSpec, ScaleType,
    };
    use crate::vizier::study_spec::{MetricSpec, ObservationNoise, ParameterSpec};

    #[tokio::test]
    async fn it_lists_studies() {
        let mut client = test_client().await;

        // create a study
        create_dummy_study(&mut client, "it_lists_studies_1".to_string()).await;
        create_dummy_study(&mut client, "it_lists_studies_2".to_string()).await;

        // list studies
        let request = client
            .mk_list_studies_request_builder()
            .with_page_size(2)
            .build();

        let studies = client.service.list_studies(request).await.unwrap();
        let study_list_resp = studies.get_ref();
        let study_list = &study_list_resp.studies;
        for t in study_list {
            dbg!(&t.name);
            dbg!(&t.display_name);
        }

        if !studies.get_ref().next_page_token.is_empty() {
            let mut page_token = studies.get_ref().next_page_token.clone();

            while !page_token.is_empty() {
                println!("There is more! - {:?}", &page_token);

                let request = client
                    .mk_list_studies_request_builder()
                    .with_page_token(page_token)
                    .with_page_size(2)
                    .build();

                let studies = client.service.list_studies(request).await.unwrap();
                let study_list = &studies.get_ref().studies;
                for t in study_list {
                    dbg!(&t.display_name);
                }

                page_token = studies.get_ref().next_page_token.clone();
            }
        }
    }

    #[tokio::test]
    async fn it_creates_studies() {
        let mut client = test_client().await;

        let study_spec =
            StudySpecBuilder::new("ALGORITHM_UNSPECIFIED".to_string(), ObservationNoise::Low)
                .with_metric_specs(vec![MetricSpec {
                    metric_id: "m1".to_string(), // FUTURE(ssoudan) unique and w/o whitespaces
                    goal: GoalType::Maximize as i32,
                    safety_config: None,
                }])
                .with_parameters(vec![
                    ParameterSpec {
                        parameter_id: "a".to_string(),
                        scale_type: ScaleType::Unspecified as i32,
                        conditional_parameter_specs: vec![],
                        parameter_value_spec: Some(ParameterValueSpec::DoubleValueSpec(
                            DoubleValueSpec {
                                min_value: 0.0,
                                max_value: 12.0,
                                default_value: Some(4.0),
                            },
                        )),
                    },
                    ParameterSpec {
                        parameter_id: "b".to_string(),
                        scale_type: ScaleType::Unspecified as i32,
                        conditional_parameter_specs: vec![],
                        parameter_value_spec: Some(ParameterValueSpec::IntegerValueSpec(
                            IntegerValueSpec {
                                min_value: 4,
                                max_value: 10,
                                default_value: Some(7),
                            },
                        )),
                    },
                ])
                .build();

        let request = client
            .mk_study_request_builder()
            .with_display_name("blah2".to_string())
            .with_study_spec(study_spec)
            .build()
            .unwrap();

        match client.service.create_study(request).await {
            Ok(study_response) => {
                let study = study_response.get_ref();
                dbg!(&study);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }

    #[tokio::test]
    async fn it_can_get_a_study() {
        let mut client = test_client().await;

        let study_name = "it_can_get_a_study".to_string();

        // create a study
        create_dummy_study(&mut client, study_name.clone()).await;

        let study_name = client.study_name(study_name);

        let request = client.mk_get_study_request(study_name);

        let study = client.service.get_study(request).await.unwrap();
        let study = study.get_ref();
        dbg!(study);
    }

    #[tokio::test]
    async fn it_deletes_a_study() {
        let mut client = test_client().await;

        let study = "blah_to_delete".to_string();
        let study_name = client.study_name(study);

        let request = client.mk_delete_study_request(study_name);

        match client.service.delete_study(request).await {
            Ok(study) => {
                let study = study.get_ref();
                dbg!(study);
            }
            Err(err) => {
                assert_eq!(err.code(), Code::Unknown);
            }
        }
    }
}

#[cfg(test)]
mod common {

    use tonic::transport::Channel;

    use crate::study::spec::StudySpecBuilder;
    use crate::vizier::study_spec::metric_spec::GoalType;
    use crate::vizier::study_spec::parameter_spec::{
        DoubleValueSpec, IntegerValueSpec, ParameterValueSpec, ScaleType,
    };
    use crate::vizier::study_spec::{MetricSpec, ObservationNoise, ParameterSpec};
    use crate::vizier::vizier_service_client::VizierServiceClient;
    use crate::VizierClient;

    pub(crate) async fn test_client() -> VizierClient<Channel> {
        let endpoint =
            std::env::var("ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_string());

        let service = VizierServiceClient::connect(endpoint).await.unwrap();

        let owner = "owner".to_string();

        VizierClient::new_with_service(owner, service)
    }

    pub(crate) async fn create_dummy_study(client: &mut VizierClient<Channel>, study_name: String) {
        let study_spec =
            StudySpecBuilder::new("ALGORITHM_UNSPECIFIED".to_string(), ObservationNoise::Low)
                .with_metric_specs(vec![MetricSpec {
                    metric_id: "m1".to_string(), // FUTURE(ssoudan) unique and w/o whitespaces
                    goal: GoalType::Maximize as i32,
                    safety_config: None,
                }])
                .with_parameters(vec![
                    ParameterSpec {
                        parameter_id: "a".to_string(),
                        scale_type: ScaleType::Unspecified as i32,
                        conditional_parameter_specs: vec![],
                        parameter_value_spec: Some(ParameterValueSpec::DoubleValueSpec(
                            DoubleValueSpec {
                                min_value: 0.0,
                                max_value: 12.0,
                                default_value: Some(4.0),
                            },
                        )),
                    },
                    ParameterSpec {
                        parameter_id: "b".to_string(),
                        scale_type: ScaleType::Unspecified as i32,
                        conditional_parameter_specs: vec![],
                        parameter_value_spec: Some(ParameterValueSpec::IntegerValueSpec(
                            IntegerValueSpec {
                                min_value: 4,
                                max_value: 10,
                                default_value: Some(7),
                            },
                        )),
                    },
                ])
                .build();

        let request = client
            .mk_study_request_builder()
            .with_display_name(study_name)
            .with_study_spec(study_spec)
            .build()
            .unwrap();

        match client.service.create_study(request).await {
            Ok(study_response) => {
                let study = study_response.get_ref();
                dbg!(&study);
            }
            Err(e) => {
                dbg!(e);
            }
        }
    }
}
