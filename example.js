const config = {
  uri: "https://opensea.io/__api/graphql/",
  host: "opensea.io",
  method: "POST",
  headers: [
    ["authority", "opensea.io"],
    ["content-type", "application/json"],
    ["origin", "https://opensea.io/__api/graphql/"],
    [
      "sec-ch-ua",
      '" Not A;Brand";v="99", "Chromium";v="99", "Google Chrome";v="99"',
    ],
    ["sec-ch-ua-mobile", "?0"],
    ["sec-ch-ua-platform", '"Windows"'],
    ["upgrade-insecure-requests", "1"],
    [
      "user-agent",
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.0.4844.51 Safari/537.36",
    ],
    ["accept", "*/*"],
    ["sec-fetch-site", "none"],
    ["sec-fetch-mode", "navigate"],
    ["sec-fetch-user", "?1"],
    ["sec-fetch-dest", "document"],
    ["accept-language", "en-US,en;q=0.9"],
    [
      "x-signed-query",
      "xxx", // use your own
    ],
    ["x-viewer-address", "xxx"], // use your own
  ],
  body: `{
    \"id\": \"NavbarQuery\",
    \"query\": \"query NavbarQuery(\\n  $identity: AddressScalar!\\n) {\\n  getAccount(address: $identity) {\\n    imageUrl\\n    id\\n  }\\n}\\n\",
    \"variables\": {
      \"identity\": \"0xf8e33110b8757e05e1db570a4528412cd907f29d\"
    }
  }`,
};

const axios = require("axios");

axios({
  ...config,
  url: config.uri,
  headers: Object.fromEntries(config.headers),
  data: config.body,
  validateStatus: () => true,
}).then((e) => console.log({ status: e.status, body: e.data }));
// {
//   status: 403,
//   body: '<!DOCTYPE html>\n' +
//     '<html lang="en-US">\n' +
//     '   <head>\n' +
//     '      <title>Access denied</title>\n' +
//     '      <meta http-equiv="X-UA-Compatible" content="IE=Edge" />\n' +
// ...

const { request } = require("./build/macos.node");

request(config, "", "").then(console.log);

// {
//   status: 200,
//   bodyJson: '{"data":{"getAccount":{"imageUrl":"https://i.seadn.io/gcs/files/27554692030796c8858c08ff5b6615a2.jpg?w=500&auto=format","id":"QWNjb3VudFR5cGU6ODY3MDYyNDQw"}}}'
// }
