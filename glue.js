import { WasmFetchResult } from './gravity_wars'

export function getWebGlContext() {
    let canvas = document.getElementById("game_canvas");
    return canvas.getContext("webgl");
}
export function fetchAsset(uri, callback) {
    const handleResponse = (response) => {
        if (!response.ok) {
            callback(WasmFetchResult.http_error(response));
            return;
        }

        response.arrayBuffer()
            .then(buf => {
                let data = new Uint8Array(buf);
                callback(WasmFetchResult.ok(data));
            })
            .catch(err => callback(WasmFetchResult.interrupted(err.message)));
    };

    fetch(uri)
        .then(handleResponse)
        .catch(err => callback(WasmFetchResult.net_error(err.message)));
}
