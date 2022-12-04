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

//! Trial suggest request builder.

use crate::vizier::SuggestTrialsRequest;
use crate::StudyName;

/// [SuggestTrialsRequest] builder.
pub struct RequestBuilder {
    study_name: StudyName,
    suggestion_count: i32,
    client_id: String,
}

impl RequestBuilder {
    /// Creates a new instance of [SuggestTrialsRequest] builder.
    pub fn new(study_name: StudyName, suggestion_count: i32, client_id: String) -> Self {
        RequestBuilder {
            study_name,
            suggestion_count,
            client_id,
        }
    }

    /// Builds the [SuggestTrialsRequest].
    pub fn build(self) -> SuggestTrialsRequest {
        SuggestTrialsRequest {
            parent: self.study_name.into(),
            suggestion_count: self.suggestion_count,
            client_id: self.client_id,
        }
    }
}
