use std::fs;
use std::{convert::Infallible, net::SocketAddr};

use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{body::Bytes, header::CONTENT_TYPE, Body, Request, Response, StatusCode};

use multer::Multipart;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
   let boundary = req
      .headers()
      .get(CONTENT_TYPE)
      .and_then(|ct| ct.to_str().ok())
      .and_then(|ct| multer::parse_boundary(ct).ok());

   if boundary.is_none() {
      return Ok(Response::builder()
         .status(StatusCode::BAD_REQUEST)
         .body(Body::from("BAD REQUEST"))
         .unwrap());
   }

   if let Err(err) = process_multipart(req.into_body(), boundary.unwrap()).await {
      return Ok(Response::builder()
         .status(StatusCode::INTERNAL_SERVER_ERROR)
         .body(Body::from(format!("INTERNAL SERVER ERROR: {}", err)))
         .unwrap());
   }

   Ok(Response::new(Body::from("Success")))
}

async fn process_multipart(body: Body, boundary: String) -> multer::Result<()> {
   let mut multipart = Multipart::new(body, boundary);

   while let Some(mut field) = multipart.next_field().await? {
      let mut file_data: Vec<Bytes> = vec![];

      while let Some(chunk) = field.chunk().await? {
         file_data.push(chunk);
      }

      let data = file_data.iter().fold(Vec::new(), |mut data, chunk| {
         data.extend_from_slice(&chunk);

         data
      });

      let new_filename: String = thread_rng()
         .sample_iter(&Alphanumeric)
         .take(30)
         .map(char::from)
         .collect();

      let type_splited = field
         .content_type()
         .unwrap()
         .to_string()
         .split("/")
         .map(|item| String::from(item))
         .collect::<Vec<String>>();

      fs::write(
         format!("./uploads/{}.{}", new_filename, type_splited[1]),
         data,
      )
      .unwrap();
   }

   Ok(())
}

#[tokio::main]
async fn main() {
   let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

   let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

   let server = Server::bind(&addr).serve(make_svc);

   if let Err(e) = server.await {
      eprintln!("server error: {}", e);
   }
}
