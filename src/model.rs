use std::sync::{Arc, Mutex};
use serde::{Serialize, Deserialize};
use axum::{
    extract::{FromRef, State}, 
    routing::post, 
    Json, 
    Router, 
};

use crate::MyResult;

#[derive(Serialize, Clone, Debug)]
pub struct Ticket{
    id: i32, 
    name: String, 
}

#[derive(Deserialize)]
pub struct TicketCreate{
    name: String, 
}

#[derive(Clone)]
pub struct MyDB{
    inner: Arc<Mutex<Vec<Ticket>>>, 
}

#[derive(Clone, FromRef)]
pub struct AppState{
    db: MyDB, 
}

impl MyDB{
    pub async fn new() -> MyResult<Self>{
        Ok(Self{inner: Arc::default()})
    }
    async fn create(&self, x: TicketCreate) -> MyResult<Ticket>{
        let mut db = self.inner.lock().unwrap();
        let res = Ticket{
            id: db.len() as i32, 
            name: x.name, 
        };
        db.push(res.clone());
        Ok(res)
    }
}

async fn handle_create_tk(
    State(mydb): State<MyDB>, 
    Json(create_tk): Json<TicketCreate>, 
) -> MyResult<Json<Ticket>>{
    let res = mydb.create(create_tk).await?;
    Ok(Json(res))
}

pub fn route_ticket(mydb: MyDB) -> Router{
    let app = AppState{db: mydb};
    Router::new()
        .route("/ticket", post(handle_create_tk))
        .with_state(app)
}



