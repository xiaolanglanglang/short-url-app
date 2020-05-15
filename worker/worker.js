addEventListener('fetch', event => {
    event.respondWith(handleRequest(event))
})

async function handleRequest(event) {
    let request = event.request;
    const {handle, need_cache} = wasm_bindgen;
    const cache = caches.default;
    // noinspection JSUnresolvedVariable
    await wasm_bindgen(wasm)
    try {
        let need_cache_flag = false;
        if (need_cache(request.url)) {
            need_cache_flag = true;
            let response = await cache.match(request.url, request);
            if (response) {
                return response;
            }
        }
        let response = await handle(request);
        if (need_cache_flag) {
            event.waitUntil(cache.put(request, response.clone()));
        }
        return response;
    } catch (e) {
        try {
            const json_str = e;
            const err = JSON.parse(json_str);
            return new Response(
                json_str, {
                    status: err.status,
                    headers: {"content-Type": "application/json"}
                }
            )
        } catch (_) {
            return new Response(e.stack || e.message || e || "unknown error", {
                status: 500
            });
        }
    }
}
