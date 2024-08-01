
const planeconnectorAddress = "http://localhost:3100/api/v1";
const autopilotAddress = "http://localhost:3200/api/v1";

//let activate_wings_level_button = document.querySelector("#activate_horizontal_wings_level_button");

async function getAutoPilotState() {
  try {
    const response = await fetch(autopilotAddress.concat("/autopilot_state"));
    const responseData = await response.json();

    return responseData;

  } catch (error) {
    console.error("Error:", error);
  }
}

async function getPlaneState() {
  try {
    const response = await fetch(planeconnectorAddress.concat("/state"));
    const responseData = await response.json();

    return responseData;

  } catch (error) {
    console.error("Error:", error);
  }
}

async function updateUI() {
	
	let state = await getAutoPilotState();

	console.log(state);

	activate_horizontal_standby_button.classList.remove("btn-outline-success", "btn-success");
	activate_horizontal_wings_level_button.classList.remove("btn-outline-success", "btn-success");
	activate_horizontal_heading_button.classList.remove("btn-outline-success", "btn-success");

	activate_vertical_standby_button.classList.remove("btn-outline-success", "btn-success");
	activate_vertical_TECS_button.classList.remove("btn-outline-success", "btn-success");

	//horizontal

	switch (state.horizontal_guidance.horizontal_mode) {
	case "Standby":
		activate_horizontal_standby_button.classList.add("btn-success");
		activate_horizontal_wings_level_button.classList.add("btn-outline-success")
		activate_horizontal_heading_button.classList.add("btn-outline-success")
		break;
	case "WingsLevel":
		activate_horizontal_standby_button.classList.add("btn-outline-success");
		activate_horizontal_wings_level_button.classList.add("btn-success")
		activate_horizontal_heading_button.classList.add("btn-outline-success")
		break;
	case "Heading":
		activate_horizontal_standby_button.classList.add("btn-outline-success");
		activate_horizontal_wings_level_button.classList.add("btn-outline-success")
		activate_horizontal_heading_button.classList.add("btn-success")		
	}

	heading_active.innerHTML = state.horizontal_guidance.heading_setpoint;

	if (document.activeElement !== heading_standby) {
		heading_standby.value = state.horizontal_guidance.heading_standby;
	}

	//vertical

	switch (state.vertical_guidance.vertical_mode) {
	case "Standby":
		activate_vertical_standby_button.classList.add("btn-success");
		activate_vertical_TECS_button.classList.add("btn-outline-success");
		break;
	case "TECS":
		activate_vertical_standby_button.classList.add("btn-outline-success");
		activate_vertical_TECS_button.classList.add("btn-success");
		break;	
	}

	let plane_state = await getPlaneState();

	let plane_state_div = document.querySelector("#plane_state");
	plane_state_div.innerHTML = JSON.stringify(plane_state, null, 2);

    setTimeout(updateUI, 500);

}

async function activateHorizontalStandby() {
  try {
    const response = await fetch(autopilotAddress.concat("/activate/horizontal/standby"), {
      method: "GET",
      headers: {
      	"Accept":"*/*",
      	"Accept-Encoding": "gzip, deflate, br"
      }
    });

    if (response.ok) {
    	console.log("Horizontal standby activated");
    } else {
    	console.error("Horizontal standby not activated");
    }

  } catch (error) {
    console.error("Error:", error);
  }

  //updateUI();
}

async function activateWingsLevel() {
  try {
    const response = await fetch(autopilotAddress.concat("/activate/horizontal/wingslevel"), {
      method: "GET",
      headers: {
      	"Accept":"*/*",
      	"Accept-Encoding": "gzip, deflate, br"
      }
    });

    if (response.ok) {
    	console.log("WingsLevel activated");
    } else {
    	console.error("WingsLevel not activated");
    }

  } catch (error) {
    console.error("Error:", error);
  }

  //updateUI();
}

async function activateHeading() {
  try {
    const response = await fetch(autopilotAddress.concat("/activate/horizontal/heading"), {
      method: "GET",
      headers: {
      	"Accept":"*/*",
      	"Accept-Encoding": "gzip, deflate, br"
      }
    });

    if (response.ok) {
    	console.log("Heading activated");
    } else {
    	console.error("Heading not activated");
    }

  } catch (error) {
    console.error("Error:", error);
  }

  //updateUI();
}

async function activateVerticalStandby() {
  try {
    const response = await fetch(autopilotAddress.concat("/activate/vertical/standby"), {
      method: "GET",
      headers: {
      	"Accept":"*/*",
      	"Accept-Encoding": "gzip, deflate, br"
      }
    });

    if (response.ok) {
    	console.log("Vertical standby activated");
    } else {
    	console.error("Vertical standby not activated");
    }

  } catch (error) {
    console.error("Error:", error);
  }

  //updateUI();
}

async function activateVerticalTECS() {
  try {
    const response = await fetch(autopilotAddress.concat("/activate/vertical/tecs"), {
      method: "GET",
      headers: {
      	"Accept":"*/*",
      	"Accept-Encoding": "gzip, deflate, br"
      }
    });

    if (response.ok) {
    	console.log("TECS activated");
    } else {
    	console.error("TECS not activated");
    }

  } catch (error) {
    console.error("Error:", error);
  }

  //updateUI();
}

async function setHeadingStandby() {

  let heading = heading_standby.value;

  try {
    const response = await fetch(autopilotAddress.concat("/set/heading/").concat(heading), {
      method: "GET",
      headers: {
      	"Accept":"*/*",
      	"Accept-Encoding": "gzip, deflate, br"
      }
    });

    if (response.ok) {
    	console.log("Heading standby set at ", heading);
    } else {
    	console.error("Heading standby not set");
    }

  } catch (error) {
    console.error("Error:", error);
  }

  //updateUI();
}

async function switchHeading() {

  try {
    const response = await fetch(autopilotAddress.concat("/switch/heading"), {
      method: "GET",
      headers: {
      	"Accept":"*/*",
      	"Accept-Encoding": "gzip, deflate, br"
      }
    });

    if (response.ok) {
    	console.log("Heading switched to active");
    } else {
    	console.error("Heading not switched");
    }

  } catch (error) {
    console.error("Error:", error);
  }

  //updateUI();
}


activate_horizontal_standby_button.addEventListener("click", () => activateHorizontalStandby());
activate_horizontal_wings_level_button.addEventListener("click", () => activateWingsLevel());
activate_horizontal_heading_button.addEventListener("click", () => activateHeading());
activate_vertical_standby_button.addEventListener("click", () => activateVerticalStandby());
activate_vertical_TECS_button.addEventListener("click", () => activateVerticalTECS());

heading_standby.addEventListener("change", () => setHeadingStandby());
switch_heading.addEventListener("click", () => switchHeading());

var map = L.map('map').setView([51.505, -0.09], 13);

updateUI();

console.log("Hello world - planepilot UI is active");
