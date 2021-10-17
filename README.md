
<a href="https://hurl.dev"><img src="https://raw.githubusercontent.com/Orange-OpenSource/hurl/master/docs/logo.svg?sanitize=true" align="center" width="264px"/></a>

<br/>

[![deploy status](https://github.com/Orange-OpenSource/hurl/workflows/CI/badge.svg)](https://github.com/Orange-OpenSource/hurl/actions)
[![Crates.io](https://img.shields.io/crates/v/hurl.svg)](https://crates.io/crates/hurl)
[![documentation](https://img.shields.io/badge/-documentation-informational)](https://hurl.dev)

# Presentation

## What's Hurl?

Hurl is a command line tool that runs <b>HTTP requests</b> defined in a simple <b>plain text format</b>.

It can perform requests, capture values and evaluate queries on headers and body response. Hurl is very 
versatile: it can be used for both <b>fetching data</b> and <b>testing HTTP</b> sessions.


```hurl
# Get home:
GET https://example.net

HTTP/1.1 200
[Captures]
csrf_token: xpath "string(//meta[@name='_csrf_token']/@content)"

# Do login!
POST https://example.net/login?user=toto&password=1234
X-CSRF-TOKEN: {{csrf_token}}

HTTP/1.1 302
```


Chaining multiple requests is easy:

```hurl
GET https://api.example.net/health
GET https://api.example.net/step1
GET https://api.example.net/step2
GET https://api.example.net/step3
```

## Also an HTTP Test Tool

Hurl can run HTTP requests but can also be used to <b>test HTTP responses</b>.
Different types of queries and predicates are supported, from [XPath] and [JSONPath] on body response, 
to assert on status code and response headers.

It is well adapted for <b>REST / JSON apis</b>

```hurl
POST https://api.example.net/tests
{
    "id": "456",
    "evaluate": true
}

HTTP/1.1 200
[Asserts]
jsonpath "$.status" == "RUNNING"    # Check the status code
jsonpath "$.tests" count == 25      # Check the number of items

```

<b>HTML content</b>

```hurl
GET https://example.net

HTTP/1.1 200
[Asserts]
xpath "normalize-space(//head/title)" == "Hello world!"
```

and even SOAP apis

```hurl
POST https://example.net/InStock
Content-Type: application/soap+xml; charset=utf-8
SOAPAction: "http://www.w3.org/2003/05/soap-envelope"
<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:m="http://www.example.org">
  <soap:Header></soap:Header>
  <soap:Body>
    <m:GetStockPrice>
      <m:StockName>GOOG</m:StockName>
    </m:GetStockPrice>
  </soap:Body>
</soap:Envelope>

HTTP/1.1 200
```

Hurl can also be used to test HTTP endpoints performances:

```hurl
GET http://api.example.org/v1/pets

HTTP/1.0 200
[Asserts]
duration < 1000  # Duration in ms
```

And responses bytes content

```hurl
GET http://example.org/data.tar.gz

HTTP/1.0 200
[Asserts]
sha256 == hex,039058c6f2c0cb492c533b0a4d14ef77cc0f78abccced5287d84a1a2011cfb81;
```


## Why Hurl?

<ul class="showcase-container">
 <li><b>Text Format:</b> for both devops and developers</li>
 <li><b>Fast CLI:</b> a command line for local dev and continuous integration</li>
 <li><b>Single Binary:</b> easy to install, with no runtime required</li>
</ul>

## Powered by curl

Hurl is a lightweight binary written in [Rust]. Under the hood, Hurl HTTP engine is 
powered by [libcurl], one of the most powerful and reliable file transfer library. 
With its text file format, Hurl adds syntactic sugar to run and tests HTTP requests, 
but it's still the [curl] that we love.

## Feedbacks

Hurl file format and runners are still in beta, any [feedback, suggestion, bugs or improvements] 
are welcome.

```hurl
POST https://hurl.dev/api/feedback
{
  "name": "John Doe",
  "feedback": "Hurl is awesome !"
}
HTTP/1.1 200
```

## Resources

[License]

[Documentation]

[GitHub]



# Usage

## {{ page.title }}
### Name

hurl - run and test HTTP requests.


### Synopsis

**hurl** [options] [FILE...]


### Description

**Hurl** is an HTTP client that performs HTTP requests defined in a simple plain text format.

Hurl is very versatile, it enables to chain HTTP requests, capture values from HTTP responses and make asserts.


```
$ hurl session.hurl
```


If no input-files are specified, input is read from stdin.


```
$ echo GET http://httpbin.org/get | hurl
    {
      "args": {},
      "headers": {
        "Accept": "*/*",
        "Accept-Encoding": "gzip",
        "Content-Length": "0",
        "Host": "httpbin.org",
        "User-Agent": "hurl/0.99.10",
        "X-Amzn-Trace-Id": "Root=1-5eedf4c7-520814d64e2f9249ea44e0"
      },
      "origin": "1.2.3.4",
      "url": "http://httpbin.org/get"
    }
```



Output goes to stdout by default. For output to a file, use the -o option:


```
$ hurl -o output input.hurl
```




By default, Hurl executes all HTTP requests and outputs the response body of the last HTTP call.



### Hurl File Format

The Hurl file format is fully documented in [https://hurl.dev/docs/hurl-file.html](https://hurl.dev/docs/hurl-file.html)

It consists of one or several HTTP requests


```hurl
GET http:/example.net/endpoint1
GET http:/example.net/endpoint2
```



#### Capturing values

A value from an HTTP response can be-reused for successive HTTP requests.

A typical example occurs with csrf tokens.


```hurl
GET https://example.net
HTTP/1.1 200
# Capture the CSRF token value from html body.
[Captures]
csrf_token: xpath "normalize-space(//meta[@name='_csrf_token']/@content)"

# Do the login !
POST https://example.net/login?user=toto&password=1234
X-CSRF-TOKEN: {{csrf_token}}
```


#### Asserts

The HTTP response defined in the Hurl session are used to make asserts.

At the minimum, the response includes the asserts on the HTTP version and status code.


```hurl
GET http:/google.com
HTTP/1.1 302
```


It can also include asserts on the response headers


```hurl
GET http:/google.com
HTTP/1.1 302
Location: http://www.google.com
```


You can also include explicit asserts combining query and predicate


```hurl
GET http:/google.com
HTTP/1.1 302
[Asserts]
xpath "//title" == "301 Moved"
```


Thanks to asserts, Hurl can be used as a testing tool to run scenarii.




### Options

Options that exist in curl have exactly the same semantic.


#### \-\-color {#color}

Colorize Output



#### -b, \-\-cookie &lt;file> {#cookie}

Read cookies from file (using the Netscape cookie file format).

Combined with [-c, \-\-cookie-jar](#cookie-jar), you can simulate a cookie storage between successive Hurl runs.


#### \-\-compressed {#compressed}

Request a compressed response using one of the algorithms br, gzip, deflate and automatically decompress the content.


#### \-\-connect-timeout &lt;seconds> {#connect-timeout}

Maximum time in seconds that you allow Hurl's connection to take.

See also [-m, \-\-max-time](#max-time) option.


#### -c, \-\-cookie-jar &lt;file> {#cookie-jar}

Write cookies to FILE after running the session (only for one session).
The file will be written using the Netscape cookie file format.

Combined with [-b, \-\-cookie](#cookie),you can simulate a cookie storage between successive Hurl runs.



#### \-\-fail-at-end {#fail-at-end}

Continue executing requests to the end of the Hurl file even when an assert error occurs.
By default, Hurl exits after an assert error in the HTTP response.

Note that this option does not affect the behavior with multiple input Hurl files.

All the input files are executed independently. The result of one file does not affect the execution of the other Hurl files.


#### \-\-file-root &lt;dir> {#file-root}

Set root filesystem to import files in Hurl. This is used for both files in multipart form data and request body.
When this is not explicitly defined, the files are relative to the current directory in which Hurl is running.




#### -h, \-\-help {#help}

Usage help. This lists all current command line options with a short description.



#### \-\-html &lt;dir> {#html}

Generate html report in dir.

If the html report already exists, it will be updated with the new test results.


#### \-\-ignore-asserts {#ignore-asserts}

Ignore all asserts defined in the Hurl file.


#### -i, \-\-include {#include}

Include the HTTP headers in the output (last entry).


#### \-\-interactive {#interactive}

Stop between requests.
This is similar to a break point, You can then continue (Press C) or quit (Press Q).


#### \-\-json &lt;file> {#json}

Write full session(s) to a json file. The format is very closed to HAR format.

If the json file already exists, the file will be updated with the new test results.


#### -k, \-\-insecure {#insecure}

This option explicitly allows Hurl to perform "insecure" SSL connections and transfers.



#### -L, \-\-location {#location}

Follow redirect.  You can limit the amount of redirects to follow by using the [\-\-max-redirs](#max-redirs) option.


#### -m, \-\-max-time &lt;seconds> {#max-time}

Maximum time in seconds that you allow a request/response to take. This is the standard timeout.

See also [\-\-connect-timeout](#connect-timeout) option.


#### \-\-max-redirs &lt;num> {#max-redirs}

Set maximum number of redirection-followings allowed
By default, the limit is set to 50 redirections. Set this option to -1 to make it unlimited.


#### \-\-no-color {#color}

Do not colorize Output



#### \-\-noproxy &lt;no-proxy-list> {#noproxy}

Comma-separated list of hosts which do not use a proxy.
Override value from Environment variable no_proxy.



#### \-\-to-entry &lt;entry-number> {#to-entry}

Execute Hurl file to ENTRY_NUMBER (starting at 1).
Ignore the remaining of the file. It is useful for debugging a session.



#### -o, \-\-output &lt;file> {#output}

Write output to &lt;file> instead of stdout.


#### \-\-progress {#progress}

Print filename and status for each test


#### \-\-summary {#summary}

Print test metrics at the end of the run

#### \-\-test {#test}

Activate test mode; equals \-\-output /dev/null \-\-progress \-\-summary


#### -x, \-\-proxy [protocol://]host[:port] {#proxy}

Use the specified proxy.

#### -u, \-\-user &lt;user:password> {#user}

Add basic Authentication header to each request.


#### \-\-variable &lt;name=value> {#variable}

Define variable (name/value) to be used in Hurl templates.
Only string values can be defined.


#### \-\-variables-file &lt;file> {#variables-file}

Set properties file in which your define your variables.

Each variable is defined as name=value exactly as with [\-\-variable](#variable) option.

Note that defining a variable twice produces an error.


#### -v, \-\-verbose {#verbose}

Turn on verbose output on standard error stream
Useful for debugging.

A line starting with '>' means data sent by Hurl.
A line staring with '&lt;' means data received by Hurl.
A line starting with '*' means additional info provided by Hurl.

If you only want HTTP headers in the output, -i, \-\-include might be the option you're looking for.


#### -V, \-\-version {#version}

Prints version information



### Environment

Environment variables can only be specified in lowercase.

Using an environment variable to set the proxy has the same effect as using
the [-x, \-\-proxy](#proxy) option.

#### http_proxy [protocol://]&lt;host>[:port]

Sets the proxy server to use for HTTP.


#### https_proxy [protocol://]&lt;host>[:port]

Sets the proxy server to use for HTTPS.


#### all_proxy [protocol://]&lt;host>[:port]

Sets the proxy server to use if no protocol-specific proxy is set.

#### no_proxy &lt;comma-separated list of hosts>

list of host names that shouldn't go through any proxy.


### Exit Codes

#### 1
Failed to parse command-line options.


#### 2
Input File Parsing Error.


#### 3
Runtime error (such as failure to connect to host).


#### 4
Assert Error.



### WWW

[https://hurl.dev](https://hurl.dev)


### See Also

curl(1)  hurlfmt(1)




# Installation / Build

## {{ page.title }}

### Binaries Installation

#### Linux

Precompiled binary is available at [hurl-{{page.hurl-version}}-x86_64-linux.tar.gz](https://github.com/Orange-OpenSource/hurl/releases/download/{{page.hurl-version}}/hurl-{{page.hurl-version}}-x86_64-linux.tar.gz)

```
INSTALL_DIR=/tmp
curl -sL https://github.com/Orange-OpenSource/hurl/releases/download/{{page.hurl-version}}/hurl-{{page.hurl-version}}-x86_64-linux.tar.gz | tar xvz -C $INSTALL_DIR
export PATH=$INSTALL_DIR/hurl-{{page.hurl-version}}:$PATH

hurl --version
hurl {{page.hurl-version}}
```


##### Debian / Ubuntu {#debian-ubuntu}

For Debian / Ubuntu, Hurl can be installed using a binary .deb file provided in each Hurl release.

```
curl -LO https://github.com/Orange-OpenSource/hurl/releases/download/{{page.hurl-version}}/hurl_{{page.hurl-version}}_amd64.deb
sudo dpkg -i hurl_{{page.hurl-version}}_amd64.deb
```

##### Arch Linux / Manjaro (via [AUR](https://wiki.archlinux.org/index.php/Arch_User_Repository)) {#arch-linux-manjaro-via-aur}

`hurl-bin` [package](https://aur.archlinux.org/packages/hurl-bin/) for Arch Linux and derived distros.

#### macOS

Precompiled binary is available at [hurl-{{page.hurl-version}}-x86_64-osx.tar.gz](https://github.com/Orange-OpenSource/hurl/releases/download/{{page.hurl-version}}/hurl-{{page.hurl-version}}-x86_64-osx.tar.gz)

Hurl can also be installed with [Homebrew](https://brew.sh):

```
brew tap jcamiel/hurl
brew install hurl

hurl --version
hurl {{page.hurl-version}}
```

#### Windows

##### Zip File

Hurl can be installed from a standalone zip file [hurl-{{page.hurl-version}}-win64.zip](https://github.com/Orange-OpenSource/hurl/releases/download/{{page.hurl-version}}/hurl-{{page.hurl-version}}-win64.zip).
You will need to update your PATH variable.


##### Installer

<span style="color:red">**!! There is an ongoing [issue](https://github.com/Orange-OpenSource/hurl/issues/267) with current installer [hurl-{{page.hurl-version}}-win64-installer.exe](https://github.com/Orange-OpenSource/hurl/releases/download/{{page.hurl-version}}/hurl-{{page.hurl-version}}-win64-installer.exe) 
for environment with PATH greater tham 1MB. You should probably save your PATH in this case !!**</span>

It should be fixed in the next release.

#### Cargo

If you're a Rust programmer, Hurl can be installed with cargo.

```
cargo install hurl
```

### Building From Sources

Hurl sources are available in [GitHub](https://github.com/Orange-OpenSource/hurl)

#### Build on Linux, macOS

Hurl depends on libssl, libcurl and libxml2 native libraries. You will need their development files in your platform.

```shell
# debian based distributions
apt install -y pkg-config libssl-dev libcurl4-openssl-dev libxml2-dev

# redhat based distributions
yum install -y pkg-config gcc openssl-devel libxml2-devel

# arch based distributions
pacman -Sy --noconfirm pkgconf gcc openssl libxml2

# osx
brew install pkg-config gcc openssl libxml2
```

Hurl is written in [Rust](https://www.rust-lang.org/). You should [install](https://www.rust-lang.org/tools/install) 
the latest stable release.

```shell
curl https://sh.rustup.rs -sSf | sh -s -- -y
source $HOME/.cargo/env
rustc --version
cargo --version
```

Build

```shell
git clone https://github.com/Orange-OpenSource/hurl
cd hurl
cargo build --release
./target/release/hurl --version
```

#### Build on Windows

Please follow the [contrib/windows section](https://github.com/Orange-OpenSource/hurl/contrib/windows/README.md)






# Samples

## {{ page.title }}

To run a sample, you can edit a file with the sample content, and use Hurl:

```
$ vi sample.hurl

GET https://example.net

$ hurl sample.hurl
```


### Getting Data

A simple GET:

```hurl
GET https://example.net
```

[Doc](https://hurl.dev/docs/request.html#method)

A simple GET with headers:

```hurl
GET https://example.net/news
User-Agent: Mozilla/5.0 
Accept: */*
Accept-Language: en-US,en;q=0.5
Accept-Encoding: gzip, deflate, br
Connection: keep-alive
```

[Doc](https://hurl.dev/docs/request.html#headers)

#### Query Params

```hurl
GET https://example.net/news
[QueryStringParams]
order: newest
search: something to search
count: 100
```

Or:

```hurl
GET https://example.net/news?order=newest&search=something%20to%20search&count=100
```

[Doc](https://hurl.dev/docs/request.html#query-parameters)

### Sending Data

#### Sending HTML Form Datas


```hurl
POST https://example.net/contact
[FormParams]
default: false
token: {{token}}
email: john.doe@rookie.org
number: 33611223344
```


[Doc](https://hurl.dev/docs/request.html#form-parameters)

#### Sending Multipart Form Datas


```hurl
POST https://example.net/upload
[MultipartFormData]
field1: value1
field2: file,example.txt;
# On can specify the file content type:
field3: file,example.zip; application/zip
```


[Doc](https://hurl.dev/docs/request.html#multipart-form-data)

#### Posting a JSON Body

With an inline JSON:

```hurl
POST https://api.example.net/tests
{
    "id": "456",
    "evaluate": true
}
```

[Doc](https://hurl.dev/docs/request.html#json-body)

With a local file:

```hurl
POST https://api.example.net/tests
Content-Type: application/json
file,data.json;
```

[Doc](https://hurl.dev/docs/request.html#file-body)

#### Templating a JSON/XML Body

Using templates with [JSON body](https://hurl.dev/docs/request.md %}#json-body) or [XML body]({% link _docs/request.html#xml
-body)
 is not currently supported in Hurl. Besides, you can use templates in [raw string body](https://hurl.dev/docs/request.html#raw
 -string-body) with variables to send a JSON or XML body:
 

~~~hurl
PUT https://api.example.net/hits
Content-Type: application/json
```
{
    "key0": "{{a_string}}",
    "key1": {{a_bool}},
    "key2": {{a_null}},
    "key3": {{a_number}}
}
```
~~~


Variables can be initialized via command line:

```bash
$ hurl --variable key0=apple \
       --variable key1=true \
       --variable key2=null \
       --variable key3=42 \
       test.hurl
```

Resulting in a PUT request with the following JSON body:

```
{
    "key0": "apple",
    "key1": true,
    "key2": null,
    "key3": 42
}
```

[Doc](https://hurl.dev/docs/request.html#raw-string-body)

### Testing Response

#### Testing Response Headers

Use implicit response asserts to test header values:

```hurl
GET http://www.example.org/index.html

HTTP/1.0 200
Set-Cookie: theme=light
Set-Cookie: sessionToken=abc123; Expires=Wed, 09 Jun 2021 10:18:14 GMT
```

[Doc](https://hurl.dev/docs/asserting-response.html#headers)


Or use explicit response asserts with [predicates](https://hurl.dev/docs/asserting-response.html#predicates):

```hurl
GET https://example.net

HTTP/1.1 302
[Asserts]
header "Location" contains "www.example.net"
```

[Doc](https://hurl.dev/docs/asserting-response.html#header-assert)


#### Testing REST Apis

Asserting JSON body response with [JSONPath](https://goessner.net/articles/JsonPath/):

```hurl
GET https//example.org/order
screencapability: low

HTTP/1.1 200
[Asserts]
jsonpath "$.validated" == true
jsonpath "$.userInfo.firstName" == "Franck"
jsonpath "$.userInfo.lastName" == "Herbert"
jsonpath "$.hasDevice" == false
jsonpath "$.links" count == 12
jsonpath "$.state" != null
```

[Doc](https://hurl.dev/docs/asserting-response.html#jsonpath-assert)

Testing status code:

```hurl
GET https//example.org/order/435

HTTP/1.1 200
```

[Doc](https://hurl.dev/docs/asserting-response.html#version-status)

```hurl
GET https//example.org/order/435

# Testing status code is in a 200-300 range
HTTP/1.1 *
[Asserts]
status >= 200
status < 300
```

[Doc](https://hurl.dev/docs/asserting-response.html#status-assert)


#### Testing HTML Response

```hurl
GET https://example.com

HTTP/1.1 200
Content-Type: text/html; charset=UTF-8

[Asserts]
xpath "string(/html/head/title)" contains "Example" # Check title
xpath "count(//p)" == 2  # Check the number of p
xpath "//p" count == 2  # Similar assert for p
xpath "boolean(count(//h2))" == false  # Check there is no h2  
xpath "//h2" not exists  # Similar assert for h2
```

[Doc](https://hurl.dev/docs/asserting-response.html#xpath-assert)

#### Testing Set-Cookie Attributes

```hurl
GET http://myserver.com/home

HTTP/1.0 200
[Asserts]
cookie "JSESSIONID" == "8400BAFE2F66443613DC38AE3D9D6239"
cookie "JSESSIONID[Value]" == "8400BAFE2F66443613DC38AE3D9D6239"
cookie "JSESSIONID[Expires]" contains "Wed, 13 Jan 2021"
cookie "JSESSIONID[Secure]" exists
cookie "JSESSIONID[HttpOnly]" exists
cookie "JSESSIONID[SameSite]" == "Lax"
```

[Doc](https://hurl.dev/docs/asserting-response.html#cookie-assert)

### Others

#### Testing Endpoint Performance

```hurl
GET https://sample.org/helloworld

HTTP/* *
[Asserts]
duration < 1000   # Check that response time is less than one second
```

[Doc](https://hurl.dev/docs/asserting-response.html#duration-assert)

#### Using SOAP Apis

```hurl
POST https://example.net/InStock
Content-Type: application/soap+xml; charset=utf-8
SOAPAction: "http://www.w3.org/2003/05/soap-envelope"
<?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope xmlns:soap="http://www.w3.org/2003/05/soap-envelope" xmlns:m="http://www.example.org">
  <soap:Header></soap:Header>
  <soap:Body>
    <m:GetStockPrice>
      <m:StockName>GOOG</m:StockName>
    </m:GetStockPrice>
  </soap:Body>
</soap:Envelope>

HTTP/1.1 200
```

[Doc](https://hurl.dev/docs/request.html#xml-body)

#### Capturing and Using a CSRF Token


```hurl
GET https://example.net

HTTP/* 200
[Captures]
csrf_token: xpath "string(//meta[@name='_csrf_token']/@content)"

POST https://example.net/login?user=toto&password=1234
X-CSRF-TOKEN: {{csrf_token}}

HTTP/* 302
```


[Doc](https://hurl.dev/docs/capturing-response.html#xpath-capture)

#### Checking Byte Order Mark (BOM) in Response Body

```hurl
GET https://example.net/data.bin

HTTP/* 200
[Asserts]
bytes startsWith hex,efbbbf;
```

[Doc](https://hurl.dev/docs/asserting-response.html#bytes-assert)


[XPath]: https://en.wikipedia.org/wiki/XPath
[JSONPath]: https://goessner.net/articles/JsonPath/
[Rust]: https://www.rust-lang.org
[curl]: https://curl.se
[the installation section]: https://hurl.dev/docs/installation.html
[feedback, suggestion, bugs or improvements]: https://github.com/Orange-OpenSource/hurl/issues
[License]: https://hurl.dev/docs/license.html
[Documentation]: https://hurl.dev/docs/man-page.html
[GitHub]: https://github.com/Orange-OpenSource/hurl
[libcurl]: https://curl.se/libcurl/





