import * as wasm from "nixstat";

const Branch = {
    NixosUnstable: 0,
    NixosUnstableSmall: 1,
    NixpkgsUnstable: 2,
    Nixos2205: 3,
    Nixos2205Small: 4,
}

async function shim() {
    event.preventDefault();
    const val = document.getElementById('prNumber').value;
    const num = parseInt(val,10);
    if (isNaN(num)){
      document.getElementById("errormsg").textContent = "Error Not a Number";
      console.log(num);
      return;
    }
    document.getElementById("errormsg").textContent = "";
    document.getElementById("loading").style.display = 'block';
    // Unstable-Small
    try {
      const result = await wasm.get_branches_status(val,Branch.NixosUnstableSmall);
      if (result) {
        var element = document.getElementById("1");
        element.classList.add("completed");
      }
      else{
      var element = document.getElementById("1");
      element.classList.remove("completed");
      }
    } catch(err) {
      if (err.message == "missing field `merge_commit_sha`") {
        document.getElementById("errormsg").textContent = "Pull Request Not Found";
        console.log("Oh oh we are in trouble");
      }
        console.log("Oh oh we are in trouble " + err.message);
    }
    //NixosUnstable
    try {
      const result = await wasm.get_branches_status(val,Branch.NixosUnstable);
      if (result) {
        var element = document.getElementById("3");
        element.classList.add("completed");
      }
      else{
      var element = document.getElementById("3");
      element.classList.remove("completed");
      }
      
    } catch(err) {
      if (err.message == "missing field `merge_commit_sha`") {
        document.getElementById("errormsg").textContent = "Pull Request Not Found";
        console.log("Oh oh we are in trouble");
      }
        console.log("Oh oh we are in trouble " + err.message);
    }
    //NixpkgsUnstable
    try {
      const result = await wasm.get_branches_status(val,Branch.NixpkgsUnstable);
      if (result) {
        var element = document.getElementById("2");
        element.classList.add("completed");
      }
      else{
      var element = document.getElementById("2");
      element.classList.remove("completed");
      }
      
    } catch(err) {
      if (err.message == "missing field `merge_commit_sha`") {
        document.getElementById("errormsg").textContent = "Pull Request Not Found";
        document.getElementById("loading").style.display = 'none';
        console.log("Oh oh we are in trouble");
      }
        console.log("Oh oh we are in trouble " + err.message);
    }

    //nixos-22.05
    try {
      const result = await wasm.get_branches_status(val,Branch.Nixos2205);
      if (result) {
        var element = document.getElementById("5");
        element.classList.add("completed");
      }
      else{
      var element = document.getElementById("5");
      element.classList.remove("completed");
      }
      
    } catch(err) {
      if (err.message == "missing field `merge_commit_sha`") {
        document.getElementById("errormsg").textContent = "Pull Request Not Found";
        document.getElementById("loading").style.display = 'none';
        console.log("Oh oh we are in trouble");
      }
        console.log("Oh oh we are in trouble " + err.message);
    }
    //nixos-22.05-small
    try {
      const result = await wasm.get_branches_status(val,Branch.Nixos2205Small);
      if (result) {
        var element = document.getElementById("4");
        element.classList.add("completed");
      }
      else{
      var element = document.getElementById("4");
      element.classList.remove("completed");
      }
      
    } catch(err) {
      if (err.message == "missing field `merge_commit_sha`") {
        document.getElementById("errormsg").textContent = "Pull Request Not Found";
        document.getElementById("loading").style.display = 'none';
        console.log("Oh oh we are in trouble");
      }
        console.log("Oh oh we are in trouble " + err.message);
    }
    document.getElementById("loading").style.display = 'none';
}

//console.log(process.env.GITHUB_TOKEN);
const form = document.getElementById('form');
form.addEventListener('submit', shim);
//(async() => {
//  console.log('1')
//  shim()
//  .then((data) => { console.log(data)})
//  console.log('2')
//})()
