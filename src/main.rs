use std::sync::atomic::{AtomicU64, Ordering};
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, web::Data};
use actix_files::Files;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result};
use serde::Deserialize;
use sysinfo::{SystemExt, DiskExt};
use futures::StreamExt;

struct AppState{
    read_count: AtomicU64,
    write_count: AtomicU64,
    startup_datetime: DateTime<Utc>,
}

impl AppState{
    fn new() -> AppState {
        let read_count= AtomicU64::new(0);
        let write_count= AtomicU64::new(0);
        let startup_datetime = Utc::now();
        AppState{read_count, write_count, startup_datetime}
    }
}

struct DB{
    conn: Connection,
}

impl DB{
    fn new() -> Result<DB>{
        let conn = Connection::open("./db/v1msgs.db")?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS msgs (
                channel TEXT PRIMARY KEY,
                msg TEXT
            )",
            (),
        )?;
        Ok(DB{conn})
    }

    fn write(&self, channel: &str, msg: &str) ->Result<()>{
        self.conn.execute(
            "INSERT OR REPLACE INTO msgs (channel, msg) VALUES (?1, ?2)",
            (channel, msg),
        )?;
        Ok(())
    }

    fn read(&self, channel: &str) -> Result<String>{
        self.conn.query_row(
            "SELECT msg FROM msgs WHERE channel = ?",
            [channel],
            |row| row.get(0),
        )
    }

    fn num_rows(&self) -> Result<i64>{
        self.conn.query_row(
            "SELECT COUNT(*) FROM msgs",
            [],
            |row| row.get(0),
        )
    }

    fn database_size(&self) -> Result<i64>{
        let page_count: i64 = self.conn.query_row(
            "PRAGMA PAGE_COUNT",
            [],
            |row| row.get(0),
        )?;
        let page_size: i64 = self.conn.query_row(
            "PRAGMA PAGE_SIZE",
            [],
            |row| row.get(0),
        )?;
        Ok(page_count * page_size)
    }
}

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
async fn welcome(app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    let channels = db.num_rows().unwrap_or(-1);
    let reads = app_state.read_count.load(Ordering::SeqCst);
    let writes = app_state.write_count.load(Ordering::SeqCst);
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
<head><title>GrugMQ</title></head>
<body>
<h3>Grug's Free Message Queue</h3>
Hi.  This is Grug's message queue.  You can use it too for free!

<h3>How to Use</h3>
Rest API using only URL.  Can read and write to channels by just visiting URLs, but intention is to use code.
<ul>
 <li> Write to channel using http get. eg. <a href=\"/v1/example_channel/write/grugsdata\">http://grugmq.com/v1/example_channel/write/grugsdata</a>
 <li> Write using http post to url <a href=\"/v1/example_channel/write\">http://grugmq.com/v1/example_channel/write</a>
 <li> Read from channel using http get.  eg. <a href=\"/v1/example_channel/read\">http://grugmq.com/v1/example_channel/read</a>
 <li> Interactively read and send messages at channel address: <a href=\"/v1/example_channel\">http://grugmq.com/v1/example_channel</a>
 <li> Create new channel by simply writing something to it. eg. <a href=\"/v1/new_channel/write/grugsdata\">http://grugmq.com/v1/new_channel/write/grugsdata</a>
</ul>

<h3>Example Python 3</h3>
<pre style=\"margin-left: 3em;\"><code>import urllib.request
# Write \"data123\" to channel python3Example
urllib.request.urlopen(\"http://grugmq.com/v1/python3_example/write/data123\")
# Read back data from channel
data = urllib.request.urlopen(\"http://grugmq.com/v1/python3_example/read\").read()
print(data) # Prints b'data123'
</code></pre>

<h3>Channels</h3>
<ul>
 <li> Each channel hold one message
 <li> Messages overwrite other messages in channel
 <li> Messages persist in database until someone clubs Grug's server
</ul>

<h3>Picture of Me</h3>
<img src=\"/images/me.jpg\">

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
 <li> Source: <a href=\"https://github.com/srleigh/grugmq\">https://github.com/srleigh/grugmq</a>
 <li> Inspiration: <a href=\"https://grugbrain.dev/\">https://grugbrain.dev/</a>
</ul>

<h3>Todo</h3>
<ul>
 <li> Nothing currently
<ul>
</body></html>");
    HttpResponse::Ok().body(html)
}

#[get("/v1/{chan}/read")]
async fn channel_read(channel: web::Path<String>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    app_state.read_count.fetch_add(1, Ordering::SeqCst);
    let channel: String = channel.to_string();
    let msg = db.read(&channel).unwrap_or("".to_string());
    HttpResponse::Ok().body(msg)
}

#[get("/v1/{chan}")]
async fn channel_interactive_get(channel: web::Path<String>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    app_state.read_count.fetch_add(1, Ordering::SeqCst);
    let channel: String = channel.to_string();
    let msg = db.read(&channel).unwrap_or("".to_string());
    let html = format!("<html><body>
    Channel message: {msg}
<form action=\"/v1/{channel}\" method=\"post\" enctype=\"application/x-www-form-urlencoded\">
 <label for=\"msg\">Enter new message: </label>
 <input type=\"text\" name=\"msg\" required>
 <input type=\"submit\" value=\"Send!\">
</form></body></html>");
    HttpResponse::Ok().body(html)
}

#[post("/v1/{chan}")]
async fn channel_interactive_post(channel: web::Path<String>, form: web::Form<FormData>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    app_state.write_count.fetch_add(1, Ordering::SeqCst);
    let channel: String = channel.to_string();
    let _ = db.write(&channel, &form.msg);
    app_state.read_count.fetch_add(1, Ordering::SeqCst);
    let msg = db.read(&channel).unwrap_or("".to_string());
    let html = format!("<html><body>
    Channel message: {msg}
<form action=\"/v1/{channel}\" method=\"post\" enctype=\"application/x-www-form-urlencoded\">
 <label for=\"msg\">Enter new message: </label>
 <input type=\"text\" name=\"msg\" required>
 <input type=\"submit\" value=\"Send!\">
</form></body></html>");
    HttpResponse::Ok().body(html)
}

#[get("/v1/{chan}/write/{data}")]
async fn channel_write_get(param: web::Path<(String, String)>, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    app_state.write_count.fetch_add(1, Ordering::SeqCst);
    let channel = param.0.to_string();
    let msg = param.1.to_string();
    let _ = db.write(&channel, &msg);
    HttpResponse::Ok()
}

#[post("/v1/{chan}/write")]
async fn channel_write_post(param: web::Path<String>, mut payload: web::Payload, app_state: Data<AppState>, db: Data<DB>) -> impl Responder {
    const MAX_SIZE: usize = 1024 * 1024;  // 1MB
    app_state.write_count.fetch_add(1, Ordering::SeqCst);
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
            .service(welcome)
            .service(channel_interactive_get)
            .service(channel_interactive_post)
            .service(channel_read)
            .service(channel_write_get)
            .service(channel_write_post)
            .service(Files::new("/images", "./images"))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}

