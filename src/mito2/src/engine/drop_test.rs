// Copyright 2023 Greptime Team
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

use object_store::util::join_path;
use store_api::region_engine::RegionEngine;
use store_api::region_request::{RegionDropRequest, RegionRequest};
use store_api::storage::RegionId;

use crate::config::MitoConfig;
use crate::test_util::{CreateRequestBuilder, TestEnv};
use crate::worker::DROPPING_MARKER_FILE;

#[tokio::test]
async fn test_engine_drop_region() {
    let mut env = TestEnv::with_prefix("drop");
    let engine = env.create_engine(MitoConfig::default()).await;

    let region_id = RegionId::new(1, 1);
    // It's okay to drop a region doesn't exist.
    engine
        .handle_request(region_id, RegionRequest::Drop(RegionDropRequest {}))
        .await
        .unwrap_err();

    let request = CreateRequestBuilder::new().build();
    engine
        .handle_request(region_id, RegionRequest::Create(request))
        .await
        .unwrap();

    let region = engine.get_region(region_id).unwrap();
    let region_dir = region.access_layer.region_dir().to_owned();
    // no dropping marker file
    assert!(!env
        .get_object_store()
        .unwrap()
        .is_exist(&join_path(&region_dir, DROPPING_MARKER_FILE))
        .await
        .unwrap());

    // create a parquet file
    env.get_object_store()
        .unwrap()
        .write(&join_path(&region_dir, "blabla.parquet"), vec![])
        .await
        .unwrap();

    // drop the created region.
    engine
        .handle_request(region_id, RegionRequest::Drop(RegionDropRequest {}))
        .await
        .unwrap();
    assert!(!engine.is_region_exists(region_id));
    // the drop marker is not removed yet
    assert!(env
        .get_object_store()
        .unwrap()
        .is_exist(&join_path(&region_dir, DROPPING_MARKER_FILE))
        .await
        .unwrap());
}
