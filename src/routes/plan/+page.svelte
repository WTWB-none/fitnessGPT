<script>
	import { disableScrollHandling } from '$app/navigation';
	import { onMount } from 'svelte';
	let workouts = $state([]);
	let meals = $state([]);
	let messages = $state([]);
	let input = $state('');
	let current_day = $state('Пн');
	let prompt =
		'Сформируй план тренировок и питания на неделю для человека весом 50кг, ростом 160см, цель — нарастить мышечную массу. Ответ в JSON с ключами workouts и meals. В свою очередь meal должен иметь поля meal, description и day(сокращенный до двух символов как принято в россии) расписанные для каждого дня, а workout day type duration и description используй для значений полей русский язык в description должен быть массив упражнений/еды с ключами exersize, rest, reps/meal(какой прием пищи), food(вся еда) эти ключи обязательны для каждого элемента массива даже если это отдых. Также у каждого элемента description должен быть ключ checked со значением false и hidden со значением true. Дни недели должны идти обязательно по порядку';

	async function fetchInitialPlan() {
		try {
			const response = await fetch(
				'https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=AIzaSyAcXLSJtXW1qVLpPZVbPEdYGwKAd9-KYFQ',
				{
					method: 'POST',
					headers: {
						'Content-Type': 'application/json'
					},
					body: JSON.stringify({
						contents: [
							{
								parts: [
									{
										text: prompt
									}
								]
							}
						]
					})
				}
			);

			const result = await response.json();
			console.log(result);
			const rawText = result.candidates[0].content.parts[0].text;
			const jsonText = rawText?.match(/\{[\s\S]*\}/)?.[0];
			if (!jsonText) throw new Error('JSON не найден в ответе');
			const plan = JSON.parse(jsonText);
			console.log(plan);
			workouts = plan.workouts;
			meals = plan.meals;
		} catch (e) {
			console.error('Ошибка при получении плана:', e);
		}
	}

	function toggleDone(index, type) {
		if (type === 'workout') workouts[index].done = !workouts[index].done;
		else meals[index].done = !meals[index].done;
	}

	function sendMessage() {
		if (input.trim()) {
			messages = [...messages, { text: input, fromUser: true }];
			input = '';
			setTimeout(() => {
				messages = [
					...messages,
					{ text: 'Ваш план обновлен на основе сообщения.', fromUser: false }
				];
			}, 500);
		}
	}

	onMount(fetchInitialPlan);
</script>

<div class="page">
	<div class="section">
		<div class="week">
			{#each workouts as workout}
				<button
					class="day"
					onclick={() => {
						current_day = workout.day;
					}}>{workout.day}</button
				>
			{/each}
		</div>
		<h2>Тренировки на неделю</h2>
		<ul>
			{#key current_day}
				{#each workouts as workout}
					{#if workout.day === current_day}
						{#each workout.description as desc}
							<li>
								<input
									type="checkbox"
									checked={desc.checked}
									value={desc.exercise}
									onchange={() => {
										desc.checked = true;
									}}
								/>
								<div class="name" onclick={() => (desc.hidden = !desc.hidden)}>{desc.exersize}</div>
							</li>
							{#if !desc.hidden}
								<div class="card">
									<div class="reps">Подходы: {desc.reps}</div>
									<div class="rest">Отдых между подходами: {desc.rest}</div>
								</div>
							{/if}
						{/each}
					{/if}
				{/each}
			{/key}
		</ul>
	</div>
	<div class="section">
		<h2>Питание</h2>
		<ul>
			{#key current_day}
				{#each meals as meal}
					{#if meal.day === current_day}
						{#each meal.description as desc}
							<li>
								<input
									type="checkbox"
									checked={desc.checked}
									value={desc.exercise}
									onchange={() => {
										desc.checked = true;
									}}
								/>
								<div class="name" onclick={() => (desc.hidden = !desc.hidden)}>{meal.meal}</div>
							</li>
							{#if !desc.hidden}
								<div class="card">
									<div class="reps">{desc.food}</div>
								</div>
							{/if}
						{/each}
					{/if}
				{/each}
			{/key}
		</ul>
	</div>

	<div class="chat">
		<div class="input-row">
			<input
				type="text"
				bind:value={input}
				placeholder="Напишите, что хотите изменить..."
				onkeydown={(e) => e.key === 'Enter' && sendMessage()}
			/>
			<button onclick={sendMessage}>Отправить</button>
		</div>
	</div>
</div>

AIzaSyAcXLSJtXW1qVLpPZVbPEdYGwKAd9-KYFQ

<style>
	.week {
		width: 100%;
		display: flex;
		justify-content: space-between;
		gap: 1em;
		padding-bottom: 1vh;
	}

	.page {
		display: grid;
		grid-template-columns: 1fr 1fr;
		min-height: 80vh;
		gap: 2rem;
		padding: 2rem;
		color: white;
		font-family: 'Segoe UI', sans-serif;
	}

	.section {
		background: rgba(255, 255, 255, 0.05);
		width: 100%;
		padding: 2rem;
		max-height: 80vh;
		border-radius: 1.5rem;
		backdrop-filter: blur(10px);
		overflow: scroll;
	}

	.chat {
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		width: 100%;
		height: 100%;
		max-height: 80vh;
	}

	.input-row {
		display: flex;
		width: 100%;
		gap: 0.5rem;
	}

	input[type='text'] {
		flex-grow: 1;
		padding: 1rem;
		border: none;
		width: 80%;
		border-radius: 0.75rem;
		font-size: 1rem;
	}

	button {
		padding: 1rem 1.5rem;
		border: none;
		border-radius: 0.75rem;
		background: #9b59b6;
		color: white;
		font-weight: bold;
		cursor: pointer;
	}

	ul {
		list-style: none;
		padding: 0;
	}

	li {
		display: flex;
		align-items: center;
		gap: 1rem;
		margin: 1rem 0;
	}

	input[type='checkbox'] {
		transform: scale(1.2);
	}

	.card {
		padding: 1em;
		background-color: rgba(255, 255, 255, 0.1);
		border-radius: 15.5px;
	}

	.day {
		width: 20%;
		padding: 0.3em;
	}
</style>
