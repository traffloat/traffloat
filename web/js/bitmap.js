import {reify_promise} from "./reified.js"

export function create_bitmap(url) {
	const promise = window.fetch(url, {mode: "cors"})
		.then(resp => resp.blob())
		.then(blob => window.createImageBitmap(blob))
	return reify_promise(promise)
}
