use axum::Json;

pub async fn handle() -> Json<()> {
    Json(())
}
