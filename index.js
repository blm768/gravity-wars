const gravity_wars = import("./gravity_wars");

var gameState;
var gameInterface;

gravity_wars.then((gravity_wars) => {
    gameInterface = gravity_wars.initInterface();
    gravity_wars.load_assets().then((assets) => {
        try {
            gameState = gravity_wars.start_game(assets);
            if (gameState) {
                gameInterface.onGameReady(gameState);
            }
        } catch (e) {
            console.log(e);
        }
    });
});
