import { WasmFetchResult } from './gravity_wars';

// Change in zoom factor for each scroll wheel change
const CAMERA_ZOOM_RATE = 0.1;
// The event code for the primary (usually left) mouse button
const PRIMARY_BUTTON = 1;

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

export class GameInterface {
    constructor() {
        let controlForm = document.getElementById('game_controls');
        this.controlForm = controlForm;
        this.angleInput = controlForm.elements.namedItem("angle");
        this.powerInput = controlForm.elements.namedItem("power");
        this.fireButton = controlForm.elements.namedItem("fire");
        this.playerIndicator = controlForm.querySelector("#current_player");
    }

    onGameReady(game) {
        if (this.gameHandle) {
            return;
        }

        this.gameHandle = game;
        this.initControls();
        this.gameHandle.onInterfaceReady(this);
    }

    initControls() {
        this.controlForm.addEventListener('submit', () => { this.sendFireEvent(); });

        let canvas = this.gameHandle.canvas();
        canvas.addEventListener('mousemove', (event) => {
            if (!(event.buttons & PRIMARY_BUTTON)) return;
            // TODO: handle touch events?
            this.gameHandle.onPan(-event.movementX, event.movementY);
        });
        canvas.addEventListener('wheel', (event) => {
            this.gameHandle.onZoom(event.deltaY * CAMERA_ZOOM_RATE);
        });
    }

    sendFireEvent() {
        let angle = parseFloat(this.angleInput.value) * Math.PI / 180.0;
        let power = parseFloat(this.powerInput.value);

        this.gameHandle.onFire(angle, power);
    }

    enableControls(doEnable) {
        this.angleInput.disabled = !doEnable;
        this.powerInput.disabled = !doEnable;
        this.fireButton.disabled = !doEnable;
    }

    updateUI() {
        this.enableControls(!this.gameHandle.hasActiveMissiles());
        var currentPlayer = this.gameHandle.currentPlayer();
        if (currentPlayer !== null) {
            this.playerIndicator.textContent = "Player " + (currentPlayer + 1);
        }
        var color = this.gameHandle.currentPlayerColor();
        if (color !== null) {
            this.playerIndicator.style.setProperty("--player-color", "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")");
        }
    }
}
