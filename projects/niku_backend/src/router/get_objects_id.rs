use std::sync::Arc;

use axum::extract::{Json, Path, State};
use niku_core::backend::ErrorResponse;
use niku_core::object::ObjectEntry;
use tokio::sync::Mutex;
use tracing::info;

use crate::errors::ServerError;
use crate::SharedData;

#[utoipa::path(
    get,
    path = "/objects/{id}",
    params(("id" = String, Path, description = "The ID of the object.")),
    responses((status = OK, body = ObjectEntry), (status = NOT_FOUND, body = ErrorResponse))
)]
/// Get the object data given it's ID.
pub(super) async fn get_objects_id(
    State(state): State<Arc<Mutex<SharedData>>>,
    Path(id): Path<String>,
) -> Result<Json<ObjectEntry>, ServerError> {
    let objects = &mut state.lock().await.objects;
    let entry = objects.get(&id).ok_or(ServerError::UnknownObject)?.clone();

    if cfg!(debug_assertions) {
        info!(?entry, "Requested object entry");
    }

    Ok(Json(entry))
}
