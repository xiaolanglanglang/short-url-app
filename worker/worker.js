addEventListener('fetch', event => {
    event.respondWith(handleRequest(event.request))
})

async function handleRequest(request) {
    const {handle} = wasm_bindgen;
    // noinspection JSUnresolvedVariable
    await wasm_bindgen(wasm)
    try {
        return await handle(request);
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
        } catch (e) {
            return new Response(e.stack || e.message || e || "unknown error", {
                status: 500
            });
        }
    }
}
