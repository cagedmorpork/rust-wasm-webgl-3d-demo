const rust = import('./pkg/rust_3d_demo');
const canvas = document.getElementById('rustCanvas');
const gl = canvas.getContext('webgl', { antialias: true });

rust.then(m => {
    if (!gl) {
        alert('Failed to init WebGl');
        return;
    }

    // ---- nb: the next two lines can be done here, but it's now done on rust ---- //
    // gl.enable(gl.BLEND); // 
    // gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    const FPS_THROTTLE = 1000.0 / 60.0; // ms/frames
    const client = new m.Client();
    const initialTime = Date.now();
    var lastDrawTime = -1; // in ms, initially -1

    function render() {
        window.requestAnimationFrame(render);
        const curTime = Date.now();

        if (curTime >= lastDrawTime + FPS_THROTTLE) {
            lastDrawTime = curTime;

            if (window.innerHeight != canvas.height || window.innerWidth != canvas.width) {
                canvas.height = window.innerHeight;
                canvas.clientHeight = window.innerHeight;
                canvas.style.height = window.innerHeight;

                canvas.width = window.innerWidth;
                canvas.clientWidth = window.innerWidth;
                canvas.style.width = window.innerWidth;

                gl.viewport(0, 0, window.innerWidth, window.innerHeight);
            }

            // if we place rust logic call here, the logic will be executed at the fps throttle 
            // for this demo, that is what we want because we don't wanto eat battery, set gfx card on fire
            let elapsedTime = curTime - initialTime;
            client.update(elapsedTime, window.innerHeight, window.innerWidth);
            client.render();
        }
    }

    render();
});
