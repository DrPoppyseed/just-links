<script lang="ts">
  import SignUpButton from "../lib/components/SignUpButton.svelte";
</script>

<div class="py-8 flex flex-col">
  <section class="mb-12">
    <h1 class="text-4xl font-serif mb-5">
      What Octal is to Hacker News, but for Pocket.
    </h1>
    <p class="text-sm mb-5">
      Save articles with Pocket and read them whenever you like with a simpler
      interface.
    </p>
    <SignUpButton />
  </section>
  <section class="mb-12">
    <h2 class="text-2xl font-serif mb-3">Just links. Maybe a bit more.</h2>
    <img alt="preview" src="/images/preview.png" />
  </section>
  <section class="mb-8">
    <h2 class="text-2xl font-serif mb-5">And yes. It's open source.</h2>
    <p class="text-sm mb-2">
      Take a look under the hood, or contribute to the codebase from our GitHub
      repo. Maybe even give it a star if you’re feeling generous.
    </p>
    <a
      class="text-sm text-blue-600"
      href="https://github.com/DrPoppyseed/just-links">DrPoppyseed/just-links</a
    >
  </section>
</div>
