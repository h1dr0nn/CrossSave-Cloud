<script lang="ts">
  export let params: { id: string };
</script>

<section class="placeholder">
  <div>
    <p class="eyebrow">Game Detail View</p>
    <h1>Game ID: {params.id}</h1>
    <p>This placeholder route will be replaced with full save management in a later task.</p>
  </div>
  <a href="/" class="back">Return to library</a>
</section>

<style>
  .placeholder {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: clamp(12px, 2vw, 18px);
    padding: clamp(18px, 3vw, 32px);
    text-align: center;
    background: linear-gradient(135deg, #e0f2fe, #eef2ff);
    color: #0f172a;
  }

  h1 {
    margin: 0;
    font-size: clamp(1.4rem, 1vw + 1.2rem, 2rem);
  }

  p {
    margin: 0;
    font-size: clamp(0.95rem, 0.5vw + 0.85rem, 1.1rem);
  }

  .eyebrow {
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: #475569;
  }

  .back {
    padding: clamp(10px, 1.2vw, 14px) clamp(14px, 2vw, 20px);
    border-radius: 999px;
    background: #0ea5e9;
    color: #fff;
    text-decoration: none;
    font-weight: 700;
    box-shadow: 0 12px 24px rgba(14, 165, 233, 0.25);
  }
</style>
