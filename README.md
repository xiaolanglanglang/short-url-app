# Short Url Generator

## Usage

- Create 3 kv libraries in cloudflare worker, named **ShortUrlData**, **ShortUrlUser**, **ShortUrlAssets**.
- Adding records to ShortUrlUser

    key: 
    
    ```{random_str}```
    
    value:
    
    ```json
    {"username": "{username}", "api_key": "{random_str}"}
    ```

- Create the wrangler.toml file.

example:

```toml
name = "{short-url-app}"
type = "rust"
account_id = "{account_id}"
workers_dev = false
route = "{https://example.com/*}"
zone_id = "{zone_id}"
kv-namespaces = [
         { binding = "ShortUrlData", id = "" },
         { binding = "ShortUrlUser", id = "" },
         { binding = "ShortUrlAssets", id = "" },
]
```

- Use the command ```wrangler publish``` to publish the application.