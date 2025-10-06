const canvas = document.getElementById("canvas");
const width = canvas.width;
const height = canvas.height;
const context = canvas.getContext("2d");
//context.imageSmoothingEnabled = false;
const scale = window.devicePixelRatio;
var wsurl = "ws://" + window.location.href.split("/")[2].split(":")[0] + ":8888";
console.log("WebSocket is connecting to " + wsurl)
const socket = new WebSocket(wsurl);
var isSocketOpened = false;


colorpicker = document.getElementById("drawing-colorpicker");
moderadio = document.getElementsByName("mode");

var pixel_size = 10;

var curr_mode = ""


canvas.width = window.innerWidth+10;
canvas.height = window.innerHeight;


class Pixel {
    x = 0;
    y = 0;
    color = "";
    constructor(x, y, color) {
        this.x = x;
        this.y = y;
        this.color = color;
    }
}

var pixels = [];

var offset_x = 0;
var offset_y = 0;

function setPixel(x, y, color) {
    context.fillStyle = color;
    context.strokeStyle = color;
    context.fillRect(Math.round(x*pixel_size+offset_x), Math.round(y*pixel_size+offset_y), Math.round(pixel_size), Math.round(pixel_size));
}

function get_cursor_pos_on_grid(mouseX, mouseY){
    const canvasScale = 1
    return [(mouseX/canvasScale*pixel_size)/pixel_size - pixel_size/2, (mouseY/canvasScale*pixel_size)/pixel_size - pixel_size/2];
}

function drawPixels() {
    pixels.forEach(function(pixel) {setPixel(pixel.x, pixel.y, pixel.color)});
}

function clearCanvas() {
    context.clearRect(0, 0, canvas.width, canvas.height);
}

function canvasUpdate() {
    clearCanvas();
    drawPixels();
}

function change_mode(mode){
    if (mode == "look"){
        canvas.style.cursor = "grab";
        curr_mode = mode
    }
    else if (mode == "draw"){
        canvas.style.cursor = "auto";
        curr_mode = mode
    }
}



socket.addEventListener("open", (event) => {
    console.log("Connection open");
    isSocketOpened = true;
});

socket.addEventListener("message", (event) => {
    console.log(event.data);
    var data = JSON.parse(event.data);
    if (Array.isArray(data)) {
        data.forEach(function (p) {pixel = new Pixel(p["x"], p["y"], p["c"]);
            pixels.push(pixel);})
        }
        else {
            pixel = new Pixel(data["x"], data["y"], data["c"]);
            pixels.push(pixel);
        }
        drawPixels();
    });
    
    
    
    var mouseHold = false;
    
    var mousedown_x_pos = 0;
    var mousedown_y_pos = 0;
    
    var mouseup_x_pos = 0;
    var mouseup_y_pos = 0;
    
    var delta_x;
    var delta_y;
    
    var t_offset_x = 0;
    var t_offset_y = 0;
    
    var drawed_pixels = []
    
    function draw(event){
        const rect = canvas.getBoundingClientRect();
        const mx = event.clientX - rect.left;
        const my = event.clientY - rect.top;
        const canvasScale = 1
        var xy = get_cursor_pos_on_grid(mx, my);
        var x = xy[0];
        var y = xy[1];
        canvasUpdate();
        var gx = Math.round((x-offset_x)/pixel_size);
        var gy = Math.round((y-offset_y)/pixel_size);
        if (mouseHold && !drawed_pixels.includes([gx, gy].toString())){
            var pixel = new Pixel(gx, gy, colorpicker.value);
        var pixel_data = {x: pixel.x, y: pixel.y, c: pixel.color};
        socket.send(JSON.stringify(pixel_data));
        console.log(gx, gy);
        drawed_pixels.push([gx, gy].toString());
        drawPixels();
    }
    context.strokeRect(Math.round(gx*pixel_size+offset_x), Math.round(gy*pixel_size+offset_y), Math.round(pixel_size), Math.round(pixel_size));
    
}

canvas.addEventListener("pointerdown", function(event) {
    mouseHold = true;
    if (curr_mode == "look"){
        const rect = canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;
        mousedown_x_pos = x;
        mousedown_y_pos = y;
        t_offset_x = offset_x;
        t_offset_y = offset_y;
        canvas.style.cursor = "grabbing";
    }
    else if(curr_mode == "draw"){
        draw(event);
    }
});

canvas.addEventListener("pointermove", function(event) {
    if (curr_mode == "look"){
        if (mouseHold)
            {
                const rect = canvas.getBoundingClientRect();
                const x = event.clientX - rect.left;
                const y = event.clientY - rect.top;
                const canvasScale = 1
                delta_x = (mousedown_x_pos - x)/canvasScale;
                delta_y = (mousedown_y_pos - y)/canvasScale;
                offset_x = -delta_x + t_offset_x;
                offset_y = -delta_y + t_offset_y;
                canvasUpdate();
            }
    }
    else if (curr_mode == "draw"){
        draw(event);
    }
});
canvas.addEventListener("pointerup", function(event) {
    mouseHold = false;
    if (curr_mode == "look"){
        offset_x = t_offset_x - delta_x;
        offset_y = t_offset_y - delta_y;
        t_offset_x = 0;
        t_offset_y = 0;
        canvasUpdate();
        canvas.style.cursor = "grab";
    }
    drawed_pixels = [];
});



canvas.addEventListener("wheel", function(event) {
    const rect = canvas.getBoundingClientRect();
    const canvasScale = 1
    const mouseX = (event.clientX - rect.left)/canvasScale;
    const mouseY = (event.clientY - rect.top)/canvasScale;
    var change = event.deltaY / 132;
    var zoom_power = 2;
    var zoom = 1;
    if (change > 0){
        pixel_size *= zoom_power;
        offset_x = (offset_x * zoom_power) - mouseX
        offset_y = (offset_y * zoom_power) - mouseY
        
    }
    else {
        pixel_size /= zoom_power;
        offset_x = (offset_x+mouseX)/zoom_power;
        offset_y = (offset_y+mouseY)/zoom_power;
    }
    
    canvasUpdate();
});

pixels.push(new Pixel(10, 10, "#000000"))
pixels.push(new Pixel(10, 12, "#000000"))
drawPixels();

change_mode("look")