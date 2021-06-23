export function set_div_lines(div, lines) {
	div.innerHTML = ""
	let first = true
	for(const line of lines.split("\n")) {
		if(!first) {
			div.appendChild(document.createElement("br"))
		}
		first = false
		div.appendChild(document.createTextNode(line))
	}
}
