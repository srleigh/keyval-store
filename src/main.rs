mod db;
use db::DB;

mod app_state;
use app_state::AppState;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, web::Data};
use actix_files::Files;
use chrono::Utc;
use serde::Deserialize;
use sysinfo::{SystemExt, DiskExt};
use futures::StreamExt;


#[derive(Deserialize)]
struct FormData {
    msg: String,
}

/// Memory stats from procfs.  (Virtual, Resident, Total)
fn memory_stats() -> anyhow::Result<(f64, f64, f64)> {
    use procfs::process::Process;
    let me = Process::myself()?;
    let page_size = procfs::page_size()?;
    let mem_virtual = ((me.statm()?.size * page_size) as f64 )/ 1024.0 / 1024.0;
    let mem_resident = ((me.statm()?.resident * page_size) as f64 )/ 1024.0 / 1024.0;
    let mem_total = (procfs::Meminfo::new()?.mem_total as f64) / 1024.0 / 1024.0;
    Ok((mem_virtual, mem_resident, mem_total))
}

#[get("/")]
async fn home(app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    let channels = db.num_rows().unwrap_or(-1);
    let reads = app_state.get_reads();
    let writes = app_state.get_writes();
    let hours_uptime = (Utc::now() - app_state.startup_datetime).num_minutes() as f64 / 60.0;
    let (mem_virtual, mem_resident, mem_total) = memory_stats().unwrap_or((-1.0,-1.0,-1.0));
    let build_time = env!("VERGEN_BUILD_TIMESTAMP");
    let build_version = env!("VERGEN_GIT_SHA");
    let database_size = db.database_size().unwrap_or(0) as f64 / 1024.0 / 1024.0;
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    let disk_total = system.disks()[0].total_space() as f64 / 1024.0 / 1024.0;
    let html = format!("
<html>
<head>
 <title>KeyVal-Store</title>
 <link rel=\"stylesheet\" href=\"/webfiles/styles.css\">
</head>
<body>
<h3>World's Simplest Key-Value Store</h3>
<img src=\"/webfiles/api.drawio.png\">
<p>Super simple free key-value store.  No setup or configuration, and did I mention it is free!

<h3>Python 3 Example</h3>
<pre style=\"margin-left: 3em;\"><code>import urllib.request as req
data_in = \"123\"
my_channel = \"http://keyval.store/v1/my_channel/\"  # api version 1 and channel 'my_channel'
req.urlopen(my_channel + \"set/\" + data_in)  # set
data_out = res.urlopen(my_channel + \"get\").read().decode('utf-8')  # get
assert(data_in == data_out)
</code></pre>

<h3>REST API</h3>
<p>Can get and set to values by just visiting URLs in a browser, but intention is to mostly use code.
<ul>
 <li> Set to key using http get. eg: <a href=\"/v1/my_key/set/mydata123\">http://keyval.store/v1/my_key/set/mydata123</a>
 <li> Set value using http post to url <a href=\"/v1/my_key/set\">http://keyval.store/v1/my_key/set</a>
 <li> Get value using http get.  eg: <a href=\"/v1/my_key/get\">http://keyval.store/v1/my_key/get</a>
 <li> Interactively get and set values at key url: <a href=\"/v1/my_key\">http://keyval.store/v1/my_key</a>
</ul>

<h3>Storage Details</h3>
<ul>
 <li> Each key holds one value
 <li> A new value will overwrite an old value for a given key
 <li> Data persists in a database until someone unplugs the <a href=\"https://xkcd.com/908\">server</a>
 <li> Max post size is 1MB
</ul>

<h3>Picture of Me</h3>
<img src=\"/webfiles/me.jpg\">

<h3>Some Details</h3>
<ul>
 <li> Channels: {channels}
 <li> Session reads: {reads}
 <li> Session writes: {writes}
 <li> Session uptime: {hours_uptime:.1} hrs
 <li> Memory usage virtual: {mem_virtual:.3} MB
 <li> Memory usage resident: {mem_resident:.3} MB
 <li> Memory total: {mem_total:.3} MB
 <li> Database size: {database_size:.3} MB
 <li> Disk space total: {disk_total:.3} MB
 <li> Build time: {build_time}
 <li> Build version: {build_version}
 <li> Source: <a href=\"https://github.com/srleigh/keyval-store\">https://github.com/srleigh/keyval-store</a>
 <li> Inspiration: <a href=\"https://grugbrain.dev/\">https://grugbrain.dev/</a>
</ul>

<h3>Todo</h3>
<ul>
 <li> Rust example
 <li> C example
 <li> Javascript example
<ul>
</body></html>");
    HttpResponse::Ok().body(html)
}

#[get("/v1/{chan}/get")]
async fn channel_get(channel: web::Path<String>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    app_state.increment_reads();
    let channel: String = channel.to_string();
    let msg = db.read(&channel).unwrap_or("".to_string());
    HttpResponse::Ok().body(msg)
}

fn interactive(channel: web::Path<String>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    app_state.increment_reads();
    let val = db.read(&channel).unwrap_or("".to_string());
    let html = format!("
    <html>
    <script>
async function updateValueLoop(){{
  while(true){{
  fetch(\"http://keyval.store/v1/{channel}/get\")
    .then((response) => response.text())
    .then((data) => document.getElementById(\"val\").innerHTML = data)
    await new Promise(r => setTimeout(r, 500));
  }}
}}
updateValueLoop();
    </script>

    <body>
    Value: <span id=\"val\">{val}</span>
<form action=\"/v1/{channel}\" method=\"post\" enctype=\"application/x-www-form-urlencoded\">
 <label for=\"msg\">Enter new value: </label>
 <input type=\"text\" name=\"msg\" required>
 <input type=\"submit\" value=\"Set!\">
</form></body></html>");
    HttpResponse::Ok().body(html)
}

#[get("/v1/{chan}")]
async fn interactive_get(channel: web::Path<String>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    interactive(channel, app_state, db)
}

#[post("/v1/{chan}")]
async fn interactive_post(channel: web::Path<String>, form: web::Form<FormData>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    app_state.increment_writes();
    let val: String = channel.to_string();
    let _ = db.write(&val, &form.msg);
    interactive(channel, app_state, db)
}

#[get("/v1/{chan}/set/{data}")]
async fn set_by_url(param: web::Path<(String, String)>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    app_state.increment_writes();
    let channel = param.0.to_string();
    let msg = param.1.to_string();
    let _ = db.write(&channel, &msg);
    HttpResponse::Ok()
}

#[post("/v1/{chan}/set")]
async fn set_by_post(param: web::Path<String>, mut payload: web::Payload, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    const MAX_SIZE: usize = 1024 * 1024;  // 1MB
    app_state.increment_writes();
    let channel = param.to_string();
    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = match chunk{
            Ok(c) => c,
            Err(_) => {return HttpResponse::PartialContent();},
        };
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return HttpResponse::PayloadTooLarge();
        }
        body.extend_from_slice(&chunk);
    }
    let body: String = match std::str::from_utf8(&body){
        Ok(s) => s.to_string(),
        Err(_) => {return HttpResponse::UnprocessableEntity();},
    };
    let _ = db.write(&channel, &body);
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let port:u16 = {
        if args.len() > 1{
            args[1].parse().unwrap()
        }
        else{
            8080
        }
    };
    println!("Starting server on port {port}");
    let app_state = Data::new(AppState::new());
    HttpServer::new(move || {
        let db = Data::new(DB::new().unwrap());
        println!("Web server thread connected to db.");
        App::new()
            .app_data(app_state.clone())
            .app_data(db)
            .service(home)
            .service(interactive_get)
            .service(interactive_post)
            .service(channel_get)
            .service(set_by_url)
            .service(set_by_post)
            .service(Files::new("/webfiles", "./webfiles"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

