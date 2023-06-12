  let socket;
  const prNumber = document.getElementById("prNumber");
  const loading = document.getElementById("submit");
  const connect = () => {
    socket = new WebSocket("wss://nixtracker.org/ws");
    socket.binaryType = 'arraybuffer';
    socket.onopen = (event) => {
      socket.send(prNumber.value);
    };

    socket.onclose = (event) => {
      if (event.wasClean && event.code != 1000) {
        document.getElementById("errormsg").textContent = `${event.reason}`;
        loading.classList.remove("loading");
        for (let i = 0; i <= 5; i++) {
          var element = document.getElementById(i);
          element.classList.remove("icon-check");
        }
      }
    };

    socket.addEventListener("message", (event) => {
      const binaryData = new Uint8Array(event.data);
    });
  };

  async function queryPR() {

    const num = parseInt(prNumber.value,10);
    if (isNaN(num)){
      console.log(num);
      document.getElementById("errormsg").textContent = "Error Not a Number";
      loading.classList.remove("loading");
      return;
    }
    if (!socket || socket.readyState === WebSocket.CLOSED) {
        connect();
    }
  else {
        socket.send(prNumber.value);
  }
    document.getElementById("errormsg").textContent = "";
      loading.classList.add("loading");
    socket.binaryType = 'arraybuffer';

    socket.addEventListener("message", (event) => {
      const binaryData = new Uint8Array(event.data);
      console.log(binaryData);
      for (let [index, val] of binaryData.entries()) {
        if (val == 1) {
          var element = document.getElementById(index);
          element.classList.add("icon-check");
        }
        else  {
            var element = document.getElementById(index);
            element.classList.remove("icon-check");
        }
      }

      loading.classList.remove('loading');
    });

  }
