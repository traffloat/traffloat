import {reify_promise} from "./reified.js"

export function create_bitmap(url) {
	const promise = window.fetch(url, {mode: "cors"})
		.then(resp => resp.blob())
		.then(blob => window.createImageBitmap(blob))
	return reify_promise(promise)
}

async function loadBitmapPromise(url) {
	const resp = await window.fetch(url, {mode: "cors"})
	const blob = await resp.blob()
	const bitmap = await window.createImageBitmap(blob)
	return bitmap
}

async function fetchTextureIndexPromise(url) {
	const resp = await window.fetch(url + ".json", {mode: "cors"})
	const json = await resp.text()
	return json
}

export function load_textures(url) {
	const promise = Await.all([
		loadBitmapPromise(url),
		fetchTextureIndexPromise(url),
	]).then(pair => ({
		bitmap: pair[0],
		index: pair[1],
	}))
	return reify_promise(promise)
}

export function get_bitmap(object) {
	return object.bitmap
}

export function get_index(object) {
	return object.index
}
