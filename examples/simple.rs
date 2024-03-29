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

//! Simple example of how to use the Vizier API.

use oss_vizier::vizier::vizier_service_client::VizierServiceClient;
use oss_vizier::VizierClient;

#[tokio::main]
async fn main() {
    let endpoint = "http://localhost:28080/";

    let service = { VizierServiceClient::connect(endpoint) }.await.unwrap();

    let owner = "owner".to_string();

    let mut client = VizierClient::new(owner, service);

    let request = client
        .mk_list_studies_request_builder()
        .with_page_size(2)
        .build();

    let studies = client.service.list_studies(request).await;

    if studies.is_err() {
        println!("Error: {}", studies.err().unwrap().message());
        return;
    }

    let studies = studies.unwrap();

    let study_list = &studies.get_ref().studies;
    for t in study_list {
        println!("- {}", &t.display_name);
    }

    if !studies.get_ref().next_page_token.is_empty() {
        let mut page_token = studies.get_ref().next_page_token.clone();

        while !page_token.is_empty() {
            let request = client
                .mk_list_studies_request_builder()
                .with_page_token(page_token)
                .with_page_size(2)
                .build();

            let studies = client.service.list_studies(request).await.unwrap();
            let study_list = &studies.get_ref().studies;
            for t in study_list {
                println!("- {}", &t.display_name);
            }

            page_token = studies.get_ref().next_page_token.clone();
        }
    }
}
