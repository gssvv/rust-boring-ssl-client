use h2::client;
use http::Request;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use boring::ssl::{ConnectConfiguration, SslConnector, SslMethod};

use once_cell::sync::OnceCell;
use std::error::Error;
use std::net::ToSocketAddrs;

use bytes::{BufMut, Bytes, BytesMut};

use neon::context::{Context, FunctionContext, ModuleContext};
use neon::prelude::*;

use tokio::runtime::Runtime;

/**
 * Neon API
 */

pub struct RequestConfig {
    pub method: String,
    pub body: String,
    pub host: String,
    pub uri: String,
    pub headers: Vec<Vec<String>>,
}

#[neon::main]
fn neon_main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("request", request_api)?;
    Ok(())
}

fn request_api(mut cx: FunctionContext) -> JsResult<JsPromise> {
    // Setting up arguments
    let config = cx.argument::<JsObject>(0).unwrap();
    let proxy_addr = cx.argument::<JsString>(1).unwrap().value(&mut cx) as String;
    let proxy_auth_in_base64 = cx.argument::<JsString>(2).unwrap().value(&mut cx) as String;

    let method =
        (config.get(&mut cx, "method").unwrap() as Handle<JsString>).value(&mut cx) as String;
    let body = (config.get(&mut cx, "body").unwrap() as Handle<JsString>).value(&mut cx) as String;
    let host = (config.get(&mut cx, "host").unwrap() as Handle<JsString>).value(&mut cx) as String;
    let uri = (config.get(&mut cx, "uri").unwrap() as Handle<JsString>).value(&mut cx) as String;
    let headers = (config.get(&mut cx, "headers").unwrap() as Handle<JsArray>)
        .to_vec(&mut cx)
        .unwrap();
    let headers: Vec<Vec<String>> = headers
        .iter()
        .map(|header| {
            (header.downcast(&mut cx).unwrap() as Handle<JsArray>)
                .to_vec(&mut cx)
                .unwrap()
                .iter()
                .map(|header_part| header_part.to_string(&mut cx).unwrap().value(&mut cx) as String)
                .collect()
        })
        .collect();

    let request_config = RequestConfig {
        body,
        host,
        uri,
        headers,
        method,
    };

    // Setting up runtime & promises
    let rt = runtime(&mut cx)?;
    let channel = cx.channel();
    let (deferred, promise) = cx.promise();

    rt.spawn(async move {
        let result = if proxy_addr.len() > 0 {
            request_with_proxy(request_config, proxy_addr, proxy_auth_in_base64)
                .await
                .unwrap()
        } else {
            request(request_config).await.unwrap()
        };

        deferred.settle_with(&channel, move |mut cx| {
            let obj = cx.empty_object();
            let status = cx.number(result.0);
            let body_json = cx.string(result.1);

            obj.set(&mut cx, "status", status)?;
            obj.set(&mut cx, "bodyJson", body_json)?;

            Ok(obj)
        });
    });

    Ok(promise)
}

fn runtime<'a, C: Context<'a>>(cx: &mut C) -> NeonResult<&'static Runtime> {
    static RUNTIME: OnceCell<Runtime> = OnceCell::new();

    RUNTIME.get_or_try_init(|| Runtime::new().or_else(|err| cx.throw_error(err.to_string())))
}

/**
 * Public Methods
 */

pub async fn request(request_config: RequestConfig) -> Result<(u16, String), Box<dyn Error>> {
    let addr = format!("{}:443", request_config.host)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let tcp = TcpStream::connect(&addr).await?;

    connect_and_send_request(tcp, request_config).await
}

pub async fn request_with_proxy(
    request_config: RequestConfig,
    proxy_addr: String,
    proxy_auth_in_base64: String,
) -> Result<(u16, String), Box<dyn Error>> {
    let addr = proxy_addr.to_socket_addrs().unwrap().next().unwrap();
    let mut tcp = TcpStream::connect(&addr).await?;

    let connect_request = [
        format!("CONNECT {}:443 HTTP/1.1", request_config.host).to_string(),
        format!("Host: {}:443", request_config.host).to_string(),
        format!("Proxy-Authorization: Basic {}", proxy_auth_in_base64),
        "User-Agent: curl/7.81.0".to_string(),
        "Connection: keep-alive".to_string(),
        "\r\n".to_string(),
    ]
    .join("\r\n");

    tcp.write_all(connect_request.as_bytes()).await.unwrap();
    let mut msg = vec![0; 1024];

    loop {
        tcp.readable().await?;

        match tcp.try_read(&mut msg) {
            Ok(n) => {
                msg.truncate(n);
                break;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    connect_and_send_request(tcp, request_config).await
}

// config: RequestConfig
async fn connect_and_send_request(
    tcp: TcpStream,
    request_config: RequestConfig,
) -> Result<(u16, String), Box<dyn Error>> {
    let connect_config = connect_config();

    let res = tokio_boring::connect(connect_config, request_config.host.as_str(), tcp).await;
    let tls = res.unwrap();

    let (mut client, h2) = client::Builder::new()
        .initial_connection_window_size(1024 * 1024 * 1024)
        .initial_window_size(1024 * 1024 * 1024)
        .handshake::<_, Bytes>(tls)
        .await
        .unwrap();

    let mut request = Request::builder()
        .version(http::version::Version::HTTP_2)
        .method(request_config.method.as_str())
        .uri(request_config.uri);

    let mut i = 0;

    while i < request_config.headers.len() {
        request = request.header(
            request_config.headers[i][0].as_str(),
            request_config.headers[i][1].as_str(),
        );
        i = i + 1;
    }

    let has_body = request_config.body.len() > 0;

    if has_body {
        request = request.header("content-length", request_config.body.len());
    }

    let request = request.body(()).unwrap();
    let (response, mut send_stream) = client.send_request(request, !has_body).unwrap();

    if has_body {
        send_stream
            .send_data(Bytes::from(request_config.body), true)
            .unwrap();
    }

    tokio::spawn(async move {
        if let Err(e) = h2.await {
            println!("GOT ERR={:?}", e);
        }
    });

    let (res_parts, mut body) = response.await?.into_parts();
    let mut response_buf = BytesMut::new();

    while let Some(chunk) = body.data().await {
        response_buf.put(chunk?);
    }

    Ok((
        res_parts.status.as_u16(),
        String::from_utf8(response_buf.to_vec())?,
    ))
}

pub fn connect_config() -> ConnectConfiguration {
    let cipher_list = "TLS_AES_128_GCM_SHA256:TLS_AES_256_GCM_SHA384:TLS_CHACHA20_POLY1305_SHA256:ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384:ECDHE-ECDSA-CHACHA20-POLY1305:ECDHE-RSA-CHACHA20-POLY1305:ECDHE-RSA-AES128-SHA:ECDHE-RSA-AES256-SHA:AES128-GCM-SHA256:AES256-GCM-SHA384:AES128-SHA:AES256-SHA";
    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_verify(boring::ssl::SslVerifyMode::NONE);
    builder.set_grease_enabled(true);
    builder.enable_ocsp_stapling();
    builder.set_cipher_list(&cipher_list).unwrap();
    builder
        .set_alpn_protos(&[2, 104, 50, 8, 104, 116, 116, 112, 47, 49, 46, 49])
        .unwrap();
    builder.enable_signed_cert_timestamps();
    let connector = builder.build();

    let mut connect_config = connector.configure().unwrap();
    connect_config.set_verify_hostname(false);

    connect_config
}
