export function handle_error(message) {
	console.error(message)
	console.error("Stack: ", new Error().stack)

	let data = `${message}\n\nStack:\n`
	for(const line of new Error().stack.toString().split("\n")) {
		if(line.includes("traffloat")) {
			data += line + "\n"
		}
	}

	alert(`The game crashed!
To report this bug, press F12, navigate to "Console" and copy ALL red text.

${data}

See console for full stack trace.`)

}
