<script lang="ts">
	import { authAPI } from "$lib/authState.svelte";

    async function handleDevLogin(): Promise<void> {
        await authAPI.verifyMagicLink("1234567890");
    }
</script>

<div class="terms">
	<p>
		By logging in, you agree to our
		<a href="/privacy" class="terms-link">Privacy Policy</a>
		and
		<a href="/terms" class="terms-link">Terms of Use</a>
	</p>
</div>

<!-- Development helper button -->
<button type="button" onclick={handleDevLogin} class="dev-button">
	enter app: (updates local storage state). Refresh after entering.
</button>

<style>
	.terms {
	  font-size: 12px;
	  color: #6b7280;
	  line-height: 1.5;
	}

	.terms p {
	  margin: 0;
	}

	.terms-link {
	  color: #6b7280;
	  text-decoration: underline;
	}

	.terms-link:hover {
	  color: #374151;
	}

	.dev-button {
	  margin-top: 1rem;
	  background: #f3f4f6;
	  border: 1px solid #d1d5db;
	  color: #374151;
	  padding: 0.5rem 1rem;
	  border-radius: 0.375rem;
	  font-size: 0.875rem;
	  cursor: pointer;
	  transition: background-color 0.2s;
	}
</style>
