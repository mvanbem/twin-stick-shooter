import init, { launch } from './twin_stick_shooter_client.js';

(async function () {
    await init();
    launch();
})();

let websocket = new Promise((resolve, reject) => {
    let url = new URL(window.location.href);
    url.protocol = "ws";
    url.pathname = "/websocket";
    let ws = new WebSocket(url);
    ws.addEventListener('open', () => {
        resolve(ws);
        ws.send('hello from javascript');
    });
    ws.addEventListener('message', e => {
        console.log('received websocket message:', e.data);
    });
    ws.addEventListener('error', e => {
        reject(new Error('websocket error: ' + e));
        console.log('websocket error:', e);
    });
    return ws;
});

(async function () {
    let conn = new RTCPeerConnection();

    let channel = conn.createDataChannel('test', {
        ordered: false,
        maxRetransmits: 0,
    });
    channel.addEventListener('open', e => {
        channel.send('test message on data channel');
    });
    channel.addEventListener('message', e => {
        console.log('RTCDataChannel message:', e.data)
    });

    let offer = await conn.createOffer();
    await conn.setLocalDescription(offer);
    let resp = await (async () => {
        let resp = await fetch('/webrtc-offer', {
            method: 'POST',
            body: offer.sdp,
        });
        if (resp.status !== 200) {
            throw new Error('unexpected response status to webrtc-offer: ' + resp.status);
        }
        return await resp.json();
    })();
    await conn.setRemoteDescription(resp.answer);
    await conn.addIceCandidate(resp.candidate);
})();
