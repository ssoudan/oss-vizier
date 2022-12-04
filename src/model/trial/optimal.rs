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

//! Trial list optimal request builder.

use crate::vizier::ListOptimalTrialsRequest;
use crate::StudyName;

/// [ListOptimalTrialsRequest] builder.
pub struct RequestBuilder {
    study_name: StudyName,
    page_token: Option<String>,
    page_size: Option<i32>,
}

impl RequestBuilder {
    /// Creates a new instance of [ListOptimalTrialsRequest] builder.
    pub fn new(study_name: StudyName) -> Self {
        RequestBuilder {
            study_name,
            page_token: None,
            page_size: None,
        }
    }

    /// Sets the page token to the [ListOptimalTrialsRequest].
    /// The page token is used to retrieve the next page of results.
    /// If not set, the first page of results is returned.
    /// The page token is returned in the response of a previous
    /// [ListOptimalTrialsRequest].
    pub fn with_page_token(mut self, page_token: String) -> Self {
        self.page_token = Some(page_token);
        self
    }

    /// Sets the page size to the [ListOptimalTrialsRequest].
    /// The page size is used to limit the number of trials returned in the response.
    pub fn with_page_size(mut self, page_size: i32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Builds the [ListOptimalTrialsRequest].
    pub fn build(self) -> ListOptimalTrialsRequest {
        ListOptimalTrialsRequest {
            parent: self.study_name.into(),
            page_token: self.page_token.unwrap_or_default(),
            page_size: self.page_size.unwrap_or(0),
        }
    }
}
