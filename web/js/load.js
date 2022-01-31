export function load_file(file) {
	return window.fetch(file)
		.then(resp => {
			if(!resp.ok) {
				throw new Error(`${resp.status} ${resp.statusText}`);
			}

			return resp.arrayBuffer()
		})
		.then(ab => new Uint8Array(ab))
}
