class ServerNotification {
    constructor(type, user_id) {
        if(type == 'sse') {
            this.sse = true;
            this.container = document.querySelector('[data-type="sse"] .notification-list');
        } else {
            this.websocket = true;
            this.container = document.querySelector('[data-type="websocket"] .notification-list');
        }
        this.user_id = user_id;
    }
    append(message) {
        const element = document.createElement('li');
        if(typeof message == 'object') {
            element.innerText = message.data;
            this.container.append(element)
        }
    }
    start() {
        if (this.sse) {
            this.eventSource = new EventSource(`/sse/${this.user_id}`);
            this.eventSource.onmessage = (event) => {
                this.append(JSON.parse(event.data).message);
            }
        } else {
            this.websocket = new WebSocket(`/ws/${this.user_id}`);
            this.websocket.addEventListener('open', (event) => {
                console.log('websocket connection established: ', event);
                const sendMessage = "PING";
                this.websocket.send(sendMessage);
                const ping_interval = 12000;
                // to Keep the connection alive
                this.interval = setInterval(() => {
                    const sendMessage = "PING";
                    this.websocket.send(sendMessage);
                }, ping_interval);
            });
    
            // subscribe to `close` event
            this.websocket.addEventListener('close', (event) => {
                console.log('websocket connectioned closed: ', event);
                clearInterval(this.interval);
            });
    
            this.websocket.onmessage = (message) => {
                console.log(message);
                if (message.data) {
                    try {
                        const notif = JSON.parse(message.data);
                        console.log(notif)
                        if(notif.message?.data) {
                            this.append(notif.message);
                        }
                    } catch (error) {
                        
                    }
                }
            }
            // subscribe to `error` event
            this.websocket.addEventListener('error', (event) => {
                console.error('an error happend in our websocket connection', event);
            });
        }
    }
}

function sendMessage(event, user_id) {
    event.preventDefault();
    fetch(`/admin/send_notification/${user_id}`, { 
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify({message: document.querySelector('input').value})
    })
}
