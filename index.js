const gravity_wars = import("./gravity_wars");

// Change in zoom factor for each scroll wheel change
const CAMERA_ZOOM_RATE = 0.1;
// The event code for the primary (usually left) mouse button
const PRIMARY_BUTTON = 1;

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
gravity_wars.then((gravity_wars) => {
    gravity_wars.load_assets().and_then((assets) => {
        try {
            game = gravity_wars.start_game(assets);
            initEvents(game);
        } catch (e) {
            console.log(e);
        }
    });
});
