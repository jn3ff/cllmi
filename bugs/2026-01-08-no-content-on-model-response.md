Issue:
Model is not responding in a way that the tool can parse
``` 
thread 'main' panicked at src/main.rs:132:10:
failed to deserialize response from the api: BadResponse("missing field `content` at line 1 column 234")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace](cluster-dev-oidc:dev ~/D/p/cllmi on  main !4 ?2 ❯ kg pods | awk print $1
awk: syntax error at source line 1
 context is
         >>> print <<<
awk: bailing out at source line 1

cluster-dev-oidc:dev ~/D/p/cllmi on  main !4 ?2 ❯ cllmi
thread 'main' panicked at src/main.rs:132:10:
failed to deserialize response from the api: BadResponse("missing field `content` at line 1 column 234")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace)
```

Debug:

- Add --debug flag to print raw model output

Step:

```
cllmi --debug

[DEBUG] Raw API response:
{"type":"error","error":{"type":"invalid_request_error","message":"Your credit balance is too low to access the Anthropic API. Please go to Plans & Billing to upgrade or purchase credits."},"request_id":"req_011CWx7pziWvo1RSZ85poimT"}


thread 'main' panicked at src/main.rs:148:10:
failed to deserialize response from the api: BadResponse("missing field `content` at line 1 column 234")
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(lol)

Solution:
- Enhance error reporting throughout the response chain (Add ResponseError::Api(String) with the error body from the api on status codes >= 300 and ResponseError::Parse(String) with details for any malformed content from the api.)


Final: 

```
cluster-dev-oidc:dev ~/D/p/cllmi on  main !5 ?3 ❯ kubectl get pods | awk print $1
awk: syntax error at source line 1
 context is
         >>> print <<<
awk: bailing out at source line 1

cluster-dev-oidc:dev ~/D/p/cllmi on  main !5 ?3 ❯ cllmi
API error: Api("Your credit balance is too low to access the Anthropic API. Please go to Plans & Billing to upgrade or purchase credits.")

cluster-dev-oidc:dev ~/D/p/cllmi on  main !5 ?3 ❯
```
