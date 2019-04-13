import init, * as gravityWars from './gravity_wars.js';

var gameState;
var gameInterface;

async function start() {
    await init('./gravity_wars_bg.wasm');
    gameInterface = gravityWars.initInterface();
    gravityWars.loadAssets().then((assets) => {
        try {
            gameState = gravityWars.startGame(assets);
            if (gameState) {
                gameInterface.onGameReady(gameState);
            }
        } catch (e) {
            console.log(e);
        }
    });
}

start();
