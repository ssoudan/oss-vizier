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

//! StudySpec builder.

use crate::vizier::study_spec::{
    AutomatedStoppingSpec, MetricSpec, ObservationNoise, ParameterSpec,
};
use crate::vizier::{KeyValue, StudySpec};

/// [StudySpec] builder.
pub struct StudySpecBuilder {
    metrics: Vec<MetricSpec>,
    parameters: Vec<ParameterSpec>,
    algorithm: String,
    observation_noise: ObservationNoise,
    automated_stopping_spec: Option<AutomatedStoppingSpec>,
    metadata: Vec<KeyValue>,
    pythia_endpoint: Option<String>,
}

impl StudySpecBuilder {
    /// Creates a new instance of [StudySpec] builder.
    pub fn new(algorithm: String, observation_noise: ObservationNoise) -> Self {
        StudySpecBuilder {
            algorithm,
            observation_noise,
            metrics: vec![],
            parameters: vec![],
            automated_stopping_spec: None,
            metadata: vec![],
            pythia_endpoint: None,
        }
    }

    /// Sets the [MetricSpec]s to the [StudySpec].
    pub fn with_metric_specs(mut self, metrics: Vec<MetricSpec>) -> Self {
        self.metrics = metrics;
        self
    }

    /// Sets the [ParameterSpec]s to the [StudySpec].
    pub fn with_parameters(mut self, parameters: Vec<ParameterSpec>) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the [AutomatedStoppingSpec] to the [StudySpec].
    pub fn with_automated_stopping_spec(
        mut self,
        automated_stopping_spec: AutomatedStoppingSpec,
    ) -> Self {
        self.automated_stopping_spec = Some(automated_stopping_spec);
        self
    }

    /// Sets the [KeyValue]s to the [StudySpec].
    /// The metadata is a collection of key-value pairs that are associated with the
    /// study.
    pub fn with_metadata(mut self, metadata: Vec<KeyValue>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the Pythia endpoint to the [StudySpec].
    /// The Pythia endpoint is the endpoint of the Pythia service that is used to
    /// generate suggestions.
    pub fn with_pythia_endpoint(mut self, pythia_endpoint: String) -> Self {
        self.pythia_endpoint = Some(pythia_endpoint);
        self
    }

    /// Builds the [StudySpec].
    pub fn build(self) -> StudySpec {
        StudySpec {
            metrics: self.metrics,
            parameters: self.parameters,
            algorithm: self.algorithm,
            observation_noise: self.observation_noise as i32,
            automated_stopping_spec: self.automated_stopping_spec,
            metadata: self.metadata,
            pythia_endpoint: self.pythia_endpoint,
        }
    }
}
