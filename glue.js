glue = {
    getElementById: (id) => document.getElementById(id),
    getWebGLContext: () => {
        let canvas = document.getElementById("game_canvas");
        return canvas.getContext("webgl");
    },
    fetchAsset: (uri, callback) => {
        fetch(uri).then((response) => {
            response.arrayBuffer().then((buf) => {
                callback(response, new Uint8Array(buf));
            });
        });
    },
    responseIsOK: (response) => response.ok,
}
