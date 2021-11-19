import {reify_promise} from "./reified.js"

async function loadBitmapPromise(url) {
	const resp = await window.fetch(url, {mode: "cors"})
	const blob = await resp.blob()
	const bitmap = await window.createImageBitmap(blob)
	return bitmap
}

export function load_textures(url) {
	return reify_promise(loadBitmapPromise(url))
}
