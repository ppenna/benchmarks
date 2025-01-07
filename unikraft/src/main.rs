use ascii::AsciiString;
use std::str::FromStr;
use tiny_http::{Server, Response, Header, HeaderField};

fn get_content_type(headers: &[Header]) -> Option<AsciiString> {
    let content_type_field = HeaderField::from_str("Content-type").unwrap();
    for header in headers {
        if header.field ==  content_type_field {
            return Some(header.value.clone());
        }
    }
    None
}

fn main() -> std::io::Result<()> {
    let server = Server::http("0.0.0.0:8080").unwrap();
    let port = server.server_addr().to_ip().unwrap().port();
    println!("Now listening on port {port}");

    for mut request in server.incoming_requests() {
        // create vector to store the body
        let mut body_vec:  Vec<u8> = Vec::new(); 
        let request_body_size = request.as_reader().read_to_end(&mut body_vec).unwrap();

        if request_body_size == 0 {
            let response = Response::from_string("Internal Server Error").with_status_code(500);
            request.respond(response)?;
            continue;
        } 

        // Get the content type
        let content_type = get_content_type(request.headers()).unwrap();

        // Create the response
        let content_type_header = Header {
            field: HeaderField::from_str("Content-type").unwrap(),
            value: content_type
        };

        let content_length_header = Header {
            field: HeaderField::from_str("Content-length").unwrap(),
            value: AsciiString::from_str(request_body_size.to_string().as_str()).unwrap(),
        };

        let response = Response::from_data(body_vec).with_header(content_type_header).with_header(content_length_header);
        request.respond(response)?;
    }

    Ok(())
}