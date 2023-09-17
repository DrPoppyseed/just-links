<script lang="ts">
</script>

<form
  action={`${import.meta.env.VITE_PUBLIC_APP_SERVER_BASE_URL}/auth/authn`}
  method="POST"
>
  <button
    type="submit"
    class="px-6 py-2 font-bold text-white bg-[#888888] rounded-md"
  >
    Sign up with Pocket
  </button>
</form>
