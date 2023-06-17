extern crate sys_info;
use std::io::{Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream,IpAddr};
use std::fs::{self, File};
use sys_info::os_type;
use pnet;
use chrono::{DateTime,Local};
use whoami::username;
use tera::{Context, Tera};
use mime_guess;

fn main() {

    /*Estabelecendo Conexão com TCPListener*/
    let listener = TcpListener::bind("127.0.0.1:59999").expect("Failed to Bind Address");
    println!("Listening on http://127.0.0.1:59999");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_connection(stream);
            }
            Err(e) => {
                eprintln!("Failed to establish a connection: {}", e);
            }
        }
    }
}

/* Função das Conexões o Web-Server*/
fn handle_connection(mut stream: TcpStream) {

    let mut buf_reader = BufReader::new(&mut stream);
    let request_line = std::mem::ManuallyDrop::new(&mut buf_reader).lines().next().expect("failed to request line").expect("cannot get error");

    /*Rota / do Servidor - Lista os arquivos da pasta static*/
    if request_line == "GET / HTTP/1.1" {

        let fileslist = list_files("static");
        let tera = Tera::new("html/*").expect("Failed to create Tera variable by the html directory");
        let mut context = Context::new();
        context.insert("n",&fileslist.len());
        let mut count = 0;
        for i in 0..fileslist.len(){
            context.insert("file".to_owned()+&count.to_string(),&fileslist[i]);
            count = count+1;
        }
        let rendered = tera.render("files.html",&context).expect("Failed to render Tera variable");
        let mut file = File::create("html/files.html").expect("Failed to create file in html/files.html");
        file.write_all(rendered.as_bytes()).expect("Failed to write in html/files.html");
        let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string("html/files.html").expect("Failed to read contents from html/files.html");
        let lenght = contents.len();
        let response = format!("{status_line}\r\nContent-Lenght: {lenght}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).expect("Failed to write a response for this stream");

    }

    /*Rota /hello do servidor - Retorna uma página de boas vindas ao cliente */
    else if request_line == "GET /hello HTTP/1.1" {

        let status_line = "HTTP/1.1 200 OK";
        let contents = fs::read_to_string("html/hello.html").expect("failed to read ccontents from html/hello.html");
        let lenght = contents.len();
        let response = format!("{status_line}\r\nContent-Lenght: {lenght}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).expect("Failed to write a response for this stream");


    }

    /*Rota /header do servidor - Retorna o cabeçalho http da requisição do cliente*/
    else if request_line == "GET /HEADER HTTP/1.1" {
        let mut headers = String::new();
        let mut line = String::new();
        let mut buf2 = std::mem::ManuallyDrop::new(&mut buf_reader);
        while let Ok(n) = buf2.read_line(&mut line) {
            if n == 0 || line == "\r\n" {
                break;
            }
            headers.push_str(&line);
            line.clear();
        }
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}", headers);
        stream.write_all(response.as_bytes()).expect("Failed to write a response for this stream");
        return;
    }

    /*Rota /info do servior - Retorna uma página com informações sobre o servidor*/
    else if request_line == "GET /info HTTP/1.1"{
        let os = os_type().expect("Failed to get information of the OS of the server");
        let ip = get_local_ip().expect("Failed to get information about the IP of the server");
        let date = getdata();
        let name = sysname();
        let junc = "Ip = ".to_owned() + &ip.to_string() + " Sistema Operacional = " + &os.to_string() + " Date & Hora = " + &date.to_string() + " Nome do Servidor = " + &name.to_string();
        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n\r\n{}", junc);
        stream.write_all(response.as_bytes()).expect("Failed to write a response for this stream");
        return;

    }

    /*Parte que cuida das requisições de download dos arquivos da pasta */
    else if request_line.starts_with("GET /static/") {
        let file_path = request_line.trim_start_matches("GET ").trim_end_matches(" HTTP/1.1").trim_start_matches("/static/");
        let file_path = format!("static/{}", file_path);
        if let Ok(contents) = fs::read(&file_path) {
            let status_line = "HTTP/1.1 200 OK";
            let length = contents.len();
            let mime_type = mime_guess::from_path(&file_path).first_or_octet_stream();
            let response = format!(
                "{status_line}\r\nContent-Type: {mime_type}\r\nContent-Disposition: attachment; filename=\"{filename}\"\r\nContent-Length: {length}\r\n\r\n",
                status_line = status_line,
                mime_type = mime_type,
                filename = file_path
            );
            stream.write_all(response.as_bytes()).expect("failed to write a response for this stream");
            match stream.write_all(&contents) {
                Ok(_) => {
                    // The write operation was successful
                    // Continue with the rest of your code
                }
                Err(err) => {
                    // Handle the error appropriately
                    eprintln!("Error occurred while writing to the stream: {}", err);
                    // Additional error handling or recovery actions can be implemented here
                }
            }
        }
        /*Caso o cliente peça uma pagina que não existe */
        else {
            let status_line = "HTTP/1.1 404 NOT FOUND";
            let contents = fs::read_to_string("html/notfound.html").expect("Failed to read the contents of this file");
            let length = contents.len();
            let response = format!(
                "{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}",
                status_line = status_line,
                length = length,
                contents = contents
            );
            stream.write_all(response.as_bytes()).expect("Failed to write a response for this stream");
        }
    }
}


/*Função que lista os arquivos da pasta static */
fn list_files(dir: &str) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for entry in fs::read_dir(dir).expect("Cannot Read Directory") {
        if let Ok(entry) = entry {
            if let Some(file_name) = entry.file_name().to_str() {
                names.push(file_name.to_string());
            }
        }
    }
    return names;
}


/* Pega o ip local do servidor*/
fn get_local_ip() -> Result<IpAddr, Box<dyn std::error::Error>> {

    let interfaces = pnet::datalink::interfaces();
    for iface in interfaces {
        for ip_network in iface.ips {
            if let IpAddr::V4(ipv4) = ip_network.ip() {
                if ipv4.is_loopback() {
                    continue;
                }
                return Ok(IpAddr::V4(ipv4));
            }
        }
    }
    Err(From::from("Não foi possível obter o endereço IP local."))
}

/*Pega a hora atual do servidor*/
fn getdata() -> DateTime<Local>{

    let local: DateTime<Local> = Local::now();
    let timezone = local.timezone();
    let current_datetime = Local::now().with_timezone(&timezone);
    return current_datetime;

}

/* Pega o nome o servidor*/
fn sysname()-> String {

    let username = username();
    return username;

}
