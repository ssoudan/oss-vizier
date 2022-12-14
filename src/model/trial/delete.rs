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

//! Trial delete request builder.

use crate::vizier::DeleteTrialRequest;
use crate::TrialName;

/// [DeleteTrialRequest] builder.
pub struct RequestBuilder {
    trial_name: TrialName,
}

impl RequestBuilder {
    /// Creates a new instance of [DeleteTrialRequest] builder.
    pub fn new(trial_name: TrialName) -> Self {
        RequestBuilder { trial_name }
    }

    /// Builds the [DeleteTrialRequest].
    pub fn build(self) -> DeleteTrialRequest {
        DeleteTrialRequest {
            name: self.trial_name.into(),
        }
    }
}
