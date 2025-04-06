<script>
	import { onMount } from 'svelte';
	let user_id = '';
	let target = '';
	let age = 0;
	let height = 0;
	let weight = 0;
	async function addUserData() {
		let request = {
			user_id: user_id,
			goal: target,
			age: Number(age),
			height: Number(height),
			weight: Number(weight)
		};

		console.log(request);

		let result = await fetch('http://localhost:8000/profile', {
			method: 'POST',
			body: JSON.stringify(request)
		});
		console.log(result);
		if (result.ok) {
			window.location.href = '/plan' + user_id;
		}
	}

	onMount(() => {
		user_id = window.location.hash.slice(1);
	});
</script>

<div class="backdrop">
	<h1>Расскажите немного о себе</h1>
	<form class="info-form">
		<div class="w-80">
			<p>Сколько тебе лет</p>
			<input type="text" bind:value={age} />
		</div>
		<div class="w-80">
			<p>Какой у тебя рост(см)?</p>
			<input type="text" bind:value={height} />
		</div>
		<div class="w-80">
			<p>Какой у тебя вес(кг)?</p>
			<input type="text" bind:value={weight} />
		</div>
		<div class="w-80">
			<p>С какой целью хочешь тренироваться?</p>
			<input type="text" bind:value={target} />
		</div>
		<button type="submit" onclick={() => addUserData()}>Продолжить</button>
	</form>
</div>

<style>
	.backdrop {
		width: 100%;
		height: 100vh;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		color: white;
	}

	h1 {
		font-size: 1.5em;
	}

	input {
		width: 100%;
		padding: 1em;
		border: none;
		border-radius: 12px;
		background-color: rgba(255, 255, 255, 0.08);
		color: white;
		font-size: 1em;
	}

	.info-form {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1em;
		width: 90%;
		max-width: 400px;
		padding: 2.5em;
		border-radius: 35px;
		background-color: rgba(255, 255, 255, 0.05);
		backdrop-filter: blur(8px);
	}

	button {
		width: 100%;
		padding: 1em;
		border: none;
		border-radius: 12px;
		background: linear-gradient(90deg, #8e44ad, #3498db, #e91e63);
		color: white;
		font-weight: bold;
		cursor: pointer;
		transition: opacity 0.3s;
	}
</style>
