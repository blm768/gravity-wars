const gravity_wars = import("./gravity_wars");

var game;
gravity_wars.then(gravity_wars => {
    gravity_wars.load_assets().and_then(assets => { game = gravity_wars.start_game(assets); });
});
