//! An example of how ructe can be used with the tide framework.
mod ructe_tide;
use ructe_tide::Render;

use httpdate::fmt_http_date;
use std::future::Future;
use std::io::{self, Write};
use std::pin::Pin;
use std::time::{Duration, SystemTime};
use templates::statics::{cloud_svg, StaticFile};
use tide::http::headers::EXPIRES;
use tide::http::Error;
use tide::{Next, Redirect, Request, Response, StatusCode};

/// Main entry point.
///
/// Set up an app and start listening for requests.
#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let mut app = tide::new();
    app.middleware(handle_error);
    app.at("/static/*path").get(static_file);
    app.at("/favicon.ico")
        .get(Redirect::new(format!("/static/{}", cloud_svg.name)));
    app.at("/").get(frontpage);

    let addr = "127.0.0.1:3000";
    println!("Starting server on http://{}/", addr);
    app.listen(addr).await?;

    Ok(())
}

/// Handler for a page in the web site.
async fn frontpage(_req: Request<()>) -> Result<Response, Error> {
    // A real site would probably have some business logic here.
    let mut res = Response::new(StatusCode::Ok);
    res.render_html(|o| {
        Ok(templates::page(o, &[("world", 5), ("tide", 7)])?)
    })?;
    Ok(res)
}

/// Handler for static files.
///
/// Ructe provides the static files as constants, and the StaticFile
/// interface to get a file by url path.
async fn static_file(req: Request<()>) -> Result<Response, Error> {
    let path = req.param::<String>("path")?;
    let data = StaticFile::get(&path)
        .ok_or_else(|| Error::from_str(StatusCode::NotFound, "not found"))?;
    Ok(Response::builder(StatusCode::Ok)
        .content_type(data.mime.clone()) // Takes Into<Mime>, not AsRef<Mime>
        .header(EXPIRES, fmt_http_date(SystemTime::now() + 180 * DAY))
        .body(data.content)
        .build())
}

/// 24 hours.
const DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// This method can be used as a "template tag", i.e. a method that
/// can be called directly from a template.
fn footer(out: &mut dyn Write) -> io::Result<()> {
    templates::footer(
        out,
        &[
            ("ructe", "https://crates.io/crates/ructe"),
            ("tide", "https://crates.io/crates/tide"),
        ],
    )
}

/// A middleware to log errors and render a html error message.
///
/// If the response has content, this function does not overwrite it.
fn handle_error<'a>(
    request: Request<()>,
    next: Next<'a, ()>,
) -> Pin<Box<dyn Future<Output = Result<Response, Error>> + Send + 'a>> {
    Box::pin(async {
        // I don't really like to create this string for every request,
        // but when I see if there is an error, the request is consumed.
        let rdesc = format!("{} {:?}", request.method(), request.url());
        let mut res = next.run(request).await;
        let status = res.status();
        if status.is_client_error() || status.is_server_error() {
            println!("Error {} on {}: {:?}", status, rdesc, res.error());
            if res.is_empty().unwrap_or(false) {
                res.render_html(|o| {
                    Ok(templates::error(
                        o,
                        status,
                        status.canonical_reason(),
                    )?)
                })?
            }
        }
        Ok(res)
    })
}

// And finally, include the generated code for templates and static files.
include!(concat!(env!("OUT_DIR"), "/templates.rs"));
