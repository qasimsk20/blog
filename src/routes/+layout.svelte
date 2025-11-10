<script>
	import '../app.css';
	import { afterNavigate } from '$app/navigation';
	import { onMount } from 'svelte';
	import { navigating } from '$app/stores';

	onMount(() => {
		document.documentElement.setAttribute('data-webtui-theme', 'catppuccin-mocha');
	});
	afterNavigate(() => {
		document.documentElement.setAttribute('data-webtui-theme', 'catppuccin-mocha');
	});
</script>

<slot />
