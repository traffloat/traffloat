export function handle_error(message) {
	message += "\n\nStack:\n\n"
	message += new Error().stack

	console.error(message)
	alert(`Fatal error: ${message}`)
}
