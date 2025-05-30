use crate::state::AppState;
use crate::volumes::models::VolumesParameters;
use crate::{
    error::ErrorResponse,
    volumes::error::{VolumesAPIError, VolumesResult},
    volumes::models::{
        FileVolume, S3TablesVolume, S3Volume, Volume, VolumeCreatePayload, VolumeCreateResponse,
        VolumeResponse, VolumeType, VolumeUpdatePayload, VolumeUpdateResponse, VolumesResponse,
    },
};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use core_metastore::error::MetastoreError;
use core_metastore::models::{AwsAccessKeyCredentials, AwsCredentials, Volume as MetastoreVolume};
use core_utils::scan_iterator::ScanIterator;
use utoipa::OpenApi;
use validator::Validate;

#[derive(OpenApi)]
#[openapi(
    paths(
        create_volume,
        get_volume,
        delete_volume,
        list_volumes,
        // update_volume,
    ),
    components(
        schemas(
            VolumeCreatePayload,
            VolumeCreateResponse,
            Volume,
            VolumeType,
            S3Volume,
            S3TablesVolume,
            FileVolume,
            AwsCredentials,
            AwsAccessKeyCredentials,
            VolumeResponse,
            VolumesResponse,
            ErrorResponse,
        )
    ),
    tags(
        (name = "volumes", description = "Volumes endpoints")
    )
)]
pub struct ApiDoc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryParameters {
    #[serde(default)]
    pub cascade: Option<bool>,
}

#[utoipa::path(
    post,
    operation_id = "createVolume",
    tags = ["volumes"],
    path = "/ui/volumes",
    request_body = VolumeCreatePayload,
    responses(
        (status = 200, description = "Successful Response", body = VolumeCreateResponse),
        (status = 401,
         description = "Unauthorized",
         headers(
            ("WWW-Authenticate" = String, description = "Bearer authentication scheme with error details")
         ),
         body = ErrorResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 422, description = "Unprocessable entity", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(level = "debug", skip(state), err, ret(level = tracing::Level::TRACE))]
pub async fn create_volume(
    State(state): State<AppState>,
    Json(volume): Json<VolumeCreatePayload>,
) -> VolumesResult<Json<VolumeCreateResponse>> {
    let embucket_volume: MetastoreVolume = volume.data.into();
    embucket_volume
        .validate()
        .map_err(|e| VolumesAPIError::Create {
            source: MetastoreError::Validation { source: e },
        })?;
    state
        .metastore
        .create_volume(&embucket_volume.ident.clone(), embucket_volume)
        .await
        .map_err(|e| VolumesAPIError::Create { source: e })
        .map(|o| {
            Json(VolumeCreateResponse {
                data: o.data.into(),
            })
        })
}

#[utoipa::path(
    get,
    operation_id = "getVolume",
    tags = ["volumes"],
    path = "/ui/volumes/{volumeName}",
    params(
        ("volumeName" = String, Path, description = "Volume name")
    ),
    responses(
        (status = 200, description = "Successful Response", body = VolumeResponse),
        (status = 401,
         description = "Unauthorized",
         headers(
            ("WWW-Authenticate" = String, description = "Bearer authentication scheme with error details")
         ),
         body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 422, description = "Unprocessable entity", body = ErrorResponse),
    )
)]
#[tracing::instrument(level = "debug", skip(state), err, ret(level = tracing::Level::TRACE))]
pub async fn get_volume(
    State(state): State<AppState>,
    Path(volume_name): Path<String>,
) -> VolumesResult<Json<VolumeResponse>> {
    match state.metastore.get_volume(&volume_name).await {
        Ok(Some(volume)) => Ok(Json(VolumeResponse {
            data: volume.data.into(),
        })),
        Ok(None) => Err(VolumesAPIError::Get {
            source: MetastoreError::VolumeNotFound {
                volume: volume_name.clone(),
            },
        }),
        Err(e) => Err(VolumesAPIError::Get { source: e }),
    }
}

#[utoipa::path(
    delete,
    operation_id = "deleteVolume",
    tags = ["volumes"],
    path = "/ui/volumes/{volumeName}",
    params(
        ("volumeName" = String, Path, description = "Volume name")
    ),
    responses(
        (status = 200, description = "Successful Response"),
        (status = 401,
         description = "Unauthorized",
         headers(
            ("WWW-Authenticate" = String, description = "Bearer authentication scheme with error details")
         ),
         body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 422, description = "Unprocessable entity", body = ErrorResponse),
    )
)]
#[tracing::instrument(level = "debug", skip(state), err, ret(level = tracing::Level::TRACE))]
pub async fn delete_volume(
    State(state): State<AppState>,
    Query(query): Query<QueryParameters>,
    Path(volume_name): Path<String>,
) -> VolumesResult<()> {
    state
        .metastore
        .delete_volume(&volume_name, query.cascade.unwrap_or_default())
        .await
        .map_err(|e| VolumesAPIError::Delete { source: e })
}

#[utoipa::path(
    put,
    operation_id = "updateVolume",
    tags = ["volumes"],
    path = "/ui/volumes/{volumeName}",
    params(
        ("volumeName" = String, Path, description = "Volume name")
    ),
    request_body = VolumeUpdatePayload,
    responses(
        (status = 200, description = "Successful Response", body = VolumeUpdateResponse),
        (status = 401,
         description = "Unauthorized",
         headers(
            ("WWW-Authenticate" = String, description = "Bearer authentication scheme with error details")
         ),
         body = ErrorResponse),
        (status = 404, description = "Not found", body = ErrorResponse),
        (status = 422, description = "Unprocessable entity", body = ErrorResponse),
    )
)]
#[tracing::instrument(level = "debug", skip(state), err, ret(level = tracing::Level::TRACE))]
pub async fn update_volume(
    State(state): State<AppState>,
    Path(volume_name): Path<String>,
    Json(volume): Json<VolumeUpdatePayload>,
) -> VolumesResult<Json<VolumeUpdateResponse>> {
    let volume: MetastoreVolume = volume.data.into();
    volume.validate().map_err(|e| VolumesAPIError::Update {
        source: MetastoreError::Validation { source: e },
    })?;
    state
        .metastore
        .update_volume(&volume_name, volume)
        .await
        .map_err(|e| VolumesAPIError::Update { source: e })
        .map(|o| {
            Json(VolumeUpdateResponse {
                data: o.data.into(),
            })
        })
}

#[utoipa::path(
    get,
    operation_id = "getVolumes",
    params(
        ("cursor" = Option<String>, Query, description = "Volumes cursor"),
        ("limit" = Option<usize>, Query, description = "Volumes limit"),
        ("search" = Option<String>, Query, description = "Volumes search (start with)"),
    ),
    tags = ["volumes"],
    path = "/ui/volumes",
    responses(
        (status = 200, body = VolumesResponse),
        (status = 401,
         description = "Unauthorized",
         headers(
            ("WWW-Authenticate" = String, description = "Bearer authentication scheme with error details")
         ),
         body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
#[tracing::instrument(level = "debug", skip(state), err, ret(level = tracing::Level::TRACE))]
pub async fn list_volumes(
    Query(parameters): Query<VolumesParameters>,
    State(state): State<AppState>,
) -> VolumesResult<Json<VolumesResponse>> {
    state
        .metastore
        .iter_volumes()
        .cursor(parameters.cursor.clone())
        .limit(parameters.limit)
        .token(parameters.search)
        .collect()
        .await
        .map_err(|e| VolumesAPIError::List {
            source: MetastoreError::UtilSlateDB { source: e },
        })
        .map(|o| {
            let next_cursor = o
                .iter()
                .last()
                .map_or(String::new(), |rw_object| rw_object.ident.clone());
            Json(VolumesResponse {
                items: o.into_iter().map(|x| x.data.into()).collect(),
                current_cursor: parameters.cursor,
                next_cursor,
            })
        })
}
