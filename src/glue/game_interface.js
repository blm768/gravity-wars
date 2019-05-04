// Change in zoom factor for each scroll wheel change
const CAMERA_ZOOM_RATE = 0.1;
// The event code for the primary (usually left) mouse button
const PRIMARY_BUTTON = 1;

class GameControls {
    constructor() {
        let controlForm = document.getElementById('game_controls');
        this.controlForm = controlForm;
        this.angleInput = controlForm.elements.namedItem("angle");
        this.powerInput = controlForm.elements.namedItem("power");
        this.fireButton = controlForm.elements.namedItem("fire");
        this.playerIndicator = controlForm.querySelector("#current_player");
    }

    enable(doEnable) {
        this.angleInput.disabled = !doEnable;
        this.powerInput.disabled = !doEnable;
        this.fireButton.disabled = !doEnable;
    }
}

export class GameInterface {
    constructor() {
        if (document.readyState == "loading") {
            document.addEventListener('DOMContentLoaded', () => this.onControlsReady());
        } else {
            this.onControlsReady();
        }
    }

    onGameReady(game) {
        if (this.gameHandle) {
            return;
        }
        this.gameHandle = game;
        this.onReady();
    }

    onControlsReady() {
        if (this.controls) {
            return;
        }
        this.controls = new GameControls;
        this.splash = document.getElementById('splash');
        this.onReady();
    }

    onReady() {
        if (!(this.gameHandle && this.controls)) {
            return;
        }

        if (this.splash) {
            this.splash.style.display = 'none';
        }

        this.controls.controlForm.addEventListener('submit', () => { this.sendFireEvent(); });

        let canvas = this.gameHandle.canvas();
        canvas.addEventListener('mousemove', (event) => {
            if (!(event.buttons & PRIMARY_BUTTON)) return;
            // TODO: handle touch events?
            this.gameHandle.onPan(-event.movementX, event.movementY);
        });
        canvas.addEventListener('wheel', (event) => {
            this.gameHandle.onZoom(event.deltaY * CAMERA_ZOOM_RATE);
        });
        this.gameHandle.onInterfaceReady(this);
    }

    sendFireEvent() {
        let angle = parseFloat(this.controls.angleInput.value) * Math.PI / 180.0;
        let power = parseFloat(this.controls.powerInput.value);

        this.gameHandle.onFire(angle, power);
    }

    updateUI() {
        this.controls.enable(this.gameHandle.isAiming());
        var currentPlayer = this.gameHandle.currentPlayer();
        if (currentPlayer !== undefined) {
            this.controls.playerIndicator.textContent = "Player " + (currentPlayer + 1);
        }
        var color = this.gameHandle.currentPlayerColor();
        if (color !== undefined) {
            this.controls.playerIndicator.style.setProperty("--player-color", "rgb(" + color[0] + "," + color[1] + "," + color[2] + ")");
        }
    }
}
