let myConfirmMessage = undefined

function myListener(event) {
	event.preventDefault()
	return event.returnValue = myConfirmMessage
}

export function set_message(confirmMessage) {
	const hadMessage = myConfirmMessage !== undefined
	myConfirmMessage = confirmMessage
	if(!hadMessage) {
		window.addEventListener("beforeunload", myListener, {capture: true})
	}
}

export function unset_messsage() {
	if(myConfirmMessage !== undefined) {
		myConfirmMessage = undefined
		window.removeEventListener("beforeunload", myListener, {capture: true})
	}
}
