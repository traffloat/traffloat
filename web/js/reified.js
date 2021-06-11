export function reify_promise(promise) {

	function ReifiedPromise() {
		this.state = 0
		this.value = undefined
	}
	const reified = new ReifiedPromise()
	promise.then(value => {
		reified.state = 1
		reified.value = value
	})
		.catch(err => {
			reified.state = 2
			reified.value = err
		})
	return reified
}

export function reified_state(reified) {
	return reified.state
}

export function reified_value(reified) {
	return reified.value
}
