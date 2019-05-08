extern crate multihash;
extern crate cid;
extern crate unicase;

extern crate rlp;
extern crate parity_bytes as bytes;
extern crate ethereum_types;
extern crate jsonrpc_core as core;
extern crate jsonrpc_http_server as http;

pub mod error;
mod route;

use std::thread;
use std::sync::{mpsc, Arc};
use std::net::{SocketAddr, IpAddr};

use core::futures::future::{self, FutureResult};
use core::futures::{self, Future};
use filesys_api::FileSysClient;
use http::hyper::{self, server, Method, StatusCode, Body,
	header::{self, HeaderValue},
};

use error::ServerError;
use route::Out;

pub use http::{AccessControlAllowOrigin, Host, DomainsValidation};

/// Request/response handler
pub struct Handler {
	/// Allowed CORS domains
	cors_domains: Option<Vec<AccessControlAllowOrigin>>,
	/// Hostnames allowed in the `Host` request header
	allowed_hosts: Option<Vec<Host>>,
	/// Reference to the Blockchain Client
	client: Arc<FileSysClient>,
}

impl Handler {
	pub fn Client(&self) -> &FileSysClient {
		&*self.client
	}

	pub fn new(cors: DomainsValidation<AccessControlAllowOrigin>, hosts: DomainsValidation<Host>, client: Arc<Client>) -> Self {
		Handler {
			cors_domains: cors.into(),
			allowed_hosts: hosts.into(),
			client: client,
		}
	}
	pub fn on_request(&self, req: hyper::Request<Body>) -> (Option<HeaderValue>, Out) {
		match *req.method() {
			Method::GET | Method::POST => {},
			_ => return (None, Out::Bad("Invalid Request")),
		}

		if !http::is_host_allowed(&req, &self.allowed_hosts) {
			return (None, Out::Bad("Disallowed Host header"));
		}

		let cors_header = http::cors_allow_origin(&req, &self.cors_domains);
		if cors_header == http::AllowCors::Invalid {
			return (None, Out::Bad("Disallowed Origin header"));
		}

		let path = req.uri().path();
		let query = req.uri().query();
		return (cors_header.into(), self.route(path, query));
	}
}

impl hyper::service::Service for Handler {
	type ReqBody = Body;
	type ResBody = Body;
	type Error = hyper::Error;
	type Future = FutureResult<hyper::Response<Body>, Self::Error>;

	fn call(&mut self, request: hyper::Request<Self::ReqBody>) -> Self::Future {
		let (cors_header, out) = self.on_request(request);

		let mut res = match out {
			Out::OctetStream(bytes) => {
				hyper::Response::builder()
					.status(StatusCode::OK)
					.header("content-type", HeaderValue::from_static("application/octet-stream"))
					.body(bytes.into())
			},
			Out::NotFound(reason) => {
				hyper::Response::builder()
					.status(StatusCode::NOT_FOUND)
					.header("content-type", HeaderValue::from_static("text/plain; charset=utf-8"))
					.body(reason.into())
			},
			Out::Bad(reason) => {
				hyper::Response::builder()
					.status(StatusCode::BAD_REQUEST)
					.header("content-type", HeaderValue::from_static("text/plain; charset=utf-8"))
					.body(reason.into())
			}
		}.expect("Response builder: Parsing 'content-type' header name will not fail; qed");

		if let Some(cors_header) = cors_header {
			res.headers_mut().append(header::ACCESS_CONTROL_ALLOW_ORIGIN, cors_header);
			res.headers_mut().append(header::VARY, HeaderValue::from_static("origin"));
		}

		future::ok(res)
	}
}

/// Add current interface (default: "127.0.0.1:5001") to list of allowed hosts
fn include_current_interface(mut hosts: Vec<Host>, interface: String, port: u16) -> Vec<Host> {
	hosts.push(match port {
		80 => interface,
		_ => format!("{}:{}", interface, port),
	}.into());

	hosts
}

#[derive(Debug)]
pub struct Listening {
	close: Option<futures::sync::oneshot::Sender<()>>,
	thread: Option<thread::JoinHandle<()>>,
}

impl Drop for Listening {
	fn drop(&mut self) {
		self.close.take().unwrap().send(()).unwrap();
		let _ = self.thread.take().unwrap().join();
	}
}

pub fn start_server(
	port: u16,
	interface: String,
	cors: DomainsValidation<AccessControlAllowOrigin>,
	hosts: DomainsValidation<Host>,
	client: Arc<FileSysClient>
) -> Result<Listening, ServerError> {

	let ip: IpAddr = interface.parse().map_err(|_| ServerError::InvalidInterface)?;
	let addr = SocketAddr::new(ip, port);
	let hosts: Option<Vec<_>> = hosts.into();
	let hosts: DomainsValidation<_> = hosts.map(move |hosts| include_current_interface(hosts, interface, port)).into();

	let (close, shutdown_signal) = futures::sync::oneshot::channel::<()>();
	let (tx, rx) = mpsc::sync_channel::<Result<(), ServerError>>(1);
	let thread = thread::spawn(move || {
		let send = |res| tx.send(res).expect("rx end is never dropped; qed");

		let server_bldr = match server::Server::try_bind(&addr) {
			Ok(s) => s,
			Err(err) => {
				send(Err(ServerError::from(err)));
				return;
			}
		};

		let new_service = move || {
			Ok::<_, ServerError>(
				Handler::new(cors.clone(), hosts.clone(), client.clone())
			)
		};

		let server = server_bldr
	        .serve(new_service)
	        .map_err(|_| ())
	        .select(shutdown_signal.map_err(|_| ()))
	        .then(|_| Ok(()));

	    hyper::rt::run(server);
		send(Ok(()));
	});

	// Wait for server to start successfuly.
	rx.recv().expect("tx end is never dropped; qed")?;

	Ok(Listening {
		close: close.into(),
		thread: thread.into(),
	})
}
