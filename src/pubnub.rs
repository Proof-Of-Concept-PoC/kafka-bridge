use crate::socket::{Socket, SocketPolicy, HasSocketPolicy};

pub(crate) struct PubNub {
    pub(crate) socket: Socket,
    pub(crate) channel: String,
}

impl PubNub {
    pub fn new(
        host: String,
        channel: String,
    ) -> Self {
    /*
        let policy = SocketPolicy {
            connected: &Self::connected,
        };
        let mut socket = Socket::new("PubNub", host, policy);
        let mut pubnub = Self {socket: socket, channel: channel};

        pubnub.socket.connect();

        pubnub
        */
    }
}

/*
impl SocketConnectivityPolicy for PubNub {
    fn connected(&self) {
        println!("{} Connected!", self.socket.name);
    }
}
*/
