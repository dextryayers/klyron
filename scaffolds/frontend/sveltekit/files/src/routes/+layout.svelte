<script lang="ts">
  import '../app.css'
  let { children } = $props()
</script>

<header>
  <nav>
    <a href="/">{{ name }}</a>
  </nav>
</header>

{@render children()}
