const glue = import("./glue");
const gravity_wars = import("./gravity_wars");

var gameState;
var gameInterface;

gravity_wars.then((gravity_wars) => {
    gravity_wars.load_assets().and_then((assets) => {
        try {
            gameState = gravity_wars.start_game(assets);
            if (gameInterface) {
                gameInterface.onGameReady(gameState);
            }
        } catch (e) {
            console.log(e);
        }
    });
});

glue.then((glue) => {
    function initInterface() {
        gameInterface = new glue.GameInterface;
        if (gameState) {
            gameInterface.onGameReady(gameState);
        }
    }

    if (document.readyState == "loading") {
        document.addEventListener('DOMContentLoaded', initInterface);
    } else {
        initInterface();
    }
});
