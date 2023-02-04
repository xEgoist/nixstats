  const prNumber = document.getElementById("prNumber");
  let socket;
  const connect = () => {
    socket = new WebSocket("ws://0.0.0.0:3000/ws");
    socket.binaryType = 'arraybuffer';
    socket.onopen = (event) => {
      socket.send(prNumber.value);
    };

    socket.onclose = (event) => {
      if (event.wasClean) {
        document.getElementById("loading").style.display = 'none';
        document.getElementById("errormsg").textContent = `${event.reason}`;
      }
    };

    socket.addEventListener("message", (event) => {
      // Convert the binary data to a Uint8Array object
      const binaryData = new Uint8Array(event.data);

      // You can do any processing you need on the binaryData here
    });
  };
  // Create a WebSocket client and connect to 0.0.0.0:3000

  // Define the sendNumber function that sends the value from the input
  async function queryPR() {
    event.preventDefault();

    const num = parseInt(prNumber.value,10);
    if (isNaN(num)){
      console.log(num);
      document.getElementById("errormsg").textContent = "Error Not a Number";
      return;
    }
    if (!socket || socket.readyState === WebSocket.CLOSED) {
        connect();
    }
  else {
        socket.send(prNumber.value);
  }
    document.getElementById("errormsg").textContent = "";
    document.getElementById("loading").style.display = 'block';
    socket.binaryType = 'arraybuffer';

    // Add an event listener to handle binary messages
    socket.addEventListener("message", (event) => {
      const binaryData = new Uint8Array(event.data);
      console.log(binaryData);
      for (let [index, val] of binaryData.entries()) {
        if (val == 1) {
          var element = document.getElementById(index);
          element.classList.add("completed");
        }
        else  {
            var element = document.getElementById(index);
            element.classList.remove("completed");
        }
      }
    });

  }