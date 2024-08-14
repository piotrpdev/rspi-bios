const socket = new WebSocket('ws://localhost:3000/ws');

socket.addEventListener('message', function (event) {
    console.log('Message from server ', event.data);
    const code = document.createElement("code")
    code.innerText = event.data;
    document.querySelector("#messages")?.appendChild(code);
});