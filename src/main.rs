mod lib;
use tokio;

#[tokio::main]
async fn main() {
    let body = String::from(
        "{
            \"id\": \"NavbarQuery\",
            \"query\": \"query NavbarQuery(\\n  $identity: AddressScalar!\\n) {\\n  getAccount(address: $identity) {\\n    imageUrl\\n    id\\n  }\\n}\\n\",
            \"variables\": {
              \"identity\": \"0xf8e33110b8757e05e1db570a4528412cd907f29d\"
            }
          }",
    );

    let config = lib::RequestConfig {
        body,
        method: "POST".to_string(),
        host: "opensea.io".to_string(),
        uri: "https://opensea.io/__api/graphql/".to_string(),
        headers: vec![
            vec!["authority".to_string(), "opensea.io".to_string()],
            vec!["content-type".to_string(),"application/json".to_string()],
            vec!["origin".to_string(), "https://opensea.io/__api/graphql/".to_string()],
            vec!["sec-ch-ua".to_string(), "\" Not A;Brand\";v=\"99\", \"Chromium\";v=\"99\", \"Google Chrome\";v=\"99\"".to_string()],
            vec!["sec-ch-ua-mobile".to_string(), "?0".to_string()],
            vec!["sec-ch-ua-platform".to_string(), "\"Windows\"".to_string()],
            vec!["upgrade-insecure-requests".to_string(), "1".to_string()],
            vec!["user-agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.51 Safari/537.36".to_string()],
            vec!["accept".to_string(), "*/*".to_string()],
            vec!["sec-fetch-site".to_string(), "none".to_string()],
            vec!["sec-fetch-mode".to_string(), "navigate".to_string()],
            vec!["sec-fetch-user".to_string(), "?1".to_string()],
            vec!["sec-fetch-dest".to_string(), "document".to_string()],
            vec!["accept-language".to_string(), "en-US,en;q=0.9".to_string()],
            vec!["x-signed-query".to_string(), "xxx".to_string()], // use your own
            vec!["x-viewer-address".to_string(), "xxx".to_string()], // use your own
          ]
    };

    let res = lib::request(config).await.unwrap();
    println!("res: {:?}", res);
}
