const gravity_wars = import("./gravity_wars");

// Change in zoom factor for each scroll wheel change
const CAMERA_ZOOM_RATE = 0.1;
// The event code for the primary (usually left) mouse button
const PRIMARY_BUTTON = 1;

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
    }

    initControls() {
        this.controlForm.addEventListener('submit', () => { this.sendFireEvent(); });
    }

    sendFireEvent() {
        let angle = parseFloat(this.angleInput.value) * Math.PI / 180.0;
        let power = parseFloat(this.powerInput.value);

        this.gameHandle.onFire(angle, power);
    }
}

function initEvents(game) {
    game.canvas().addEventListener('mousemove', (event) => {
        if (!(event.buttons & PRIMARY_BUTTON)) return;
        // TODO: handle touch events?
        game.onPan(-event.movementX, event.movementY);
    });
    game.canvas().addEventListener('wheel', (event) => {
        game.onZoom(event.deltaY * CAMERA_ZOOM_RATE);
    });
}

var game;
var gameInterface;

gravity_wars.then((gravity_wars) => {
    gravity_wars.load_assets().and_then((assets) => {
        try {
            game = gravity_wars.start_game(assets);
            initEvents(game);
            if (gameInterface) {
                gameInterface.onGameReady(game);
            }
        } catch (e) {
            console.log(e);
        }
    });
});

document.addEventListener('DOMContentLoaded', () => {
    gameInterface = new GameInterface;
    if (game) {
        gameInterface.onGameReady(game);
    }
})
